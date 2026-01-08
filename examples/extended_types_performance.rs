// Performance optimization examples for extended BindProxy types
//
// This example demonstrates performance best practices and optimization techniques
// when working with extended data types in sqlx_struct_enhanced.

#[cfg(all(feature = "postgres", feature = "all-types"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::{FromRow, PgPool};
    use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
    use chrono::{NaiveDate, NaiveDateTime, Utc, TimeZone};
    use std::time::Instant;

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

    let pool = PgPool::connect(&database_url).await?;

    println!("⚡ Extended Types Performance Optimization Guide");
    println!("================================================\n");

    // Setup: Create test table with indexes
    setup_performance_test_table(&pool).await?;

    // ============================================================================
    // Performance Tip 1: Use Direct Binding When Possible
    // ============================================================================

    println!("💡 Tip 1: Use Direct Binding for Native Types");
    println!("---------------------------------------------");

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "performance_test"]
    struct PerformanceRecord {
        pub id: String,
        pub user_id: i32,
        pub score: i16,
        pub rating: f32,
        pub timestamp: String,
    }

    // Fast: Native types (i8, i16, f32) bind directly without conversion
    let start = Instant::now();
    for i in 0..1000 {
        let mut record = PerformanceRecord {
            id: format!("perf-{}", i),
            user_id: i,
            score: (i % 100) as i16,
            rating: 4.5f32,
            timestamp: Utc.now().format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string(),
        };
        record.insert_bind(&pool).await?;
    }
    let duration = start.elapsed();

    println!("✅ Inserted 1000 records with native types: {:?}", duration);
    println!("   Native types (i16, f32) have ZERO conversion overhead");

    // ============================================================================
    // Performance Tip 2: Batch Operations with Prepared Statements
    // ============================================================================

    println!("\n💡 Tip 2: Batch Operations for Better Throughput");
    println!("------------------------------------------------");

    // Clean table
    sqlx::query("TRUNCATE TABLE performance_test").execute(&pool).await?;

    // Batch insert: Use transactions for better performance
    let mut tx = pool.begin().await?;

    let start = Instant::now();
    for i in 0..1000 {
        let mut record = PerformanceRecord {
            id: format!("batch-{}", i),
            user_id: i,
            score: (i % 100) as i16,
            rating: 4.5f32,
            timestamp: Utc.now().format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string(),
        };
        record.insert_bind(&mut tx).await?;
    }
    tx.commit().await?;

    let batch_duration = start.elapsed();
    println!("✅ Batch insert (transaction): {:?}", batch_duration);
    println!("   Transactions reduce round-trips and improve throughput");

    // ============================================================================
    // Performance Tip 3: Efficient Date Queries with Indexes
    // ============================================================================

    println!("\n💡 Tip 3: Create Indexes for Date/Time Columns");
    println!("----------------------------------------------");

    // Create index on timestamp column
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_timestamp ON performance_test (timestamp)")
        .execute(&pool)
        .await?;

    println!("✅ Created index on timestamp column");

    // Query with date range
    let search_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let start = Instant::now();
    let results = PerformanceRecord::where_query("timestamp >= {}")
        .bind_proxy(search_date)
        .fetch_all(&pool)
        .await?;
    let query_duration = start.elapsed();

    println!("✅ Date range query: {:?} (found {} records)", query_duration, results.len());
    println!("   Indexes significantly speed up date/time queries");

    // ============================================================================
    // Performance Tip 4: String Conversion Cost Analysis
    // ============================================================================

    println!("\n💡 Tip 4: Understand String Conversion Costs");
    println!("--------------------------------------------");

    // Unsigned integers convert to String (has overhead)
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = 255u8.to_string();  // u8 → String conversion
        let _ = 65535u16.to_string(); // u16 → String conversion
    }
    let conversion_time = start.elapsed();

    println!("✅ 10,000 unsigned integer → String conversions: {:?}", conversion_time);
    println!("   Cost: ~{} ns per conversion", conversion_time.as_nanos() / 10000);
    println!("   💡 Mitigation: Use signed integers (i8, i16, i32) when possible");

    // ============================================================================
    // Performance Tip 5: Binary Data Efficiency
    // ============================================================================

    println!("\n💡 Tip 5: Binary Data (Vec<u8>) is Zero-Copy");
    println!("--------------------------------------------");

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "performance_test"]
    struct RecordWithBinary {
        pub id: String,
        pub user_id: i32,
        pub score: i16,
        pub rating: f32,
        pub timestamp: String,
        pub binary_data: Option<Vec<u8>>,
    }

    // Binary data binds directly as Vec<u8> (no conversion)
    let binary_data: Vec<u8> = vec![0u8; 1024]; // 1KB of data

    let start = Instant::now();
    let mut record = RecordWithBinary {
        id: "binary-test-1".to_string(),
        user_id: 1,
        score: 100,
        rating: 5.0f32,
        timestamp: Utc.now().format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string(),
        binary_data: Some(binary_data),
    };
    record.insert_bind(&pool).await?;
    let insert_duration = start.elapsed();

    println!("✅ Inserted 1KB binary data: {:?}", insert_duration);
    println!("   Vec<u8> binding has ZERO overhead (direct pass-through)");

    // ============================================================================
    // Performance Tip 6: JSON Serialization Considerations
    // ============================================================================

    println!("\n💡 Tip 6: JSON Serialization Performance");
    println!("----------------------------------------");

    use serde_json::json;

    let complex_json = json!({
        "user": {
            "id": 12345,
            "name": "Performance Test User",
            "email": "test@example.com"
        },
        "metadata": {
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-15T12:30:45Z",
            "tags": ["performance", "benchmark", "optimization"]
        },
        "settings": {
            "notifications": true,
            "theme": "dark",
            "language": "en"
        }
    });

    let start = Instant::now();
    let json_string = complex_json.to_string();
    let serialization_time = start.elapsed();

    println!("✅ JSON serialization time: {:?}", serialization_time);
    println!("   💡 Tip: Cache JSON strings when reusing metadata");

    // ============================================================================
    // Performance Tip 7: Query Optimization with Type Selection
    // ============================================================================

    println!("\n💡 Tip 7: Choose Appropriate Types for Your Use Case");
    println!("----------------------------------------------------");

    // Type size comparison
    println!("Type Memory Usage:");
    println!("   i8, u8:    1 byte");
    println!("   i16, u16:  2 bytes");
    println!("   i32, u32:  4 bytes");
    println!("   i64, u64:  8 bytes");
    println!("   f32:       4 bytes");
    println!("   f64:       8 bytes");
    println!("   String:    Variable (heap allocation)");

    println!("\n💡 Best Practices:");
    println!("   • Use smallest sufficient integer type (i8 vs i32)");
    println!("   • Use f32 instead of f64 when precision isn't critical");
    println!("   • Avoid unnecessary String conversions");
    println!("   • Prefer signed integers over unsigned (avoids conversion)");

    // ============================================================================
    // Performance Tip 8: Connection Pooling
    // ============================================================================

    println!("\n💡 Tip 8: Leverage Connection Pooling");
    println!("--------------------------------------");

    println!("Current pool size: {}", pool.size());
    println!("Idle connections: {}", pool.num_idle());
    println!("💡 Tip: Set appropriate pool size based on workload");

    // ============================================================================
    // Performance Tip 9: Prepared Statement Caching
    // ============================================================================

    println!("\n💡 Tip 9: SQL Query Caching is Automatic");
    println!("----------------------------------------");

    // Repeated queries benefit from SQL statement caching
    let user_id = 42i32;

    let start = Instant::now();
    for _ in 0..100 {
        let _ = PerformanceRecord::where_query("user_id = {}")
            .bind_proxy(user_id)
            .fetch_optional(&pool)
            .await?;
    }
    let repeated_query_time = start.elapsed();

    println!("✅ 100 repeated queries: {:?}", repeated_query_time);
    println!("   Average per query: {:?}", repeated_query_time / 100);
    println!("   💡 SQLx caches prepared statements automatically");

    // ============================================================================
    // Performance Tip 10: Benchmark Summary
    // ============================================================================

    println!("\n📊 Performance Summary");
    println!("======================");

    println!("\nDirect Binding Types (Fastest - Zero Overhead):");
    println!("   ✅ i8, i16, i32, i64 (signed integers)");
    println!("   ✅ f32, f64 (floating-point)");
    println!("   ✅ bool (boolean)");
    println!("   ✅ Vec<u8>, &[u8] (binary data)");
    println!("   ✅ String (already a string)");

    println!("\nString Conversion Types (Fast - Minor Overhead):");
    println!("   ⚠️  u8, u16, u32, u64 (unsigned integers → String)");
    println!("   ⚠️  rust_decimal::Decimal → String");
    println!("   ⚠️  chrono::NaiveDate → String");
    println!("   ⚠️  chrono::NaiveTime → String");
    println!("   ⚠️  chrono::NaiveDateTime → String");
    println!("   ⚠️  chrono::DateTime<Utc> → String");
    println!("   ⚠️  uuid::Uuid → String");
    println!("   ⚠️  serde_json::Value → JSON String");

    println!("\nOptimization Recommendations:");
    println!("   1. Use signed integers (i8, i16, i32) instead of unsigned when possible");
    println!("   2. Create indexes on frequently queried date/time columns");
    println!("   3. Use transactions for bulk operations");
    println!("   4. Cache JSON/serialized strings when reused");
    println!("   5. Choose smallest sufficient type (i8 vs i32, f32 vs f64)");
    println!("   6. Leverage automatic SQL statement caching");
    println!("   7. Use connection pooling appropriately");
    println!("   8. Consider prepared statements for repeated queries");

    // ============================================================================
    // Cleanup
    // ============================================================================

    sqlx::query("DROP TABLE performance_test CASCADE").execute(&pool).await?;
    println!("\n✅ Performance test complete!");

    Ok(())
}

#[cfg(all(feature = "postgres", feature = "all-types"))]
async fn setup_performance_test_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Drop table if exists
    sqlx::query("DROP TABLE IF EXISTS performance_test CASCADE")
        .execute(pool)
        .await?;

    // Create table with various column types
    sqlx::query(r#"
        CREATE TABLE performance_test (
            id VARCHAR(36) PRIMARY KEY,
            user_id INTEGER NOT NULL,
            score SMALLINT,
            rating REAL,
            timestamp TEXT NOT NULL,
            binary_data BYTEA
        )
    "#).execute(pool).await?;

    // Create indexes for performance testing
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_user_id ON performance_test (user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_score ON performance_test (score)")
        .execute(pool)
        .await?;

    Ok(())
}

// ============================================================================
// Feature-Gated Main for Other Databases
// ============================================================================

#[cfg(all(feature = "mysql", feature = "all-types", not(feature = "postgres")))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ MySQL Performance Example");
    println!("Run with: cargo run --example extended_types_performance --features 'mysql,all-types'");
    Ok(())
}

#[cfg(all(feature = "sqlite", feature = "all-types", not(feature = "postgres"), not(feature = "mysql")))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ SQLite Performance Example");
    println!("Run with: cargo run --example extended_types_performance --features 'sqlite,all-types'");
    Ok(())
}

#[cfg(not(feature = "all-types"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 'all-types' feature");
    println!("Run with: cargo run --example extended_types_performance --features 'postgres,all-types'");
    Ok(())
}
