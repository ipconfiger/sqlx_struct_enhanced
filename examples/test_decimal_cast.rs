// Test DECIMAL field casting with ::numeric in INSERT/UPDATE and cast_as in SELECT
use sqlx::FromRow;
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx::{Execute, Row};
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct TestProduct {
    #[crud(decimal(precision = 10, scale = 2), cast_as = "TEXT")]
    pub price: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    pub discount: Option<String>,

    pub name: String,
    pub id: String,
}

fn main() {
    // This test verifies that DECIMAL fields get ::numeric cast in INSERT/UPDATE SQL
    println!("=== Testing DECIMAL field casting ===\n");

    // Create test instance
    let mut product = TestProduct {
        id: "test-id".to_string(),
        name: "Test Product".to_string(),
        price: Some("99.99".to_string()),
        discount: Some("12.50".to_string()),
    };

    // Generate SQL by calling the methods
    println!("1. INSERT SQL:");
    let insert_sql = product.insert_bind();
    println!("   {}\n", insert_sql.sql());

    println!("2. UPDATE SQL:");
    let mut product2 = product.clone();
    let update_sql = product2.update_bind();
    println!("   {}\n", update_sql.sql());

    println!("3. BULK INSERT SQL:");
    let products = vec![product.clone()];
    let bulk_sql = TestProduct::bulk_insert(&products);
    println!("   {}\n", bulk_sql.sql());

    println!("4. SELECT SQL:");
    let select_query = TestProduct::by_pk();
    println!("   {}\n", select_query.sql());

    println!("=== Expected behavior ===");
    println!("- INSERT/UPDATE: should have ::numeric cast for 'price' and 'discount'");
    println!("  Example: VALUES ($1, $2::numeric, $3::numeric, $4)");
    println!("- SELECT: should have ::TEXT as 'price' (but NOT 'discount' which has no cast_as)");
    println!("  Example: SELECT \"id\", \"name\", \"price\"::TEXT as \"price\", \"discount\" FROM...");
}
