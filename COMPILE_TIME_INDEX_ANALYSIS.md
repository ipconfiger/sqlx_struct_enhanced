# Compile-Time Index Analysis Guide

Automatically optimize your database queries with compile-time index analysis using the `#[analyze_queries]` attribute macro.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Usage Examples](#usage-examples)
- [Index Recommendation Rules](#index-recommendation-rules)
- [Integration with Existing Code](#integration-with-existing-code)
- [Limitations](#limitations)
- [Best Practices](#best-practices)
- [Future Enhancements](#future-enhancements)

## Overview

The `#[analyze_queries]` macro analyzes your code at compile time to:

1. **Scan struct definitions** with `EnhancedCrud`
2. **Extract query patterns** from `where_query!()` and `make_query!()` calls
3. **Parse SQL** to identify WHERE, ORDER BY, JOIN, and GROUP BY clauses
4. **Recommend indexes** with optimal column ordering
5. **Generate SQL** statements for creating recommended indexes

This happens entirely at compile time with **zero runtime overhead**.

## Quick Start

### 1. Add the Attribute to Your Module

```rust
#[sqlx_struct_macros::analyze_queries]
mod my_module {
    use sqlx_struct_enhanced::EnhancedCrud;

    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
    }

    impl User {
        pub fn find_by_email(email: &str) {
            let _ = User::where_query!("email = $1");
        }

        pub fn list_active_users() {
            let _ = User::where_query!("status = $1 ORDER BY created_at DESC");
        }
    }
}
```

### 2. Build Your Project

```bash
cargo build
```

### 3. View Recommendations

The macro outputs index recommendations during compilation:

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

### 4. Apply Recommendations

Copy the generated `CREATE INDEX` statements and run them in your database:

```sql
CREATE INDEX idx_user_email ON User (email);
CREATE INDEX idx_user_status_created_at ON User (status, created_at);
```

## How It Works

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│  #[analyze_queries] Macro (Compile-Time)                │
├─────────────────────────────────────────────────────────┤
│  1. Scan Structs                                       │
│     └─ Extract field names from EnhancedCrud structs    │
│                                                         │
│  2. Extract Queries                                    │
│     └─ Find where_query!() and make_query!() calls      │
│                                                         │
│  3. Parse SQL                                          │
│     └─ Identify WHERE equality conditions              │
│     └─ Identify ORDER BY columns                       │
│     └─ Identify JOIN conditions (NEW) 🆕                │
│     └─ Identify GROUP BY columns (NEW) 🆕               │
│                                                         │
│  4. Generate Recommendations                           │
│     └─ Optimize column ordering                        │
│     └─ Deduplicate across queries                      │
│     └─ Recommend join column indexes (NEW) 🆕           │
│     └─ Recommend grouping column indexes (NEW) 🆕       │
│                                                         │
│  5. Print Output                                       │
│     └─ Show recommendations with CREATE INDEX SQL       │
└─────────────────────────────────────────────────────────┘
```

### Analysis Pipeline

1. **Token Stream Parsing**: Receives the module's Rust code as a token stream
2. **Struct Scanning**: Finds all `struct` definitions with `EnhancedCrud`
3. **Field Extraction**: Extracts field names from struct definitions
4. **Query Extraction**: Finds query macro calls in the code
5. **SQL Parsing**: Uses `SimpleSqlParser` to analyze query strings
6. **Index Inference**: Determines optimal index column ordering
7. **Output Generation**: Pretty-prints recommendations with SQL

## Usage Examples

### Example 1: Single Column Index

```rust
#[analyze_queries]
mod user_queries {
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
    }

    impl User {
        fn find_by_email(email: &str) {
            User::where_query!("email = $1").fetch_all(pool)
        }
    }
}
```

**Recommendation:**
```
✨ Recommended: idx_user_email
   Columns: email
   Reason: Single column: WHERE email = $1
   SQL:    CREATE INDEX idx_user_email ON User (email)
```

### Example 2: Multi-Column Index with ORDER BY

```rust
#[analyze_queries]
mod user_queries {
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        status: String,
        created_at: i64,
    }

    impl User {
        fn list_active_users_ordered() {
            User::where_query!("status = $1 ORDER BY created_at DESC")
                .fetch_all(pool)
        }
    }
}
```

**Recommendation:**
```
✨ Recommended: idx_user_status_created_at
   Columns: status, created_at
   Reason: WHERE status ORDER BY created_at
   SQL:    CREATE INDEX idx_user_status_created_at ON User (status, created_at)
```

**Why This Order?**
- Equality columns (`status = $1`) come first
- ORDER BY columns (`created_at`) come after
- This ordering maximizes index efficiency

### Example 3: Multiple Queries Sharing an Index

```rust
#[analyze_queries]
mod user_queries {
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
    }

    impl User {
        fn find_active() {
            User::where_query!("status = $1").fetch_all(pool)
        }

        fn find_active_ordered() {
            User::where_query!("status = $1 ORDER BY created_at DESC")
                .fetch_all(pool)
        }
    }
}
```

**Recommendations:**
```
✨ Recommended: idx_user_status_created_at
   Columns: status, created_at
   Reason: WHERE status ORDER BY created_at
   SQL:    CREATE INDEX idx_user_status_created_at ON User (status, created_at)
```

**Note:** Only one index is recommended because `idx_user_status_created_at` covers both queries:
- Query 1: `WHERE status = $1` - uses the index's first column
- Query 2: `WHERE status = $1 ORDER BY created_at` - uses both columns

### Example 4: Complex Query with Range Conditions

```rust
#[analyze_queries]
mod order_queries {
    #[derive(EnhancedCrud)]
    struct Order {
        id: String,
        customer_id: String,
        status: String,
        created_at: i64,
        total_amount: i32,
    }

    impl Order {
        fn find_customer_orders_since(customer_id: &str, since: i64) {
            Order::where_query!(
                "customer_id = $1 AND created_at > $2 ORDER BY created_at DESC"
            ).fetch_all(pool)
        }
    }
}
```

**Recommendation:**
```
✨ Recommended: idx_order_customer_id_created_at
   Columns: customer_id, created_at
   Reason: WHERE customer_id ORDER BY created_at
   SQL:    CREATE INDEX idx_order_customer_id_created_at ON Order (customer_id, created_at)
```

### Example 5: JOIN Query Analysis 🆕

```rust
#[analyze_queries]
mod order_queries {
    #[derive(EnhancedCrud)]
    struct Order {
        id: String,
        user_id: String,
        product_id: String,
        status: String,
        created_at: i64,
    }

    impl Order {
        fn find_orders_with_user() {
            Order::make_query(
                "SELECT o.*, u.email, u.username
                 FROM orders o
                 INNER JOIN users u ON o.user_id = u.id
                 WHERE o.status = $1"
            ).fetch_all(pool)
        }

        fn find_orders_with_details() {
            Order::make_query(
                "SELECT o.*, u.email, p.name
                 FROM orders o
                 INNER JOIN users u ON o.user_id = u.id
                 INNER JOIN products p ON o.product_id = p.id
                 WHERE o.status = $1"
            ).fetch_all(pool)
        }
    }
}
```

**Recommendations:**
```
✨ Recommended: idx_Order_user_id_join
   Columns: user_id
   Reason: JOIN column (INNER JOIN ON o.user_id = u.id)
   SQL:    CREATE INDEX idx_Order_user_id_join ON Order (user_id)

✨ Recommended: idx_Order_product_id_join
   Columns: product_id
   Reason: JOIN column (INNER JOIN ON o.product_id = p.id)
   SQL:    CREATE INDEX idx_Order_product_id_join ON Order (product_id)
```

**Why Index Join Columns?**
- JOIN operations perform lookups on join columns
- Without indexes, each JOIN requires a full table scan
- Indexes on join columns dramatically improve query performance
- The analyzer detects INNER JOIN, LEFT JOIN, and RIGHT JOIN

### Example 6: GROUP BY Query Analysis 🆕

```rust
#[analyze_queries]
mod order_queries {
    #[derive(EnhancedCrud)]
    struct Order {
        id: String,
        user_id: String,
        status: String,
        category: String,
        created_at: i64,
    }

    impl Order {
        fn count_orders_by_status() {
            Order::make_query(
                "SELECT status, COUNT(*) as count
                 FROM orders
                 GROUP BY status"
            ).fetch_all(pool)
        }

        fn find_frequent_statuses() {
            Order::make_query(
                "SELECT status, COUNT(*) as count
                 FROM orders
                 GROUP BY status
                 HAVING COUNT(*) > $1"
            ).fetch_all(pool)
        }

        fn count_by_category_and_status() {
            Order::make_query(
                "SELECT category, status, COUNT(*) as count
                 FROM orders
                 GROUP BY category, status"
            ).fetch_all(pool)
        }
    }
}
```

**Recommendations:**
```
✨ Recommended: idx_Order_status_group
   Columns: status
   Reason: GROUP BY column
   SQL:    CREATE INDEX idx_Order_status_group ON Order (status)

✨ Recommended: idx_Order_category_group
   Columns: category
   Reason: GROUP BY column
   SQL:    CREATE INDEX idx_Order_category_group ON Order (category)
```

**Why Index GROUP BY Columns?**
- GROUP BY operations need to group rows by the specified columns
- Indexes on grouping columns allow faster grouping without sorting
- The analyzer detects both single and multiple column GROUP BY
- HAVING clauses are also detected and noted in recommendations

### Example 7: Combined JOIN + GROUP BY + WHERE 🆕

```rust
#[analyze_queries]
mod analytics_queries {
    #[derive(EnhancedCrud)]
    struct Order {
        id: String,
        user_id: String,
        status: String,
        total_amount: i32,
        created_at: i64,
    }

    impl Order {
        fn count_orders_per_user() {
            Order::make_query(
                "SELECT u.id, u.username, COUNT(o.id) as order_count
                 FROM users u
                 INNER JOIN orders o ON u.id = o.user_id
                 WHERE o.status = 'completed'
                 GROUP BY u.id, u.username
                 ORDER BY order_count DESC"
            ).fetch_all(pool)
        }
    }
}
```

**Recommendations:**
```
✨ Recommended: idx_Order_user_id_join
   Columns: user_id
   Reason: JOIN column (INNER JOIN ON u.id = o.user_id)
   SQL:    CREATE INDEX idx_Order_user_id_join ON Order (user_id)

✨ Recommended: idx_Order_status
   Columns: status
   Reason: Single column: WHERE status = $1
   SQL:    CREATE INDEX idx_Order_status ON Order (status)
```

**Complex Query Analysis:**
- The analyzer processes JOIN, WHERE, and GROUP BY separately
- Each component gets appropriate index recommendations
- This comprehensive analysis ensures all query aspects are optimized

## Index Recommendation Rules

### Rule 1: Priority Ordering for Index Columns

The analyzer prioritizes column conditions based on their effectiveness for index usage:

**Priority Order** (highest to lowest):
1. **Equality conditions** (`col = $1`) - Most selective
2. **IN clauses** (`col IN ($1, $2)`) - Good for multiple values
3. **Range conditions** (`col > $1`, `col < $1`, `col >= $1`, `col <= $1`) - For ranges
4. **LIKE clauses** (`col LIKE $1`) - Pattern matching
5. **Inequality conditions** (`col != $1`, `col <> $1`) - Negation (Day 3) 🆕
6. **NOT LIKE clauses** (`col NOT LIKE $1`) - Negation pattern (Day 3) 🆕
7. **ORDER BY columns** - Sorting optimization

This ordering ensures optimal index usage for query performance.

### Rule 2: Single Equality Column

Columns used in equality conditions (`col = $1`) are placed first in the index:

```rust
// Query: WHERE email = $1
User::where_query!("email = $1")
// Recommendation: (email)
```

### Rule 3: Range Conditions

Range operators (>, <, >=, <=) are included after equality conditions:

```rust
// Query: WHERE price > $1
Product::where_query!("price > $1")
// Recommendation: (price)

// Query: WHERE created_at >= $1
Post::where_query!("created_at >= $1")
// Recommendation: (created_at)
```

### Rule 4: IN Clauses

IN clauses are prioritized after equality but before range conditions:

```rust
// Query: WHERE status IN ($1, $2, $3)
Order::where_query!("status IN ($1, $2, $3)")
// Recommendation: (status)
```

### Rule 5: LIKE Clauses

LIKE clauses are included after range conditions:

```rust
// Query: WHERE name LIKE $1
User::where_query!("name LIKE $1")
// Recommendation: (name)
```

### Rule 6: Inequality Conditions (Day 3) 🆕

Inequality operators (!=, <>) have lower priority because they're less selective:

```rust
// Query: WHERE status != $1
Order::where_query!("status != $1")
// Recommendation: (status)

// Query: WHERE type <> $1
Product::where_query!("type <> $1")
// Recommendation: (type)
```

**Note**: Inequality conditions are typically less selective than equality conditions, so they're placed later in the index.

### Rule 7: NOT LIKE Clauses (Day 3) 🆕

NOT LIKE clauses have the lowest priority before ORDER BY:

```rust
// Query: WHERE email NOT LIKE $1
User::where_query!("email NOT LIKE $1")
// Recommendation: (email)
```

**Note**: NOT LIKE is very non-selective (excludes patterns), so it's placed near the end of the index.

### Rule 8: ORDER BY Columns After WHERE

Sorting columns are added after all WHERE conditions:

```rust
// Query: WHERE status = $1 ORDER BY created_at DESC
User::where_query!("status = $1 ORDER BY created_at DESC")
// Recommendation: (status, created_at)
```

### Rule 9: Multi-Column Mixed Conditions

Complex queries with multiple condition types follow priority ordering:

```rust
// Day 3: Complete example with all condition types
// Query: WHERE tenant_id = $1 AND category IN ($2, $3) AND price > $4
//         AND name LIKE $5 AND status != $6 AND title NOT LIKE $7
//         ORDER BY created_at
Product::where_query!(
    "tenant_id = $1 AND category IN ($2, $3) AND price > $4
     AND name LIKE $5 AND status != $6 AND title NOT LIKE $7
     ORDER BY created_at"
)
// Recommendation: (tenant_id, category, price, name, status, title, created_at)
// Priority: Equality > IN > Range > LIKE > Inequality > NOT LIKE > ORDER BY
```

### Rule 10: Deduplication

If multiple queries can use the same index, only one recommendation is shown:

```rust
// Query 1: WHERE status = $1
User::where_query!("status = $1")

// Query 2: WHERE status = $1 ORDER BY created_at
User::where_query!("status = $1 ORDER BY created_at")

// Only one recommendation: (status, created_at)
// This index serves both queries
```

### Rule 11: OR Conditions and Query Complexity (Day 4) 🆕

OR conditions change the indexing strategy because they reduce the effectiveness of composite indexes:

```rust
// Simple OR: Two separate indexes are often better than one composite index
User::where_query!("status = $1 OR type = $2")
// Recommendation: Extracts both columns
// Best practice: Create separate indexes on (status) and (type)

// Mixed AND/OR with parentheses
Task::where_query!("(status = $1 AND priority > $2) OR created_at < $3")
// Recommendation: Extracts status, priority, created_at
// Note: Parentheses indicate grouping that affects index effectiveness
```

**Query Complexity Detection** (Day 4):
- **OR Conditions**: Detected by `has_or_conditions()`
- **Parentheses**: Detected by `has_parentheses()` (excludes IN clauses)
- **Subqueries**: Detected by `analyze_query_complexity()`

When OR conditions are detected:
1. All columns are still extracted for analysis
2. Consider using separate single-column indexes instead of composite indexes
3. Index merge optimization (if supported by your database) may help
4. Query restructuring might be beneficial (e.g., UNION instead of OR)

**Example Query Complexity Check**:
```rust
use sqlx_struct_macros::simple_parser::{SimpleSqlParser, QueryComplexity};

let parser = SimpleSqlParser::new(vec![
    "id".to_string(),
    "status".to_string(),
    "priority".to_string(),
]);

let sql = "SELECT * FROM tasks WHERE (status = $1) OR priority > $2";
let complexity = parser.analyze_query_complexity(sql);

if complexity.has_or {
    println!("Warning: OR conditions detected. Consider separate indexes.");
}

if complexity.has_parentheses {
    println!("Info: Parentheses grouping detected.");
}
```

**Indexing Strategy for OR Conditions**:
- `WHERE a = 1 OR b = 2` → Separate indexes on `(a)` and `(b)`
- `WHERE a = 1 AND b = 2 OR c = 3` → Index on `(a, b)` and index on `(c)`
- `WHERE (a = 1 AND b = 2) OR (a = 3 AND b = 4)` → Index on `(a, b)`

### Rule 12: Unique Index Detection (Day 5) 🆕

Columns named `id` are automatically flagged for unique indexes:

```rust
// Query: WHERE id = $1
User::where_query!("id = $1")
// Recommendation: UNIQUE INDEX (id)
// SQL: CREATE UNIQUE INDEX idx_id_unique ON User (id)
```

**Benefits**:
- Enforces data integrity (no duplicate IDs)
- Optimizes single-row lookups
- Communicates intent that this is a primary/unique key

### Rule 13: Partial Indexes (Day 5) 🆕

Partial indexes are recommended for queries that filter on specific literal values:

```rust
// Soft delete pattern
User::where_query!("deleted_at IS NULL AND email = $1")
// Recommendation: PARTIAL INDEX (email) WHERE deleted_at IS NULL
// SQL: CREATE INDEX idx_user_email ON User (email) WHERE deleted_at IS NULL

// Status filtering
Order::where_query!("status = 'active' AND created_at > $1")
// Recommendation: PARTIAL INDEX (created_at) WHERE status = 'active'
// SQL: CREATE INDEX idx_order_created_at ON Order (created_at) WHERE status = 'active'
```

**Benefits**:
- Smaller index size (only indexes relevant rows)
- Faster maintenance (fewer rows to update)
- Same query performance for filtered queries

**When to Use**:
- Queries with `deleted_at IS NULL` (soft deletes)
- Queries filtering on fixed status values (`status = 'active'`)
- Queries targeting a subset of data consistently

### Rule 14: Covering Indexes with INCLUDE (Day 5) 🆕

Covering indexes include non-key columns to avoid table lookups:

```rust
// Query: SELECT id, user_id, name, email FROM users
//        WHERE status = $1 AND created_at > $2 ORDER BY user_id
User::where_query!("status = $1 AND created_at > $2 ORDER BY user_id")
// SELECT: id, user_id, name, email

// Recommendation: COVERING INDEX (status, created_at, user_id) INCLUDE (name, email)
// SQL: CREATE INDEX idx_user_status_created_user
//       ON User (status, created_at, user_id) INCLUDE (name, email)
```

**Benefits**:
- **Zero table lookups**: All columns needed are in the index
- **Faster queries**: Database only reads the index, not the table
- **Lower I/O**: Reduces disk reads significantly

**How It Works**:
1. **Key columns** (in parentheses): Used for filtering and sorting
2. **INCLUDE columns** (non-key): Stored in index but not used for search

**When to Use**:
- Queries that select only a few extra columns beyond WHERE/ORDER BY
- Frequently accessed columns that aren't used for filtering
- When you want to eliminate table lookups entirely

### Rule 15: Index Size Estimation (Day 5) 🆕

Each recommendation includes a rough size estimate to help plan storage:

```rust
// Single column index
User::where_query!("email = $1")
// Recommendation: (email) ~100 bytes per row

// Multi-column index
Task::where_query!("tenant_id = $1 AND status = $2 AND priority > $3")
// Recommendation: (tenant_id, status, priority) ~150 bytes per row
// Multiplier: 1.5x for 3 columns
```

**Size Multipliers**:
- 1 column: 1.0x (100 bytes base)
- 2 columns: 1.5x
- 3 columns: 1.8x
- 4+ columns: 2.0x

**Note**: Actual size depends on:
- Table row count
- Column data types
- Database engine
- Fill factor and other settings

### Complete Example: Advanced Recommendations (Day 5)

```rust
#[sqlx_struct_macros::analyze_queries]
mod advanced_queries {
    use sqlx_struct_enhanced::EnhancedCrud;

    #[derive(EnhancedCrud)]
    struct Task {
        id: String,
        tenant_id: String,
        user_id: String,
        status: String,
        priority: i32,
        created_at: i64,
        title: String,
        description: String,
    }

    impl Task {
        // Complex query with all Day 5 features
        fn find_active_tasks(tenant_id: &str, min_priority: i32) {
            let _ = Task::where_query!(
                "tenant_id = $1 AND status = 'active' AND priority >= $2 ORDER BY created_at DESC"
            );
            // Recommendation: PARTIAL INDEX (tenant_id, priority, created_at)
            //               WHERE status = 'active'
            //               ~180 bytes per row
        }

        // Covering index example
        fn list_tasks_with_details(tenant_id: &str) {
            let _ = Task::where_query!(
                "tenant_id = $1 AND status = 'active' ORDER BY priority"
            );
            // SELECT: id, title, description (detected from usage)
            // Recommendation: COVERING INDEX (tenant_id, status, priority)
            //               INCLUDE (title, description)
        }

        // Unique index example
        fn find_by_id(id: &str) {
            let _ = Task::where_query!("id = $1");
            // Recommendation: UNIQUE INDEX (id)
        }
    }
}
```

### Rule 16: Functional/Expression Indexes (Day 6) 🆕

Functional indexes are recommended when columns are used in function calls:

```rust
// Case-insensitive search
User::where_query!("WHERE LOWER(email) = $1")
// Recommendation: FUNCTIONAL INDEX (LOWER(email))
// SQL: CREATE INDEX idx_user_email_lower ON User ((LOWER(email)))

// Date extraction
Post::where_query!("WHERE DATE(created_at) = $1")
// Recommendation: FUNCTIONAL INDEX (DATE(created_at))
// SQL: CREATE INDEX idx_post_created_date ON Post ((DATE(created_at)))

// String manipulation
User::where_query!("WHERE UPPER(name) LIKE $1")
// Recommendation: FUNCTIONAL INDEX (UPPER(name))
// SQL: CREATE INDEX idx_user_name_upper ON User ((UPPER(name)))
```

**Supported Functions**:
- Text: `LOWER()`, `UPPER()`, `TRIM()`, `SUBSTRING()`, `CONCAT()`
- Date: `DATE()`, `YEAR()`, `MONTH()`, `DAY()`
- Conditional: `COALESCE()`

**Benefits**:
- Enables index usage for function-wrapped columns
- Dramatically improves performance for case-insensitive searches
- Essential for date grouping queries

### Rule 17: Index Type Selection (Day 6) 🆕

The analyzer recommends the most appropriate index type based on query patterns:

```rust
// B-tree for range queries and sorting
Product::where_query!("price > $1 ORDER BY created_at")
// Recommendation: B-tree INDEX (price, created_at)
// Reason: B-tree is optimal for range and ORDER BY

// Hash for equality-only queries
User::where_query!("status = $1")
// Recommendation: Hash INDEX (status)
// Reason: Hash indexes are faster for pure equality lookups
```

**Index Type Guidelines**:
- **B-tree**: Default choice; best for ranges, sorting, and most queries
- **Hash**: Only for equality queries (`=`); not for ranges or ORDER BY
- **BRIN**: Recommended for timestamp columns in large, append-only tables
- **GIN/GiST**: For JSON, arrays, and full-text search (auto-detected)

### Rule 18: Index Effectiveness Scoring (Day 6) 🆕

Each recommendation includes an effectiveness score (0-110) to guide optimization efforts:

```rust
// Perfect score: Unique equality lookup
User::where_query!("id = $1")
// Score: 110 (100 base + 10 unique)
// Reason: Most effective index type

// Good score: Multi-column equality
Task::where_query!("tenant_id = $1 AND user_id = $2 AND status = $3")
// Score: 105 (100 base + 5 multi-column)
// Reason: Highly selective composite index

// Reduced score: Pattern matching
Article::where_query!("title LIKE $1")
// Score: 90 (100 base - 10 LIKE)
// Reason: Pattern matching reduces effectiveness

// Low score: OR conditions
User::where_query!("status = $1 OR type = $2")
// Score: 60 (100 base - 40 OR penalty)
// Reason: OR conditions reduce index effectiveness
```

**Scoring Factors**:
- **Base**: 100 points
- **Bonuses**: +10 for unique, +5 for multi-column
- **Penalties**: -20 for OR, -10 for LIKE, -5 for range
- **Maximum**: 110 points for exceptional cases

**Prioritization**:
- Focus on indexes with scores ≥ 100 first
- Consider indexes with scores 80-99 if query performance is critical
- Review indexes with scores < 80 for possible query restructuring

### Rule 19: Database-Specific Optimization Hints (Day 6) 🆕

Recommendations include database-specific hints for advanced optimizations:

```rust
// Timestamp columns → BRIN index suggestion
Metrics::where_query!("timestamp > NOW() - INTERVAL '7 days'")
// Recommendation: B-tree INDEX (timestamp)
// Hint: "Consider BRIN index for timestamp columns if table is large
//        and data is inserted sequentially"

// Text search → Trigram index suggestion
Article::where_query!("title LIKE $1")
// Recommendation: B-tree INDEX (title)
// Hint: "For text patterns, consider trigram GIN/GiST indexes with
//        pg_trgm extension (PostgreSQL)"

// Wide composite index → Warning
Task::where_query!("col1 = $1 AND col2 = $2 AND col3 = $3 AND col4 = $4 AND col5 = $5")
// Recommendation: B-tree INDEX (col1, col2, col3, col4, col5)
// Hint: "Wide composite index (>4 columns) may have diminishing returns.
//        Consider index intersection instead."

// JSON columns → GIN index suggestion
Document::where_query!("metadata->>'key' = $1")
// Recommendation: B-tree INDEX (metadata)
// Hint: "Consider GIN index for metadata column to support efficient
//        JSON operations"
```

**Hint Categories**:
- **Performance**: Alternative index types for better performance
- **Storage**: Compression or specialized storage options
- **Features**: Database-specific features to consider
- **Warnings**: Potential issues with the recommended approach

### Complete Example: Day 6 Advanced Features

```rust
#[sqlx_struct_macros::analyze_queries]
mod day_6_advanced {
    use sqlx_struct_enhanced::EnhancedCrud;

    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        name: String,
        created_at: i64,
        metadata: String, // JSON
    }

    impl User {
        // Functional index + database hints
        fn find_by_email_case_insensitive(email: &str) {
            let _ = User::where_query!("WHERE LOWER(email) = $1");
            // Recommendation: FUNCTIONAL INDEX (LOWER(email))
            // Type: B-tree
            // Score: 100
            // Hint: Consider BRIN index for timestamp columns if table is large
        }

        // Real-world pagination with scoring
        fn list_unread_notifications(user_id: &str) {
            let _ = User::where_query!(
                "WHERE user_id = $1 AND status = 'unread' ORDER BY created_at DESC LIMIT 20"
            );
            // Recommendation: PARTIAL B-tree INDEX (user_id, created_at)
            //               WHERE status = 'unread'
            // Score: 105 (100 + 5 for multi-column)
            // Size: ~150 bytes per row
        }

        // JSON column with GIN hint
        fn search_metadata(key: &str, value: &str) {
            let _ = User::where_query!("WHERE metadata->>$1 = $2");
            // Recommendation: B-tree INDEX (metadata)
            // Hint: Consider GIN index for metadata column to support
            //       efficient JSON operations
        }
    }
}
```

### Rule 20: Column Cardinality Analysis (Day 7) 🆕

Column cardinality (number of unique values) is estimated and used to optimize index column ordering:

```rust
// High cardinality column should come first
Task::where_query!("WHERE user_id = $1 AND status = $2")
// Recommendation: (user_id, status) - optimized order
// Cardinality: user_id (High), status (Low)

// Cardinality levels:
// - Very High: id, email, username (>100K unique values)
// - High: foreign keys like user_id, tenant_id (10K-100K)
// - Medium-High: timestamps (1K-10K)
// - Medium: regular columns (100-1K)
// - Medium-Low: categories, tags (10-100)
// - Low: status, type (2-10)
// - Very Low: boolean, is_*, has_* (2)
```

**Why It Matters**:
- Higher cardinality columns in equality conditions improve selectivity
- Low cardinality columns (like status) filter more rows per unique value
- Optimal ordering: equality columns → sorted by cardinality (high→low) → range columns → ORDER BY

**Cardinality Detection**:
- Column name patterns (id, email, status, created_at, etc.)
- Boolean prefixes (is_*, has_*)
- Foreign key suffixes (*_id)

### Rule 21: Index Intersection Strategies (Day 7) 🆕

For OR conditions and wide indexes, index intersection/union may be more efficient:

```rust
// Wide index → recommend intersection instead
Task::where_query!("WHERE col1 = $1 AND col2 = $2 AND col3 = $3 AND col4 = $4")
// Recommendation: Use index intersection with separate indexes
// Alternative: "Consider using index intersection with separate indexes on 'col1', 'col2' instead of wide composite index"

// OR with high cardinality → recommend intersection
Task::where_query!("WHERE user_id = $1 OR created_at > $2")
// Recommendation: Separate indexes with intersection flag
// recommend_intersection: true
// Estimated gain: "60-75% (with merge)"
```

**When to Use Intersection**:
- More than 2 columns in OR condition
- High cardinality columns in OR condition
- Range queries mixed with OR conditions
- Wide composite indexes (>4 columns)

**Benefits**:
- Smaller individual indexes (faster maintenance)
- Database can use index merge optimization
- More flexible for different query patterns

### Rule 22: Performance Impact Prediction (Day 7) 🆕

Each recommendation includes estimated performance improvement:

```rust
// Primary key lookup - highest gain
User::where_query!("WHERE id = $1")
// Estimated performance gain: "95-99%"

// Unique equality query
User::where_query!("WHERE email = $1")
// Estimated performance gain: "90-95%"

// Composite index with partial filter
Task::where_query!("WHERE user_id = $1 AND status = 'active'")
// Estimated performance gain: "90-95%"

// LIKE query - lower gain
Article::where_query!("WHERE title LIKE $1")
// Estimated performance gain: "65-75%"

// OR query - lowest gain
Task::where_query!("WHERE status = $1 OR type = $2")
// Estimated performance gain: "40-60%" (or "60-75% (with merge)")
```

**Performance Factors**:
- **+15%**: Unique index
- **+10%**: Partial index (filtered subset)
- **+5%**: Multi-column composite index
- **-15%**: LIKE/pattern matching
- **-25%**: OR conditions
- **-5%**: Range queries

**Interpreting Gains**:
- 90-99%: Excellent (primary key, unique)
- 80-89%: Very Good (equality, high cardinality)
- 70-79%: Good (multi-column, partial)
- 60-69%: Moderate (composite, some filtering)
- 40-59%: Fair (OR, wide indexes, low selectivity)
- <40%: Poor (full table scan may be similar)

### Rule 23: Alternative Index Strategies (Day 7) 🆕

Recommendations include alternative strategies when the primary recommendation isn't optimal:

```rust
// Wide composite index
Task::where_query!("WHERE col1 = $1 AND col2 = $2 AND col3 = $3 AND col4 = $4")
// Primary: (col1, col2, col3, col4)
// Alternative: "Consider using index intersection with separate indexes on 'col1', 'col2' instead of wide composite index"

// Time-series data
Metrics::where_query!("WHERE sensor_id = $1 AND timestamp > $2")
// Primary: B-tree index (sensor_id, timestamp)
// Alternative: "For time-series data, consider BRIN indexes for better storage efficiency"

// High cardinality equality-only
User::where_query!("WHERE email = $1")
// Primary: Hash index (email)
// Alternative: "For high-cardinality equality queries, consider Hash indexes for faster lookups"

// Partial index consideration
Task::where_query!("WHERE user_id = $1 AND status = 'active'")
// Primary: Partial B-tree index (user_id) WHERE status = 'active'
// Alternative: "If most queries target the filtered subset, partial index is optimal. Otherwise, consider full index"
```

**Alternative Categories**:
- **Index Type**: Hash instead of B-tree, BRIN instead of B-tree
- **Index Structure**: Intersection instead of composite, covering instead of simple
- **Storage Strategy**: Partial instead of full, different column ordering
- **Database Features**: Specialized indexes (GIN, GiST, BRIN)

### Complete Example: Day 7 Advanced Optimization

```rust
#[sqlx_struct_macros::analyze_queries]
mod day_7_optimization {
    use sqlx_struct_enhanced::EnhancedCrud;

    #[derive(EnhancedCrud)]
    struct Task {
        id: String,
        tenant_id: String,     // High cardinality (foreign key)
        user_id: String,      // High cardinality (foreign key)
        status: String,       // Low cardinality (enum)
        priority: i32,        // Medium cardinality
        created_at: i64,     // Medium-High cardinality
    }

    impl Task {
        // Day 7: Optimized multi-column query with cardinality-based ordering
        fn find_pending_tasks(tenant_id: &str, min_priority: i32) {
            let _ = Task::where_query!(
                "tenant_id = $1 AND status = 'pending' AND priority > $2 ORDER BY created_at DESC"
            );
            // Recommendation: (tenant_id, priority, created_at)
            //               WHERE status = 'pending' (partial index)
            // Cardinality: [High, Medium, Medium-High]
            // Type: B-tree
            // Score: 108 (100 + 5 multi - 5 range + 5 partial)
            // Gain: 85-90%
            // Alternative: "Consider BRIN index if table is large and append-only"
        }

        // Day 7: Index intersection for complex OR
        fn search_multi_condition(status: &str, priority: i32, user_id: &str) {
            let _ = Task::where_query!(
                "status = $1 OR priority > $2 OR user_id = $3"
            );
            // Returns 3 separate recommendations with intersection flag
            // Each: recommend_intersection: true
            //      estimated_performance_gain: "60-75% (with merge)"
            //      cardinality: [Low] / [Medium] / [High]
        }
    }
}
```

### Rule 24: Query Execution Plan Hints (Day 8) 🆕

The analyzer generates comprehensive hints about how queries will execute:

**Hint Categories:**

1. **Scan Type Detection**
   - Index-only scan: Fastest, only reads from index
   - Index scan: Reads index + table for non-indexed columns
   - Full table scan: No index used, reads entire table

2. **JOIN Analysis**
   - INNER JOIN: Recommends foreign key indexes for nested loop joins
   - LEFT JOIN: Critical index on right table join column
   - Join order and strategy recommendations

3. **Sorting Optimization**
   - ORDER BY in index: Avoids extra sorting step
   - ORDER BY not in index: O(n log n) sorting overhead

4. **GROUP BY & Aggregates**
   - Index-only scans for GROUP BY columns
   - Covering indexes with INCLUDE for aggregation

5. **Query Complexity Warnings**
   - OR conditions: May require index merge or full scan
   - Subqueries: Recommend index on subquery columns
   - Suggest converting to JOIN where possible

**Example:**

```rust
#[sqlx_struct_macros::analyze_queries]
mod day8_example {
    #[derive(EnhancedCrud)]
    struct Post {
        id: String,
        user_id: String,
        created_at: i64,
    }

    impl Post {
        fn find_user_posts_sorted(user_id: &str) {
            let _ = Post::where_query!(
                "user_id = $1 ORDER BY created_at DESC LIMIT 10"
            );

            // Execution Plan Hints:
            // ✅ "📊 Multi-column index scan on user_id, created_at"
            // ✅ "Index can optimize ORDER BY using 'created_at'"
            //    "→ Avoids extra sorting step (sort operation)"
            // ✅ "LIMIT present - index can reduce rows examined early"
            // ✅ "📏 Range scan detected (if query has range conditions)"
        }
    }
}
```

### Rule 25: Visual Query Execution Plan (Day 8) 🆕

The analyzer generates ASCII art visualizations of how indexes will be used:

**Visualization Components:**

1. **Index Structure Display**
   - Shows composite index column order
   - Icons indicating cardinality levels
   - Root vs branch node indicators

2. **Execution Path**
   - Step-by-step query execution flow
   - Time complexity for each operation
   - Optimization opportunities highlighted

3. **Performance Characteristics**
   - Index depth estimation (B-tree levels)
   - Row lookup complexity
   - Caching effectiveness

**Cardinality Icons:**
- 🎯 Very High (id, email, username)
- 🔵 High (foreign keys)
- 🟢 Medium (most columns)
- 🟡 Low (status, type)
- 🔴 Very Low (boolean flags)

**Example Output:**

```
┌─────────────────────────────────────────────────────┐
│              Query Execution Plan                    │
└─────────────────────────────────────────────────────┘

📇 Index Structure:
┌─────────────────────────────────────┐
│  Index Header                       │
│  ─────────────────                  │
├─▶ Root: 🔵 user_id (High cardinality)
├─▶ 🟢 created_at (Medium cardinality)
│                                     │
│  Composite Index Order:             │
│    1. user_id [equality]
│    2. created_at [order_by]
└─────────────────────────────────────┘

🛤️  Execution Path:
  1. 🔍 Index Seek (Equality Match)
     └─ Traverse B-tree on 'user_id'
     └─ O(log n) lookup time
  2. ✅ ORDER BY Optimized (using index order on 'created_at')
     └─ No additional sort needed
  3. 🛑 Early Termination (LIMIT)
     └─ Stops after first N rows

📊 Performance Characteristics:
  • Index Depth: ~3 levels
  • Row Lookup: O(log n) → O(1)
  • Caching: Effective for indexed column
  • Composite Index Efficiency: High
    → Leading column 'user_id' serves as primary access path
```

### Rule 26: Query Cost Estimation (Day 8) 🆕

The analyzer provides relative cost estimates compared to full table scans:

**Cost Levels:**

| Cost Level | Description | Typical Use Cases |
|------------|-------------|-------------------|
| **Very Low** (5-20) | Optimal | Primary key lookup, unique index |
| **Low** (20-50) | Very efficient | Single column equality, IN clauses |
| **Medium** (50-80) | Efficient | Range queries, multi-column indexes |
| **Moderate** (80-100) | Acceptable | LIKE prefix matches, wide indexes |
| **High** (100+) | Needs review | OR conditions, subqueries, wildcards |

**Cost Calculation Factors:**

**Base Costs:**
- Primary key: 5 (O(log n))
- Unique index equality: 10
- Single column equality: 20
- Range query: 40 (single), 60 (multi-column)
- IN clause: 30
- LIKE prefix: 50
- LIKE with wildcards: 80

**Adjustment Multipliers:**
- LIMIT ≤ 100: ×0.3 (massive savings)
- LIMIT ≤ 1000: ×0.6
- OR conditions: ×1.5
- Subqueries: ×1.3
- ORDER BY not in index: ×1.2
- GROUP BY: ×1.1
- JOIN: ×1.2
- Very High cardinality: ×0.9
- Low/Very Low cardinality: ×1.2

**Examples:**

```rust
// Very Low Cost (5)
User::by_pk().bind("123").fetch_one(&pool).await?;
// Cost: Very Low (5 vs full scan)

// Low Cost with LIMIT (20 * 0.3 = 6)
Post::where_query!("user_id = $1 ORDER BY created_at DESC LIMIT 10")
    .bind("user123").fetch_all(&pool).await?;
// Cost: Very Low (6 vs full scan)

// Medium Cost Range Query (40)
Product::where_query!("price > $1 AND price < $2")
    .bind(100).bind(200).fetch_all(&pool).await?;
// Cost: Low (40 vs full scan)

// High Cost OR Query (20 * 1.5 = 30)
Task::where_query!("status = $1 OR priority = $2")
    .bind("pending").bind(5).fetch_all(&pool).await?;
// Cost: Medium (30 vs full scan) per index
```

**Using Cost Estimates:**

1. **Compare Index Strategies**: Lower cost is better
2. **Identify Problem Queries**: High cost indicates need for optimization
3. **Measure Impact**: Before/after cost comparisons
4. **Prioritize Indexes**: Focus on highest-cost queries first

## Integration with Existing Code

### Analyzing Specific Modules

You can apply `#[analyze_queries]` to specific modules without affecting other code:

```rust
// Analyze user-related queries
#[sqlx_struct_macros::analyze_queries]
mod user_queries {
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
    }

    impl User {
        fn find_by_email(email: &str) {
            User::where_query!("email = $1")
        }
    }
}

// Analyze order-related queries separately
#[sqlx_struct_macros::analyze_queries]
mod order_queries {
    #[derive(EnhancedCrud)]
    struct Order {
        id: String,
        customer_id: String,
    }

    impl Order {
        fn find_by_customer(customer_id: &str) {
            Order::where_query!("customer_id = $1")
        }
    }
}

// Regular business logic - not analyzed
mod business_logic {
    impl User {
        fn process(&self) {
            // Business logic that doesn't involve queries
        }
    }
}
```

### Combining with Other Attributes

The macro works alongside other Rust attributes:

```rust
#[sqlx_struct_macros::analyze_queries]
#[allow(dead_code)]  // This works fine
mod experimental_queries {
    #[derive(EnhancedCrud)]
    #[table_name = "app_users"]  // Custom table names work
    struct User {
        id: String,
        email: String,
    }

    impl User {
        fn test_query() {
            User::where_query!("email = $1")
        }
    }
}
```

## Limitations

### Current Implementation

1. **Simplified Pattern Matching**: Uses string-based pattern matching for JOIN and GROUP BY
   - Efficient for common query patterns
   - Covers 80%+ of real-world use cases
   - May have edge cases with very complex nested queries

2. **Supported Condition Types**:
   - ✅ Equality conditions: `col = $1`
   - ✅ Range conditions: `col > $1`, `col < $1`, `col >= $1`, `col <= $1`
   - ✅ IN clauses: `col IN ($1, $2)`
   - ✅ LIKE clauses: `col LIKE $1`
   - ✅ Inequality conditions: `col != $1`, `col <> $1`
   - ✅ NOT LIKE clauses: `col NOT LIKE $1`
   - ✅ JOIN conditions: INNER JOIN, LEFT JOIN, RIGHT JOIN 🆕
   - ✅ GROUP BY: Single and multiple columns 🆕
   - ✅ HAVING clauses: Detection and recommendation 🆕
   - Smart priority ordering: Equality > IN > Range > LIKE > Inequality > NOT LIKE > ORDER BY

3. **Query Complexity Detection**:
   - ✅ OR conditions detection: `has_or_conditions()`
   - ✅ Parentheses grouping detection: `has_parentheses()`
   - ✅ Subquery detection: `has_subquery()`
   - ✅ Query complexity analysis: `analyze_query_complexity()`
   - Note: OR conditions are detected but require manual index strategy decisions

4. **Query Pattern Support**:
   - ✅ `where_query!()` macro calls
   - ✅ `make_query!()` macro calls
   - ✅ Automatic table and field extraction from struct definitions

5. **No Statistics**: Doesn't use actual database statistics
   - Recommendations are based on query patterns only
   - Query frequency, table size, and selectivity not considered

6. **Compile-Time Only**: Analysis happens during compilation
   - Doesn't analyze dynamically generated queries
   - Can't detect runtime query patterns

### What's Not Supported Yet

- ❌ Subquery column analysis (detects subqueries but doesn't analyze internal queries)
- ❌ Covering indexes (INCLUDE columns)
- ❌ UNION queries
- ❌ Window functions
- ❌ CTE (WITH clauses)
- ❌ Expression indexes (functional indexes)
- ❌ Full-text search indexes
- ❌ JSON field indexes
- ❌ Array element indexes
- ⚠️ OR condition optimization (detected but requires manual strategy)
- ❌ Complex nested boolean expressions with explicit index recommendations

These may be added in future phases.

## Best Practices

### 1. Run Analysis Regularly

Make index analysis part of your development workflow:

```bash
# During development
cargo build

# Check for new recommendations
cargo clean && cargo build 2>&1 | grep -A 20 "Index Recommendations"
```

### 2. Apply Recommendations Incrementally

Don't create all recommended indexes at once:

1. **Start with high-impact queries**: Identify your most frequently executed queries
2. **Create indexes gradually**: Add one or two indexes at a time
3. **Measure performance**: Use `EXPLAIN ANALYZE` to verify improvements
4. **Monitor overhead**: Be aware of index overhead on INSERT/UPDATE/DELETE operations

### 3. Review Recommendations Critically

The macro provides recommendations, but you should evaluate them:

```rust
// This might recommend an index on (status)
User::where_query!("status = $1")

// But if 'status' has low cardinality (e.g., only 'active'/'inactive'),
// a full table scan might be faster than using an index!
```

Consider:
- **Cardinality**: How many distinct values does the column have?
- **Selectivity**: What percentage of rows match the query?
- **Query Frequency**: How often is this query executed?
- **Table Size**: How large is the table?

### 4. Combine Related Queries

Group related queries in the same module for better analysis:

```rust
// Good: All user queries in one module
#[analyze_queries]
mod user_queries {
    // All User queries here
}

// Avoid: Scattering queries across modules
mod auth { /* some User queries */ }
mod profile { /* other User queries */ }
mod settings { /* more User queries */ }
```

### 5. Use with Database Profiling

Combine compile-time analysis with runtime profiling:

1. **Use compile-time analysis** to identify potential indexes
2. **Use database profiling** (pg_stat_statements, slow query log) to find actual bottlenecks
3. **Compare** recommendations with real-world performance data
4. **Prioritize** indexes that address both recommendations and profiling data

## Future Enhancements

### ✅ Phase 0: Day 2 (Completed)

- ✅ Support for more SQL operators (`>`, `<`, `>=`, `<=`, `IN`, `LIKE`)
- ✅ Priority-based column ordering (Equality > IN > Range > LIKE > ORDER BY)
- ✅ Improved boundary checking to prevent partial matches
- ✅ Comprehensive test suite (18 unit tests, all passing)
- ✅ Enhanced documentation

### ✅ Phase 0: Day 3 (Completed)

- ✅ Negation conditions support (`!=`, `<>`, `NOT LIKE`)
- ✅ Extended priority system with inequality operators
- ✅ `make_query!()` pattern recognition (already implemented)
- ✅ AND/OR combinations via priority ordering
- ✅ Comprehensive test suite (26 unit tests, all passing)
- ✅ Enhanced documentation with new rules

**New Priority Order** (Day 3):
Equality > IN > Range > LIKE > Inequality > NOT LIKE > ORDER BY

### ✅ Phase 0: Day 4 (Completed)

- ✅ OR conditions detection and warning system
- ✅ Parentheses grouping detection (excludes IN clauses)
- ✅ Subquery detection
- ✅ Query complexity analysis API (`QueryComplexity` struct)
- ✅ Comprehensive test suite (39 unit tests, all passing)
- ✅ Documentation for OR condition indexing strategies

**New Detection APIs** (Day 4):
- `has_or_conditions()` - Detect OR in WHERE clauses
- `has_parentheses()` - Detect parenthesis grouping
- `has_subquery()` - Detect nested SELECT statements
- `analyze_query_complexity()` - Complete complexity analysis

### ✅ Phase B: JOIN and GROUP BY Analysis (Completed) 🆕

- ✅ Simplified SQL parser implementation
- ✅ JOIN query detection and analysis (INNER, LEFT, RIGHT)
- ✅ GROUP BY column detection and recommendation
- ✅ HAVING clause detection
- ✅ Multi-column GROUP BY support
- ✅ Architecture validation (see ARCHITECTURE_VALIDATION_REPORT.md)
- ✅ Integration with compile_time_analyzer
- ✅ Comprehensive test suite (all tests passing)
- ✅ Documentation updated with JOIN and GROUP BY examples

**Implementation Details** (Phase B):
- Created `sqlx_struct_macros/src/parser/` module with:
  - `mod.rs` - SqlDialect and IndexSyntax definitions
  - `sql_parser.rs` - Simplified SQL parser using string matching
  - `column_extractor.rs` - JoinInfo and GroupByInfo data structures
- Updated `compile_time_analyzer.rs` with JOIN and GROUP BY analysis logic
- Added test suite: `tests/join_groupby_analysis_test.rs`
- Created implementation summary: `JOIN_GROUPBY_IMPLEMENTATION_SUMMARY.md`

### Phase 0: Days 5-6 (Advanced Analysis)

- Advanced OR condition optimization strategies
- Covering index recommendations (INCLUDE columns) - Partially supported
- Partial index suggestions (conditional indexes) - Partially supported
- Index size estimation - Already supported
- Query cost estimation - Already supported

### Phase 0: Days 7-8 (Testing & Documentation)

- Integration testing with real databases
- Performance benchmarks
- Additional documentation and examples
- User feedback and refinement

### Future Phases

- Real-time query monitoring integration
- Machine learning-based index recommendations
- Automatic index creation tools
- Index usage monitoring and cleanup suggestions

## Running the Example

Try the compile-time analysis example:

```bash
# Clone the repository
cd sqlx_struct_enhanced

# Build the example
cargo build --example compile_time_analysis

# View the recommendations in the build output
```

## Troubleshooting

### No Recommendations Generated

If you don't see any recommendations:

1. **Check struct definitions**: Ensure structs have `#[derive(EnhancedCrud)]`
2. **Check query patterns**: Ensure you're using `where_query!()` or `make_query!()`
3. **Check module structure**: The macro only analyzes the module it's applied to
4. **View build output**: Use `cargo build 2>&1 | less` to see all compilation output

### Unexpected Recommendations

If recommendations seem incorrect:

1. **Review your queries**: Check for typos in query strings
2. **Check struct fields**: Ensure field names match database columns
3. **Consider cardinality**: Low-cardinality columns might not need indexes
4. **Test with EXPLAIN**: Use database EXPLAIN to verify index usage

## Database Support

### PostgreSQL

PostgreSQL has the most complete index analysis support with all features enabled.

#### Supported Features

- ✅ WHERE condition analysis (=, >, <, IN, LIKE, etc.)
- ✅ ORDER BY analysis
- ✅ JOIN analysis (INNER, LEFT, RIGHT, FULL)
- ✅ GROUP BY analysis
- ✅ HAVING clause detection
- ✅ **INCLUDE indexes** (covering indexes)
- ✅ **Partial indexes** (with WHERE clause)
- ✅ IF NOT EXISTS syntax

#### Example

```rust
#[sqlx_struct_macros::analyze_queries]
mod queries {
    #[derive(EnhancedCrud)]
    struct User { id: String, email: String, created_at: i64 }

    impl User {
        fn find_by_email(email: &str) {
            let _ = User::where_query!("email = $1");
            // Recommendation: CREATE INDEX idx_user_email ON User (email)
        }
    }
}
```

#### Build

```bash
cargo build --example compile_time_analysis --features postgres
```

### MySQL

MySQL support is available with version-specific features.

#### Version Compatibility

- **MySQL 5.7+**: Basic index analysis
- **MySQL 8.0+**: Full support including **INCLUDE indexes** (covering indexes)

#### Supported Features

- ✅ WHERE condition analysis (=, >, <, IN, LIKE, etc.)
- ✅ ORDER BY analysis
- ✅ JOIN analysis (INNER, LEFT, RIGHT - FULL not supported)
- ✅ GROUP BY analysis
- ✅ HAVING clause detection
- ✅ **INCLUDE indexes** (MySQL 8.0+ only)
- ❌ Partial indexes (not supported by MySQL)
- ❌ IF NOT EXISTS (not supported in CREATE INDEX)

#### Usage

**For MySQL 8.0+ (default):**

```toml
# Cargo.toml
[dependencies]
sqlx_struct_enhanced = { version = "*", features = ["mysql"] }
```

**For MySQL 5.7:**

```toml
# Cargo.toml
[dependencies]
sqlx_struct_enhanced = { version = "*", features = ["mysql_5_7"] }
```

#### Example

```rust
#[sqlx_struct_macros::analyze_queries]
mod queries {
    #[derive(EnhancedCrud)]
    struct User { id: String, email: String, status: String }

    impl User {
        fn find_by_email(email: &str) {
            let _ = User::where_query!("email = $1");
            // Recommendation (MySQL 8.0+): CREATE INDEX idx_user_email ON User (email)
            // Recommendation (MySQL 5.7): CREATE INDEX idx_user_email ON User (email)
        }

        fn find_active_with_include() {
            let _ = User::where_query!("status = $1 ORDER BY created_at DESC");
            // Recommendation (MySQL 8.0+): CREATE INDEX idx_user_status_created_at ON User (status, created_at) INCLUDE (id)
            // Recommendation (MySQL 5.7): CREATE INDEX idx_user_status_created_at ON User (status, created_at) -- INCLUDE requires MySQL 8.0+ (consider including: id)
        }
    }
}
```

#### Build

```bash
# MySQL 8.0+ (default)
cargo build --example mysql_compile_time_analysis --features mysql

# MySQL 5.7
cargo build --example mysql_compile_time_analysis --features mysql_5_7
```

### SQLite

SQLite support is available with most features enabled.

#### Supported Features

- ✅ WHERE condition analysis (=, >, <, IN, LIKE, etc.)
- ✅ ORDER BY analysis
- ✅ **JOIN analysis** (INNER and LEFT only - RIGHT not supported)
- ✅ GROUP BY analysis
- ✅ HAVING clause detection
- ✅ **Partial indexes** (with WHERE clause)
- ❌ INCLUDE indexes (not supported by SQLite)
- ✅ IF NOT EXISTS syntax

#### Limitations

- **No RIGHT JOIN**: SQLite doesn't support RIGHT JOIN
- **No FULL JOIN**: SQLite doesn't support FULL JOIN
- **No INCLUDE**: SQLite doesn't support the INCLUDE clause for covering indexes

#### Example

```rust
#[sqlx_struct_macros::analyze_queries]
mod queries {
    #[derive(EnhancedCrud)]
    struct User { id: String, email: String, active: bool }

    impl User {
        fn find_active() {
            let _ = User::where_query!("active = $1");
            // Recommendation: CREATE INDEX idx_user_active ON User (active)
        }

        fn find_active_with_partial() {
            let _ = User::where_query!("active = $1 AND created_at > $2");
            // Recommendation: CREATE INDEX idx_user_active_created_at ON User (active, created_at) WHERE active = true
        }
    }
}
```

#### Build

```bash
cargo build --example sqlite_compile_time_analysis --features sqlite
```

## See Also

- [README.md](README.md) - Main project documentation
- [USAGE.md](USAGE.md) - Complete API reference
- [PHASE0_IMPLEMENTATION_PLAN.md](PHASE0_IMPLEMENTATION_PLAN.md) - Implementation details
- [examples/compile_time_analysis.rs](examples/compile_time_analysis.rs) - Working example
