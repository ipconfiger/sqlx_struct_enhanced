# SQLx Struct Enhanced - Usage Guide for Claude Code

This guide provides comprehensive information about using `sqlx_struct_enhanced` in Rust projects, enabling Claude Code to effectively work with this crate.

## Overview

`sqlx_struct_enhanced` is a Rust crate that provides auto-generated CRUD operations for SQLx through derive macros. It eliminates boilerplate code by generating type-safe SQL queries based on struct definitions.

**Key Benefits:**
- Zero boilerplate CRUD operations via derive macro
- Type-safe SQL queries with compile-time verification
- Support for PostgreSQL, MySQL, and SQLite
- Bulk operations for efficient batch processing
- Flexible WHERE queries and custom table names
- Transaction support helpers

## Quick Start

### 1. Add Dependencies

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }  # or mysql/sqlite
sqlx_struct_enhanced = { version = "0.1", features = ["postgres"] }     # or mysql/sqlite
```

### 2. Derive the Macro

```rust
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(EnhancedCrud)]
struct User {
    id: String,           // First field = primary key
    name: String,
    email: String,
    age: i32,
}
// Generated table name: "user" (snake_case of struct name)
```

### 3. Use CRUD Operations

```rust
// Insert
let mut user = User {
    id: "123".to_string(),
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
    age: 30,
};
user.insert_bind().execute(&pool).await?;

// Select by primary key
let user = User::by_pk().bind("123").fetch_one(&pool).await?;

// Update
user.name = "Alice Updated".to_string();
user.update_bind().execute(&pool).await?;

// Delete
user.delete_bind().execute(&pool).await?;

// Custom query
let users = User::make_query("SELECT * FROM user WHERE age > 25")
    .fetch_all(&pool).await?;

// WHERE query
let users = User::where_query("age > 25").fetch_all(&pool).await?;

// Count
let (count,) = User::count_query("age > 25").fetch_one(&pool).await?;
```

## API Reference

### Instance Methods (operate on struct instances)

#### `insert_bind()`
Inserts the struct instance as a new row.
```rust
let mut user = User { /* fields */ };
user.insert_bind().execute(&pool).await?;
```

#### `update_bind()`
Updates the row matching the primary key.
```rust
user.name = "New Name".to_string();
user.update_bind().execute(&pool).await?;
```

#### `delete_bind()`
Deletes the row matching the primary key.
```rust
user.delete_bind().execute(&pool).await?;
```

### Static Methods (called on the struct type)

#### `by_pk()`
Creates a query to select by primary key.
```rust
let user = User::by_pk().bind("user-id").fetch_one(&pool).await?;
```

#### `make_query(sql: &str)`
Executes a custom SQL query returning the struct type.
```rust
let users = User::make_query("SELECT * FROM user WHERE age > 25")
    .fetch_all(&pool).await?;
```

#### `where_query(where_stmt: &str)`
Creates a SELECT query with WHERE clause.
```rust
// Use {} as placeholder for parameters
let users = User::where_query("age > {} AND name = {}")
    .bind(25)
    .bind("Alice")
    .fetch_all(&pool).await?;
```

#### `count_query(where_stmt: &str)`
Counts rows matching the WHERE clause.
```rust
let (count,) = User::count_query("age > 25").fetch_one(&pool).await?;
```

#### `delete_where_query(where_stmt: &str)`
Deletes rows matching the WHERE clause.
```rust
User::delete_where_query("age < 18").execute(&pool).await?;
```

### Bulk Operations

#### `bulk_insert(items: &[Self])`
Inserts multiple rows in a single query.
```rust
let users = vec![
    User { id: "1".to_string(), /* ... */ },
    User { id: "2".to_string(), /* ... */ },
];
User::bulk_insert(&users).execute(&pool).await?;
```

#### `bulk_select(ids: &[String])`
Selects multiple rows by primary keys using WHERE IN.
```rust
let ids = vec!["1".to_string(), "2".to_string(), "3".to_string()];
let users = User::bulk_select(&ids).fetch_all(&pool).await?;
```

**Note:** Order is not guaranteed. Sort in application code if needed.

#### `bulk_delete(ids: &[String])`
Deletes multiple rows by primary keys.
```rust
let ids = vec!["1".to_string(), "2".to_string()];
User::bulk_delete(&ids).execute(&pool).await?;
```

#### `bulk_update(items: &[Self])`
Updates multiple rows, matching by primary key.
```rust
let users = vec![
    User { id: "1".to_string(), name: "Updated 1".to_string(), /* ... */ },
    User { id: "2".to_string(), name: "Updated 2".to_string(), /* ... */ },
];
User::bulk_update(&users).execute(&pool).await?;
```

**Note:** Only updates non-primary-key fields.

## Advanced Features

### Custom Table Names

Override the auto-generated table name using `#[table_name]` attribute:

```rust
#[derive(EnhancedCrud)]
#[table_name = "app_users"]
struct User {
    id: String,
    name: String,
}
// Uses "app_users" table instead of "user"
```

### Transaction Helpers

The crate provides type-safe transaction helpers:

```rust
use sqlx_struct_enhanced::transaction;

// Simple transaction
transaction(&pool, |tx| async move {
    // Execute queries in transaction
    user.insert_bind().execute(tx).await?;
    Ok(())
}).await?;

// Nested transaction
transaction(&pool, |tx1| async move {
    user.insert_bind().execute(tx1).await?;

    nested_transaction(tx1, |tx2| async move {
        // Nested operations
        Ok(())
    }).await?;

    Ok(())
}).await?;
```

## Database Support

### PostgreSQL (default)
```toml
sqlx_struct_enhanced = { version = "0.1", features = ["postgres"] }
```
- Parameter syntax: `$1, $2, $3...`
- Field wrapping: `"field_name"`

### MySQL
```toml
sqlx_struct_enhanced = { version = "0.1", features = ["mysql"] }
```
- Parameter syntax: `?`
- Field wrapping: `` `field_name` ``

### SQLite
```toml
sqlx_struct_enhanced = { version = "0.1", features = ["sqlite"] }
```
- Parameter syntax: `?`
- Field wrapping: `field_name` (no wrapping)

## Important Conventions

### Primary Key
**The first struct field is always treated as the primary key.**

```rust
#[derive(EnhancedCrud)]
struct Product {
    id: String,      // ← Primary key (first field)
    name: String,
    price: i32,
}
```

### Table Naming
Table names are auto-generated from struct names using snake_case:

```rust
struct UserProfile {}  // → table: "user_profile"
struct APIKey {}       // → table: "a_p_i_key"  (Note: all caps gets underscored)
struct HttpHandler {}  // → table: "http_handler"
```

Use `#[table_name]` attribute to override:

```rust
#[derive(EnhancedCrud)]
#[table_name = "api_keys"]
struct APIKey {
    id: String,
    key: String,
}
```

### Parameter Placeholders
In `where_query()` and `delete_where_query()`, use `{}` as a placeholder:

```rust
// PostgreSQL: {} → $1, $2, $3...
let users = User::where_query("age > {} AND name = {}")
    .bind(25)
    .bind("Alice")
    .fetch_all(&pool).await?;
// Generates: SELECT * FROM user WHERE age > $1 AND name = $2

// MySQL/SQLite: {} → ?, ?, ?...
```

## Common Patterns

### Pagination

```rust
let page_size = 20;
let page = 1;
let offset = (page - 1) * page_size;

let users = User::make_query(&format!(
    "SELECT * FROM user ORDER BY created_at LIMIT {} OFFSET {}",
    page_size, offset
)).fetch_all(&pool).await?;
```

### Batch Processing

```rust
// Process users in batches of 100
let all_ids = get_all_user_ids();
for chunk in all_ids.chunks(100) {
    let users = User::bulk_select(chunk).fetch_all(&pool).await?;
    process_users(users).await?;
}
```

### Conditional Updates

```rust
// Update only if version matches
let updated = User::make_query(
    "UPDATE user SET name = $1, version = version + 1 WHERE id = $2 AND version = $3"
)
.bind(new_name)
.bind(&user.id)
.bind(user.version)
.execute(&pool).await?;

if updated.rows_affected() == 0 {
    return Err("Optimistic lock failed".into());
}
```

### Soft Deletes

```rust
#[derive(EnhancedCrud)]
struct User {
    id: String,
    name: String,
    deleted_at: Option<i64>,  // Unix timestamp
}

// Soft delete
User::delete_where_query("id = {}")
    .bind(user_id)
    .bind(chrono::Utc::now().timestamp())
    .execute(&pool).await?;

// Query only non-deleted
let users = User::where_query("deleted_at IS NULL").fetch_all(&pool).await?;
```

## Testing

### Unit Tests
The crate includes comprehensive unit tests that verify SQL generation without requiring a database:

```bash
cargo test  # Runs 61+ unit tests
```

### Integration Tests
Integration tests require a running PostgreSQL instance:

```bash
# Start PostgreSQL
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password postgres:16

# Run integration tests
cargo test --test test

# Run specific test
cargo test test_bulk_operations
```

**Note:** Integration tests use `serial_test` to ensure sequential execution and avoid data conflicts.

## Best Practices

### 1. Use Bulk Operations
Avoid N+1 queries by using bulk operations:
```rust
// ❌ Bad: N+1 query
for id in user_ids {
    let user = User::by_pk().bind(&id).fetch_one(&pool).await?;
    process(user);
}

// ✅ Good: Single query
let users = User::bulk_select(&user_ids).fetch_all(&pool).await?;
for user in users {
    process(user);
}
```

### 2. Leverage SQL Caching
SQL strings are cached automatically, so don't worry about calling the same query multiple times.

### 3. Custom Table Names for Clarity
Use explicit table names for complex schemas:
```rust
#[derive(EnhancedCrud)]
#[table_name = "user_profiles"]
struct UserProfile { /* ... */ }
```

### 4. First Field is Always ID
Never put a non-ID field first:
```rust
// ❌ Wrong: name is not the primary key
struct User {
    name: String,
    id: String,
}

// ✅ Correct: id is first
struct User {
    id: String,
    name: String,
}
```

### 5. Handle Empty Results
Always handle the case where no rows are found:
```rust
match User::by_pk().bind(id).fetch_optional(&pool).await? {
    Some(user) => Ok(Some(user)),
    None => Ok(None),
}
```

## Performance Considerations

- **SQL Generation**: SQL strings are generated once and cached (using `Box::leak` to get `&'static str`)
- **Bulk Operations**: Significantly faster than individual queries for batch operations
- **Parameter Binding**: All queries use prepared statements for security and performance
- **Database Indexes**: Ensure your database has proper indexes on primary keys and frequently queried columns

## Error Handling

All operations return `Result<T, sqlx::Error>`:

```rust
use sqlx::Error;

async fn get_user(id: &str) -> Result<User, Error> {
    match User::by_pk().bind(id).fetch_optional(&pool).await? {
        Some(user) => Ok(user),
        None => Err(Error::RowNotFound),
    }
}
```

## Limitations

1. **Single Primary Key**: Only supports single-column primary keys (first field)
2. **No Auto-Generation**: Does not auto-generate IDs (you must set them)
3. **No Relationships**: Does not handle foreign keys or JOIN operations (use raw SQLx for those)
4. **Order Not Guaranteed**: `bulk_select()` does not guarantee return order (sort in app code)
5. **All-or-Nothing Updates**: `bulk_update()` updates all non-primary-key fields

## Troubleshooting

### Issue: "table not found"
**Cause:** Table name mismatch (struct name → snake_case)

**Solution:** Check the generated table name or use `#[table_name]`:
```rust
#[derive(EnhancedCrud)]
#[table_name = "your_table_name"]
struct YourStruct { /* ... */ }
```

### Issue: "column does not exist"
**Cause:** Field name mismatch

**Solution:** Ensure struct fields match database columns exactly (case-sensitive in some databases)

### Issue: "parameter $1 does not exist"
**Cause:** Wrong database feature or parameter syntax mismatch

**Solution:** Ensure you're using the correct feature flag (`postgres`, `mysql`, or `sqlite`)

### Issue: "first field must be primary key"
**Cause:** Non-ID field is first in struct

**Solution:** Reorder fields so the primary key is first

## Migration from Raw SQLx

### Before (raw SQLx)
```rust
// Insert
sqlx::query("INSERT INTO user (id, name, email) VALUES ($1, $2, $3)")
    .bind(&user.id)
    .bind(&user.name)
    .bind(&user.email)
    .execute(&pool).await?;

// Select
let user = sqlx::query_as::<_, User>("SELECT * FROM user WHERE id = $1")
    .bind(id)
    .fetch_one(&pool).await?;

// Update
sqlx::query("UPDATE user SET name = $1 WHERE id = $2")
    .bind(&new_name)
    .bind(&id)
    .execute(&pool).await?;

// Delete
sqlx::query("DELETE FROM user WHERE id = $1")
    .bind(id)
    .execute(&pool).await?;
```

### After (sqlx_struct_enhanced)
```rust
// Insert
user.insert_bind().execute(&pool).await?;

// Select
let user = User::by_pk().bind(id).fetch_one(&pool).await?;

// Update
user.name = new_name;
user.update_bind().execute(&pool).await?;

// Delete
user.delete_bind().execute(&pool).await?;
```

**Benefits:**
- 75% less code
- Type-safe (no field name typos)
- DRY (Don't Repeat Yourself)
- Easier refactoring

## Examples Repository

For complete working examples, see the `tests/` directory in the crate:
- `tests/test.rs` - Integration tests demonstrating all features
- `tests/phase3_features.rs` - Phase 3 feature examples

## Version History

- **v0.1.0** - Initial release with:
  - Basic CRUD operations (insert, update, delete, select by PK)
  - WHERE queries and count queries
  - Bulk operations (insert, select, delete, update)
  - Custom table names
  - Transaction helpers
  - Support for PostgreSQL, MySQL, and SQLite

## Contributing

When contributing to this crate:
1. Add unit tests for new features
2. Ensure all 3 database backends are supported (Postgres, MySQL, SQLite)
3. Run `cargo test` and `cargo clippy` before committing
4. Update this documentation with examples of new features

## License

[Your License Here]

---

**For Claude Code Users:**

When working with projects using `sqlx_struct_enhanced`, Claude Code can:
- Generate new structs with `#[derive(EnhancedCrud)]`
- Add CRUD operations to existing structs
- Write database queries using the crate's API
- Debug common issues (table name mismatches, parameter binding errors)
- Refactor raw SQLx code to use this crate

**Example prompt:** "Add a new User struct with id, name, email fields and create a function to insert users into the database"

**Claude Code response:** Will create the struct with `EnhancedCrud` derive and implement the insert function using `insert_bind()`.
