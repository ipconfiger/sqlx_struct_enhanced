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

## Supported Data Types with BindProxy

The `bind_proxy` method provides automatic type conversion for complex Rust types when binding parameters to queries. This feature is essential when working with types that don't natively map to database types.

### Enabling Extended Type Support

To use extended data types, enable the appropriate feature flags:

```toml
[dependencies]
sqlx_struct_enhanced = { version = "0.1", features = ["postgres", "all-types"] }
# Or enable individual features:
# sqlx_struct_enhanced = { version = "0.1", features = ["postgres", "chrono", "json", "uuid"] }
```

**Available Features:**
- `decimal` - Rust `Decimal` type (via `rust_decimal` crate)
- `chrono` - Date and time types (via `chrono` crate)
- `json` - JSON type (via `serde_json` crate)
- `uuid` - UUID type (via `uuid` crate)
- `all-types` - Enables all of the above

### Basic Types (No Feature Required)

These types work without any feature flags:

**Signed Integers:**
- `i8`, `i16`, `i32`, `i64` - Direct binding (zero overhead)

**Floating Point:**
- `f32`, `f64` - Direct binding (zero overhead)

**Binary Data:**
- `Vec<u8>`, `&[u8]` - Direct binding (zero overhead)

**Other:**
- `String`, `&str` - Native string types
- `bool` - Boolean values

### Extended Types (Feature-Gated)

#### Unsigned Integers → String Conversion

**Note:** SQLx doesn't natively support unsigned integers for all databases. These types automatically convert to String:

```rust
use sqlx_struct_enhanced::EnhancedCrudExt;

// u8, u16, u32, u64 automatically convert to String
let users = User::where_query("age_group = {}")
    .bind_proxy(255u8)       // → String "255"
    .bind_proxy(65535u16)    // → String "65535"
    .bind_proxy(4294967295u32) // → String "4294967295"
    .bind_proxy(18446744073709551615u64) // → String "18446744073709551615"
    .fetch_all(&pool)
    .await?;
```

#### Chrono Date/Time Types (Feature: `chrono`)

All chrono types convert to ISO 8601 string format:

```rust
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Utc};

// NaiveDate → "YYYY-MM-DD"
let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
let events = Event::where_query("event_date >= {}")
    .bind_proxy(date)  // → "2024-01-15"
    .fetch_all(&pool)
    .await?;

// NaiveTime → "HH:MM:SS.nnnnnnnnn"
let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
let schedules = Schedule::where_query("start_time = {}")
    .bind_proxy(time)  // → "14:30:00.000000000"
    .fetch_all(&pool)
    .await?;

// NaiveDateTime → "YYYY-MM-DD HH:MM:SS.nnnnnnnnn"
let dt = NaiveDateTime::from_timestamp_opt(1704067200, 0).unwrap();
let logs = Log::where_query("created_at >= {}")
    .bind_proxy(dt)  // → "2024-01-01 00:00:00.000000000"
    .fetch_all(&pool)
    .await?;

// DateTime<Utc> → "YYYY-MM-DD HH:MM:SS.nnnnnnnnn+00:00"
let utc_dt = Utc::now();
let orders = Order::where_query("order_date = {}")
    .bind_proxy(utc_dt)
    .fetch_all(&pool)
    .await?;
```

#### UUID Type (Feature: `uuid`)

```rust
use uuid::Uuid;

let user_id = Uuid::new_v4();
let users = User::where_query("id = {}")
    .bind_proxy(user_id)  // → UUID string format
    .fetch_one(&pool)
    .await?;

// Parse UUID from string
let id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?;
let users = User::where_query("parent_id = {}")
    .bind_proxy(id)
    .fetch_all(&pool)
    .await?;
```

#### JSON Type (Feature: `json`)

```rust
use serde_json::json;

let metadata = json!({
    "name": "John Doe",
    "age": 30,
    "tags": ["vip", "premium"]
});

let users = User::where_query("metadata = {}")
    .bind_proxy(metadata)  // → JSON string
    .fetch_all(&pool)
    .await?;

// Query with JSON contains
let search_term = json!({"vip": true});
let users = User::where_query("metadata LIKE {}")
    .bind_proxy("%\"vip\": true%")
    .fetch_all(&pool)
    .await?;
```

#### Decimal Type (Feature: `decimal`)

```rust
use rust_decimal::Decimal;

// For DECIMAL/NUMERIC columns
let price = Decimal::from_str_exact("99.99").unwrap();
let products = Product::where_query("price >= {}")
    .bind_proxy(price)  // → String "99.99"
    .fetch_all(&pool)
    .await?;
```

### Type Conversion Summary

| Rust Type | Database Type | Conversion | Feature | Overhead |
|-----------|--------------|------------|---------|----------|
| `i8`, `i16`, `i32`, `i64` | SMALLINT/INT/BIGINT | None | - | Zero |
| `f32`, `f64` | REAL/DOUBLE | None | - | Zero |
| `Vec<u8>`, `&[u8]` | BYTEA/BLOB | None | - | Zero |
| `u8`, `u16`, `u32`, `u64` | TEXT | → String | - | Minimal |
| `chrono::NaiveDate` | TEXT | → ISO 8601 | chrono | Minimal |
| `chrono::NaiveTime` | TEXT | → ISO 8601 | chrono | Minimal |
| `chrono::NaiveDateTime` | TEXT | → ISO 8601 | chrono | Minimal |
| `chrono::DateTime<Utc>` | TEXT | → ISO 8601 | chrono | Minimal |
| `uuid::Uuid` | TEXT/UUID | → String | uuid | Minimal |
| `serde_json::Value` | TEXT/JSON | → JSON String | json | Minimal |
| `rust_decimal::Decimal` | TEXT/NUMERIC | → String | decimal | Minimal |

### Using bind_proxy with WHERE Queries

The `bind_proxy` method works seamlessly with `where_query`:

```rust
use sqlx_struct_enhanced::EnhancedCrudExt;
use chrono::NaiveDate;

// Multiple type conversions in one query
let results = Order::where_query("created_at >= {} AND total_amount >= {} AND status = {}")
    .bind_proxy(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()) // Date
    .bind_proxy(100.0f32)                                        // Float
    .bind_proxy("pending")                                       // String
    .fetch_all(&pool)
    .await?;
```

### Performance Considerations

1. **Direct binding is fastest**: Use signed integers (`i8`, `i16`, `i32`) instead of unsigned when possible
2. **String conversions have minimal overhead**: The conversion cost is typically < 100ns per value
3. **Type safety is maintained**: All conversions are type-safe and checked at compile time
4. **Zero runtime overhead for native types**: `i8`, `i16`, `f32`, `f64`, `Vec<u8>` bind directly
5. **Automatic SQL caching**: Repeated queries benefit from SQLx's prepared statement caching

### Examples

For comprehensive examples using all extended types, see:
- `examples/extended_types_simple.rs` - Basic usage examples
- `examples/extended_types_real_world.rs` - E-commerce scenario
- `examples/extended_types_performance.rs` - Performance optimization guide

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

## Aggregation Queries

The crate provides a fluent query builder for SQL aggregation operations including SUM, AVG, COUNT, MIN, MAX with support for GROUP BY, HAVING, ORDER BY, and LIMIT/OFFSET.

### Basic Aggregations

Simple aggregation without grouping:

```rust
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(EnhancedCrud)]
struct Order {
    id: String,
    category: String,
    amount: i32,
    status: String,
}

// SUM all amounts
let sql = Order::agg_query()
    .sum("amount")
    .build();
// Generates: SELECT SUM(amount) FROM order

let (total,): (i64,) = sqlx::query_as(sql)
    .fetch_one(&pool)
    .await?;

// AVG amount
let sql = Order::agg_query()
    .avg("amount")
    .build();

// COUNT rows
let sql = Order::agg_query()
    .count()
    .build();

// MIN/MAX
let sql = Order::agg_query()
    .min("amount")
    .max("amount")
    .build();
// Generates: SELECT MIN(amount), MAX(amount) FROM order
```

### GROUP BY Aggregations

Group results by column with aggregations:

```rust
// Total amount per category
let sql = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .build();
// Generates: SELECT category, SUM(amount) FROM order GROUP BY category

let results: Vec<(String, i64)> = sqlx::query_as(sql)
    .fetch_all(&pool)
    .await?;

// Multiple aggregates
let sql = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .avg("amount")
    .count()
    .build();
// Generates: SELECT category, SUM(amount), AVG(amount), COUNT(*)
//           FROM order GROUP BY category
```

### Custom Column Aliases

Use `_as` suffix methods to specify custom aliases for aggregate columns:

```rust
// Custom aliases for better result mapping
let sql = Order::agg_query()
    .group_by("category")
    .sum_as("amount", "total_amount")
    .avg_as("amount", "average_amount")
    .count_as("order_count")
    .build();
// Generates: SELECT category,
//           SUM(amount) AS total_amount,
//           AVG(amount) AS average_amount,
//           COUNT(*) AS order_count
//           FROM order GROUP BY category
```

**Available alias methods:**
- `.sum_as(column, alias)` - SUM with custom alias
- `.avg_as(column, alias)` - AVG with custom alias
- `.count_as(alias)` - COUNT(*) with custom alias
- `.count_column_as(column, alias)` - COUNT(column) with custom alias
- `.min_as(column, alias)` - MIN with custom alias
- `.max_as(column, alias)` - MAX with custom alias

### WHERE Clause

Filter rows before aggregation:

```rust
let sql = Order::agg_query()
    .where_("status = {}", &["active"])
    .group_by("category")
    .sum("amount")
    .build();
// Generates: SELECT category, SUM(amount) FROM order
//           WHERE status = $1 GROUP BY category

// Multiple conditions
let sql = Order::agg_query()
    .where_("status = {} AND amount > {}", &["active", "100"])
    .group_by("category")
    .sum("amount")
    .build();
// Generates: SELECT category, SUM(amount) FROM order
//           WHERE status = $1 AND amount > $2 GROUP BY category
```

### HAVING Clause

Filter grouped results using HAVING:

```rust
// Filter using aggregate function
let sql = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .having("SUM(amount) > {}", &[&1000i64])
    .build();
// Generates: SELECT category, SUM(amount) FROM order
//           GROUP BY category HAVING SUM(amount) > $1

// Filter using alias (recommended)
let sql = Order::agg_query()
    .group_by("category")
    .sum_as("amount", "total")
    .having("total > {}", &[&1000i64])
    .build();
// Generates: SELECT category, SUM(amount) AS total FROM order
//           GROUP BY category HAVING total > $1
```

**Note:** Parameter indexing is automatic - HAVING parameters come after WHERE parameters.

### ORDER BY

Sort aggregated results:

```rust
// Sort by aggregate in descending order
let sql = Order::agg_query()
    .group_by("category")
    .sum_as("amount", "total")
    .order_by("total", "DESC")
    .build();
// Generates: SELECT category, SUM(amount) AS total FROM order
//           GROUP BY category ORDER BY total DESC

// Sort by column in ascending order
let sql = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .order_by("category", "ASC")
    .build();
```

### LIMIT and OFFSET

Paginate aggregated results:

```rust
// Top 10 categories by total amount
let sql = Order::agg_query()
    .group_by("category")
    .sum_as("amount", "total")
    .order_by("total", "DESC")
    .limit(10)
    .build();
// Generates: SELECT category, SUM(amount) AS total FROM order
//           GROUP BY category ORDER BY total DESC LIMIT $1

// Pagination with OFFSET
let sql = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .limit(10)
    .offset(20)
    .build();
// Generates: SELECT category, SUM(amount) FROM order
//           GROUP BY category LIMIT $1 OFFSET $2
```

### Complex Queries

Combine all features for powerful queries:

```rust
// Complete example: Active orders, grouped by category,
// with totals > 1000, sorted by total, top 10 results
let sql = Order::agg_query()
    .where_("status = {}", &["active"])
    .group_by("category")
    .sum_as("amount", "total")
    .avg_as("amount", "average")
    .having("total > {}", &[&1000i64])
    .order_by("total", "DESC")
    .limit(10)
    .build();
// Generates:
// SELECT category, SUM(amount) AS total, AVG(amount) AS average
// FROM order
// WHERE status = $1
// GROUP BY category
// HAVING total > $2
// ORDER BY total DESC
// LIMIT $3

// Fetch results with custom struct
#[derive(sqlx::FromRow)]
struct CategoryStats {
    category: String,
    total: i64,
    average: Option<f64>,
}

let stats: Vec<CategoryStats> = sqlx::query_as(sql)
    .fetch_all(&pool)
    .await?;
```

### Parameter Indexing

The builder automatically handles parameter placeholder indexing:

```rust
// WHERE + HAVING + LIMIT + OFFSET
let sql = Order::agg_query()
    .where_("status = {} AND amount > {}", &["active", "100"])  // $1, $2
    .group_by("category")
    .sum_as("amount", "total")
    .having("total > {}", &[&500i64])  // $3
    .order_by("total", "DESC")
    .limit(10)  // $4
    .offset(20)  // $5
    .build();
```

### SQL Caching

Aggregation queries use SQL caching for performance:

```rust
let sql1 = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .build();

let sql2 = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .build();

// Both return the same cached &str (same memory address)
assert_eq!(sql1, sql2);
```

### Direct Execution Methods

For cleaner code, you can execute aggregation queries directly without manually calling `build()` and `sqlx::query_as()`:

#### Specialized Methods (Recommended)

For common aggregation types, use specialized methods that return simple types:

```rust
// COUNT - Returns i64 directly
let count: i64 = User::agg_query()
    .where_("role = {}", &[&"admin"])
    .count()
    .fetch_count(&pool)
    .await?;

// AVG - Returns Option<f64> (NULL if no rows)
let average_score: Option<f64> = Rating::agg_query()
    .where_("category = {}", &[&"tech"])
    .avg("score")
    .fetch_avg(&pool)
    .await?;

// SUM - Returns Option<f64> (NULL if no rows)
let total_amount: Option<f64> = Order::agg_query()
    .where_("status = {}", &[&"completed"])
    .sum("amount")
    .fetch_sum(&pool)
    .await?;
```

**Benefits:**
- No manual type specification needed
- Cleaner, more readable code
- Automatic parameter binding
- Consistent with CRUD operations

#### Generic Methods

For flexible result types, use generic methods:

```rust
// fetch_one<T>() - Single row result
let (avg, count): (Option<f64>, i64) = Rating::agg_query()
    .where_("product_id = {}", &[&"prod123"])
    .avg("score")
    .count()
    .fetch_one(&pool)
    .await?;

// fetch_all<T>() - Multiple row results (GROUP BY)
let results: Vec<(String, i64)> = Order::agg_query()
    .group_by("status")
    .count()
    .fetch_all(&pool)
    .await?;

// fetch_optional<T>() - Optional result (None if no rows)
let max_value: Option<(i32,)> = Metric::agg_query()
    .where_("sensor_id = {}", &[&"sensor1"])
    .max("value")
    .fetch_optional(&pool)
    .await?;
```

**When to use:**
- Multiple aggregates in one query (AVG + COUNT, etc.)
- GROUP BY queries with custom result types
- Queries that might return zero rows

#### Code Comparison

**Old way (using build()):**
```rust
// 8 lines, manual parameter binding
let id_str = id.to_string();
let sql = User::agg_query()
    .where_("operation_center_id = {}", &[&id_str])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)
    .bind(id)
    .fetch_one(&pool)
    .await?;
```

**New way (using fetch_count()):**
```rust
// 2 lines, automatic binding
let count = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_count(&pool)
    .await?;
```

**Reduction: 75% less code!**

#### Available Methods

**Specialized Methods:**
- `.fetch_count(&pool)` → Returns `Result<i64>`
- `.fetch_avg(&pool)` → Returns `Result<Option<f64>>`
- `.fetch_sum(&pool)` → Returns `Result<Option<f64>>`

**Generic Methods:**
- `.fetch_one<T>(&pool)` → Returns `Result<T>` (single row)
- `.fetch_all<T>(&pool)` → Returns `Result<Vec<T>>` (multiple rows)
- `.fetch_optional<T>(&pool)` → Returns `Result<Option<T>>` (0 or 1 rows)

All methods automatically handle:
- WHERE clause parameters
- HAVING clause parameters
- LIMIT parameter
- OFFSET parameter

#### Complete Examples

```rust
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(EnhancedCrud)]
struct Order {
    id: String,
    customer_id: String,
    amount: f64,
    status: String,
}

// Example 1: Simple count with specialized method
let active_order_count = Order::agg_query()
    .where_("status = {}", &[&"active"])
    .count()
    .fetch_count(&pool)
    .await?;

// Example 2: AVG + COUNT with generic method
let (avg_amount, order_count): (Option<f64>, i64) = Order::agg_query()
    .where_("customer_id = {}", &[&"cust123"])
    .avg("amount")
    .count()
    .fetch_one(&pool)
    .await?;

// Example 3: GROUP BY with fetch_all
let status_counts: Vec<(String, i64)> = Order::agg_query()
    .group_by("status")
    .count()
    .order_by("count", "DESC")
    .fetch_all(&pool)
    .await?;

// Example 4: Pagination with LIMIT/OFFSET
let page_results: Vec<(String, Option<f64>)> = Order::agg_query()
    .group_by("customer_id")
    .avg("amount")
    .order_by("avg", "DESC")
    .limit(10)
    .offset(20)
    .fetch_all(&pool)
    .await?;

// Example 5: Complex query with all features
let results: Vec<(String, i64)> = Order::agg_query()
    .where_("status = {} AND amount > {}", &["completed", "100"])
    .group_by("customer_id")
    .count()
    .having("count > {}", &[&5i64])
    .order_by("count", "DESC")
    .limit(10)
    .fetch_all(&pool)
    .await?;
```

### JOIN Support

Combine data from multiple tables for complex analytics:

#### INNER JOIN

```rust
// Join orders with customers to analyze by customer attributes
let sql = Order::agg_query()
    .join("customers", "orders.customer_id = customers.id")
    .where_("customers.status = {} AND orders.status = {}", &["active", "completed"])
    .group_by("customers.region")
    .sum_as("orders.amount", "total_revenue")
    .count_as("order_count")
    .order_by("total_revenue", "DESC")
    .build();
// Generates: SELECT customers.region, SUM(orders.amount) AS total_revenue, COUNT(*) AS order_count
//           FROM orders INNER JOIN customers ON orders.customer_id = customers.id
//           WHERE customers.status = $1 AND orders.status = $2
//           GROUP BY customers.region ORDER BY total_revenue DESC
```

#### LEFT JOIN

```rust
// Include all products, even those with no sales
let sql = Order::agg_query()
    .join_left("products", "orders.product_id = products.id")
    .group_by("products.category")
    .sum_as("orders.amount", "total_sales")
    .count_as("order_count")
    .build();
// Generates: SELECT products.category, SUM(orders.amount) AS total_sales, COUNT(*) AS order_count
//           FROM orders LEFT JOIN products ON orders.product_id = products.id
//           GROUP BY products.category
```

#### RIGHT JOIN and FULL JOIN

```rust
// RIGHT JOIN - Focus on the right table
let sql = Order::agg_query()
    .join_right("customers", "orders.customer_id = customers.id")
    .group_by("customers.region")
    .sum("orders.amount")
    .build();

// FULL JOIN - All records from both tables
let sql = Order::agg_query()
    .join_full("customers", "orders.customer_id = customers.id")
    .group_by("customers.region")
    .sum("orders.amount")
    .build();
```

#### Multiple Joins

```rust
// Join both customers and products
let sql = Order::agg_query()
    .join("customers", "orders.customer_id = customers.id")
    .join("products", "orders.product_id = products.id")
    .where_("customers.tier = {} AND orders.status = {}", &["gold", "completed"])
    .group_by("customers.region")
    .group_by("products.category")
    .sum_as("orders.amount", "total_revenue")
    .count_as("order_count")
    .having("total_revenue > {}", &[&1000i64])
    .order_by("total_revenue", "DESC")
    .limit(10)
    .build();
// Generates: SELECT customers.region, products.category, SUM(orders.amount) AS total_revenue, COUNT(*) AS order_count
//           FROM orders INNER JOIN customers ON orders.customer_id = customers.id
//                  INNER JOIN products ON orders.product_id = products.id
//           WHERE customers.tier = $1 AND orders.status = $2
//           GROUP BY customers.region, products.category
//           HAVING total_revenue > $3 ORDER BY total_revenue DESC LIMIT $4
```

#### Self-Join Pattern

Join the same table twice for comparing related records:

```rust
// Compare orders with reference orders
let sql = Order::agg_query()
    .join("orders AS ref_orders", "orders.ref_id = ref_orders.id")
    .where_("ref_orders.status = {}", &["completed"])
    .group_by("ref_orders.category")
    .sum_as("orders.amount", "pending_total")
    .sum_as("ref_orders.amount", "completed_total")
    .build();
```

**Available JOIN Methods:**
- `.join(table, condition)` - INNER JOIN (default)
- `.join_left(table, condition)` - LEFT JOIN
- `.join_right(table, condition)` - RIGHT JOIN
- `.join_full(table, condition)` - FULL JOIN

**Important Notes:**
- Always specify table names with column references in joins (e.g., `orders.amount` not just `amount`)
- JOIN conditions use the format: `"left_table.left_col = right_table.right_col"`
- Multiple `.group_by()` calls are needed for grouping by multiple columns
- Parameter indexing is automatic across WHERE, HAVING, LIMIT, OFFSET

### Real-World Examples

#### Sales Report by Category

```rust
#[derive(EnhancedCrud)]
struct SalesOrder {
    id: String,
    category: String,
    amount: i32,
    status: String,
    created_at: String,
}

// Monthly sales by category
let sql = SalesOrder::agg_query()
    .where_("status = {}", &["completed"])
    .group_by("category")
    .sum_as("amount", "total_sales")
    .count_as("order_count")
    .avg_as("amount", "avg_order_value")
    .having("total_sales > {}", &[&10000i64])
    .order_by("total_sales", "DESC")
    .build();

#[derive(sqlx::FromRow)]
struct SalesReport {
    category: String,
    total_sales: i64,
    order_count: i64,
    avg_order_value: Option<f64>,
}

let reports: Vec<SalesReport> = sqlx::query_as(sql)
    .fetch_all(&pool)
    .await?;
```

#### Top N Customers

```rust
#[derive(EnhancedCrud)]
struct CustomerOrder {
    id: String,
    customer_id: String,
    amount: i32,
}

// Top 10 customers by total order amount
let sql = CustomerOrder::agg_query()
    .group_by("customer_id")
    .sum_as("amount", "total_spent")
    .count_as("order_count")
    .order_by("total_spent", "DESC")
    .limit(10)
    .build();

#[derive(sqlx::FromRow)]
struct TopCustomer {
    customer_id: String,
    total_spent: i64,
    order_count: i64,
}

let top_customers: Vec<TopCustomer> = sqlx::query_as(sql)
    .fetch_all(&pool)
    .await?;
```

#### Pagination for Dashboard

```rust
// Paginated category statistics (page 2, 20 per page)
let page = 2;
let per_page = 20;
let offset = (page - 1) * per_page;

let sql = Order::agg_query()
    .where_("status = {}", &["active"])
    .group_by("category")
    .sum_as("amount", "total")
    .avg_as("amount", "average")
    .min_as("amount", "minimum")
    .max_as("amount", "maximum")
    .order_by("total", "DESC")
    .limit(per_page)
    .offset(offset)
    .build();

let stats: Vec<CategoryStats> = sqlx::query_as(sql)
    .fetch_all(&pool)
    .await?;
```

### Database Support

All aggregation features work across all supported databases:

- **PostgreSQL** - Uses `$1, $2, $3` parameter placeholders
- **MySQL** - Uses `?, ?, ?` parameter placeholders
- **SQLite** - Uses `?, ?, ?` parameter placeholders

The query builder automatically generates the correct parameter syntax for your database.

### Performance Tips

1. **Use aliases for HAVING clauses** - More readable and easier to maintain
2. **Limit result sets early** - Use LIMIT to reduce data transfer
3. **Index GROUP BY columns** - For better query performance
4. **Cache query builders** - Reuse builder instances for similar queries
5. **Use custom result structs** - Better type safety than tuples

### Limitations

- Only single-column GROUP BY is currently supported
- Column names are not validated at compile time (trust user input)
- JOIN support is not yet available (planned for future release)
- Subqueries in aggregates are not supported

## Advanced Features

### DECIMAL/NUMERIC Support

The crate supports PostgreSQL DECIMAL/NUMERIC columns with automatic type casting for Rust String types:

#### New Simplified Syntax (Recommended)

**Default cast_as = "TEXT"** (applied automatically):

```rust
#[derive(EnhancedCrud)]
#[table_name = "products"]
struct Product {
    id: String,
    name: String,

    // Simplified syntax: cast_as defaults to "TEXT"
    #[crud(decimal(precision = 10, scale = 2))]
    price: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    discount: Option<String>,

    quantity: i32,
}
```

**Custom cast_as type:**

```rust
#[derive(EnhancedCrud)]
struct Product {
    id: String,

    // Custom cast type
    #[crud(decimal(precision = 10, scale = 2, cast_as = "VARCHAR"))]
    price: Option<String>,
}
```

#### Legacy Syntax (Still Supported)

```rust
#[derive(EnhancedCrud)]
struct Product {
    id: String,

    // Old two-attribute syntax (backward compatible)
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    price: Option<String>,
}
```

**How it works:**
- `#[crud(decimal(precision = N, scale = M))]` - Defines NUMERIC(N,M) for migration generation
- `cast_as` parameter (optional) - Specifies SQL cast type, defaults to "TEXT" for DECIMAL fields
- Adds type casting in SELECT queries: `price::TEXT as price`
- Generated SELECT: `SELECT id, name, price::TEXT as price, discount::TEXT as discount, quantity FROM products`

**Use cases:**
- Financial data where exact decimal precision is required
- Storing decimal values as String in Rust to avoid floating-point errors
- Automatic type conversion between database NUMERIC and Rust String

**Example:**
```rust
// Insert
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
assert_eq!(product.price, Some("1299.99".to_string()));
```

**Multiple Attributes:**
You can use multiple `#[crud(...)]` attributes on a single field:

```rust
#[derive(EnhancedCrud)]
struct OrderItem {
    id: String,

    // Multiple attributes for same field
    #[crud(decimal(precision = 12, scale = 4))]
    #[crud(cast_as = "TEXT")]
    unit_price: Option<String>,

    quantity: i32,
}
```

### DECIMAL Helper Methods

When you annotate a field with `#[crud(decimal(precision = N, scale = M))]`, the library automatically generates **27 helper methods** for that field. These methods provide type-safe operations for DECIMAL values stored as `String` or `Option<String>`.

**Quick Overview:**

The DECIMAL helper methods provide:

- **Type Conversion** - Convert DECIMAL strings to f64 for calculations
- **Arithmetic Operations** - Add, subtract, multiply, divide with chainable syntax
- **Validation** - Ensure values fit within NUMERIC(precision, scale) constraints
- **Value Checks** - Test for positive, negative, or zero values
- **Formatting** - Format with thousands separators, currency symbols, and percentages
- **Precision Control** - Query precision/scale metadata, clamp values, get min/max

**Why Use DECIMAL Helpers?**

```rust
// Without DECIMAL helpers: Manual string parsing and error handling
let amount_str = &order.amount;
let amount_f64: f64 = amount_str.parse()?;
let new_amount = format!("{:.2}", amount_f64 * 1.1);
order.amount = Some(new_amount);

// With DECIMAL helpers: Clean, type-safe, chainable
order.amount_add_f64(50.0)?
     .amount_mul_f64(1.1)?
     .amount_round(2)?;
```

#### Supported Field Types

The macro automatically detects the field type and generates appropriate methods:

- **`String` fields** - Non-optional DECIMAL values (e.g., `available_balance: String`)
- **`Option<String>` fields** - Nullable DECIMAL values (e.g., `estimated_price: Option<String>`)

#### Generated Methods

For each DECIMAL field, the following methods are automatically generated (using `amount` as the example field name):

##### Type Conversion Methods

```rust
// Convert DECIMAL field to f64
// For String:      -> DecimalResult<f64>
// For Option<String>: -> DecimalResult<Option<f64>>
order.amount_as_f64()?;

// Convert with default value if field is None (only for Option<String>)
order.amount_as_f64_or(0.0)?;  // -> DecimalResult<f64>

// Convert and unwrap (panics if None or invalid)
order.amount_as_f64_unwrap();  // -> f64
```

##### Chainable Arithmetic Operations

All arithmetic methods return `&mut Self` for method chaining:

```rust
// Add string value
order.amount_add("100.00")?;

// Subtract string value
order.amount_sub("50.00")?;

// Multiply by string value
order.amount_mul("1.5")?;

// Divide by string value (returns error if dividing by zero)
order.amount_div("2.0")?;

// Add f64 value
order.amount_add_f64(100.0)?;

// Subtract f64 value
order.amount_sub_f64(50.0)?;

// Multiply by f64 value
order.amount_mul_f64(1.5)?;

// Divide by f64 value (returns error if dividing by zero)
order.amount_div_f64(2.0)?;

// Round to N decimal places
order.amount_round(2)?;

// Absolute value
order.amount_abs()?;

// Negate value
order.amount_neg()?;
```

##### Validation Methods

```rust
// Validate against NUMERIC(precision, scale) constraints
// Returns Err(DecimalError::Overflow) if value exceeds precision
order.amount_validate()?;  // -> DecimalResult<bool>

// Check if positive (> 0)
// For String:      -> bool
// For Option<String>: -> Option<bool>
order.amount_is_positive();

// Check if negative (< 0)
order.amount_is_negative();

// Check if zero (= 0)
order.amount_is_zero();
```

##### Formatting Methods

```rust
// Format with thousands separator and 2 decimal places
// For String:      -> DecimalResult<String>
// For Option<String>: -> DecimalResult<Option<String>>
order.amount_format()?;  // "1,234.56" (with thousands separator)

// Format with currency symbol and thousands separator
order.amount_format_currency("$")?;  // "$1,234.56"

// Format as percentage (multiplies by 100 and adds %)
// Note: Expects decimal value like 0.7550 for 75.50%
order.amount_format_percent()?;  // "75.50%"

// Truncate to N decimal places (no rounding)
order.amount_truncate(2)?;
```

**Formatting examples:**
```rust
let amount = "1234567.89";

// Basic formatting with thousands separator
order.amount = Some(amount.to_string());
order.amount_format().unwrap();  // Some("1,234,567.89")

// Currency formatting
order.amount_format_currency("$").unwrap();  // Some("$1,234,567.89")
order.amount_format_currency("€").unwrap();  // Some("€1,234,567.89")

// Percentage formatting
let rate = "0.0825";  // 8.25%
order.rate = Some(rate.to_string());
order.rate_format_percent().unwrap();  // Some("8.25%")
```

##### Precision Information

```rust
// Get precision from #[crud(decimal(precision = N))]
order.amount_precision();  // -> u8 (e.g., 10)

// Get scale from #[crud(decimal(scale = N))]
order.amount_scale();  // -> u8 (e.g., 2)

// Get maximum value for NUMERIC(precision, scale)
order.amount_max_value()?;  // -> DecimalResult<String> (e.g., "99999999.99")

// Get minimum value
order.amount_min_value()?;  // -> DecimalResult<String> (e.g., "-99999999.99")

// Clamp value to fit within precision/scale constraints
order.amount_clamp()?;
```

#### Complete Example

```rust
#[derive(EnhancedCrud)]
#[table_name = "orders"]
struct Order {
    id: String,
    user_id: String,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    total_amount: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    commission_rate: Option<String>,

    created_at: chrono::DateTime<chrono::Utc>,
}

// Usage examples
async fn process_order(pool: &PgPool, mut order: Order) -> Result<(), Box<dyn std::error::Error>> {
    // Type conversion
    let amount = order.total_amount_as_f64_or(0.0)?;
    println!("Order amount: {}", amount);  // 1000.0

    // Check value
    if order.total_amount_is_positive().unwrap_or(false) {
        println!("Order has positive amount");
    }

    // Chainable arithmetic
    order.total_amount_add_f64(50.0)?
         .total_amount_mul_f64(1.1)?;  // Add tax
    order.total_amount_round(2)?;

    println!("Amount after tax: {}", order.total_amount_as_f64_unwrap());  // 1155.0

    // Format for display (with thousands separator)
    let formatted = order.total_amount_format()?;
    println!("Total: {}", formatted.unwrap());  // "1,155.00"

    // Format as currency
    let currency = order.total_amount_format_currency("$")?;
    println!("Total: {}", currency.unwrap());  // "$1,155.00"

    // Validate against NUMERIC(10,2) constraints
    order.total_amount_validate()?;

    // Clamp if value exceeds precision
    order.total_amount_clamp()?;

    Ok(())
}
```

#### Error Handling

All DECIMAL helper methods return `DecimalResult<T>` which is `Result<T, DecimalError>`:

```rust
use sqlx_struct_enhanced::decimal_helpers::{DecimalError, DecimalResult};

enum DecimalError {
    InvalidFormat(String),           // String cannot be parsed as f64
    Overflow { value: String, precision: u8, scale: u8 },  // Exceeds NUMERIC constraints
    DivisionByZero,                  // Attempted division by zero
    NullValue,                       // Operation on None field (for Option<String>)
}
```

**Error handling example:**

```rust
match order.total_amount_as_f64() {
    Ok(Some(amount)) => println!("Amount: {}", amount),
    Ok(None) => println!("No amount set"),
    Err(DecimalError::InvalidFormat(s)) => {
        eprintln!("Invalid decimal format: '{}'", s);
    }
    Err(DecimalError::Overflow { value, precision, scale }) => {
        eprintln!("Value '{}' exceeds NUMERIC({}, {})", value, precision, scale);
    }
    Err(e) => eprintln!("Error: {:?}", e),
}
```

#### Differences Between String and Option<String>

| Method | `String` Field | `Option<String>` Field |
|--------|----------------|------------------------|
| `as_f64()` | `DecimalResult<f64>` | `DecimalResult<Option<f64>>` |
| `as_f64_or()` | `DecimalResult<f64>` | `DecimalResult<f64>` |
| `is_positive()` | `bool` | `Option<bool>` |
| Mutation on None | N/A (always has value) | Returns `Err(NullValue)` |

#### Complete Method Reference

| Method Category | Method Name | String Return | Option<String> Return | Description |
|----------------|-------------|---------------|----------------------|-------------|
| **Type Conversion** | `as_f64()` | `DecimalResult<f64>` | `DecimalResult<Option<f64>>` | Convert to f64 |
| | `as_f64_or(default)` | `DecimalResult<f64>` | `DecimalResult<f64>` | Convert with default |
| | `as_f64_unwrap()` | `f64` | `f64` | Convert, panic on error |
| **Arithmetic** | `add(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Add decimal string |
| | `add_f64(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Add f64 |
| | `sub(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Subtract decimal string |
| | `sub_f64(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Subtract f64 |
| | `mul(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Multiply by decimal string |
| | `mul_f64(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Multiply by f64 |
| | `div(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Divide by decimal string |
| | `div_f64(value)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Divide by f64 |
| | `round(places)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Round to N places |
| | `truncate(places)` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Truncate to N places |
| | `abs()` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Absolute value |
| | `neg()` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Negate value |
| **Validation** | `validate()` | `DecimalResult<bool>` | `DecimalResult<bool>` | Check constraints |
| **Value Checks** | `is_positive()` | `bool` | `Option<bool>` | Check if > 0 |
| | `is_negative()` | `bool` | `Option<bool>` | Check if < 0 |
| | `is_zero()` | `bool` | `Option<bool>` | Check if = 0 |
| **Formatting** | `format()` | `DecimalResult<String>` | `DecimalResult<Option<String>>` | Format with separators |
| | `format_currency(symbol)` | `DecimalResult<String>` | `DecimalResult<Option<String>>` | Format as currency |
| | `format_percent()` | `DecimalResult<String>` | `DecimalResult<Option<String>>` | Format as percentage |
| **Precision** | `precision()` | `u8` | `u8` | Get precision value |
| | `scale()` | `u8` | `u8` | Get scale value |
| | `max_value()` | `DecimalResult<String>` | `DecimalResult<String>` | Get maximum value |
| | `min_value()` | `DecimalResult<String>` | `DecimalResult<String>` | Get minimum value |
| | `clamp()` | `DecimalResult<&mut Self>` | `DecimalResult<&mut Self>` | Clamp to valid range |

**Total: 27 methods per DECIMAL field**

#### When to Use DECIMAL Helpers

✅ **Use DECIMAL helpers when:**
- Working with financial data (prices, amounts, balances)
- Need exact decimal precision (avoiding floating-point errors)
- Storing DECIMAL as String in Rust
- Performing arithmetic on decimal values
- Validating against NUMERIC constraints
- Formatting for display (currency, percentages)

❌ **Don't use DECIMAL helpers when:**
- Working with simple integers (use `i32`, `i64`)
- Using approximate values (use `f64` directly)
- Field is not a DECIMAL/NUMERIC type

#### Real-World Examples

**Example 1: E-commerce Order Processing**

```rust
#[derive(EnhancedCrud, Debug, Clone)]
struct Order {
    id: String,
    customer_id: String,

    #[crud(decimal(precision = 12, scale = 2))]
    #[crud(cast_as = "TEXT")]
    subtotal: Option<String>,

    #[crud(decimal(precision = 5, scale = 4))]
    #[crud(cast_as = "TEXT")]
    tax_rate: Option<String>,

    #[crud(decimal(precision = 12, scale = 2))]
    #[crud(cast_as = "TEXT")]
    total: Option<String>,

    status: String,
}

// Calculate order total with tax
fn calculate_order_total(order: &mut Order) -> Result<(), DecimalError> {
    let subtotal = order.subtotal_as_f64_or(0.0)?;
    let tax_rate = order.tax_rate_as_f64_or(0.0)?;

    // Calculate total: subtotal * (1 + tax_rate)
    order.total = Some("0".to_string());
    order.total_add_f64(subtotal)?
         .total_mul_f64(1.0 + tax_rate)?
         .total_round(2)?;

    Ok(())
}

// Display formatted price
fn display_order_price(order: &Order) -> Result<String, DecimalError> {
    let total = order.total_format_currency("$")?;
    total.ok_or(DecimalError::NullValue)
}

// Apply discount
fn apply_discount(order: &mut Order, discount_percent: f64) -> Result<(), DecimalError> {
    // discount_percent should be like 10.0 for 10%
    let discount_factor = 1.0 - (discount_percent / 100.0);
    order.total_mul_f64(discount_factor)?;
    order.total_round(2)?;
    Ok(())
}
```

**Example 2: Financial Account Balance**

```rust
#[derive(EnhancedCrud, Debug, Clone)]
struct Account {
    id: String,
    user_id: String,

    #[crud(decimal(precision = 15, scale = 2))]
    #[crud(cast_as = "TEXT")]
    balance: Option<String>,

    #[crud(decimal(precision = 15, scale = 2))]
    #[crud(cast_as = "TEXT")]
    available_balance: Option<String>,

    #[crud(decimal(precision = 15, scale = 2))]
    #[crud(cast_as = "TEXT")]
    frozen_amount: Option<String>,
}

// Check if sufficient funds
fn has_sufficient_funds(account: &Account, required_amount: &str) -> Result<bool, DecimalError> {
    let available = account.available_balance_as_f64_or(0.0)?;
    let required = required_amount.parse::<f64>()
        .map_err(|_| DecimalError::InvalidFormat(required_amount.to_string()))?;
    Ok(available >= required)
}

// Freeze funds
fn freeze_funds(account: &mut Account, amount: &str) -> Result<(), DecimalError> {
    // Subtract from available_balance
    account.available_balance_sub(amount)?;

    // Add to frozen_amount
    account.frozen_amount_add(amount)?;

    Ok(())
}

// Get formatted balance for display
fn get_balance_display(account: &Account) -> Result<String, DecimalError> {
    account.balance_format_currency("$")?.ok_or(DecimalError::NullValue)
}

// Validate balance doesn't exceed NUMERIC constraints
fn validate_account_balance(account: &Account) -> Result<bool, DecimalError> {
    account.balance_validate()?;
    account.available_balance_validate()?;
    account.frozen_amount_validate()?;
    Ok(true)
}
```

**Example 3: Product Pricing**

```rust
#[derive(EnhancedCrud, Debug, Clone)]
struct Product {
    id: String,
    name: String,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    base_price: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    discount_percent: Option<String>,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    sale_price: Option<String>,
}

// Calculate sale price with discount
fn calculate_sale_price(product: &mut Product) -> Result<(), DecimalError> {
    let base = product.base_price_as_f64_or(0.0)?;
    let discount = product.discount_percent_as_f64_or(0.0)?;

    // sale_price = base_price * (1 - discount_percent/100)
    let discount_factor = 1.0 - (discount / 100.0);
    product.sale_price = Some("0".to_string());
    product.sale_price_add_f64(base * discount_factor)?
                .sale_price_round(2)?;

    Ok(())
}

// Format price for display
fn format_product_price(product: &Product) -> Result<String, DecimalError> {
    let price = product.sale_price_format_currency("$")?.ok_or(DecimalError::NullValue)?;

    if product.discount_is_positive().unwrap_or(false) {
        Ok(format!("{} (was {})", price, product.base_price_format_currency("$")?.ok_or(DecimalError::NullValue)?))
    } else {
        Ok(price)
    }
}
```

**Example 4: Percentage Calculations**

```rust
#[derive(EnhancedCrud, Debug, Clone)]
struct Report {
    id: String,
    period: String,

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    growth_rate: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    profit_margin: Option<String>,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    revenue: Option<String>,
}

// Format growth rate as percentage
fn display_growth_rate(report: &Report) -> Result<String, DecimalError> {
    let growth = report.growth_rate_format_percent()?.ok_or(DecimalError::NullValue)?;

    if report.growth_rate_is_positive().unwrap_or(false) {
        Ok(format!("▲ {}", growth))  // "▲ 15.50%"
    } else if report.growth_rate_is_negative().unwrap_or(false) {
        Ok(format!("▼ {}", growth))  // "▼ 5.20%"
    } else {
        Ok(format!("＝ {}", growth))  // "＝ 0.00%"
    }
}

// Check if profitable
fn is_profitable(report: &Report) -> bool {
    report.profit_margin_is_positive().unwrap_or(false)
}
```

**Example 5: Bulk Operations with DECIMAL Fields**

```rust
// Apply price increase to multiple products
async fn bulk_price_increase(products: &mut Vec<Product>, increase_percent: f64) -> Result<(), DecimalError> {
    let multiplier = 1.0 + (increase_percent / 100.0);

    for product in products {
        product.base_price_mul_f64(multiplier)?
                .base_price_round(2)?;

        // Recalculate sale price
        calculate_sale_price(product)?;
    }

    Ok(())
}

// Validate all products have valid prices
fn validate_product_prices(products: &[Product]) -> Result<(), DecimalError> {
    for product in products {
        product.base_price_validate()?;

        if let Some(sale_price) = &product.sale_price {
            let sale = sale_price.parse::<f64>()
                .map_err(|_| DecimalError::InvalidFormat(sale_price.clone()))?;
            let base = product.base_price_as_f64_or(0.0)?;

            // Sale price should be less than or equal to base price
            if sale > base {
                return Err(DecimalError::Overflow {
                    value: sale_price.clone(),
                    precision: 10,
                    scale: 2,
                });
            }
        }
    }
    Ok(())
}
```

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
  - **DECIMAL/NUMERIC helper methods** - Auto-generated 26 helper methods for DECIMAL fields:
    - Type conversion (String → f64)
    - Chainable arithmetic operations (add, sub, mul, div, round, abs, neg)
    - Validation and value checks (is_positive, is_negative, is_zero, validate)
    - Formatting (format, format_currency, format_percent, truncate)
    - Precision information (precision, scale, max_value, min_value, clamp)
    - Support for both `String` and `Option<String>` field types

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
