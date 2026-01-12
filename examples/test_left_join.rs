use sqlx_struct_enhanced::{EnhancedCrud, join::JoinTuple2};
use sqlx::{FromRow, PgPool, Postgres, Row};
use sqlx::query::Query;
use sqlx::query::QueryAs;
use sqlx::database::HasArguments;

// Import Row trait at module level for derive macro
use sqlx::Row as _;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "orders"]
struct Order {
    pub id: String,
    pub customer_id: String,
    pub product_id: String,
    pub amount: i32,
    pub status: String,
}

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "customers"]
struct Customer {
    pub id: String,
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio").await?;

    // Setup
    sqlx::query("CREATE TABLE IF NOT EXISTS customers (id VARCHAR(36) PRIMARY KEY, name VARCHAR(100), email VARCHAR(100))")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS orders (id VARCHAR(36) PRIMARY KEY, customer_id VARCHAR(36), product_id VARCHAR(36), amount INTEGER, status VARCHAR(20))")
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM orders").execute(&pool).await?;
    sqlx::query("DELETE FROM customers").execute(&pool).await?;

    sqlx::query("INSERT INTO customers (id, name, email) VALUES ('cust-1', 'Alice', 'alice@test.com')")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO orders (id, customer_id, product_id, amount, status) VALUES ('order-1', 'cust-1', 'prod-1', 100, 'pending')")
        .execute(&pool)
        .await?;

    // Add orphan
    sqlx::query("INSERT INTO orders (id, customer_id, product_id, amount, status) VALUES ('order-orphan', 'cust-999', 'prod-1', 100, 'pending')")
        .execute(&pool)
        .await?;

    // Test LEFT JOIN
    println!("Testing LEFT JOIN...");
    let results: Vec<JoinTuple2<Order, Customer>> = Order::join_left::<Customer>(
        "orders.customer_id = customers.id"
    )
    .fetch_all(&pool)
    .await?;

    println!("Got {} results", results.len());

    for (i, result) in results.iter().enumerate() {
        match (&result.0, &result.1) {
            (Some(order), Some(customer)) => {
                println!("{}: Order {} with Customer {}", i, order.id, customer.name);
            }
            (Some(order), None) => {
                println!("{}: Order {} with NO Customer", i, order.id);
            }
            _ => {
                println!("{}: Unexpected", i);
            }
        }
    }

    // Cleanup
    sqlx::query("DROP TABLE IF EXISTS orders").execute(&pool).await?;
    sqlx::query("DROP TABLE IF EXISTS customers").execute(&pool).await?;

    Ok(())
}
