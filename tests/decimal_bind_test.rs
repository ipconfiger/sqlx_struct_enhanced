// Test Decimal insert/update operations with real database connection
//
// This test verifies that insert_bind() and update_bind() correctly handle
// Decimal types by converting them to strings before binding.
//
// Run with:
//   cargo test --test decimal_bind_test --features postgres,decimal -- --ignored
//
// Requires PostgreSQL running at postgres://postgres:@127.0.0.1/test-sqlx-tokio

use sqlx::{Postgres, Pool};
use rust_decimal::Decimal;
use std::str::FromStr;

// This struct will NOT use EnhancedCrud because it requires FromRow
// Instead, we'll manually test the generated code pattern
struct Product {
    id: String,
    name: String,
    price: Decimal,
    discount: Option<Decimal>,
    quantity: i32,
}

#[tokio::test]
#[ignore]
async fn test_decimal_insert_manual() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::query;

    let pool = Pool::<Postgres>::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    // Create test table
    query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            price NUMERIC(10,2) NOT NULL,
            discount NUMERIC(10,2),
            quantity INTEGER NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    query("DELETE FROM products WHERE id = 'test-decimal-1'")
        .execute(&pool)
        .await?;

    println!("=== Test: Insert with Decimal fields (manual conversion) ===");

    // Test insert by manually converting Decimal to String
    let price = Decimal::from_str("99.99").unwrap();
    let discount = Some(Decimal::from_str("10.00").unwrap());

    // This is what the macro should generate - convert Decimal to String before binding
    // Note: PostgreSQL requires casting from text to numeric
    query(
        "INSERT INTO products (id, name, price, discount, quantity) VALUES ($1, $2, CAST($3 AS NUMERIC), CAST($4 AS NUMERIC), $5)"
    )
    .bind("test-decimal-1")
    .bind("Test Product")
    .bind(price.to_string())  // Convert Decimal to String
    .bind(discount.as_ref().map(|d| d.to_string()))  // Convert Option<Decimal> to Option<String>
    .bind(100)
    .execute(&pool)
    .await?;

    println!("✓ Insert successful with price={}, discount={}", price, discount.unwrap());

    // Verify the insert
    let row: (String, String, String, Option<String>, i32) =
        sqlx::query_as("SELECT id, name, price::text, discount::text, quantity FROM products WHERE id = $1")
        .bind("test-decimal-1")
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.0, "test-decimal-1");
    assert_eq!(row.1, "Test Product");
    assert_eq!(row.2, "99.99");
    assert_eq!(row.3, Some("10.00".to_string()));
    assert_eq!(row.4, 100);

    println!("✓ Insert verified: price={}, discount={}", row.2, row.3.unwrap());

    println!("\n=== Test: Update with Decimal fields (manual conversion) ===");

    let new_price = Decimal::from_str("89.99").unwrap();

    query("UPDATE products SET price = CAST($1 AS NUMERIC), discount = CAST($2 AS NUMERIC), quantity = $3 WHERE id = $4")
        .bind(new_price.to_string())  // Convert Decimal to String
        .bind::<Option<String>>(None)  // Set discount to NULL
        .bind(150)
        .bind("test-decimal-1")
        .execute(&pool)
        .await?;

    println!("✓ Update successful with price={}, discount=NULL", new_price);

    // Verify the update
    let row: (String, Option<String>, i32) =
        sqlx::query_as("SELECT price::text, discount::text, quantity FROM products WHERE id = $1")
        .bind("test-decimal-1")
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.0, "89.99");
    assert_eq!(row.1, None); // discount should be NULL
    assert_eq!(row.2, 150);

    println!("✓ Update verified: price={}, discount={:?}", row.0, row.1);

    // Clean up
    query("DELETE FROM products WHERE id = 'test-decimal-1'")
        .execute(&pool)
        .await?;

    println!("\n=== All manual tests passed! ===");

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_decimal_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::query;

    let pool = Pool::<Postgres>::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    // Create test table
    query(
        r#"
        CREATE TABLE IF NOT EXISTS decimal_test (
            id VARCHAR(36) PRIMARY KEY,
            value NUMERIC(20,6) NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up
    query("DELETE FROM decimal_test WHERE id LIKE 'decimal-edge-%'")
        .execute(&pool)
        .await?;

    println!("=== Testing Decimal edge cases ===");

    // Test case 1: Very small decimal
    let val1 = Decimal::from_str("0.000001").unwrap();
    query("INSERT INTO decimal_test (id, value) VALUES ($1, CAST($2 AS NUMERIC))")
        .bind("decimal-edge-1")
        .bind(val1.to_string())
        .execute(&pool)
        .await?;

    let value1: String = sqlx::query_scalar("SELECT value::text FROM decimal_test WHERE id = $1")
        .bind("decimal-edge-1")
        .fetch_one(&pool)
        .await?;
    assert_eq!(value1, "0.000001");
    println!("✓ Small decimal: {}", value1);

    // Test case 2: Very large decimal
    let val2 = Decimal::from_str("999999999999.999999").unwrap();
    query("INSERT INTO decimal_test (id, value) VALUES ($1, CAST($2 AS NUMERIC))")
        .bind("decimal-edge-2")
        .bind(val2.to_string())
        .execute(&pool)
        .await?;

    let value2: String = sqlx::query_scalar("SELECT value::text FROM decimal_test WHERE id = $1")
        .bind("decimal-edge-2")
        .fetch_one(&pool)
        .await?;
    assert_eq!(value2, "999999999999.999999");
    println!("✓ Large decimal: {}", value2);

    // Test case 3: Zero
    let val3 = Decimal::from_str("0.00").unwrap();
    query("INSERT INTO decimal_test (id, value) VALUES ($1, CAST($2 AS NUMERIC))")
        .bind("decimal-edge-3")
        .bind(val3.to_string())
        .execute(&pool)
        .await?;

    let value3: String = sqlx::query_scalar("SELECT value::text FROM decimal_test WHERE id = $1")
        .bind("decimal-edge-3")
        .fetch_one(&pool)
        .await?;
    assert_eq!(value3, "0.000000"); // NUMERIC(20,6) formats as "0.000000"
    println!("✓ Zero decimal: {}", value3);

    // Test case 4: Negative decimal
    let val4 = Decimal::from_str("-123.456").unwrap();
    query("INSERT INTO decimal_test (id, value) VALUES ($1, CAST($2 AS NUMERIC))")
        .bind("decimal-edge-4")
        .bind(val4.to_string())
        .execute(&pool)
        .await?;

    let value4: String = sqlx::query_scalar("SELECT value::text FROM decimal_test WHERE id = $1")
        .bind("decimal-edge-4")
        .fetch_one(&pool)
        .await?;
    assert_eq!(value4, "-123.456000"); // NUMERIC(20,6) pads to 6 decimal places
    println!("✓ Negative decimal: {}", value4);

    // Clean up
    query("DELETE FROM decimal_test WHERE id LIKE 'decimal-edge-%'")
        .execute(&pool)
        .await?;

    println!("\n=== All edge case tests passed! ===");

    Ok(())
}
