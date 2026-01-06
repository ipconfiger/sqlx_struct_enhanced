# sqlx_struct_enhanced

Auto-generate CRUD SQL operations for SQLx with type-safe query building.

## Features

- ✅ **Auto-generated SQL** for INSERT, UPDATE, DELETE, SELECT
- ✅ **Batch Operations**: Bulk insert, update, and delete for efficient multi-row operations
- ✅ **Transaction Support**: Atomic multi-operation transactions with automatic rollback
- ✅ **Conditional Operations**: WHERE queries, count queries, and conditional deletes
- ✅ **Multiple Database Backends**: PostgreSQL, MySQL, SQLite
- ✅ **Compile-time SQL Generation**: No runtime overhead
- ✅ **Global SQL Caching**: Efficient query reuse
- ✅ **Custom Table Names**: Override default table names
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

## Documentation

- **[USAGE.md](USAGE.md)** - Complete usage guide and API reference (⭐ Start here)
- [TESTING.md](TESTING.md) - Testing guide and CI/CD setup
- [PHASE3_FEATURES.md](PHASE3_FEATURES.md) - New Phase 3 features (custom table names)
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
- Phase 2: Testing and optimization (52 unit tests)
- Phase 3: Custom table names
- Phase 3: Conditional delete (`delete_where_query`)
- Phase 3: Batch delete (`bulk_delete`)
- Phase 3: Batch insert (`bulk_insert`)
- Phase 3: Batch update (`bulk_update`)
- Phase 3: Transaction support (`transaction` helper)

### Planned 🚧
- Savepoint support for nested transactions
- Connection pool integration helpers
- Async streaming queries

## Contributing

See [CLAUDE.md](CLAUDE.md) for development guidelines.

## License

This project follows the same license as SQLx.
