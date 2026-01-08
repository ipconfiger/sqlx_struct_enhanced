// Simple examples demonstrating extended BindProxy data types
//
// This example shows basic usage of all new data types supported by BindProxy:
// - Additional numeric types (i8, i16, u8, u16, u32, u64, f32)
// - Binary types (Vec<u8>, &[u8])
// - Chrono date/time types (NaiveDate, NaiveTime, NaiveDateTime, DateTime<Utc>)
// - UUID types
// - JSON types

#[cfg(all(feature = "postgres", feature = "all-types"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::{FromRow, PgPool};
    use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
    use chrono::{NaiveDate, Utc, TimeZone};

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

    let pool = PgPool::connect(&database_url).await?;

    // Create test table
    sqlx::query("DROP TABLE IF EXISTS products")
        .execute(&pool)
        .await?;

    sqlx::query(r#"
        CREATE TABLE products (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            quantity SMALLINT,
            rating REAL,
            price TEXT,
            release_date TEXT,
            created_at TEXT,
            metadata TEXT
        )
    "#).execute(&pool).await?;

    // ============================================================================
    // Example 1: Numeric Types
    // ============================================================================

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "products"]
    struct Product {
        pub id: String,
        pub name: String,
        pub quantity: Option<i16>,  // i8 or i16
        pub rating: Option<f32>,     // f32
        pub price: Option<String>,   // Could use rust_decimal::Decimal
        pub release_date: Option<String>,
        pub created_at: Option<String>,
        pub metadata: Option<String>,
    }

    println!("📦 Example 1: Insert product with numeric types");

    let mut product = Product {
        id: "prod-1".to_string(),
        name: "Laptop".to_string(),
        quantity: Some(50i16),           // Small integer
        rating: Some(4.7f32),             // Single-precision float
        price: Some("1299.99".to_string()),
        release_date: None,
        created_at: None,
        metadata: None,
    };

    product.insert_bind(&pool).await?;
    println!("✅ Inserted product: {} (quantity: {}, rating: {})",
        product.name, product.quantity.unwrap(), product.rating.unwrap());

    // ============================================================================
    // Example 2: Unsigned Integers (convert to String)
    // ============================================================================

    println!("\n📊 Example 2: Query with unsigned integers");

    // Query using bind_proxy with unsigned integers
    // u8, u16, u32, u64 automatically convert to String
    let products = Product::where_query("quantity > {} AND rating >= {}")
        .bind_proxy(10u8)      // u8 → String "10"
        .bind_proxy(4.0f32)    // f32 direct binding
        .fetch_all(&pool)
        .await?;

    println!("✅ Found {} products with quantity > 10 and rating >= 4.0", products.len());

    // ============================================================================
    // Example 3: Chrono Date/Time Types
    // ============================================================================

    println!("\n📅 Example 3: Insert with chrono date/time types");

    let release_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let created_at = Utc.with_ymd_and_hms(2024, 1, 10, 14, 30, 0).unwrap();

    let mut product2 = Product {
        id: "prod-2".to_string(),
        name: "Smartphone".to_string(),
        quantity: Some(100i16),
        rating: Some(4.5f32),
        price: Some("799.99".to_string()),
        release_date: Some(release_date.format("%Y-%m-%d").to_string()),
        created_at: Some(created_at.format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string()),
        metadata: None,
    };

    product2.insert_bind(&pool).await?;
    println!("✅ Inserted product with release_date: {} and created_at: {}",
        product2.release_date.unwrap(), product2.created_at.unwrap());

    // Query using bind_proxy with NaiveDate
    let search_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let products = Product::where_query("release_date >= {}")
        .bind_proxy(search_date)  // NaiveDate → String "2024-01-01"
        .fetch_all(&pool)
        .await?;

    println!("✅ Found {} products released on or after 2024-01-01", products.len());

    // ============================================================================
    // Example 4: Binary Types
    // ============================================================================

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "products"]
    struct ProductWithImage {
        pub id: String,
        pub name: String,
        pub quantity: Option<i16>,
        pub rating: Option<f32>,
        pub price: Option<String>,
        pub release_date: Option<String>,
        pub created_at: Option<String>,
        pub metadata: Option<String>,
    }

    // Binary data example (thumbnail image)
    let thumbnail_data: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];

    println!("\n🖼️  Example 4: Working with binary data");
    println!("✅ Binary data size: {} bytes", thumbnail_data.len());
    println!("   Note: Binary data binds as Vec<u8> directly");

    // ============================================================================
    // Example 5: UUID Types
    // ============================================================================

    use uuid::Uuid;

    println!("\n🔑 Example 5: Using UUID types");

    let product_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();

    println!("✅ Generated UUID: {}", product_id);
    println!("   UUIDs automatically convert to String for database storage");

    // Query using bind_proxy with UUID
    // let product = Product::where_query("id = {}")
    //     .bind_proxy(product_id)  // Uuid → String
    //     .fetch_one(&pool)
    //     .await?;

    // ============================================================================
    // Example 6: JSON Types
    // ============================================================================

    use serde_json::json;

    println!("\n📝 Example 6: Using JSON types");

    let metadata = json!({
        "brand": "TechCorp",
        "warranty_months": 24,
        "features": ["waterproof", "wireless", "fast-charging"]
    });

    let mut product3 = Product {
        id: "prod-3".to_string(),
        name: "Smart Watch".to_string(),
        quantity: Some(75i16),
        rating: Some(4.8f32),
        price: Some("299.99".to_string()),
        release_date: None,
        created_at: None,
        metadata: Some(metadata.to_string()),
    };

    product3.insert_bind(&pool).await?;
    println!("✅ Inserted product with JSON metadata");
    println!("   Metadata: {}", product3.metadata.unwrap());

    // Query with JSON in WHERE clause
    let products = Product::where_query("metadata LIKE {}")
        .bind_proxy("%waterproof%")  // String matching
        .fetch_all(&pool)
        .await?;

    println!("✅ Found {} products with 'waterproof' feature", products.len());

    // ============================================================================
    // Example 7: Combining Multiple Types
    // ============================================================================

    println!("\n🔧 Example 7: Complex query with multiple type conversions");

    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let min_rating = 4.0f32;
    let max_price = "1000.00";  // Could use Decimal

    let products = Product::where_query("release_date >= {} AND rating >= {} AND price <= {}")
        .bind_proxy(start_date)      // NaiveDate → String
        .bind_proxy(min_rating)      // f32 direct binding
        .bind_proxy(max_price)       // String
        .fetch_all(&pool)
        .await?;

    println!("✅ Complex query found {} products", products.len());

    for product in products {
        println!("   - {} (rating: {}, price: {})",
            product.name,
            product.rating.unwrap_or(0.0),
            product.price.unwrap_or_else(|| "N/A".to_string())
        );
    }

    // ============================================================================
    // Cleanup
    // ============================================================================

    sqlx::query("DROP TABLE products").execute(&pool).await?;

    println!("\n✅ All examples completed successfully!");

    Ok(())
}

// ============================================================================
// Feature-Gated Main for Other Databases
// ============================================================================

#[cfg(all(feature = "mysql", feature = "all-types", not(feature = "postgres")))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MySQL example - similar to PostgreSQL example");
    println!("Run with: cargo run --example extended_types_simple --features 'mysql,all-types'");
    Ok(())
}

#[cfg(all(feature = "sqlite", feature = "all-types", not(feature = "postgres"), not(feature = "mysql")))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SQLite example - similar to PostgreSQL example");
    println!("Run with: cargo run --example extended_types_simple --features 'sqlite,all-types'");
    Ok(())
}

#[cfg(not(feature = "all-types"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 'all-types' feature");
    println!("Run with: cargo run --example extended_types_simple --features 'postgres,all-types'");
    Ok(())
}
