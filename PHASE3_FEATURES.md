# Phase 3 Features

## Custom Table Names

The `EnhancedCrud` derive macro now supports custom table names via the `table_name` attribute.

### Usage

```rust
use sqlx_struct_enhanced::EnhancedCrud;
use sqlx::FromRow;

// Without attribute - uses snake_case of struct name
#[derive(EnhancedCrud)]
struct MyUser {      // Table name: "my_user"
    id: String,
    name: String,
}

// With custom table name
#[derive(EnhancedCrud)]
#[table_name = "app_users"]
struct CustomUser {  // Table name: "app_users"
    id: String,
    username: String,
}

// With prefixes
#[derive(EnhancedCrud)]
#[table_name = "legacy.users"]  // Can include schema prefix
struct LegacyModel {
    id: String,
    data: String,
}
```

### Why Use Custom Table Names?

1. **Legacy Databases**: Match existing table names that don't follow naming conventions
2. **Multi-tenant Apps**: Use prefixes like `tenant1_users`
3. **Naming Conflicts**: Avoid conflicts with SQL keywords or reserved words
4. **Schema Prefixes**: Include schema information like `public.users` or `app.users`

### Examples

#### Example 1: E-commerce Platform
```rust
#[derive(EnhancedCrud)]
#[table_name = "shop.customers"]
struct Customer {
    customer_id: String,
    email: String,
}

// Generates SQL: INSERT INTO shop.customers VALUES ($1,$2)
```

#### Example 2: Multi-tenant Application
```rust
#[derive(EnhancedCrud)]
#[table_name = "tenant_{tenant_id}.users"]
struct User {
    id: String,
    name: String,
}
```

#### Example 3: Legacy System Integration
```rust
#[derive(EnhancedCrud)]
#[table_name = "tbl_users"]
struct User {
    id: String,
    name: String,
}

// Works with legacy "tbl_" prefixed tables
```

## Implementation Details

### How It Works

1. The macro checks for a `#[table_name = "..."]` attribute on the struct
2. If found, uses the provided name for all SQL generation
3. If not found, converts the struct name to snake_case (default behavior)

### Code Generation Comparison

**Default (no attribute)**:
```rust
#[derive(EnhancedCrud)]
struct UserProfile { ... }

// Generated: SELECT * FROM user_profile WHERE id=$1
```

**Custom table name**:
```rust
#[derive(EnhancedCrud)]
#[table_name = "user_profiles"]
struct UserProfile { ... }

// Generated: SELECT * FROM user_profiles WHERE id=$1
```

## Testing

Unit tests verify that custom table names work correctly:

```rust
#[test]
fn test_table_name_with_underscores() {
    let scheme = Scheme {
        table_name: "user_profile_settings".to_string(),
        ...
    };
    let sql = scheme.gen_insert_sql_static();
    assert_eq!(sql, "INSERT INTO user_profile_settings VALUES ($1)");
}
```

See `tests/phase3_features.rs` for integration examples.

## Future Enhancements

The following features are planned for future releases:

### Batch Operations

Batch insert, update, and delete operations for better performance:

```rust
// Planned API
let users = vec![user1, user2, user3];
User::bulk_insert(users, &pool).await?;

User::bulk_update(users, &pool).await?;
User::bulk_delete(user_ids, &pool).await?;
```

### Transaction Support

Easy transaction management:

```rust
// Planned API
User::transaction(&pool, |tx| async move {
    user.insert_bind().execute(tx).await?;
    profile.update_bind().execute(tx).await?;
    Ok(())
}).await?;
```

### Connection Pool Integration

Direct pool support:

```rust
// Planned API
impl User {
    async fn find_by_email(pool: &PgPool, email: &str) -> Result<Self> {
        // Direct pool queries with connection management
    }
}
```

## Migration Guide

### From Default Names to Custom Names

**Before**:
```rust
#[derive(EnhancedCrud)]
struct User { id: String }  // Uses table "user"
```

**After**:
```rust
#[derive(EnhancedCrud)]
#[table_name = "users"]
struct User { id: String }  // Uses table "users"
```

No other code changes needed - all CRUD operations work the same way.

### Compatibility

- ✅ Fully backward compatible
- ✅ Works with all database backends (PostgreSQL, MySQL, SQLite)
- ✅ No breaking changes to existing code
- ✅ Mix default and custom table names in the same project

## Performance Considerations

Custom table names have **zero runtime overhead**:
- Table name is parsed at compile time
- SQL is generated and cached at compile time
- No performance difference between custom and default names

## Known Limitations

1. **Static Only**: Table names must be known at compile time (no dynamic table names)
2. **No Validation**: The macro doesn't validate that the table exists
3. **Case Sensitivity**: Some databases are case-sensitive for table names

## Best Practices

1. **Use for Legacy Systems**: When integrating with existing databases
2. **Be Consistent**: Choose either default or custom names throughout your project
3. **Document Deviations**: If using custom names, document the rationale
4. **Test Thoroughly**: Verify SQL generation matches your database schema

## Examples in Production

See `tests/phase3_features.rs` for real-world usage examples.
