// DateTime<Utc> CRUD Integration Test
//
// This test verifies complete CRUD operations with DateTime<Utc> fields using EnhancedCrud derive macro
//
// Run with:
//   cargo test --test datetime_utc_crud_test --features postgres,chrono -- --ignored
//
// Requires PostgreSQL running at postgres://alex:@127.0.0.1/test_sqlx_tokio

use sqlx::{Postgres, FromRow, Row};
use sqlx::postgres::PgPoolOptions;
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use chrono::{DateTime, Utc};
use serial_test::serial;
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "datetime_utc_test"]
struct DateTimeUtcTest {
    id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    optional_date: Option<DateTime<Utc>>,
    count: i32,
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_datetime_utc_insert() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Recreate test table
    sqlx::query("DROP TABLE IF EXISTS datetime_utc_test")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE datetime_utc_test (
            id VARCHAR(50) PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            optional_date TIMESTAMPTZ,
            count INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id = 'test-1'")
        .execute(&pool)
        .await?;

    println!("=== Test: Insert DateTime<Utc> fields ===");

    let now = Utc::now();
    let mut record = DateTimeUtcTest {
        id: "test-1".to_string(),
        created_at: now,
        updated_at: now + chrono::Duration::hours(1),
        optional_date: Some(now + chrono::Duration::days(1)),
        count: 100,
    };

    record.insert_bind().execute(&pool).await?;
    println!("✓ Insert successful with created_at={}", record.created_at);

    // Verify insert
    let retrieved = DateTimeUtcTest::by_pk()
        .bind("test-1")
        .fetch_one(&pool)
        .await?;

    assert_eq!(retrieved.id, "test-1");
    assert_eq!(retrieved.count, 100);
    assert!(retrieved.optional_date.is_some());
    println!("✓ Insert verified: created_at={}, updated_at={}",
             retrieved.created_at, retrieved.updated_at);

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id = 'test-1'")
        .execute(&pool)
        .await?;

    println!("=== Insert test passed! ===\n");

    Ok(())
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_datetime_utc_update() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Recreate test table
    sqlx::query("DROP TABLE IF EXISTS datetime_utc_test")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE datetime_utc_test (
            id VARCHAR(50) PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            optional_date TIMESTAMPTZ,
            count INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up and insert initial data
    sqlx::query("DELETE FROM datetime_utc_test WHERE id = 'test-2'")
        .execute(&pool)
        .await?;

    let now = Utc::now();
    let mut record = DateTimeUtcTest {
        id: "test-2".to_string(),
        created_at: now,
        updated_at: now,
        optional_date: Some(now),
        count: 50,
    };

    record.insert_bind().execute(&pool).await?;

    println!("=== Test: Update DateTime<Utc> fields ===");

    // Update record with new DateTime values
    record.updated_at = Utc::now();
    record.optional_date = None;
    record.count = 150;

    record.update_bind().execute(&pool).await?;
    println!("✓ Update successful with updated_at={}", record.updated_at);

    // Verify update
    let retrieved = DateTimeUtcTest::by_pk()
        .bind("test-2")
        .fetch_one(&pool)
        .await?;

    assert_eq!(retrieved.id, "test-2");
    assert_eq!(retrieved.count, 150);
    assert!(retrieved.optional_date.is_none());
    println!("✓ Update verified: optional_date is NULL");

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id = 'test-2'")
        .execute(&pool)
        .await?;

    println!("=== Update test passed! ===\n");

    Ok(())
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_datetime_utc_query() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Recreate test table
    sqlx::query("DROP TABLE IF EXISTS datetime_utc_test")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE datetime_utc_test (
            id VARCHAR(50) PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            optional_date TIMESTAMPTZ,
            count INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id LIKE 'test-query-%'")
        .execute(&pool)
        .await?;

    println!("=== Test: Query DateTime<Utc> fields ===");

    // Insert test data
    let now = Utc::now();
    let mut record1 = DateTimeUtcTest {
        id: "test-query-1".to_string(),
        created_at: now,
        updated_at: now,
        optional_date: Some(now),
        count: 10,
    };
    record1.insert_bind().execute(&pool).await?;

    let mut record2 = DateTimeUtcTest {
        id: "test-query-2".to_string(),
        created_at: now + chrono::Duration::hours(1),
        updated_at: now + chrono::Duration::hours(1),
        optional_date: None,
        count: 20,
    };
    record2.insert_bind().execute(&pool).await?;

    // Test make_query
    let all_records = DateTimeUtcTest::make_query("SELECT * FROM datetime_utc_test WHERE id LIKE 'test-query-%'")
        .fetch_all(&pool)
        .await?;

    assert_eq!(all_records.len(), 2);
    println!("✓ make_query retrieved {} records", all_records.len());

    // Test count_query
    let (count,) = DateTimeUtcTest::count_query("id LIKE 'test-query-%'")
        .fetch_one(&pool)
        .await?;

    assert_eq!(count, 2);
    println!("✓ count_query returned {}", count);

    // Test where_query
    let filtered = DateTimeUtcTest::where_query("count = $1")
        .bind(10)
        .fetch_all(&pool)
        .await?;

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "test-query-1");
    println!("✓ where_query filtered correctly");

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id LIKE 'test-query-%'")
        .execute(&pool)
        .await?;

    println!("=== Query test passed! ===\n");

    Ok(())
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_datetime_utc_null_handling() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Recreate test table
    sqlx::query("DROP TABLE IF EXISTS datetime_utc_test")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE datetime_utc_test (
            id VARCHAR(50) PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            optional_date TIMESTAMPTZ,
            count INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id = 'test-null'")
        .execute(&pool)
        .await?;

    println!("=== Test: DateTime<Utc> NULL handling ===");

    let now = Utc::now();
    let mut record = DateTimeUtcTest {
        id: "test-null".to_string(),
        created_at: now,
        updated_at: now,
        optional_date: None,  // NULL
        count: 0,
    };

    record.insert_bind().execute(&pool).await?;
    println!("✓ Insert with NULL optional_date successful");

    let retrieved = DateTimeUtcTest::by_pk()
        .bind("test-null")
        .fetch_one(&pool)
        .await?;

    assert!(retrieved.optional_date.is_none());
    println!("✓ NULL value correctly retrieved");

    // Update from NULL to Some
    let mut record = retrieved;
    record.optional_date = Some(Utc::now());
    record.update_bind().execute(&pool).await?;

    let retrieved2 = DateTimeUtcTest::by_pk()
        .bind("test-null")
        .fetch_one(&pool)
        .await?;

    assert!(retrieved2.optional_date.is_some());
    println!("✓ Updated NULL to Some value");

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id = 'test-null'")
        .execute(&pool)
        .await?;

    println!("=== NULL handling test passed! ===\n");

    Ok(())
}

#[tokio::test]
#[serial]
#[ignore]
async fn test_datetime_utc_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Recreate test table
    sqlx::query("DROP TABLE IF EXISTS datetime_utc_test")
        .execute(&pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE datetime_utc_test (
            id VARCHAR(50) PRIMARY KEY,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL,
            optional_date TIMESTAMPTZ,
            count INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id LIKE 'test-edge-%'")
        .execute(&pool)
        .await?;

    println!("=== Test: DateTime<Utc> edge cases ===");

    // Test case 1: Minimum date
    let min_dt = DateTime::from_timestamp(0, 0).unwrap(); // 1970-01-01
    let mut record1 = DateTimeUtcTest {
        id: "test-edge-min".to_string(),
        created_at: min_dt,
        updated_at: min_dt,
        optional_date: None,
        count: 1,
    };
    record1.insert_bind().execute(&pool).await?;
    println!("✓ Minimum date inserted: {}", min_dt);

    // Test case 2: Far future date
    let future_dt = DateTime::from_timestamp(4000000000, 0).unwrap(); // 2096-09-27
    let mut record2 = DateTimeUtcTest {
        id: "test-edge-future".to_string(),
        created_at: future_dt,
        updated_at: future_dt,
        optional_date: Some(future_dt),
        count: 2,
    };
    record2.insert_bind().execute(&pool).await?;
    println!("✓ Future date inserted: {}", future_dt);

    // Test case 3: High precision (microseconds)
    let precise_dt = Utc::now();
    let mut record3 = DateTimeUtcTest {
        id: "test-edge-precise".to_string(),
        created_at: precise_dt,
        updated_at: precise_dt,
        optional_date: Some(precise_dt),
        count: 3,
    };
    record3.insert_bind().execute(&pool).await?;

    let retrieved = DateTimeUtcTest::by_pk()
        .bind("test-edge-precise")
        .fetch_one(&pool)
        .await?;
    println!("✓ High precision date - inserted: {}, retrieved: {}",
             precise_dt, retrieved.created_at);

    // Clean up
    sqlx::query("DELETE FROM datetime_utc_test WHERE id LIKE 'test-edge-%'")
        .execute(&pool)
        .await?;

    println!("=== Edge cases test passed! ===\n");

    Ok(())
}
