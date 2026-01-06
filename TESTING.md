# Testing Guide

This document describes how to run tests for the `sqlx_struct_enhanced` crate.

## Prerequisites

### PostgreSQL Tests (Default)
PostgreSQL is the default database backend. Tests can be run directly:

```bash
# Run all library unit tests
cargo test --lib

# Run integration tests (requires running PostgreSQL instance)
cargo test
```

**Requirements**:
- PostgreSQL instance running on `127.0.0.1:5432`
- Database: `test-sqlx-tokio`
- User: `postgres` with no password

### MySQL Tests

To test with MySQL backend:

```bash
# Run unit tests
cargo test --lib --no-default-features --features mysql

# Run integration tests (requires MySQL instance)
cargo test --no-default-features --features mysql
```

**Requirements**:
- MySQL instance running on `127.0.0.1:3306`
- Database: `test_sqlx_tokio`
- User: `root` with no password or configure as needed

### SQLite Tests

SQLite tests don't require a running server:

```bash
# Run all tests
cargo test --lib --no-default-features --features sqlite
```

## Unit Tests

The crate includes comprehensive unit tests for SQL generation:

- **21 unit tests** covering:
  - SQL generation for INSERT, UPDATE, DELETE, SELECT
  - Parameter placeholder translation
  - WHERE clause preparation
  - SQL caching functionality
  - Edge cases (single field, large schemas, complex conditions)
  - Custom SQL placeholder replacement

Run unit tests:
```bash
cargo test --lib --features postgres  # or mysql/sqlite
```

## Test Coverage

### Current Test Coverage

| Component | Tests | Coverage |
|-----------|-------|----------|
| SQL Generation | 21 tests | ✅ Comprehensive |
| PostgreSQL | 21 tests passing | ✅ Full |
| MySQL | Unit tests need config | ⚠️ Partial |
| SQLite | Unit tests need config | ⚠️ Partial |
| Integration | 1 basic test | ⚠️ Needs expansion |

### Test Examples

#### Unit Test Example
```rust
#[test]
fn test_scheme_insert_sql_generation() {
    let scheme = Scheme {
        table_name: "users".to_string(),
        insert_fields: vec!["id".to_string(), "name".to_string()],
        update_fields: vec!["name".to_string()],
        id_field: "id".to_string(),
    };

    let sql = scheme.gen_insert_sql_static();
    assert_eq!(sql, "INSERT INTO users VALUES ($1,$2)");
}
```

#### Integration Test Example
```rust
#[tokio::test]
async fn test_crud_operations() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    let mut user = User { id: "1".to_string(), name: "Alice".to_string() };
    user.insert_bind().execute(&pool).await?;

    let user = User::by_pk().bind("1").fetch_one(&pool).await?;
    assert_eq!(user.name, "Alice");

    Ok(())
}
```

## CI/CD Considerations

For CI/CD pipelines:

1. **PostgreSQL** (primary backend):
   ```yaml
   - run: |
       docker run -d -p 5432:5432 -e POSTGRES_PASSWORD= postgres postgres
       cargo test --features postgres
   ```

2. **MySQL**:
   ```yaml
   - run: |
       docker run -d -p 3306:3306 -e MYSQL_ALLOW_EMPTY_PASSWORD=1 mysql
       cargo test --no-default-features --features mysql
   ```

3. **SQLite**:
   ```yaml
   - run: cargo test --no-default-features --features sqlite
   ```

## Adding New Tests

When adding new functionality:

1. Add unit tests in `src/lib.rs` under the `#[cfg(test)] mod tests` module
2. Include database-specific assertions using `#[cfg(feature = "...")]`
3. Add integration tests in `tests/test.rs` if database interaction is required
4. Ensure tests cover both success and error cases

## Known Testing Limitations

1. **Feature Flag Interaction**: Unit tests for MySQL/SQLite require `--no-default-features` to avoid conflicts with the default PostgreSQL feature
2. **Database Requirements**: Integration tests require actual database instances
3. **Test Isolation**: Current integration tests may leave test data in the database

## Future Improvements

- [ ] Add test database cleanup between test runs
- [ ] Use test containers for automatic database provisioning
- [ ] Add benchmarks for SQL generation performance
- [ ] Add property-based testing for SQL generation
- [ ] Add integration tests for MySQL and SQLite backends
