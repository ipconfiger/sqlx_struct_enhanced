// Real-world scenario examples for extended BindProxy data types
//
// Run with: cargo run --example extended_types_real_world --features "postgres,all-types"

#[cfg(all(feature = "postgres", feature = "all-types"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏪 Real-World Scenario: E-commerce Order Management");
    println!("===================================================\n");

    println!("This document outlines realistic use cases for extended BindProxy types\n");

    println!("📋 Scenario 1: Customer Registration");
    println!("--------------------------------------");
    println!("Use chrono::NaiveDate for date of birth");
    println!("Use chrono::DateTime<Utc> for registration timestamp");
    println!("Use uuid::Uuid for customer IDs");
    println!("Use serde_json::Value for customer preferences metadata\n");

    println!("📦 Scenario 2: Product Catalog");
    println!("------------------------------");
    println!("Use i16 for small stock counts (direct binding, zero overhead)");
    println!("Use f32 for ratings (direct binding, zero overhead)");
    println!("Use u8/u16 for category codes (converts to String)");
    println!("Use chrono::NaiveDate for release dates\n");

    println!("🛒 Scenario 3: Order Processing");
    println!("------------------------------");
    println!("Use chrono::NaiveDateTime for order dates");
    println!("Use rust_decimal::Decimal for prices (with decimal feature)");
    println!("Use uuid::Uuid for order and customer IDs");
    println!("Use serde_json::Value for shipping address and metadata\n");

    println!("🔍 Scenario 4: Product Search");
    println!("----------------------------");
    println!("// Find products released after a certain date");
    println!("use sqlx_struct_enhanced::EnhancedCrudExt;");
    println!("use chrono::NaiveDate;");
    println!("");
    println!("let products = Product::where_query(\"release_date >= {{}}\")");
    println!("    .bind_proxy(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())");
    println!("    .fetch_all(&pool)");
    println!("    .await?;\n");

    println!("// Find products in price range with minimum rating");
    println!("let products = Product::where_query(\"price BETWEEN {{}} AND {{}} AND rating >= {{}}\")");
    println!("    .bind_proxy(100.0f32)");
    println!("    .bind_proxy(500.0f32)");
    println!("    .bind_proxy(4.5f32)");
    println!("    .fetch_all(&pool)");
    println!("    .await?;\n");

    println!("💾 Scenario 5: Data Storage Patterns");
    println!("-----------------------------------");
    println!("Binary Data: Vec<u8> for thumbnails, images, documents");
    println!("JSON Metadata: User preferences, settings, tags, categories");
    println!("UUID Fields: Primary keys, foreign keys, references");
    println!("Date/Time: Created_at, updated_at, birth_date, expiration\n");

    println!("⚡ Performance Tips:");
    println!("-------------------");
    println!("1. Use signed integers (i8, i16) instead of unsigned when possible");
    println!("2. Create indexes on frequently queried date/time columns");
    println!("3. Use f32 instead of f64 when precision isn't critical");
    println!("4. Cache JSON/serialized strings when reused");
    println!("5. Leverage automatic SQL statement caching\n");

    println!("📖 For working code examples, see:");
    println!("   - tests/extended_types_integration_test.rs");
    println!("   - USAGE.md (Supported Data Types section)");
    println!("   - README.md (Extended Data Types Support section)");

    Ok(())
}

#[cfg(not(all(feature = "postgres", feature = "all-types")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 'postgres' and 'all-types' features");
    println!("\nRun with:");
    println!("  cargo run --example extended_types_real_world --features 'postgres,all-types'");
    Ok(())
}
