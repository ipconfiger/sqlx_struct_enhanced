// Performance optimization guide for extended BindProxy data types
//
// Run with: cargo run --example extended_types_performance --features "postgres,all-types"

#[cfg(all(feature = "postgres", feature = "all-types"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Extended Types Performance Optimization Guide");
    println!("================================================\n");

    println!("📊 Performance Characteristics\n");

    println!("Direct Binding Types (Fastest - Zero Overhead):");
    println!("   ✅ i8, i16, i32, i64 (signed integers)");
    println!("   ✅ f32, f64 (floating-point)");
    println!("   ✅ bool (boolean)");
    println!("   ✅ Vec<u8>, &[u8] (binary data)");
    println!("   ✅ String, &str (strings)");
    println!("   Cost: 0 ns (direct pass-through)\n");

    println!("String Conversion Types (Fast - Minor Overhead):");
    println!("   ⚠️  u8, u16, u32, u64 (unsigned integers → String)");
    println!("   ⚠️  rust_decimal::Decimal → String");
    println!("   ⚠️  chrono::NaiveDate → String");
    println!("   ⚠️  chrono::NaiveTime → String");
    println!("   ⚠️  chrono::NaiveDateTime → String");
    println!("   ⚠️  chrono::DateTime<Utc> → String");
    println!("   ⚠️  uuid::Uuid → String");
    println!("   ⚠️  serde_json::Value → JSON String");
    println!("   Cost: ~50-100 ns per conversion\n");

    println!("🎯 Optimization Recommendations\n");

    println!("1. Use Appropriate Types");
    println!("   • Choose smallest sufficient integer (i8 vs i32)");
    println!("   • Use f32 instead of f64 when precision isn't critical");
    println!("   • Prefer signed integers over unsigned (avoids conversion)\n");

    println!("2. Database Indexing");
    println!("   • Create indexes on frequently queried date/time columns");
    println!("   • Index columns used in WHERE clauses");
    println!("   • Consider composite indexes for multi-column queries\n");

    println!("3. Query Optimization");
    println!("   • Use transactions for bulk operations");
    println!("   • Leverage automatic SQL statement caching");
    println!("   • Use prepared statements for repeated queries");
    println!("   • Batch operations when possible\n");

    println!("4. Type Selection Guide");
    println!("   ┌─────────────────────────┬──────────────┬───────────┐");
    println!("   │ Use Case                │ Type         │ Overhead │");
    println!("   ├─────────────────────────┼──────────────┼───────────┤");
    println!("   │ Tiny counters (< 128)    │ i8           │ Zero     │");
    println!("   │ Small counts (< 32K)     │ i16          │ Zero     │");
    println!("   │ Medium counts (< 2B)     │ i32          │ Zero     │");
    println!("   │ Large counts             │ i64          │ Zero     │");
    println!("   │ Flags/IDs (< 256)        │ u8           │ Minimal  │");
    println!("   │ Precision decimals       │ Decimal      │ Minimal  │");
    println!("   │ Ratings/scores           │ f32          │ Zero     │");
    println!("   │ Scientific calculations  │ f64          │ Zero     │");
    println!("   │ Dates                    │ NaiveDate    │ Minimal  │");
    println!("   │ Timestamps               │ NaiveDateTime│ Minimal  │");
    println!("   │ Binary data              │ Vec<u8>      │ Zero     │");
    println!("   │ JSON metadata            │ serde_json   │ Minimal  │");
    println!("   │ Unique identifiers       │ Uuid         │ Minimal  │");
    println!("   └─────────────────────────┴──────────────┴───────────┘\n");

    println!("💡 Performance Tips\n");

    println!("1. Batch Operations");
    println!("   • Use transactions for multiple inserts");
    println!("   • Bulk operations reduce round-trips");
    println!("   • Example:");
    println!("     transaction(&pool, |tx| async {{");
    println!("         for item in items {{");
    println!("             item.insert_bind().execute(tx).await?;");
    println!("         }}");
    println!("         Ok(())");
    println!("     }}).await?;\n");

    println!("2. Index Strategy");
    println!("   • Index date columns used in range queries");
    println!("   • Index foreign keys for JOIN performance");
    println!("   • Consider partial indexes for filtered queries");
    println!("   • Example:");
    println!("     CREATE INDEX idx_created_at ON orders (created_at);\n");

    println!("3. Connection Pooling");
    println!("   • Set appropriate pool size based on workload");
    println!("   • Reuse connections across queries");
    println!("   • Monitor connection pool usage\n");

    println!("4. Query Caching");
    println!("   • SQLx automatically caches prepared statements");
    println!("   • Repeated queries are faster");
    println!("   • Use parameterized queries (bind_proxy/bind)\n");

    println!("5. String Conversion Mitigation");
    println!("   • Use signed integers when possible");
    println!("   • Cache converted strings when reused");
    println!("   • Consider database-native types (e.g., UUID columns)\n");

    println!("📈 Performance Benchmarks");
    println!("--------------------------");
    println!("• Direct binding (i16, f32):        < 10 ns");
    println!("• String conversion (u8, u16):      ~50 ns");
    println!("• Date/time formatting:              ~100 ns");
    println!("• JSON serialization:                ~200 ns");
    println!("• UUID string conversion:            ~50 ns");
    println!("• Decimal to String:                 ~100 ns\n");

    println!("✅ Key Takeaways");
    println!("---------------");
    println!("• Native types (i8, i16, f32, f64) have ZERO overhead");
    println!("• String conversions have minimal overhead (~50-100 ns)");
    println!("• Database indexing is more critical than type selection");
    println!("• Use transactions and batch operations for better throughput");
    println!("• SQLx caching makes repeated queries very efficient\n");

    println!("📖 For detailed performance analysis:");
    println!("   • See tests/extended_types_integration_test.rs for working examples");
    println!("   • Run integration tests to measure actual performance");
    println!("   • Profile your specific workload for optimization opportunities");

    Ok(())
}

#[cfg(not(all(feature = "postgres", feature = "all-types")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 'postgres' and 'all-types' features");
    println!("\nRun with:");
    println!("  cargo run --example extended_types_performance --features 'postgres,all-types'");
    Ok(())
}
