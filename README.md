# sqlx_struct_enhanced

Auto-generate CRUD SQL operations for SQLx with type-safe query building.

## Features

- ✅ **Auto-generated SQL** for INSERT, UPDATE, DELETE, SELECT
- ✅ **Batch Operations**: Bulk insert, update, and delete for efficient multi-row operations
- ✅ **Transaction Support**: Atomic multi-operation transactions with automatic rollback
- ✅ **Conditional Operations**: WHERE queries, count queries, and conditional deletes
- ✅ **Multiple Database Backends**: PostgreSQL, MySQL, SQLite
- ✅ **DECIMAL/NUMERIC Support** 🆕: Type-safe decimal handling with automatic casting
- ✅ **Extended BindProxy Types** 🆕: Auto-conversion for dates, JSON, binary, and more
- ✅ **Compile-time SQL Generation**: No runtime overhead
- ✅ **Global SQL Caching**: Efficient query reuse
- ✅ **Custom Table Names**: Override default table names
- ✅ **Index Analysis** 🆕: Compile-time query analysis with automatic index recommendations
- ✅ **Type-Safe**: Full type safety with Rust's type system
- ✅ **Zero-Copy**: No unnecessary allocations for cached queries

## Quick Start

> **📖 For comprehensive documentation and API reference, see [USAGE.md](USAGE.md)**

### Installation

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
sqlx_struct_enhanced = { version = "0.1", features = ["postgres"] }
```

### Basic Usage

```rust
use sqlx_struct_enhanced::EnhancedCrud;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct User {
    id: String,
    name: String,
    email: String,
}

// Insert
let mut user = User {
    id: "1".to_string(),
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};
user.insert_bind().execute(&pool).await?;

// Select by ID
let user = User::by_pk().bind("1").fetch_one(&pool).await?;

// Update
user.name = "Bob".to_string();
user.update_bind().execute(&pool).await?;

// Delete single record
user.delete_bind().execute(&pool).await?;

// Bulk insert multiple records
let new_users = vec![
    User { id: "1".to_string(), name: "Alice".to_string(), email: "alice@example.com".to_string() },
    User { id: "2".to_string(), name: "Bob".to_string(), email: "bob@example.com".to_string() },
    User { id: "3".to_string(), name: "Charlie".to_string(), email: "charlie@example.com".to_string() },
];
User::bulk_insert(&new_users).execute(&pool).await?;

// Bulk delete multiple records
let ids_to_delete = vec!["1".to_string(), "2".to_string(), "3".to_string()];
User::bulk_delete(&ids_to_delete).execute(&pool).await?;

// Bulk update multiple records
let users_to_update = vec![
    User { id: "1".to_string(), name: "Alice Updated".to_string(), email: "alice.new@example.com".to_string() },
    User { id: "2".to_string(), name: "Bob Updated".to_string(), email: "bob.new@example.com".to_string() },
];
User::bulk_update(&users_to_update).execute(&pool).await?;

// Custom queries
let users = User::where_query("email LIKE '%@example.com'")
    .fetch_all(&pool).await?;

let (count,) = User::count_query("active = true")
    .fetch_one(&pool).await?;
```

## DECIMAL/NUMERIC Support

For financial data or other use cases requiring exact decimal precision:

```rust
#[derive(EnhancedCrud)]
#[table_name = "products"]
struct Product {
    id: String,
    name: String,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    price: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    discount: Option<String>,

    quantity: i32,
}

// Insert product with decimal prices
let mut product = Product {
    id: "1".to_string(),
    name: "Laptop".to_string(),
    price: Some("1299.99".to_string()),
    discount: Some("15.00".to_string()),
    quantity: 10,
};
product.insert_bind().execute(&pool).await?;

// Select - automatically casts NUMERIC to TEXT
let product = Product::by_pk().bind("1").fetch_one(&pool).await?;
println!("Price: {}", product.price.unwrap()); // "1299.99"
```

**How it works:**
- `#[crud(decimal(precision = N, scale = M))]` - For migration generation (NUMERIC columns)
- `#[crud(cast_as = "TEXT")]` - Adds type casting in SELECT queries
- Generated SQL: `SELECT id, name, price::TEXT as price, ... FROM products`

**Benefits:**
- ✅ Exact decimal precision (no floating-point errors)
- ✅ Type-safe String storage in Rust
- ✅ Automatic type casting in queries

> **📖 For full DECIMAL documentation, see [DECIMAL_USAGE_GUIDE.md](DECIMAL_USAGE_GUIDE.md)**

## Extended Data Types Support 🆕

The `bind_proxy` method provides automatic type conversion for complex Rust types, making it easy to work with dates, JSON, binary data, and more.

### Installation with Extended Types

```toml
[dependencies]
sqlx_struct_enhanced = { version = "0.1", features = ["postgres", "all-types"] }
# Or enable individual features:
# sqlx_struct_enhanced = { version = "0.1", features = ["postgres", "chrono", "json", "uuid"] }
```

### Supported Types

#### Additional Numeric Types (No Feature Required)
```rust
use sqlx_struct_enhanced::EnhancedCrudExt;

// i8, i16, i32, i64 - Direct binding (zero overhead)
let products = Product::where_query("stock_count = {}")
    .bind_proxy(100i16)
    .fetch_all(&pool)
    .await?;

// f32, f64 - Direct binding (zero overhead)
let products = Product::where_query("rating >= {}")
    .bind_proxy(4.5f32)
    .fetch_all(&pool)
    .await?;

// u8, u16, u32, u64 - Auto-convert to String
let users = User::where_query("age_group = {}")
    .bind_proxy(255u8)  // → String "255"
    .fetch_all(&pool)
    .await?;

// Vec<u8>, &[u8] - Direct binding (zero overhead)
let files = File::where_query("data = {}")
    .bind_proxy(vec![0x00, 0x01, 0x02])
    .fetch_all(&pool)
    .await?;
```

#### Chrono Date/Time Types (Feature: `chrono`)
```rust
use chrono::{NaiveDate, NaiveDateTime, Utc};

// NaiveDate → ISO 8601 string
let events = Event::where_query("event_date >= {}")
    .bind_proxy(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
    .fetch_all(&pool)
    .await?;

// NaiveDateTime → ISO 8601 string
let logs = Log::where_query("created_at >= {}")
    .bind_proxy(NaiveDateTime::from_timestamp_opt(1704067200, 0).unwrap())
    .fetch_all(&pool)
    .await?;

// DateTime<Utc> → ISO 8601 with timezone
let orders = Order::where_query("order_date = {}")
    .bind_proxy(Utc::now())
    .fetch_all(&pool)
    .await?;
```

#### UUID Type (Feature: `uuid`)
```rust
use uuid::Uuid;

let user_id = Uuid::new_v4();
let users = User::where_query("id = {}")
    .bind_proxy(user_id)  // → UUID string
    .fetch_one(&pool)
    .await?;
```

#### JSON Type (Feature: `json`)
```rust
use serde_json::json;

let metadata = json!({
    "name": "John Doe",
    "tags": ["vip", "premium"]
});

let users = User::where_query("metadata = {}")
    .bind_proxy(metadata)  // → JSON string
    .fetch_all(&pool)
    .await?;
```

### Type Conversion Summary

| Rust Type | Conversion | Feature | Overhead |
|-----------|------------|---------|----------|
| `i8`, `i16`, `i32`, `i64` | None | - | Zero |
| `f32`, `f64` | None | - | Zero |
| `Vec<u8>`, `&[u8]` | None | - | Zero |
| `u8`, `u16`, `u32`, `u64` | → String | - | Minimal |
| `chrono::*` | → ISO 8601 | chrono | Minimal |
| `uuid::Uuid` | → String | uuid | Minimal |
| `serde_json::Value` | → JSON String | json | Minimal |

> **📖 For complete documentation, see [USAGE.md](USAGE.md#supported-data-types-with-bindproxy)**

## Table Naming

By default, table names are automatically converted to `snake_case`:

```rust
#[derive(EnhancedCrud)]
struct UserProfile { ... }  // Table: "user_profile"
```

Use the `table_name` attribute to customize:

```rust
#[derive(EnhancedCrud)]
#[table_name = "app_users"]
struct UserProfile { ... }  // Table: "app_users"
```

See [PHASE3_FEATURES.md](PHASE3_FEATURES.md) for details.

## Database Support

### PostgreSQL (Default)

```toml
sqlx_struct_enhanced = { version = "0.1", features = ["postgres"] }
```

Generates PostgreSQL-style parameters: `$1, $2, $3`

### MySQL

```toml
sqlx_struct_enhanced = { version = "0.1", features = ["mysql"] }
```

Generates MySQL-style parameters: `?`

### SQLite

```toml
sqlx_struct_enhanced = { version = "0.1", features = ["sqlite"] }
```

Generates SQLite-style parameters: `?`

## Advanced Usage

### Custom Queries

```rust
// Using make_query for custom SQL
let users = User::make_query("SELECT * FROM user_profile WHERE created_at > NOW()")
    .fetch_all(&pool).await?;

// Using make_execute for statements without return values
User::make_execute("DELETE FROM user_profile WHERE created_at < NOW() - INTERVAL '30 days'")
    .execute(&pool).await?;
```

### WHERE Clauses

The `where_query`, `count_query`, and `delete_where_query` methods support parameterized WHERE clauses:

```rust
// Simple condition
User::where_query("active = true").fetch_all(&pool).await?;

// Multiple conditions with parameters
User::where_query("status = ? AND created_at > ?")
    .bind("active")
    .bind("2024-01-01")
    .fetch_all(&pool).await?;

// Counting
let (count,) = User::count_query("department = 'engineering'")
    .fetch_one(&pool).await?;

// Conditional delete
User::delete_where_query("status = 'inactive' AND last_login < NOW() - INTERVAL '90 days'")
    .execute(&pool).await?;

// Delete with parameters
User::delete_where_query("expired = ? AND created_at < ?")
    .bind(true)
    .bind("2024-01-01")
    .execute(&pool).await?;
```

### Batch Operations

#### Bulk Insert

The `bulk_insert` method allows efficient insertion of multiple records in a single SQL query:

```rust
// Insert multiple users at once
let new_users = vec![
    User { id: "1".to_string(), name: "Alice".to_string(), email: "alice@example.com".to_string() },
    User { id: "2".to_string(), name: "Bob".to_string(), email: "bob@example.com".to_string() },
    User { id: "3".to_string(), name: "Charlie".to_string(), email: "charlie@example.com".to_string() },
];
User::bulk_insert(&new_users).execute(&pool).await?;

// The generated SQL will be:
// PostgreSQL: INSERT INTO users VALUES ($1,$2,$3),($4,$5,$6),($7,$8,$9)
// MySQL/SQLite: INSERT INTO users VALUES (?,?,?),(?,?,?),(?,?,?)

// Large batch insertions are efficient
let many_users: Vec<User> = (1..=1000).map(|i| {
    User {
        id: format!("user{}", i),
        name: format!("User {}", i),
        email: format!("user{}@example.com", i),
    }
}).collect();
User::bulk_insert(&many_users).execute(&pool).await?;
```

#### Bulk Delete

The `bulk_delete` method allows efficient deletion of multiple records in a single SQL query:

```rust
// Delete multiple users by their IDs
let user_ids = vec![
    "user1".to_string(),
    "user2".to_string(),
    "user3".to_string(),
];
User::bulk_delete(&user_ids).execute(&pool).await?;

// The generated SQL will be:
// PostgreSQL: DELETE FROM users WHERE id IN ($1,$2,$3)
// MySQL/SQLite: DELETE FROM users WHERE id IN (?,?,?)

// Large batch deletions are efficient
let many_ids: Vec<String> = (1..=1000).map(|i| format!("user{}", i)).collect();
User::bulk_delete(&many_ids).execute(&pool).await?;
```

#### Bulk Update

The `bulk_update` method allows efficient updating of multiple records in a single SQL query using CASE WHEN statements:

```rust
// Update multiple users at once
let users_to_update = vec![
    User { id: "1".to_string(), name: "Alice Smith".to_string(), email: "alice.smith@example.com".to_string() },
    User { id: "2".to_string(), name: "Bob Jones".to_string(), email: "bob.jones@example.com".to_string() },
];
User::bulk_update(&users_to_update).execute(&pool).await?;

// The generated SQL will be:
// PostgreSQL: UPDATE users SET name=CASE WHEN $1 THEN $2 WHEN $3 THEN $4 END,
//                              email=CASE WHEN $5 THEN $6 WHEN $7 THEN $8 END
//                          WHERE id IN ($9,$10)
// MySQL/SQLite: UPDATE users SET name=CASE WHEN ? THEN ? WHEN ? THEN ? END,
//                                email=CASE WHEN ? THEN ? WHEN ? THEN ? END
//                            WHERE id IN (?,?)

// Large batch updates are efficient
let many_users: Vec<User> = (1..=100).map(|i| {
    User {
        id: format!("user{}", i),
        name: format!("Updated User {}", i),
        email: format!("updated{}@example.com", i),
    }
}).collect();
User::bulk_update(&many_users).execute(&pool).await?;
```

**Benefits of bulk operations:**
- Single database round-trip instead of N individual operations
- Automatic SQL caching for each batch size
- Type-safe with compile-time SQL generation
- Works with all three database backends

**Note:** SQL is cached based on the number of items/IDs in the batch, so repeated operations with the same batch size are very efficient.

### Transaction Support

The `transaction` helper function allows grouping multiple operations into a single atomic transaction:

```rust
use sqlx_struct_enhanced::transaction;

// Execute multiple operations in a transaction
transaction(&pool, |tx| async move {
    // Insert user
    user.insert_bind().execute(tx).await?;

    // Update profile in same transaction
    profile.update_bind().execute(tx).await?;

    // Bulk insert related records
    User::bulk_insert(&new_users).execute(tx).await?;

    Ok(())
}).await?;

// If any operation fails, all are rolled back automatically
transaction(&pool, |tx| async move {
    user.insert_bind().execute(tx).await?;

    if some_condition {
        return Err(MyError::ValidationFailed);
        // Transaction automatically rolled back
    }

    profile.update_bind().execute(tx).await?;
    Ok(())
}).await?;

// Cross-table transactions
transaction(&pool, |tx| async {
    // Create user and profile atomically
    user.insert_bind().execute(tx).await?;
    profile.insert_bind().execute(tx).await?;

    // Update both in one transaction
    user.update_bind().execute(tx).await?;
    profile.update_bind().execute(tx).await?;

    Ok(())
}).await?;
```

**Transaction Guarantees:**
- **Atomic**: All operations succeed or all fail
- **Automatic Rollback**: On error, transaction is rolled back
- **Automatic Commit**: On success, transaction is committed
- **Type-Safe**: Full type safety with Rust's type system
- **Works with All Operations**: Insert, update, delete, bulk operations

### Nested Transactions with Savepoints

The `nested_transaction` helper allows creating nested transactions within an existing transaction using savepoints:

```rust
use sqlx_struct_enhanced::{transaction, nested_transaction};

// Main transaction with nested transaction
transaction(&pool, |parent_tx| async move {
    // Main transaction work
    user.insert_bind().execute(parent_tx).await?;

    // Nested transaction with savepoint
    nested_transaction(parent_tx, |nested_tx| async move {
        profile.update_bind().execute(nested_tx).await?;
        log.insert_bind().execute(nested_tx).await?;

        // If this fails, only rolls back to savepoint
        if validation_fails {
            return Err(MyError::ValidationFailed);
        }

        Ok(())
    }).await?; // Nested transaction commits/rolls back independently

    // Parent transaction continues after nested transaction
    settings.update_bind().execute(parent_tx).await?;

    Ok(())
}).await?;
```

**Nested Transaction Guarantees:**
- **Partial Rollback**: Nested transaction can fail without failing parent
- **Automatic Savepoint Management**: Savepoints created and released automatically
- **Unique Names**: UUID-based savepoint names prevent conflicts
- **Multi-Level Nesting**: Supports multiple levels of nesting
- **Full ACID Compliance**: Maintains database consistency at all levels

**Use Cases:**
- Retryable operations within larger transactions
- Optional side-operations that can fail independently
- Complex multi-step processes with partial rollback capability
- Error isolation in nested business logic

### Compile-Time Index Analysis

The `#[analyze_queries]` attribute macro automatically analyzes your queries at compile time and recommends indexes to optimize performance:

```rust
#[sqlx_struct_macros::analyze_queries]
mod user_queries {
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
    }

    impl User {
        fn find_by_email(email: &str) {
            // Query: WHERE email = $1
            let _ = User::where_query!("email = $1");
        }

        fn find_active_users_since(timestamp: i64) {
            // Query: WHERE status = $1 AND created_at > $2 ORDER BY created_at DESC
            let _ = User::where_query!("status = $1 AND created_at > $2 ORDER BY created_at DESC");
        }
    }
}
```

When you compile this code, the macro outputs index recommendations:

```
🔍 ======================================================
🔍   SQLx Struct - Index Recommendations
🔍 ======================================================

📊 Table: User

   ✨ Recommended: idx_user_email
      Columns: email
      Reason: Single column: WHERE email = $1
      SQL:    CREATE INDEX idx_user_email ON User (email)

   ✨ Recommended: idx_user_status_created_at
      Columns: status, created_at
      Reason: WHERE status ORDER BY created_at
      SQL:    CREATE INDEX idx_user_status_created_at ON User (status, created_at)

🔍 ======================================================
🔍   End of Recommendations
🔍 ======================================================
```

**How It Works:**
1. **Parses struct definitions** to extract field names
2. **Scans for query patterns** like `where_query!()` and `make_query!()`
3. **Analyzes SQL WHERE clauses** to find conditions:
   - Equality: `col = $1`
   - Range: `col > $1`, `col < $1`, `col >= $1`, `col <= $1`
   - IN clauses: `col IN ($1, $2)`
   - LIKE clauses: `col LIKE $1`
4. **Analyzes ORDER BY clauses** to find sorting columns
5. **Generates recommendations** with optimal priority ordering:
   - Equality > IN > Range > LIKE > ORDER BY
6. **Deduplicates indexes** across multiple queries

**Benefits:**
- ✅ **Zero Runtime Overhead**: All analysis happens at compile time
- ✅ **Automatic Optimization**: No manual query analysis needed
- ✅ **Performance Guidance**: Get index recommendations as you write code
- ✅ **Smart Priority Ordering**: Optimizes column order based on condition types
- ✅ **Deduplication**: Identifies which indexes serve multiple queries
- ✅ **SQL Ready**: Copy-paste the generated CREATE INDEX statements

**Try It:**
```bash
cargo build --example compile_time_analysis
```

## Documentation

- **[USAGE.md](USAGE.md)** - Complete usage guide and API reference (⭐ Start here)
- **[DECIMAL_USAGE_GUIDE.md](DECIMAL_USAGE_GUIDE.md)** - DECIMAL/NUMERIC support guide (🆕)
- **[DECIMAL_QUICK_START.md](DECIMAL_QUICK_START.md)** - DECIMAL quick start examples (🆕)
- **[DECIMAL_FEATURE_SUMMARY.md](DECIMAL_FEATURE_SUMMARY.md)** - DECIMAL feature overview (🆕)
- **[COMPILE_TIME_INDEX_ANALYSIS.md](COMPILE_TIME_INDEX_ANALYSIS.md)** - Compile-time index analysis guide (🆕)
- [CLAUDE.md](CLAUDE.md) - Development guidelines

## Architecture

### Components

1. **Derive Macro** (`sqlx_struct_macros`)
   - Parses struct attributes and fields
   - Generates SQL query code at compile time

2. **SQL Generation** (`src/lib.rs`)
   - `Scheme` struct manages table metadata
   - Global cache stores SQL strings as `&'static str`
   - Database-specific parameter translation

3. **Trait Definitions** (`src/traits.rs`)
   - `EnhancedCrud` trait defines CRUD operations
   - Database-specific implementations

### Performance

- **Zero Memory Leaks**: No `Box::leak()` usage (fixed in v0.1)
- **Effective Caching**: Global cache stores generated SQL
- **No Runtime Overhead**: All SQL generation at compile time

## Examples

See the `tests/` directory for complete examples:
- `tests/test.rs` - Integration tests
- `tests/phase3_features.rs` - Phase 3 feature examples

## Limitations

- First struct field must be the ID/primary key
- Table names must be known at compile time
- No savepoint support (nested transactions)

## Roadmap

### Completed ✅
- Phase 1: P0 fixes (memory leaks, cache, feature flags)
- Phase 1: High priority issues (redundant code, typos, docs)
- Phase 2: Testing and optimization (62 unit tests)
- Phase 3: Custom table names
- Phase 3: Conditional delete (`delete_where_query`)
- Phase 3: Batch delete (`bulk_delete`)
- Phase 3: Batch insert (`bulk_insert`)
- Phase 3: Batch update (`bulk_update`)
- Phase 3: Transaction support (`transaction` helper)
- **DECIMAL/NUMERIC Support** 🆕
  - Type-safe decimal handling with automatic casting
  - `#[crud(decimal(precision = N, scale = M))]` attribute for migration generation
  - `#[crud(cast_as = "TYPE")]` attribute for query-time type casting
  - Full integration test coverage
  - Bug fixes: SQL cache deadlock, attribute parsing, placeholder replacement
- **Extended BindProxy Data Types** 🆕
  - Additional numeric types: i8, i16, u8, u16, u32, u64, f32
  - Chrono date/time types: NaiveDate, NaiveTime, NaiveDateTime, DateTime<Utc>
  - UUID support with automatic string conversion
  - JSON type support via serde_json
  - Binary data support: Vec<u8>, &[u8]
  - 93 comprehensive unit tests, all passing
  - Zero overhead for native types (i8, i16, f32, f64, Vec<u8>)
  - Minimal overhead for types requiring String conversion
  - Cross-database consistency (PostgreSQL, MySQL, SQLite)
  - Complete integration tests and example code
- Phase 0: Compile-time index analysis (`analyze_queries` macro) 🆕
  - ✅ Day 1: Basic equality and ORDER BY analysis
  - ✅ Day 2: Enhanced pattern recognition (Range, IN, LIKE operators)
    - Support for `>`, `<`, `>=`, `<=` operators
    - Support for `IN` clauses
    - Support for `LIKE` clauses
    - Smart priority ordering (Equality > IN > Range > LIKE > ORDER BY)
    - 18 comprehensive unit tests, all passing
    - Enhanced documentation and examples
  - ✅ Day 3: Negation conditions and extended features
    - Support for `!=`, `<>` inequality operators
    - Support for `NOT LIKE` clauses
    - `make_query!()` pattern recognition
    - Extended priority: Equality > IN > Range > LIKE > Inequality > NOT LIKE > ORDER BY
    - 26 comprehensive unit tests, all passing
    - Enhanced documentation with new rules
  - ✅ Day 4: OR conditions and query complexity detection 🆕
    - OR conditions detection (`has_or_conditions()`)
    - Parentheses grouping detection (`has_parentheses()`)
    - Subquery detection (`has_subquery()`)
    - Query complexity analysis API (`analyze_query_complexity()`)
    - 39 comprehensive unit tests, all passing
    - Documentation for OR condition indexing strategies
  - ✅ Day 5: Advanced analysis and recommendations 🆕
    - Unique index detection for `id` columns
    - Partial index detection (soft deletes, status filters)
    - Covering indexes with INCLUDE columns
    - Index size estimation (byte-level estimates)
    - OR condition separate index recommendations
    - 18 comprehensive unit tests, all passing (total: 57 tests)
    - Enhanced documentation with Rules 12-15
  - ✅ Day 6: Advanced multi-column optimization and database-specific features
    - Functional/expression index detection (LOWER, DATE, UPPER, etc.)
    - Index type selection (B-tree, Hash, BRIN, GIN/GiST recommendations)
    - Index effectiveness scoring (0-110 scale with detailed factors)
    - Database-specific optimization hints (PostgreSQL BRIN, trigram, GIN)
    - Real-world query pattern testing (pagination, search, time-series)
    - 20 comprehensive unit tests, all passing (total: 77 tests)
    - Enhanced documentation with Rules 16-19
  - ✅ Day 7: Index intersection strategies and performance prediction
    - Column cardinality analysis (Very High, High, Medium, Low, Very Low)
    - Column order optimization (cardinality-based within condition types)
    - Index intersection vs composite index recommendations
    - Query performance gain prediction (20-99% estimates)
    - Alternative index strategies for complex scenarios
    - 16 comprehensive unit tests, all passing (total: 93 tests)
    - Enhanced documentation with Rules 20-23
  - ✅ Day 8: Query plan visualization and cost analysis 🆕
    - Query execution plan hints (JOIN, ORDER BY, GROUP BY analysis)
    - Visual ASCII art representation of index structure and execution path
    - Query cost estimation (Very Low to High, compared to full table scan)
    - Performance characteristics and optimization recommendations
    - 18 comprehensive unit tests, all passing (total: 110 tests)
    - Enhanced documentation with Rules 24-26

### Planned 🚧
- Savepoint support for nested transactions
- Connection pool integration helpers
- Async streaming queries

## Contributing

See [CLAUDE.md](CLAUDE.md) for development guidelines.

## License

This project follows the same license as SQLx.
