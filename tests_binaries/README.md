# Integration Test Binaries

This directory contains standalone binary programs for integration testing, designed to avoid Cargo feature resolution issues with derive macros in workspace test files.

## Background

When using sqlx's `#[derive(FromRow)]` macro in a workspace context with feature-gated database backends, we encountered issues where the derive macro wouldn't generate the correct `FromRow<'r, MySqlRow>` implementation, even when the `mysql` feature was enabled.

**Solution**: Create standalone binary crates with independent workspace configurations to ensure proper feature propagation.

## Structure

```
tests_binaries/
├── Cargo.toml                      # Standalone workspace configuration
├── mysql_test.rs                   # MySQL integration tests
└── target/                         # Build artifacts
```

## MySQL Integration Tests

### Prerequisites

Start MySQL using Docker Compose:

```bash
cd /Users/alex/Projects/workspace/sqlx_struct_enhanced
docker compose up -d mysql
```

### Running Tests

```bash
cd tests_binaries
cargo run --bin mysql_integration_test
```

### Test Coverage

The MySQL integration test suite includes 7 test scenarios:

1. **Numeric Types** (`test_mysql_extended_types_insert_select_numeric`)
   - Tests i8, i16, f32, u8-u64 types
   - Validates automatic type conversion for unsigned integers (to String)

2. **Chrono Date/Time Types** (`test_mysql_extended_types_chrono_datetime`)
   - Tests NaiveDate, NaiveTime, NaiveDateTime, DateTime<Utc>
   - Validates ISO 8601 string formatting

3. **Binary Types** (`test_mysql_extended_types_binary`)
   - Tests Vec<u8> for binary data storage
   - Validates data integrity after round-trip

4. **UUID Types** (`test_mysql_extended_types_uuid`)
   - Tests uuid::Uuid conversion to String
   - Validates UUID format preservation

5. **JSON Types** (`test_mysql_extended_types_json`)
   - Tests serde_json::Value serialization
   - Validates JSON string storage

6. **Complex WHERE Queries** (`test_mysql_extended_types_complex_where`)
   - Tests multiple bind_proxy calls with different types
   - Validates BETWEEN, >=, > operators

7. **Unsigned Integers in WHERE** (`test_mysql_extended_types_unsigned_where`)
   - Tests u8, u16, u32 binding in WHERE clauses
   - Validates automatic String conversion

### Expected Output

```
🚀 MySQL Integration Tests - Binary Program
==========================================

🔧 Connecting to MySQL: mysql://root:test@127.0.0.1:3306/test_sqlx
✅ Connected to MySQL
✅ Table 'extended_types_test' created

🔧 Starting test_mysql_extended_types_insert_select_numeric...
✅ Inserted record
✅ Numeric types test passed

🔧 Starting test_mysql_extended_types_chrono_datetime...
✅ Chrono date/time types test passed

🔧 Starting test_mysql_extended_types_binary...
✅ Binary types test passed

🔧 Starting test_mysql_extended_types_uuid...
✅ UUID types test passed

🔧 Starting test_mysql_extended_types_json...
✅ JSON types test passed

🔧 Starting test_mysql_extended_types_complex_where...
✅ Complex WHERE query test passed

🔧 Starting test_mysql_extended_types_unsigned_where...
✅ Unsigned integers WHERE clause test passed

==========================================
✅ All MySQL integration tests passed!
==========================================
```

## Database Configuration

The test program uses the following default MySQL connection:

```rust
mysql://root:test@127.0.0.1:3306/test_sqlx
```

You can override this using the `MYSQL_DATABASE_URL` environment variable:

```bash
export MYSQL_DATABASE_URL="mysql://user:pass@host:port/database"
cargo run --bin mysql_integration_test
```

## Key Implementation Details

### 1. Independent Workspace

The binary crate has its own `[workspace]` section to avoid inheriting the parent workspace's feature configuration:

```toml
[workspace]
# This creates a standalone workspace to avoid inheriting parent workspace features
```

### 2. Explicit Feature Configuration

All features are explicitly enabled in the binary's Cargo.toml:

```toml
[dependencies]
sqlx = { version = "0.7.3", default-features = false,
         features = ["runtime-tokio-rustls", "json", "mysql", "macros"] }
sqlx_struct_enhanced = { path = "..", default-features = false,
                         features = ["mysql", "all-types"] }
```

### 3. Data Cleanup

Each test cleans up all data before execution to avoid test interference:

```rust
sqlx::query("DELETE FROM extended_types_test")
    .execute(pool)
    .await
    .expect("Failed to clean up test data");
```

### 4. MySQL-Specific Syntax

Tests use MySQL's `?` placeholder syntax instead of PostgreSQL's `$1, $2`:

```rust
let results = ExtendedTypesTest::where_query_ext("tiny_int >= ? AND small_int > ?")
    .bind_proxy(3i16)
    .bind_proxy(1002i16)
    .fetch_all(pool)
    .await?;
```

## Troubleshooting

### Connection Errors

If you see "Failed to connect to MySQL test database", ensure:

1. MySQL container is running: `docker compose ps`
2. Container is healthy: `docker compose logs mysql`
3. Port 3306 is accessible: `telnet 127.0.0.1 3306`

### Compilation Errors

If you see trait bound errors like `FromRow<'r, MySqlRow> is not satisfied`:

1. Ensure you're in the `tests_binaries` directory
2. Ensure you're using the standalone binary, not workspace tests
3. Clean and rebuild: `cargo clean && cargo build`

### Test Failures

If tests fail with assertion errors:

1. Check if previous test data is interfering (cleanup should handle this)
2. Verify table schema: look for "✅ Table 'extended_types_test' created"
3. Run with `RUST_BACKTRACE=1` for detailed stack traces

## Future Extensions

This pattern can be extended for other database backends:

```
tests_binaries/
├── mysql_test.rs       # ✅ Complete
├── sqlite_test.rs      # TODO
└── postgres_test.rs    # Optional (workspace tests already work)
```

## References

- Main project documentation: `/Users/alex/Projects/workspace/sqlx_struct_enhanced/README.md`
- Usage guide: `/Users/alex/Projects/workspace/sqlx_struct_enhanced/USAGE.md`
- Issue analysis: `/Users/alex/Projects/workspace/sqlx_struct_enhanced/MySQL_INTEGRATION_TEST_ISSUE.md`
