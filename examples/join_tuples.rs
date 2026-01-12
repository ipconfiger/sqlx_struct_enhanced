// JOIN Queries - Entity Tuple Examples
//
// This example demonstrates the new JOIN query functionality that returns
// type-safe entity tuples, allowing you to join related tables and get
// structured results.
//
// Run with: cargo run --example join_tuples

use sqlx_struct_enhanced::{EnhancedCrud, join::JoinTuple2};
use sqlx::{PgPool, Postgres, Row};
use sqlx::query::Query;
use sqlx::query::QueryAs;
use sqlx::database::HasArguments;

// ========================================================================
// Domain Models
// ========================================================================
//
// All fields must be pub for JOIN query deserialization.
// This is a requirement of the current implementation.

use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "orders"]
struct Order {
    pub id: String,
    pub customer_id: String,
    pub product_id: String,
    pub amount: i32,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "customers"]
struct Customer {
    pub id: String,
    pub name: String,
    pub email: String,
    pub region: String,
    pub tier: String,
}

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products"]
struct Product {
    pub id: String,
    pub name: String,
    pub category: String,
    pub price: i32,
}

// ========================================================================
// Example Functions
// ========================================================================

/// Basic INNER JOIN example
///
/// INNER JOIN returns only rows where both tables have matching data.
/// Both entities in the tuple will always be Some(value).
async fn example_inner_join(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 1: INNER JOIN ===\n");
    println!("Find all orders with their customer information\n");

    let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
        "orders.customer_id = customers.id"
    )
    .fetch_all(pool)
    .await?;

    for result in results {
        match (result.0, result.1) {
            (Some(order), Some(customer)) => {
                println!("Order {} by {} ({}): ${}",
                    order.id,
                    customer.name,
                    customer.email,
                    order.amount
                );
            }
            _ => {
                println!("Unexpected: INNER JOIN should never have NULL entities");
            }
        }
    }

    Ok(())
}

/// LEFT JOIN example
///
/// LEFT JOIN returns all rows from the left table (orders),
/// and matching rows from the right table (customers).
/// The right entity may be None if there's no match.
async fn example_left_join(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 2: LEFT JOIN ===\n");
    println!("Find all orders, including those without customer records\n");

    let results: Vec<JoinTuple2<Order, Customer>> = Order::join_left::<Customer>(
        "orders.customer_id = customers.id"
    )
    .fetch_all(pool)
    .await?;

    for result in results {
        match (result.0, result.1) {
            (Some(order), Some(customer)) => {
                println!("Order {} by {}: ${}",
                    order.id,
                    customer.name,
                    order.amount
                );
            }
            (Some(order), None) => {
                println!("Order {} has no customer record", order.id);
            }
            _ => {
                println!("Unexpected: LEFT JOIN should always have Order");
            }
        }
    }

    Ok(())
}

/// JOIN with WHERE clause
async fn example_join_with_where(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 3: INNER JOIN with WHERE ===\n");
    println!("Find completed orders by gold-tier customers\n");

    let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
        "orders.customer_id = customers.id"
    )
    .where_("orders.status = {} AND customers.tier = {}", &["completed", "gold"])
    .fetch_all(pool)
    .await?;

    for result in results {
        if let (Some(order), Some(customer)) = (result.0, result.1) {
            println!("Order {} by {} ({} tier): ${}",
                order.id,
                customer.name,
                customer.tier,
                order.amount
            );
        }
    }

    Ok(())
}

/// 3-table JOIN example
async fn example_three_table_join(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 4: 3-Table JOIN ===\n");
    println!("Find orders with customer and product information\n");

    // First join orders with customers
    let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
        "orders.customer_id = customers.id"
    )
    .where_("orders.status = {}", &["completed"])
    .fetch_all(pool)
    .await?;

    // Then for each result, fetch the product
    for result in results {
        if let (Some(order), Some(customer)) = (result.0, result.1) {
            // Fetch product for this order
            let product_result: Vec<JoinTuple2<Order, Product>> = Order::join_inner::<Product>(
                "orders.product_id = products.id"
            )
            .where_("orders.id = {}", &[&order.id])
            .fetch_all(pool)
            .await?;

            if let Some(product_tuple) = product_result.first() {
                if let (Some(_), Some(product)) = (&product_tuple.0, &product_tuple.1) {
                    println!("Order {}: {} bought {} for ${}",
                        order.id,
                        customer.name,
                        product.name,
                        order.amount
                    );
                }
            }
        }
    }

    Ok(())
}

/// Fetch single result
async fn example_fetch_one(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 5: Fetch Single Result ===\n");
    println!("Find a specific order with customer information\n");

    let result: JoinTuple2<Order, Customer> = Order::join_inner::<Customer>(
        "orders.customer_id = customers.id"
    )
    .where_("orders.id = {}", &["order-123"])
    .fetch_one(pool)
    .await?;

    if let (Some(order), Some(customer)) = (result.0, result.1) {
        println!("Found Order {} by {} ({}): ${}",
            order.id,
            customer.name,
            customer.email,
            order.amount
        );
    }

    Ok(())
}

/// Fetch optional result (may not exist)
async fn example_fetch_optional(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 6: Fetch Optional Result ===\n");
    println!("Try to find an order that may not exist\n");

    let result: Option<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
        "orders.customer_id = customers.id"
    )
    .where_("orders.id = {}", &["non-existent"])
    .fetch_optional(pool)
    .await?;

    match result {
        Some(tuple) => {
            if let (Some(order), Some(customer)) = (tuple.0, tuple.1) {
                println!("Found Order {} by {}", order.id, customer.name);
            }
        }
        None => {
            println!("No order found with that ID");
        }
    }

    Ok(())
}

/// RIGHT JOIN example
///
/// RIGHT JOIN returns all rows from the right table (customers),
/// and matching rows from the left table (orders).
/// The left entity may be None if there's no match.
async fn example_right_join(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("\n=== Example 7: RIGHT JOIN ===\n");
    println!("Find all customers, including those without orders\n");

    let results: Vec<JoinTuple2<Order, Customer>> = Order::join_right::<Customer>(
        "orders.customer_id = customers.id"
    )
    .fetch_all(pool)
    .await?;

    for result in results {
        match (result.0, result.1) {
            (Some(order), Some(customer)) => {
                println!("Customer {} has order {}: ${}",
                    customer.name,
                    order.id,
                    order.amount
                );
            }
            (None, Some(customer)) => {
                println!("Customer {} has no orders", customer.name);
            }
            _ => {
                println!("Unexpected: RIGHT JOIN should always have Customer");
            }
        }
    }

    Ok(())
}

// ========================================================================
// Main
// ========================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PostgreSQL
    println!("Connecting to PostgreSQL...");
    let pool = PgPool::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    println!("Setting up test data...");

    // Create test tables
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS customers (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) NOT NULL,
            region VARCHAR(50),
            tier VARCHAR(20)
        )
    "#)
    .execute(&pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS products (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            category VARCHAR(50),
            price INTEGER NOT NULL
        )
    "#)
    .execute(&pool)
    .await?;

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS orders (
            id VARCHAR(36) PRIMARY KEY,
            customer_id VARCHAR(36) NOT NULL,
            product_id VARCHAR(36) NOT NULL,
            amount INTEGER NOT NULL,
            status VARCHAR(20),
            created_at VARCHAR(50)
        )
    "#)
    .execute(&pool)
    .await?;

    // Clear existing data
    sqlx::query("DELETE FROM orders").execute(&pool).await?;
    sqlx::query("DELETE FROM products").execute(&pool).await?;
    sqlx::query("DELETE FROM customers").execute(&pool).await?;

    // Insert test data
    sqlx::query(r#"
        INSERT INTO customers (id, name, email, region, tier) VALUES
            ('cust-1', 'Alice Johnson', 'alice@example.com', 'north', 'gold'),
            ('cust-2', 'Bob Smith', 'bob@example.com', 'south', 'silver'),
            ('cust-3', 'Carol White', 'carol@example.com', 'east', 'bronze'),
            ('cust-4', 'David Brown', 'david@example.com', 'west', 'gold')
    "#)
    .execute(&pool)
    .await?;

    sqlx::query(r#"
        INSERT INTO products (id, name, category, price) VALUES
            ('prod-1', 'Laptop', 'Electronics', 1200),
            ('prod-2', 'Mouse', 'Electronics', 25),
            ('prod-3', 'Desk', 'Furniture', 500),
            ('prod-4', 'Chair', 'Furniture', 150)
    "#)
    .execute(&pool)
    .await?;

    sqlx::query(r#"
        INSERT INTO orders (id, customer_id, product_id, amount, status, created_at) VALUES
            ('order-1', 'cust-1', 'prod-1', 1200, 'completed', '2025-01-01'),
            ('order-2', 'cust-1', 'prod-2', 25, 'completed', '2025-01-02'),
            ('order-3', 'cust-2', 'prod-3', 500, 'pending', '2025-01-03'),
            ('order-4', 'cust-2', 'prod-4', 150, 'shipped', '2025-01-04'),
            ('order-5', 'cust-3', 'prod-1', 1200, 'completed', '2025-01-05')
    "#)
    .execute(&pool)
    .await?;

    println!("\nRunning JOIN query examples...\n");
    for _ in 0..60 { print!("="); }
    println!();

    // Run all examples
    example_inner_join(&pool).await?;
    example_left_join(&pool).await?;
    example_join_with_where(&pool).await?;
    example_three_table_join(&pool).await?;
    example_fetch_one(&pool).await?;
    example_fetch_optional(&pool).await?;
    example_right_join(&pool).await?;

    for _ in 0..60 { print!("="); }
    println!();
    println!("\nAll examples completed successfully!");

    Ok(())
}
