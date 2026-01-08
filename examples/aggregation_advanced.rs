// Aggregation Queries - Advanced Features
//
// This example demonstrates advanced aggregation operations including
// HAVING, ORDER BY, LIMIT/OFFSET, and complex queries.
//
// Run with: cargo run --example aggregation_advanced

use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::{FromRow, PgPool, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct Order {
    id: String,
    category: String,
    amount: i32,
    status: String,
    region: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    println!("=== Aggregation Queries - Advanced Features ===\n");

    // ========================================================================
    // Example 1: HAVING Clause with Aggregate Function
    // ========================================================================
    println!("1. HAVING Clause with Aggregate Function");
    println!("   Query: Categories with total amount > 1000");

    let sql = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .having("SUM(amount) > {}", &[&1000i64])
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind(1000i64)
        .fetch_all(&pool)
        .await?;

    println!("   Results (categories with total > 1000):");
    for (category, total) in &results {
        println!("     - {}: {}", category, total);
    }
    println!();

    // ========================================================================
    // Example 2: HAVING Clause with Alias (Recommended)
    // ========================================================================
    println!("2. HAVING Clause with Alias");
    println!("   Query: Categories with total amount > 1000 using alias");

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total")
        .having("total > {}", &[&1000i64])
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind(1000i64)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for (category, total) in &results {
        println!("     - {}: {}", category, total);
    }
    println!();

    // ========================================================================
    // Example 3: WHERE + GROUP BY + HAVING
    // ========================================================================
    println!("3. WHERE + GROUP BY + HAVING");
    println!("   Query: Active categories with total amount > 500");

    let sql = Order::agg_query()
        .where_("status = {}", &["active"])
        .group_by("category")
        .sum_as("amount", "total")
        .having("total > {}", &[&500i64])
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind("active")
        .bind(500i64)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for (category, total) in &results {
        println!("     - {}: {}", category, total);
    }
    println!();

    // ========================================================================
    // Example 4: ORDER BY with DESC
    // ========================================================================
    println!("4. ORDER BY DESC");
    println!("   Query: Categories ordered by total amount (descending)");

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total")
        .order_by("total", "DESC")
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results (sorted by total DESC):");
    for (index, (category, total)) in results.iter().enumerate() {
        println!("     {}. {}: {}", index + 1, category, total);
    }
    println!();

    // ========================================================================
    // Example 5: ORDER BY with ASC
    // ========================================================================
    println!("5. ORDER BY ASC");
    println!("   Query: Categories ordered by name (ascending)");

    let sql = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .order_by("category", "ASC")
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results (sorted by category ASC):");
    for (category, total) in &results {
        println!("     - {}: {}", category, total);
    }
    println!();

    // ========================================================================
    // Example 6: LIMIT - Top N Results
    // ========================================================================
    println!("6. LIMIT - Top 5 Categories");
    println!("   Query: Top 5 categories by total amount");

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total")
        .order_by("total", "DESC")
        .limit(5)
        .build();

    println!("   SQL: {}", sql);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind(5i64)
        .fetch_all(&pool)
        .await?;

    println!("   Results (top 5):");
    for (index, (category, total)) in results.iter().enumerate() {
        println!("     {}. {}: {}", index + 1, category, total);
    }
    println!();

    // ========================================================================
    // Example 7: LIMIT + OFFSET - Pagination
    // ========================================================================
    println!("7. LIMIT + OFFSET - Pagination");
    println!("   Query: Page 2 of categories (5 per page)");

    let page = 2;
    let per_page = 5;
    let offset = (page - 1) * per_page;

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total")
        .order_by("total", "DESC")
        .limit(per_page)
        .offset(offset)
        .build();

    println!("   SQL: {}", sql);
    println!("   Parameters: limit={}, offset={}", per_page, offset);

    let results: Vec<(String, i64)> = sqlx::query_as(sql)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&pool)
        .await?;

    println!("   Results (page 2):");
    for (index, (category, total)) in results.iter().enumerate() {
        let global_index = offset + index + 1;
        println!("     {}. {}: {}", global_index, category, total);
    }
    println!();

    // ========================================================================
    // Example 8: Complete Complex Query
    // ========================================================================
    println!("8. Complete Complex Query");
    println!("   Query: Active categories, total > 1000, sorted DESC, top 3");

    let sql = Order::agg_query()
        .where_("status = {}", &["active"])
        .group_by("category")
        .sum_as("amount", "total")
        .avg_as("amount", "average")
        .count_as("count")
        .having("total > {}", &[&1000i64])
        .order_by("total", "DESC")
        .limit(3)
        .build();

    println!("   SQL: {}", sql);

    #[derive(FromRow, Debug)]
    struct ComplexStats {
        category: String,
        total: i64,
        average: Option<f64>,
        count: i64,
    }

    let results: Vec<ComplexStats> = sqlx::query_as(sql)
        .bind("active")
        .bind(1000i64)
        .bind(3i64)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for (index, stat) in results.iter().enumerate() {
        println!(
            "     {}. {}: total={}, avg={:?}, count={}",
            index + 1,
            stat.category,
            stat.total,
            stat.average,
            stat.count
        );
    }
    println!();

    // ========================================================================
    // Example 9: Multiple HAVING Conditions
    // ========================================================================
    println!("9. Multiple HAVING Conditions");
    println!("   Query: Categories with total > 500 AND count > 2");

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total")
        .count_as("count")
        .having("total > {} AND count > {}", &[&500i64, &2i32])
        .order_by("total", "DESC")
        .build();

    println!("   SQL: {}", sql);

    #[derive(FromRow, Debug)]
    struct MultipleHavingStats {
        category: String,
        total: i64,
        count: i64,
    }

    let results: Vec<MultipleHavingStats> = sqlx::query_as(sql)
        .bind(500i64)
        .bind(2i32)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for stat in &results {
        println!(
            "     - {}: total={}, count={}",
            stat.category, stat.total, stat.count
        );
    }
    println!();

    // ========================================================================
    // Example 10: ORDER BY Direction Case-Insensitive
    // ========================================================================
    println!("10. ORDER BY Direction Case-Insensitive");

    // All generate the same SQL
    let sql1 = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .order_by("category", "DESC")
        .build();

    let sql2 = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .order_by("category", "desc")
        .build();

    let sql3 = Order::agg_query()
        .group_by("category")
        .sum("amount")
        .order_by("category", "DeSc")
        .build();

    println!("   DESC:  {}", sql1);
    println!("   desc:  {}", sql2);
    println!("   DeSc:  {}", sql3);
    println!("   All identical: {}\n", sql1 == sql2 && sql2 == sql3);

    // ========================================================================
    // Example 11: All Aggregate Functions with Aliases
    // ========================================================================
    println!("11. All Aggregate Functions with Custom Aliases");
    println!("   Query: Full statistics per category");

    let sql = Order::agg_query()
        .group_by("category")
        .sum_as("amount", "total_amount")
        .avg_as("amount", "average_amount")
        .count_as("order_count")
        .min_as("amount", "min_amount")
        .max_as("amount", "max_amount")
        .build();

    println!("   SQL: {}", sql);

    #[derive(FromRow, Debug)]
    struct FullStats {
        category: String,
        total_amount: i64,
        average_amount: Option<f64>,
        order_count: i64,
        min_amount: i64,
        max_amount: i64,
    }

    let results: Vec<FullStats> = sqlx::query_as(sql)
        .fetch_all(&pool)
        .await?;

    println!("   Results:");
    for stat in &results {
        println!(
            "     - {}: total={}, avg={:?}, count={}, min={}, max={}",
            stat.category,
            stat.total_amount,
            stat.average_amount,
            stat.order_count,
            stat.min_amount,
            stat.max_amount
        );
    }
    println!();

    // ========================================================================
    // Example 12: Parameter Indexing Demonstration
    // ========================================================================
    println!("12. Automatic Parameter Indexing");
    println!("   Query: WHERE + HAVING + LIMIT + OFFSET");

    let sql = Order::agg_query()
        .where_("status = {} AND amount > {}", &["active", "100"])  // $1, $2
        .group_by("category")
        .sum_as("amount", "total")
        .having("total > {}", &[&500i64])  // $3
        .order_by("total", "DESC")
        .limit(10)  // $4
        .offset(20)  // $5
        .build();

    println!("   SQL: {}", sql);
    println!("   Parameter mapping:");
    println!("     $1 = active (WHERE status)");
    println!("     $2 = 100 (WHERE amount)");
    println!("     $3 = 500 (HAVING total)");
    println!("     $4 = 10 (LIMIT)");
    println!("     $5 = 20 (OFFSET)\n");

    println!("=== All Advanced Examples Completed ===");

    Ok(())
}
