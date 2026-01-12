// Aggregation Queries - JOIN Examples
//
// This example demonstrates JOIN support in aggregation queries,
// showing how to combine data from multiple tables for complex analytics.
//
// Run with: cargo run --example aggregation_joins

use sqlx_struct_enhanced::{EnhancedCrud, Scheme, AggQueryBuilder, JoinType};
use sqlx::{FromRow, PgPool, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;
use sqlx::Row;

// ========================================================================
// Domain Models
// ========================================================================

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "orders"]
struct Order {
    id: String,
    customer_id: String,
    product_id: String,
    amount: i32,
    status: String,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "customers"]
struct Customer {
    id: String,
    name: String,
    email: String,
    region: String,
    tier: String,  // gold, silver, bronze
    status: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "products"]
struct Product {
    id: String,
    name: String,
    category: String,
    price: i32,
}

// ========================================================================
// Result Structs
// ========================================================================

#[derive(FromRow, Debug)]
struct RegionalSales {
    region: String,
    total_revenue: i64,
    order_count: i64,
    avg_order_value: Option<f64>,
}

#[derive(FromRow, Debug)]
struct ProductSales {
    category: String,
    total_revenue: i64,
    units_sold: i64,
    avg_price: Option<f64>,
}

#[derive(FromRow, Debug)]
struct CustomerTierStats {
    tier: String,
    total_revenue: i64,
    customer_count: i64,
    avg_revenue_per_customer: Option<f64>,
}

#[derive(FromRow, Debug)]
struct CategoryRegionStats {
    product_category: String,
    customer_region: String,
    total_revenue: i64,
    order_count: i64,
}

// ========================================================================
// Example Functions
// ========================================================================

/// Example 1: Basic INNER JOIN
async fn example_1_basic_join(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 1: Basic INNER JOIN                                   ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Join orders with customers to get regional sales
    let sql = Order::agg_query()
        .join("customers", "orders.customer_id = customers.id")
        .where_("customers.status = {} AND orders.status = {}", &["active", "completed"])
        .group_by("customers.region")
        .sum_as("orders.amount", "total_revenue")
        .count_as("order_count")
        .avg_as("orders.amount", "avg_order_value")
        .order_by("total_revenue", "DESC")
        .build();

    println!("Query: Total sales by customer region");
    println!("SQL:\n{}\n", sql);

    let results: Vec<RegionalSales> = sqlx::query_as(sql)
        .bind("active")
        .bind("completed")
        .fetch_all(pool)
        .await?;

    println!("Results:");
    println!("┌──────────────────┬──────────────┬──────────────┬─────────────────┐");
    println!("│ Region          │ Revenue      │ Orders       │ Avg Order Value │");
    println!("├──────────────────┼──────────────┼──────────────┼─────────────────┤");

    for result in &results {
        println!(
            "│ {:14} │ ${:>10} │ {:>12} │ ${:>14} │",
            result.region,
            result.total_revenue,
            result.order_count,
            result.avg_order_value.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string())
        );
    }

    println!("└──────────────────┴──────────────┴──────────────┴─────────────────┘");

    Ok(())
}

/// Example 2: LEFT JOIN for All Products
async fn example_2_left_join(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 2: LEFT JOIN - Include All Products                    ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // LEFT JOIN ensures we get all products, even those with no orders
    let sql = Order::agg_query()
        .join_left("products", "orders.product_id = products.id")
        .where_("orders.status = {}", &["completed"])
        .group_by("products.category")
        .sum_as("orders.amount", "total_revenue")
        .count_as("units_sold")
        .order_by("total_revenue", "DESC")
        .build();

    println!("Query: Sales by product category (includes products with no sales)");
    println!("SQL:\n{}\n", sql);

    let results: Vec<ProductSales> = sqlx::query_as(sql)
        .bind("completed")
        .fetch_all(pool)
        .await?;

    println!("Results:");
    println!("┌──────────────────┬──────────────┬──────────────┐");
    println!("│ Category        │ Revenue      │ Units Sold   │");
    println!("├──────────────────┼──────────────┼──────────────┤");

    for result in &results {
        println!(
            "│ {:14} │ ${:>10} │ {:>12} │",
            result.category, result.total_revenue, result.units_sold
        );
    }

    println!("└──────────────────┴──────────────┴──────────────┘");

    Ok(())
}

/// Example 3: Multiple Joins
async fn example_3_multiple_joins(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 3: Multiple Joins (Orders + Customers + Products)       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Join both customers and products for detailed analysis
    let sql = Order::agg_query()
        .join("customers", "orders.customer_id = customers.id")
        .join("products", "orders.product_id = products.id")
        .where_("customers.tier = {} AND orders.status = {}", &["gold", "completed"])
        .group_by("customers.region")
        .group_by("products.category")
        .sum_as("orders.amount", "total_revenue")
        .count_as("order_count")
        .order_by("total_revenue", "DESC")
        .limit(10)
        .build();

    println!("Query: Gold customer sales by region and product category");
    println!("SQL:\n{}\n", sql);

    let results: Vec<CategoryRegionStats> = sqlx::query_as(sql)
        .bind("gold")
        .bind("completed")
        .bind(10i64)
        .fetch_all(pool)
        .await?;

    println!("Results (Top 10):");
    println!("┌──────────────────┬──────────────────┬──────────────┬──────────────┐");
    println!("│ Product Category │ Customer Region │ Revenue      │ Orders       │");
    println!("├──────────────────┼──────────────────┼──────────────┼──────────────┤");

    for (index, result) in results.iter().enumerate() {
        println!(
            "│ {:16} │ {:16} │ ${:>10} │ {:>12} │",
            result.product_category,
            result.customer_region,
            result.total_revenue,
            result.order_count
        );
        if index >= 9 {
            break;
        }
    }

    println!("└──────────────────┴──────────────────┴──────────────┴──────────────┘");

    Ok(())
}

/// Example 4: Customer Tier Analysis with HAVING
async fn example_4_having_with_join(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 4: Customer Tier Analysis with HAVING                 ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Analyze revenue by customer tier, filtering high-revenue tiers
    let sql = Order::agg_query()
        .join("customers", "orders.customer_id = customers.id")
        .where_("orders.status = {}", &["completed"])
        .group_by("customers.tier")
        .sum_as("orders.amount", "total_revenue")
        .count_as("customer_count")
        .avg_as("orders.amount", "avg_revenue_per_customer")
        .having("total_revenue > {}", &[&10000i64])
        .order_by("total_revenue", "DESC")
        .build();

    println!("Query: Revenue by customer tier (only tiers > $10,000 total)");
    println!("SQL:\n{}\n", sql);

    let results: Vec<CustomerTierStats> = sqlx::query_as(sql)
        .bind("completed")
        .bind(10000i64)
        .fetch_all(pool)
        .await?;

    println!("Results:");
    println!("┌────────┬──────────────┬────────────────┬──────────────────────┐");
    println!("│ Tier   │ Revenue      │ Customers       │ Avg Revenue/Customer  │");
    println!("├────────┼──────────────┼────────────────┼──────────────────────┤");

    for result in &results {
        println!(
            "│ {:6} │ ${:>10} │ {:>14} │ ${:>20} │",
            result.tier,
            result.total_revenue,
            result.customer_count,
            result.avg_revenue_per_customer.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string())
        );
    }

    println!("└────────┴──────────────┴────────────────┴──────────────────────┘");

    Ok(())
}

/// Example 5: Complex Multi-Join with Pagination
async fn example_5_pagination(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 5: Complex Query with Pagination                       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let page = 2;
    let per_page = 5;
    let offset = (page - 1) * per_page;

    // Complex query: Join customers and products, filter by multiple conditions
    // Group by multiple columns, apply HAVING, sort, and paginate
    let sql = Order::agg_query()
        .join("customers", "orders.customer_id = customers.id")
        .join_left("products", "orders.product_id = products.id")
        .where_("customers.status = {} AND orders.status = {} AND orders.amount > {}",
                &["active", "completed", "100"])
        .group_by("customers.region")
        .group_by("products.category")
        .sum_as("orders.amount", "total_revenue")
        .count_as("order_count")
        .having("total_revenue > {}", &[&1000i64])
        .order_by("total_revenue", "DESC")
        .limit(per_page)
        .offset(offset)
        .build();

    println!("Query: Page {} of detailed sales report ({} per page)", page, per_page);
    println!("SQL:\n{}\n", sql);

    let results: Vec<CategoryRegionStats> = sqlx::query_as(sql)
        .bind("active")
        .bind("completed")
        .bind("100")
        .bind(1000i64)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

    println!("Results - Page {}:", page);
    println!("┌────┬──────────────────┬──────────────────┬──────────────┬──────────────┐");
    println!("│ #  │ Product Category │ Customer Region │ Revenue      │ Orders       │");
    println!("├────┼──────────────────┼──────────────────┼──────────────┼──────────────┤");

    for (index, result) in results.iter().enumerate() {
        let global_rank = offset + index + 1;
        println!(
            "│ {:>2} │ {:16} │ {:16} │ ${:>10} │ {:>12} │",
            global_rank,
            result.product_category,
            result.customer_region,
            result.total_revenue,
            result.order_count
        );
    }

    println!("└────┴──────────────────┴──────────────────┴──────────────┴──────────────┘");

    Ok(())
}

/// Example 6: All JOIN Types
async fn example_6_join_types(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 6: Demonstrating All JOIN Types                          ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // INNER JOIN - Only matching records
    let sql_inner = Order::agg_query()
        .join("customers", "orders.customer_id = customers.id")
        .group_by("customers.region")
        .sum("orders.amount")
        .build();

    println!("INNER JOIN - Only customers with orders:");
    println!("{}\n", sql_inner);

    // LEFT JOIN - All customers, orders or null
    let sql_left = Order::agg_query()
        .join_left("customers", "orders.customer_id = customers.id")
        .group_by("customers.region")
        .sum("orders.amount")
        .build();

    println!("LEFT JOIN - All customers, including those with no orders:");
    println!("{}\n", sql_left);

    // RIGHT JOIN - All orders, customers or null
    let sql_right = Order::agg_query()
        .join_right("customers", "orders.customer_id = customers.id")
        .group_by("customers.region")
        .sum("orders.amount")
        .build();

    println!("RIGHT JOIN - Focus on orders, match customers where possible:");
    println!("{}\n", sql_right);

    // FULL JOIN - All records from both tables
    let sql_full = Order::agg_query()
        .join_full("customers", "orders.customer_id = customers.id")
        .group_by("customers.region")
        .sum("orders.amount")
        .build();

    println!("FULL JOIN - All customers and all orders:");
    println!("{}\n", sql_full);

    Ok(())
}

/// Example 7: Self-Join Pattern
async fn example_7_self_join(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║ Example 7: Self-Join Pattern (Same Table, Different Context)       ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Join the same table twice with different aliases
    // This is useful for comparing related records
    let sql = Order::agg_query()
        .join("orders AS ref_orders", "orders.id = ref_orders.id")
        .where_("ref_orders.status = {}", &["completed"])
        .where_("orders.status = {}", &["pending"])
        .sum("ref_orders.amount")
        .build();

    println!("Query: Compare pending vs completed orders (self-join pattern)");
    println!("SQL:\n{}\n", sql);

    println!("Note: Self-joins are useful for:");
    println!("  - Comparing records over time periods");
    println!("  - Finding relationships between rows in the same table");
    println!("  - Hierarchical data (parent-child relationships)");

    Ok(())
}

// ========================================================================
// Main Entry Point
// ========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║   Aggregation Queries with JOIN - Complete Examples             ║");
    println!("║   Demonstrating multi-table analytics and complex relationships   ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // Run all examples
    example_1_basic_join(&pool).await?;
    example_2_left_join(&pool).await?;
    example_3_multiple_joins(&pool).await?;
    example_4_having_with_join(&pool).await?;
    example_5_pagination(&pool).await?;
    example_6_join_types(&pool).await?;
    example_7_self_join(&pool).await?;

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   All JOIN examples completed successfully!                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
