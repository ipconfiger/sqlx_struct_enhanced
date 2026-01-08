// Simple examples demonstrating extended BindProxy data types
//
// Run with: cargo run --example extended_types_simple --features "postgres,all-types"

#[cfg(all(feature = "postgres", feature = "all-types"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Extended BindProxy Data Types - Simple Examples");
    println!("==================================================\n");

    // This example demonstrates the type conversions available with bind_proxy
    // Note: This is a demonstration file - see integration tests for working examples

    println!("✅ Supported Types with bind_proxy:\n");

    println!("1. Additional Numeric Types (No Feature Required):");
    println!("   - i8, i16, i32, i64 (direct binding, zero overhead)");
    println!("   - f32, f64 (direct binding, zero overhead)");
    println!("   - u8, u16, u32, u64 (convert to String)\n");

    println!("2. Binary Types (No Feature Required):");
    println!("   - Vec<u8>, &[u8] (direct binding, zero overhead)\n");

    println!("3. Chrono Date/Time Types (Feature: chrono):");
    println!("   - chrono::NaiveDate → ISO 8601 date string");
    println!("   - chrono::NaiveTime → ISO 8601 time string");
    println!("   - chrono::NaiveDateTime → ISO 8601 datetime string");
    println!("   - chrono::DateTime<Utc> → ISO 8601 with timezone\n");

    println!("4. UUID Type (Feature: uuid):");
    println!("   - uuid::Uuid → UUID string format\n");

    println!("5. JSON Type (Feature: json):");
    println!("   - serde_json::Value → JSON string\n");

    println!("💡 Usage Example:");
    println!("```rust");
    println!("use sqlx_struct_enhanced::EnhancedCrudExt;");
    println!("use chrono::NaiveDate;");
    println!("");
    println!("// Query with date type");
    println!("let users = User::where_query(\"created_at >= {{}}\")");
    println!("    .bind_proxy(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())");
    println!("    .fetch_all(&pool)");
    println!("    .await?;");
    println!("```\n");

    println!("📖 For complete documentation, see USAGE.md");
    println!("🧪 For working examples, see tests/extended_types_integration_test.rs");

    Ok(())
}

#[cfg(not(all(feature = "postgres", feature = "all-types")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 'postgres' and 'all-types' features");
    println!("\nRun with:");
    println!("  cargo run --example extended_types_simple --features 'postgres,all-types'");
    Ok(())
}
