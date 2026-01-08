// Aggregation Queries - Basic Examples
//
// This example demonstrates basic aggregation operations including
// SUM, AVG, COUNT, MIN, MAX with GROUP BY.
//
// Run with: cargo run --example aggregation_basics

use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::{FromRow, PgPool, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct Order {
    id: String,
    category: String,
    amount: i32,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database connection
    let pool = PgPool::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    println!("=== Aggregation Queries - Basic Examples ===\n");

    // ========================================================================
    // Example 1: Simple SUM Aggregation
    // ========================================================================
    println!("1. Simple SUM Aggregation");
    println!("   Query: Total amount across all orders");

    let sql = Order::agg_query()
        .sum("amount")
        .build();

    println!("   SQL: {}", sql);

    let (total,): (i64,) = sqlx::query_as(sql)
        .fetch_one(&pool)
        .await?;

    println!("   Result: Total amount = {}\n", total);

    // ========================================================================
    // Example 2: Multiple Aggregates
    // ========================================================================
    println!("2. Multiple Aggregates (SUM, AVG, COUNT, MIN, MAX)");
    println!("   Query: Statistics across all orders");

    let sql = Order::agg_query()
        .sum("amount")
        .avg("amount")
        .count()
        .min("amount")
        .max("amount")
        .build();

    println!("   SQL: {}", sql);

    let (sum, avg, count, min, max): (i64, Option<f64>, i64, i64, i64) =
        sqlx::query_as(sql)
            .fetch_one(&pool)
            .await?;

    println!("   Results:");
    println!("     - Sum: {}", sum);
    println!("     - Avg: {:?}", avg);
    println!("     - Count: {}", count);
    println!("     - Min: {}", min);
    println!("     - Max: {}\n", max);

    // ========================================================================
    // Example 3: GROUP BY Aggregation
    // ========================================================================
    println!("3. GROUP BY Aggregation");
    println!("   Query: Total amount per category");

    let sql = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for (category, total) in &results {
        println!("     - {}: {}", category, total);
    }
    println!();

    // ========================================================================
    // Example 4: GROUP BY with Multiple Aggregates
    // ========================================================================
    println!("4. GROUP BY with Multiple Aggregates");
    println!("   Query: Statistics per category");

    let sql = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .avg("amount")
        .count()
        .build();

    println!("   SQL: {}", sql);

    #[derive(FromRow, Debug)]
    struct CategoryStats {
        category: String,
        sum: i64,
        avg: Option<f64>,
        count: i64,
    }

    let results: Vec<CategoryStats> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for stat in &results {
        println!(
            "     - {}: sum={}, avg={:?}, count={}",
            stat.category, stat.sum, stat.avg, stat.count
        );
    }
    println!();

    // ========================================================================
    // Example 5: COUNT Variations
    // ========================================================================
    println!("5. COUNT Variations");

    // COUNT(*)
    let sql = Order::agg_query()
        .count()
        .build();

    println!("   COUNT(*) SQL: {}", sql);

    let (count_all,): (i64,) = sqlx::query_as(sql)
        .fetch_one(&pool)
        .await?;

    println!("   COUNT(*) Result: {}", count_all);

    // COUNT(column)
    let sql = Order::agg_query()
        .count_column("id")
        .build();

    println!("   COUNT(id) SQL: {}", sql);

    let (count_id,): (i64,) = sqlx::query_as(sql)
        .fetch_one(&pool)
        .await?;

    println!("   COUNT(id) Result: {}\n", count_id);

    // ========================================================================
    // Example 6: Custom Column Aliases
    // ========================================================================
    println!("6. Custom Column Aliases");
    println!("   Query: Total and average per category with custom names");

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total_amount")
        .avg_as("amount", "average_amount")
        .count_as("order_count")
        .build();

    println!("   SQL: {}", sql);

    #[derive(FromRow, Debug)]
    struct CategoryStatsWithAliases {
        category: String,
        total_amount: i64,
        average_amount: Option<f64>,
        order_count: i64,
    }

    let results: Vec<CategoryStatsWithAliases> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for stat in &results {
        println!(
            "     - {}: total={}, avg={:?}, count={}",
            stat.category, stat.total_amount, stat.average_amount, stat.order_count
        );
    }
    println!();

    // ========================================================================
    // Example 7: WHERE Clause
    // ========================================================================
    println!("7. WHERE Clause with Aggregation");
    println!("   Query: Total amount for active orders");

    let sql = Order::agg_query()
        .where_("status = {}", &["active"])
        .sum("amount")
        .build();

    println!("   SQL: {}", sql);

    let (total,): (i64,) = sqlx::query_as(sql)
        .bind("active")
        .fetch_one(&pool)
        .await?;

    println!("   Result: Total for active orders = {}\n", total);

    // ========================================================================
    // Example 8: WHERE with GROUP BY
    // ========================================================================
    println!("8. WHERE + GROUP BY");
    println!("   Query: Total amount per category for active orders");

    let sql = Order::agg_query()
        .where_("status = {}", &["active"])
        .group_by("category")
        .sum_as("amount", "total")
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind("active")
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for (category, total) in &results {
        println!("     - {}: {}", category, total);
    }
    println!();

    // ========================================================================
    // Example 9: MIN and MAX
    // ========================================================================
    println!("9. MIN and MAX Aggregates");
    println!("   Query: Min and max amounts per category");

    let sql = Order::agg_query()
        .group_by("category")
        .min("amount")
        .max("amount")
        .build();

    println!("   SQL: {}", sql);

    #[derive(FromRow, Debug)]
    struct MinMaxStats {
        category: String,
        min: i64,
        max: i64,
    }

    let results: Vec<MinMaxStats> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for stat in &results {
        println!("     - {}: min={}, max={}", stat.category, stat.min, stat.max);
    }
    println!();

    // ========================================================================
    // Example 10: SQL Caching
    // ========================================================================
    println!("10. SQL Caching Demonstration");
    println!("     Building identical queries twice...");

    let sql1 = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .build();

    let sql2 = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .build();

    println!("     First SQL:  {}", sql1);
    println!("     Second SQL: {}", sql2);
    println!("     Same address (cached): {}\n", sql1.as_ptr() == sql2.as_ptr());

    println!("=== All Basic Examples Completed ===");

    Ok(())
}
