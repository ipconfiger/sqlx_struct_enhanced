# Aggregation Query Examples

This directory contains three comprehensive example programs demonstrating the aggregation query features of `sqlx_struct_enhanced`.

## Examples Overview

### 1. aggregation_basics.rs
**Level:** Beginner
**Topics:** Basic aggregations, GROUP BY, custom aliases, WHERE clause

Demonstrates:
- Simple SUM, AVG, COUNT, MIN, MAX aggregations
- GROUP BY with single and multiple aggregates
- Custom column aliases (`sum_as`, `avg_as`, etc.)
- WHERE clause filtering
- COUNT variations (COUNT(*) vs COUNT(column))
- SQL caching demonstration

**Run:** `cargo run --example aggregation_basics`

### 2. aggregation_advanced.rs
**Level:** Intermediate
**Topics:** HAVING, ORDER BY, LIMIT/OFFSET, complex queries

Demonstrates:
- HAVING clause with aggregate functions and aliases
- WHERE + GROUP BY + HAVING combinations
- ORDER BY (ASC/DESC, case-insensitive)
- LIMIT for top-N queries
- OFFSET for pagination
- Complete complex queries combining all features
- Automatic parameter indexing

**Run:** `cargo run --example aggregation_advanced`

### 3. aggregation_real_world.rs
**Level:** Advanced
**Topics:** Real-world business use cases

Demonstrates:
- **E-Commerce Sales Dashboard** - Revenue by category with filtering
- **Customer Leaderboard** - Top 10 customers by total spend
- **Regional Performance Analysis** - Sales by geographic region
- **Website Analytics Dashboard** - Top pages by views
- **Inventory Valuation** - Category-based inventory summaries
- **Paginated Reports** - Page 2 of sales report with pagination

**Run:** `cargo run --example aggregation_real_world`

## Database Setup

All examples require a running PostgreSQL instance. By default, they connect to:

```
postgres://postgres:@127.0.0.1/test-sqlx-tokio
```

You can modify the connection string in each example's `main()` function:

```rust
let pool = PgPool::connect("postgres://user:pass@host/database").await?;
```

## Creating Test Tables

To run the examples, you'll need to create the appropriate tables:

### For aggregation_basics.rs and aggregation_advanced.rs:

```sql
CREATE TABLE order (
    id VARCHAR PRIMARY KEY,
    category VARCHAR NOT NULL,
    amount INTEGER NOT NULL,
    status VARCHAR NOT NULL
);

INSERT INTO order (id, category, amount, status) VALUES
    ('1', 'electronics', 1000, 'active'),
    ('2', 'electronics', 500, 'active'),
    ('3', 'clothing', 200, 'active'),
    ('4', 'electronics', 1500, 'completed'),
    ('5', 'clothing', 300, 'completed');
```

### For aggregation_real_world.rs:

```sql
CREATE TABLE sales_orders (
    id VARCHAR PRIMARY KEY,
    customer_id VARCHAR NOT NULL,
    product_category VARCHAR NOT NULL,
    amount INTEGER NOT NULL,
    status VARCHAR NOT NULL,
    region VARCHAR NOT NULL,
    created_at VARCHAR NOT NULL
);

CREATE TABLE website_events (
    id VARCHAR PRIMARY KEY,
    event_type VARCHAR NOT NULL,
    page_url VARCHAR NOT NULL,
    user_id VARCHAR,
    session_id VARCHAR NOT NULL,
    created_at VARCHAR NOT NULL
);

CREATE TABLE inventory_items (
    id VARCHAR PRIMARY KEY,
    product_name VARCHAR NOT NULL,
    category VARCHAR NOT NULL,
    quantity INTEGER NOT NULL,
    unit_cost INTEGER NOT NULL,
    warehouse_location VARCHAR NOT NULL
);
```

## Key Concepts Demonstrated

### Fluent API
All examples use the fluent builder pattern:

```rust
let sql = Order::agg_query()
    .where_("status = {}", &["active"])
    .group_by("category")
    .sum_as("amount", "total")
    .having("total > {}", &[&1000i64])
    .order_by("total", "DESC")
    .limit(10)
    .build();
```

### Result Structs
Using custom structs with `sqlx::FromRow` for type-safe results:

```rust
#[derive(FromRow)]
struct CategoryStats {
    category: String,
    total: i64,
    average: Option<f64>,
}

let stats: Vec<CategoryStats> = sqlx::query_as(sql)
    .bind("active")
    .bind(1000i64)
    .bind(10i64)
    .fetch_all(&pool)
    .await?;
```

### Parameter Binding
Automatic parameter indexing and binding:

```rust
// $1 = status
// $2 = HAVING value
// $3 = LIMIT value
let sql = Order::agg_query()
    .where_("status = {}", &["active"])
    .group_by("category")
    .sum("amount")
    .having("SUM(amount) > {}", &[&1000i64])
    .limit(10)
    .build();

let results = sqlx::query_as(sql)
    .bind("active")
    .bind(1000i64)
    .bind(10i64)
    .fetch_all(&pool)
    .await?;
```

## Learning Path

1. **Start with aggregation_basics.rs**
   - Run the example
   - Read through each example section
   - Modify queries and see results

2. **Move to aggregation_advanced.rs**
   - Understand HAVING vs WHERE
   - Learn ORDER BY and LIMIT
   - Practice pagination with OFFSET

3. **Study aggregation_real_world.rs**
   - See real-world applications
   - Understand table structure design
   - Adapt patterns to your use cases

## Common Patterns

### Top N Results
```rust
let sql = Order::agg_query()
    .group_by("category")
    .sum_as("amount", "total")
    .order_by("total", "DESC")
    .limit(10)
    .build();
```

### Pagination
```rust
let page = 2;
let per_page = 20;
let offset = (page - 1) * per_page;

let sql = Order::agg_query()
    .group_by("category")
    .sum("amount")
    .order_by("amount", "DESC")
    .limit(per_page)
    .offset(offset)
    .build();
```

### Filtering Groups
```rust
let sql = Order::agg_query()
    .where_("status = {}", &["active"])
    .group_by("category")
    .sum_as("amount", "total")
    .having("total > {}", &[&1000i64])
    .build();
```

## Troubleshooting

### Compilation Errors
If you see errors about missing imports, ensure you have:
```rust
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::{FromRow, PgPool, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;
```

### Connection Errors
- Verify PostgreSQL is running: `psql postgres://postgres:@127.0.0.1/test-sqlx-tokio`
- Check connection string in the example
- Ensure the database exists

### Empty Results
- Insert test data into the tables
- Verify data matches query conditions (status, category, etc.)

## Additional Resources

- [USAGE.md](../USAGE.md) - Complete API documentation
- [aggregate_tests.rs](../tests/aggregate_tests.rs) - Test cases with assertions
- [query_builder.rs](../src/aggregate/query_builder.rs) - Implementation details

## Contributing

To add new examples:
1. Create a new file: `examples/aggregation_YOUR_TOPIC.rs`
2. Follow the existing structure and style
3. Add imports and table setup instructions
4. Update this README with your example

---

**Last Updated:** Phase 2 Implementation - Complete aggregation query support with HAVING, ORDER BY, LIMIT/OFFSET
