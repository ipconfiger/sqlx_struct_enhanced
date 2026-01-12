// Tests for JOIN support in aggregation queries
// Tests the new JOIN functionality with INNER, LEFT, RIGHT, and FULL joins

use sqlx_struct_enhanced::{EnhancedCrud, Scheme, AggQueryBuilder, Join, JoinType};
use sqlx::{FromRow, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;
use sqlx::Row;

// Test struct with foreign key relationship
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct Order {
    id: String,
    customer_id: String,
    product_id: String,
    amount: i32,
    status: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct Customer {
    id: String,
    name: String,
    region: String,
    status: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct Product {
    id: String,
    name: String,
    category: String,
    price: i32,
}

// ============================================================================
// Basic JOIN Tests
// ============================================================================

#[test]
fn test_inner_join() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("SELECT SUM(order.amount) FROM order INNER JOIN customer ON order.customer_id = customer.id"));
}

#[test]
fn test_left_join() {
    let builder = Order::agg_query()
        .join_left("customer", "order.customer_id = customer.id")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("LEFT JOIN customer"));
}

#[test]
fn test_right_join() {
    let builder = Order::agg_query()
        .join_right("customer", "order.customer_id = customer.id")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("RIGHT JOIN customer"));
}

#[test]
fn test_full_join() {
    let builder = Order::agg_query()
        .join_full("customer", "order.customer_id = customer.id")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("FULL JOIN customer"));
}

// ============================================================================
// JOIN with GROUP BY Tests
// ============================================================================

#[test]
fn test_join_with_single_group_by() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("SELECT customer.region, SUM(order.amount) FROM order INNER JOIN customer ON order.customer_id = customer.id GROUP BY customer.region"));
}

#[test]
fn test_join_with_multiple_group_by() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .join("product", "order.product_id = product.id")
        .group_by("customer.region")
        .group_by("product.category")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("INNER JOIN customer"));
    assert!(sql.contains("INNER JOIN product"));
    assert!(sql.contains("GROUP BY customer.region, product.category"));
}

#[test]
fn test_join_with_multiple_aggregates() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum_as("order.amount", "total")
        .avg_as("order.amount", "average")
        .count_as("count");

    let sql = builder.build();
    assert!(sql.contains("SELECT customer.region, SUM(order.amount) AS total, AVG(order.amount) AS average, COUNT(*) AS count"));
}

// ============================================================================
// JOIN with WHERE Tests
// ============================================================================

#[test]
fn test_join_with_where() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .where_("customer.status = {}", &["active"])
        .group_by("customer.region")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("WHERE customer.status = $1"));
}

#[test]
fn test_join_with_complex_where() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .where_("customer.status = {} AND order.amount > {}", &["active", "100"])
        .group_by("customer.region")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("WHERE customer.status = $1 AND order.amount > $2"));
}

// ============================================================================
// JOIN with HAVING Tests
// ============================================================================

#[test]
fn test_join_with_having() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum_as("order.amount", "total")
        .having("total > {}", &[&1000i64]);

    let sql = builder.build();
    assert!(sql.contains("HAVING total > $1"));
}

#[test]
fn test_join_with_where_and_having() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .where_("customer.status = {}", &["active"])
        .group_by("customer.region")
        .sum_as("order.amount", "total")
        .having("total > {}", &[&500i64]);

    let sql = builder.build();
    assert!(sql.contains("WHERE customer.status = $1"));
    assert!(sql.contains("HAVING total > $2"));
}

// ============================================================================
// JOIN with ORDER BY Tests
// ============================================================================

#[test]
fn test_join_with_order_by() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum_as("order.amount", "total")
        .order_by("total", "DESC");

    let sql = builder.build();
    assert!(sql.contains("ORDER BY total DESC"));
}

#[test]
fn test_join_with_order_by_asc() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount")
        .order_by("customer.region", "ASC");

    let sql = builder.build();
    assert!(sql.contains("ORDER BY customer.region ASC"));
}

// ============================================================================
// JOIN with LIMIT/OFFSET Tests
// ============================================================================

#[test]
fn test_join_with_limit() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount")
        .limit(10);

    let sql = builder.build();
    assert!(sql.contains("LIMIT $1"));
}

#[test]
fn test_join_with_offset() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount")
        .offset(20);

    let sql = builder.build();
    assert!(sql.contains("OFFSET $1"));
}

#[test]
fn test_join_with_limit_and_offset() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount")
        .limit(10)
        .offset(20);

    let sql = builder.build();
    assert!(sql.contains("LIMIT $1"));
    assert!(sql.contains("OFFSET $2"));
}

// ============================================================================
// Complex JOIN Tests
// ============================================================================

#[test]
fn test_complex_join_query() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .join_left("product", "order.product_id = product.id")
        .where_("customer.status = {} AND order.amount > {}", &["active", "100"])
        .group_by("customer.region")
        .group_by("product.category")
        .sum_as("order.amount", "total")
        .avg_as("order.amount", "average")
        .having("total > {}", &[&500i64])
        .order_by("total", "DESC")
        .limit(10);

    let sql = builder.build();
    assert!(sql.contains("INNER JOIN customer"));
    assert!(sql.contains("LEFT JOIN product"));
    assert!(sql.contains("WHERE customer.status = $1 AND order.amount > $2"));
    assert!(sql.contains("GROUP BY customer.region, product.category"));
    assert!(sql.contains("HAVING total > $3"));
    assert!(sql.contains("ORDER BY total DESC"));
    assert!(sql.contains("LIMIT $4"));
}

#[test]
fn test_three_way_join() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .join("product", "order.product_id = product.id")
        .group_by("customer.region")
        .group_by("product.category")
        .sum("order.amount")
        .count();

    let sql = builder.build();
    assert!(sql.contains("INNER JOIN customer"));
    assert!(sql.contains("INNER JOIN product"));
    assert!(sql.contains("GROUP BY customer.region, product.category"));
}

#[test]
fn test_join_with_all_aggregates() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum_as("order.amount", "total")
        .avg_as("order.amount", "average")
        .count_as("count")
        .min_as("order.amount", "minimum")
        .max_as("order.amount", "maximum");

    let sql = builder.build();
    assert!(sql.contains("SELECT customer.region, SUM(order.amount) AS total"));
    assert!(sql.contains("AVG(order.amount) AS average"));
    assert!(sql.contains("COUNT(*) AS count"));
    assert!(sql.contains("MIN(order.amount) AS minimum"));
    assert!(sql.contains("MAX(order.amount) AS maximum"));
}

// ============================================================================
// JOIN Type Tests
// ============================================================================

#[test]
fn test_join_type_display() {
    assert_eq!(format!("{}", JoinType::Inner), "INNER JOIN");
    assert_eq!(format!("{}", JoinType::Left), "LEFT JOIN");
    assert_eq!(format!("{}", JoinType::Right), "RIGHT JOIN");
    assert_eq!(format!("{}", JoinType::Full), "FULL JOIN");
}

#[test]
fn test_join_equality() {
    let join1 = Join {
        join_type: JoinType::Inner,
        table: "customer".to_string(),
        condition: "order.customer_id = customer.id".to_string(),
    };

    let join2 = Join {
        join_type: JoinType::Inner,
        table: "customer".to_string(),
        condition: "order.customer_id = customer.id".to_string(),
    };

    assert_eq!(join1, join2);
}

// ============================================================================
// SQL Caching with JOINs
// ============================================================================

#[test]
fn test_join_sql_caching() {
    let builder1 = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount");

    let builder2 = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .group_by("customer.region")
        .sum("order.amount");

    // Should generate the same SQL and cache it
    let sql1 = builder1.build();
    let sql2 = builder2.build();
    assert_eq!(sql1, sql2);
}

#[test]
fn test_different_join_order_not_cached() {
    // Different join order should produce different SQL
    let builder1 = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .join("product", "order.product_id = product.id")
        .sum("order.amount");

    let builder2 = Order::agg_query()
        .join("product", "order.product_id = product.id")
        .join("customer", "order.customer_id = customer.id")
        .sum("order.amount");

    let sql1 = builder1.build();
    let sql2 = builder2.build();

    // Different join order means different SQL
    assert_ne!(sql1, sql2);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_join_without_group_by() {
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("SELECT SUM(order.amount) FROM order INNER JOIN customer"));
    assert!(!sql.contains("GROUP BY"));
}

#[test]
fn test_multiple_joins_same_table_different_conditions() {
    let builder = Order::agg_query()
        .join("customer AS c1", "order.customer_id = c1.id")
        .join("customer AS c2", "order.delivery_customer_id = c2.id")
        .sum("order.amount");

    let sql = builder.build();
    assert!(sql.contains("INNER JOIN customer AS c1"));
    assert!(sql.contains("INNER JOIN customer AS c2"));
}

// ============================================================================
// Real-World Scenarios
// ============================================================================

#[test]
fn test_sales_by_region_with_customer_info() {
    // Typical business intelligence query
    let builder = Order::agg_query()
        .join("customer", "order.customer_id = customer.id")
        .where_("customer.status = {} AND order.status = {}", &["active", "completed"])
        .group_by("customer.region")
        .sum_as("order.amount", "total_sales")
        .count_as("order_count")
        .avg_as("order.amount", "avg_order_value")
        .order_by("total_sales", "DESC")
        .limit(10);

    let sql = builder.build();
    assert!(sql.contains("WHERE customer.status = $1 AND order.status = $2"));
    assert!(sql.contains("GROUP BY customer.region"));
    assert!(sql.contains("ORDER BY total_sales DESC"));
    assert!(sql.contains("LIMIT $3"));
}

#[test]
fn test_product_sales_by_category() {
    // Product category analysis
    let builder = Order::agg_query()
        .join("product", "order.product_id = product.id")
        .where_("order.status = {}", &["completed"])
        .group_by("product.category")
        .sum_as("order.amount", "revenue")
        .count_as("units_sold")
        .having("revenue > {}", &[&1000i64])
        .order_by("revenue", "DESC");

    let sql = builder.build();
    assert!(sql.contains("INNER JOIN product"));
    assert!(sql.contains("WHERE order.status = $1"));
    assert!(sql.contains("GROUP BY product.category"));
    assert!(sql.contains("HAVING revenue > $2"));
    assert!(sql.contains("ORDER BY revenue DESC"));
}
