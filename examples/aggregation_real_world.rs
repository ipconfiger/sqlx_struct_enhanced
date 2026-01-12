// Aggregation Queries - Real-World Use Cases
//
// This example demonstrates practical, real-world scenarios for aggregation
// queries including sales reports, leaderboards, analytics dashboards, etc.
//
// Run with: cargo run --example aggregation_real_world

use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::{FromRow, PgPool, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;
use sqlx::Row;

// ========================================================================
// Domain Models
// ========================================================================

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "sales_orders"]
struct SalesOrder {
    id: String,
    customer_id: String,
    product_category: String,
    amount: i32,
    status: String,
    region: String,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "website_events"]
struct WebsiteEvent {
    id: String,
    event_type: String,
    page_url: String,
    user_id: Option<String>,
    session_id: String,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "inventory_items"]
struct InventoryItem {
    id: String,
    product_name: String,
    category: String,
    quantity: i32,
    unit_cost: i32,
    warehouse_location: String,
}

// ========================================================================
// Result Structs
// ========================================================================

#[derive(FromRow, Debug)]
struct SalesReport {
    product_category: String,
    total_revenue: i64,
    order_count: i64,
    avg_order_value: Option<f64>,
}

#[derive(FromRow, Debug)]
struct TopCustomer {
    customer_id: String,
    total_spent: i64,
    order_count: i64,
    avg_order_value: Option<f64>,
}

#[derive(FromRow, Debug)]
struct RegionalStats {
    region: String,
    total_revenue: i64,
    order_count: i64,
    top_category: String,
}

#[derive(FromRow, Debug)]
struct PageStats {
    page_url: String,
    page_views: i64,
    unique_visitors: i64,
}

#[derive(FromRow, Debug)]
struct InventoryValue {
    category: String,
    total_quantity: i64,
    total_value: i64,
    avg_unit_cost: Option<f64>,
}

#[derive(FromRow, Debug)]
struct DailyStats {
    date: String,
    total_revenue: i64,
    order_count: i64,
    avg_order_value: Option<f64>,
}

// ========================================================================
// Use Case Functions
// ========================================================================

/// Use Case 1: E-Commerce Sales Dashboard
/// Shows revenue by category with filtering and sorting
async fn sales_dashboard_example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use Case 1: E-Commerce Sales Dashboard");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Sales by completed status, grouped by category, sorted by revenue
    let sql = SalesOrder::agg_query()
        .where_("status = {}", &["completed"])
        .group_by("product_category")
        .sum_as("amount", "total_revenue")
        .count_as("order_count")
        .avg_as("amount", "avg_order_value")
        .having("total_revenue > {}", &[&10000i64])
        .order_by("total_revenue", "DESC")
        .build();

    println!("Query: Sales by category (completed orders > $10,000)");
    println!("SQL:\n{}\n", sql);

    let reports: Vec<SalesReport> = sqlx::query_as(sql)
        .bind("completed")
        .bind(10000i64)
        .fetch_all(pool)
        .await?;

    println!("Sales Dashboard Results:");
    println!("┌────────────────────┬──────────────┬──────────────┬─────────────────┐");
    println!("│ Category           │ Revenue      │ Orders       │ Avg Order Value │");
    println!("├────────────────────┼──────────────┼──────────────┼─────────────────┤");

    for report in &reports {
        println!(
            "│ {:18} │ ${:>10} │ {:>12} │ ${:>14} │",
            report.product_category,
            report.total_revenue,
            report.order_count,
            report.avg_order_value.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string())
        );
    }

    println!("└────────────────────┴──────────────┴──────────────┴─────────────────┘");

    Ok(())
}

/// Use Case 2: Customer Leaderboard
/// Shows top customers by total spend
async fn customer_leaderboard_example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use Case 2: Customer Leaderboard");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Top 10 customers by total spend
    let sql = SalesOrder::agg_query()
        .where_("status = {}", &["completed"])
        .group_by("customer_id")
        .sum_as("amount", "total_spent")
        .count_as("order_count")
        .avg_as("amount", "avg_order_value")
        .order_by("total_spent", "DESC")
        .limit(10)
        .build();

    println!("Query: Top 10 customers by total spend");
    println!("SQL:\n{}\n", sql);

    let customers: Vec<TopCustomer> = sqlx::query_as(sql)
        .bind("completed")
        .bind(10i64)
        .fetch_all(pool)
        .await?;

    println!("Customer Leaderboard (Top 10):");
    println!("┌──────────┬──────────────┬──────────────┬─────────────────┐");
    println!("│ Rank     │ Customer ID  │ Total Spent  │ Orders          │");
    println!("├──────────┼──────────────┼──────────────┼─────────────────┤");

    for (index, customer) in customers.iter().enumerate() {
        println!(
            "│ {:8} │ {:12} │ ${:>10} │ {:>14} │",
            index + 1,
            customer.customer_id,
            customer.total_spent,
            customer.order_count
        );
    }

    println!("└──────────┴──────────────┴──────────────┴─────────────────┘");

    Ok(())
}

/// Use Case 3: Regional Performance Analysis
/// Shows sales performance by region with top categories
async fn regional_analysis_example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use Case 3: Regional Performance Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // This is a simplified example - in reality you'd need more complex queries
    // or multiple queries to get the top category per region
    let sql = SalesOrder::agg_query()
        .where_("status = {}", &["completed"])
        .group_by("region")
        .sum_as("amount", "total_revenue")
        .count_as("order_count")
        .order_by("total_revenue", "DESC")
        .build();

    println!("Query: Regional sales performance");
    println!("SQL:\n{}\n", sql);

    let regions: Vec<RegionalStats> = sqlx::query_as(sql)
        .bind("completed")
        .fetch_all(pool)
        .await?;

    println!("Regional Performance:");
    println!("┌──────────────┬──────────────┬──────────────┐");
    println!("│ Region       │ Revenue      │ Orders       │");
    println!("├──────────────┼──────────────┼──────────────┤");

    for region in &regions {
        println!(
            "│ {:12} │ ${:>10} │ {:>12} │",
            region.region, region.total_revenue, region.order_count
        );
    }

    println!("└──────────────┴──────────────┴──────────────┘");

    Ok(())
}

/// Use Case 4: Website Analytics
/// Shows page views and unique visitors
async fn website_analytics_example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use Case 4: Website Analytics Dashboard");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Page views by URL
    let sql = WebsiteEvent::agg_query()
        .where_("event_type = {}", &["page_view"])
        .group_by("page_url")
        .count_as("page_views")
        .order_by("page_views", "DESC")
        .limit(10)
        .build();

    println!("Query: Top 10 pages by views");
    println!("SQL:\n{}\n", sql);

    let pages: Vec<PageStats> = sqlx::query_as(sql)
        .bind("page_view")
        .bind(10i64)
        .fetch_all(pool)
        .await?;

    println!("Top Pages (by page views):");
    println!("┌───┬──────────────────────────────────┬──────────────┐");
    println!("│ # │ Page URL                        │ Views        │");
    println!("├───┼──────────────────────────────────┼──────────────┤");

    for (index, page) in pages.iter().enumerate() {
        let truncated_url = if page.page_url.len() > 32 {
            format!("{}...", &page.page_url[..29])
        } else {
            page.page_url.clone()
        };

        println!("│ {:1} │ {:32} │ {:>12} │",
            index + 1, truncated_url, page.page_views);
    }

    println!("└───┴──────────────────────────────────┴──────────────┘");

    Ok(())
}

/// Use Case 5: Inventory Valuation
/// Shows total inventory value by category
async fn inventory_valuation_example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use Case 5: Inventory Valuation by Category");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Calculate inventory value: SUM(quantity * unit_cost)
    // Note: This is simplified - you'd need to do the multiplication in SQL
    // or use a more complex query for actual value calculation
    let sql = InventoryItem::agg_query()
        .group_by("category")
        .sum_as("quantity", "total_quantity")
        .count_as("item_count")
        .order_by("total_quantity", "DESC")
        .build();

    println!("Query: Inventory by category (quantity-based)");
    println!("SQL:\n{}\n", sql);

    let inventory: Vec<InventoryValue> = sqlx::query_as(sql)
        .fetch_all(pool)
        .await?;

    println!("Inventory Summary:");
    println!("┌────────────────────┬──────────────┬──────────────┐");
    println!("│ Category           │ Items        │ Quantity     │");
    println!("├────────────────────┼──────────────┼──────────────┤");

    for item in &inventory {
        println!(
            "│ {:18} │ {:>12} │ {:>12} │",
            item.category, item.total_quantity, item.total_quantity
        );
    }

    println!("└────────────────────┴──────────────┴──────────────┘");

    Ok(())
}

/// Use Case 6: Paginated Report
/// Demonstrates pagination for large datasets
async fn paginated_report_example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Use Case 6: Paginated Sales Report");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let page = 2;
    let per_page = 5;
    let offset = (page - 1) * per_page;

    let sql = SalesOrder::agg_query()
        .where_("status = {}", &["completed"])
        .group_by("product_category")
        .sum_as("amount", "total_revenue")
        .count_as("order_count")
        .order_by("total_revenue", "DESC")
        .limit(per_page)
        .offset(offset)
        .build();

    println!("Query: Page {} of sales report ({} per page)", page, per_page);
    println!("SQL:\n{}\n", sql);

    let reports: Vec<SalesReport> = sqlx::query_as(sql)
        .bind("completed")
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

    println!("Sales Report - Page {}:", page);
    println!("┌────┬────────────────────┬──────────────┬──────────────┐");
    println!("│ #  │ Category           │ Revenue      │ Orders       │");
    println!("├────┼────────────────────┼──────────────┼──────────────┤");

    for (index, report) in reports.iter().enumerate() {
        let global_rank = offset + index + 1;
        println!(
            "│ {:>2} │ {:18} │ ${:>10} │ {:>12} │",
            global_rank, report.product_category, report.total_revenue, report.order_count
        );
    }

    println!("└────┴────────────────────┴──────────────┴──────────────┘");

    Ok(())
}

// ========================================================================
// Main Entry Point
// ========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║   Real-World Aggregation Query Examples                        ║");
    println!("║   Demonstrating practical use cases for business analytics     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // Run all use case examples
    sales_dashboard_example(&pool).await?;
    customer_leaderboard_example(&pool).await?;
    regional_analysis_example(&pool).await?;
    website_analytics_example(&pool).await?;
    inventory_valuation_example(&pool).await?;
    paginated_report_example(&pool).await?;

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   All real-world examples completed successfully!             ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}
