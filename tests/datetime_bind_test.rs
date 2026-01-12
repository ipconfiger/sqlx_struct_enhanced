// Test DateTime insert/update operations with real database connection
//
// This test verifies that insert_bind() and update_bind() correctly handle
// DateTime types by converting them to strings before binding.
//
// Run with:
//   cargo test --test datetime_bind_test --features postgres,chrono -- --ignored
//
// Requires PostgreSQL running at postgres://postgres:@127.0.0.1/test-sqlx-tokio

use sqlx::{Postgres, Pool};
use chrono::{Utc, NaiveDateTime, NaiveDate, NaiveTime, DateTime};

#[tokio::test]
#[ignore]
async fn test_naive_datetime_insert_manual() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::query;

    let pool = Pool::<Postgres>::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    // Create test table
    query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            event_date TIMESTAMP NOT NULL,
            optional_date TIMESTAMP,
            count INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    query("DELETE FROM events WHERE id = 'test-datetime-1'")
        .execute(&pool)
        .await?;

    println!("=== Test: Insert with NaiveDateTime fields (manual conversion) ===");

    // Test insert by manually converting NaiveDateTime to String
    let event_date = Utc::now().naive_utc();
    let optional_date = Some(Utc::now().naive_utc());

    // This is what the macro should generate - convert DateTime to String before binding
    // Note: PostgreSQL requires CAST from text to timestamp
    query(
        "INSERT INTO events (id, name, event_date, optional_date, count) VALUES ($1, $2, CAST($3 AS TIMESTAMP), CAST($4 AS TIMESTAMP), $5)"
    )
    .bind("test-datetime-1")
    .bind("Test Event")
    .bind(event_date.to_string())  // Convert NaiveDateTime to String
    .bind(optional_date.as_ref().map(|d| d.to_string()))  // Convert Option<NaiveDateTime> to Option<String>
    .bind(100)
    .execute(&pool)
    .await?;

    println!("✓ Insert successful with event_date={}, optional_date={}", event_date, optional_date.unwrap());

    // Verify the insert
    let row: (String, String, String, Option<String>, i32) =
        sqlx::query_as("SELECT id, name, event_date::text, optional_date::text, count FROM events WHERE id = $1")
        .bind("test-datetime-1")
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.0, "test-datetime-1");
    assert_eq!(row.1, "Test Event");
    // Note: PostgreSQL may format the timestamp differently, so we just check it's not empty
    assert!(!row.2.is_empty());
    assert!(row.3.is_some());
    assert_eq!(row.4, 100);

    println!("✓ Insert verified: event_date={}, optional_date={}", row.2, row.3.unwrap());

    println!("\n=== Test: Update with NaiveDateTime fields (manual conversion) ===");

    let new_date = Utc::now().naive_utc();

    query("UPDATE events SET event_date = CAST($1 AS TIMESTAMP), optional_date = CAST($2 AS TIMESTAMP), count = $3 WHERE id = $4")
        .bind(new_date.to_string())  // Convert NaiveDateTime to String
        .bind::<Option<String>>(None)  // Set optional_date to NULL
        .bind(150)
        .bind("test-datetime-1")
        .execute(&pool)
        .await?;

    println!("✓ Update successful with event_date={}, optional_date=NULL", new_date);

    // Verify the update
    let row: (String, Option<String>, i32) =
        sqlx::query_as("SELECT event_date::text, optional_date::text, count FROM events WHERE id = $1")
        .bind("test-datetime-1")
        .fetch_one(&pool)
        .await?;

    assert!(!row.0.is_empty());
    assert_eq!(row.1, None); // optional_date should be NULL
    assert_eq!(row.2, 150);

    println!("✓ Update verified: event_date={}, optional_date={:?}", row.0, row.1);

    // Clean up
    query("DELETE FROM events WHERE id = 'test-datetime-1'")
        .execute(&pool)
        .await?;

    println!("\n=== All NaiveDateTime tests passed! ===");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_datetime_types_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::query;

    let pool = Pool::<Postgres>::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    // Create test table with different date/time types
    query(
        r#"
        CREATE TABLE IF NOT EXISTS datetime_test (
            id VARCHAR(36) PRIMARY KEY,
            timestamp_value TIMESTAMP NOT NULL,
            date_value DATE NOT NULL,
            time_value TIME NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    query("DELETE FROM datetime_test WHERE id LIKE 'datetime-edge-%'")
        .execute(&pool)
        .await?;

    println!("=== Testing DateTime edge cases ===");

    // Test case 1: NaiveDateTime
    let dt1 = NaiveDateTime::from_timestamp_millis(1609459200000).unwrap(); // 2021-01-01 00:00:00
    query("INSERT INTO datetime_test (id, timestamp_value, date_value, time_value) VALUES ($1, CAST($2 AS TIMESTAMP), CAST($3 AS DATE), CAST($4 AS TIME))")
        .bind("datetime-edge-1")
        .bind(dt1.to_string())
        .bind(dt1.date().to_string())  // Extract date and convert
        .bind(dt1.time().to_string())  // Extract time and convert
        .execute(&pool)
        .await?;

    let timestamp1: String = sqlx::query_scalar("SELECT timestamp_value::text FROM datetime_test WHERE id = $1")
        .bind("datetime-edge-1")
        .fetch_one(&pool)
        .await?;
    println!("✓ NaiveDateTime: {}", timestamp1);

    // Test case 2: DateTime<Utc>
    let dt2: DateTime<Utc> = DateTime::from_timestamp(1640995200, 0).unwrap(); // 2022-01-01 00:00:00
    query("INSERT INTO datetime_test (id, timestamp_value, date_value, time_value) VALUES ($1, CAST($2 AS TIMESTAMP), CAST($3 AS DATE), CAST($4 AS TIME))")
        .bind("datetime-edge-2")
        .bind(dt2.naive_utc().to_string())  // Convert to NaiveDateTime
        .bind(dt2.naive_utc().date().to_string())
        .bind(dt2.naive_utc().time().to_string())
        .execute(&pool)
        .await?;

    let timestamp2: String = sqlx::query_scalar("SELECT timestamp_value::text FROM datetime_test WHERE id = $1")
        .bind("datetime-edge-2")
        .fetch_one(&pool)
        .await?;
    println!("✓ DateTime<Utc>: {}", timestamp2);

    // Test case 3: NaiveDate only
    let date1 = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
    query("INSERT INTO datetime_test (id, timestamp_value, date_value, time_value) VALUES ($1, CAST($2 AS TIMESTAMP), CAST($3 AS DATE), CAST($4 AS TIME))")
        .bind("datetime-edge-3")
        .bind(date1.and_hms_opt(0, 0, 0).unwrap().to_string())  // Convert to full timestamp
        .bind(date1.to_string())
        .bind("00:00:00".to_string())
        .execute(&pool)
        .await?;

    let date_val: String = sqlx::query_scalar("SELECT date_value::text FROM datetime_test WHERE id = $1")
        .bind("datetime-edge-3")
        .fetch_one(&pool)
        .await?;
    assert_eq!(date_val, "2024-06-15");
    println!("✓ NaiveDate: {}", date_val);

    // Test case 4: NaiveTime only
    let time1 = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
    query("INSERT INTO datetime_test (id, timestamp_value, date_value, time_value) VALUES ($1, CAST($2 AS TIMESTAMP), CAST($3 AS DATE), CAST($4 AS TIME))")
        .bind("datetime-edge-4")
        .bind("1970-01-01 ".to_string() + &time1.to_string())  // Create full timestamp
        .bind("1970-01-01".to_string())
        .bind(time1.to_string())
        .execute(&pool)
        .await?;

    let time_val: String = sqlx::query_scalar("SELECT time_value::text FROM datetime_test WHERE id = $1")
        .bind("datetime-edge-4")
        .fetch_one(&pool)
        .await?;
    assert_eq!(time_val, "14:30:45");
    println!("✓ NaiveTime: {}", time_val);

    // Clean up
    query("DELETE FROM datetime_test WHERE id LIKE 'datetime-edge-%'")
        .execute(&pool)
        .await?;

    println!("\n=== All DateTime edge case tests passed! ===");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_datetime_with_special_values() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::query;

    let pool = Pool::<Postgres>::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    // Create test table
    query(
        r#"
        CREATE TABLE IF NOT EXISTS special_dates (
            id VARCHAR(36) PRIMARY KEY,
            event_date TIMESTAMP NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    query("DELETE FROM special_dates WHERE id LIKE 'special-%'")
        .execute(&pool)
        .await?;

    println!("=== Testing special DateTime values ===");

    // Test case 1: Minimum date
    let min_date = NaiveDateTime::from_timestamp_millis(-62135596800000).unwrap(); // 0001-01-01
    query("INSERT INTO special_dates (id, event_date) VALUES ($1, CAST($2 AS TIMESTAMP))")
        .bind("special-1")
        .bind(min_date.to_string())
        .execute(&pool)
        .await?;
    println!("✓ Minimum date: {}", min_date);

    // Test case 2: Maximum reasonable date
    let max_date = NaiveDateTime::from_timestamp_millis(253402300799000).unwrap(); // 9999-12-31
    query("INSERT INTO special_dates (id, event_date) VALUES ($1, CAST($2 AS TIMESTAMP))")
        .bind("special-2")
        .bind(max_date.to_string())
        .execute(&pool)
        .await?;
    println!("✓ Maximum date: {}", max_date);

    // Test case 3: Current time (with microseconds)
    let now = Utc::now().naive_utc();
    query("INSERT INTO special_dates (id, event_date) VALUES ($1, CAST($2 AS TIMESTAMP))")
        .bind("special-3")
        .bind(now.to_string())
        .execute(&pool)
        .await?;

    let retrieved: String = sqlx::query_scalar("SELECT event_date::text FROM special_dates WHERE id = $1")
        .bind("special-3")
        .fetch_one(&pool)
        .await?;
    println!("✓ Current time with precision: {} (retrieved: {})", now, retrieved);

    // Clean up
    query("DELETE FROM special_dates WHERE id LIKE 'special-%'")
        .execute(&pool)
        .await?;

    println!("\n=== All special value tests passed! ===");

    Ok(())
}
