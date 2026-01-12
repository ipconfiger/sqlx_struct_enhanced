// Integration Tests for Entity Tuple JOIN Queries
//
// Tests the new JOIN query functionality that returns type-safe entity tuples
// like Vec<(Option<Order>, Option<Customer>)>

#[cfg(test)]
mod join_tuple_integration_tests {
    use sqlx_struct_enhanced::{EnhancedCrud, join::JoinTuple2};
    use sqlx::{FromRow, PgPool, Postgres, Row};
    use sqlx::query::Query;
    use sqlx::query::QueryAs;
    use sqlx::database::HasArguments;
    use serial_test::serial;

    // Import Row trait at module level for derive macro
    use sqlx::Row as _;

    // All fields must be pub for JOIN query deserialization
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
        pub email: String,
        pub region: String,
    }

    #[derive(Debug, Clone, FromRow, EnhancedCrud)]
    #[table_name = "products"]
    struct Product {
        pub id: String,
        pub name: String,
        pub category: String,
        pub price: i32,
    }

    // ============================================================================
    // Test Setup
    // ============================================================================

    async fn get_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

        sqlx::PgPool::connect(&database_url).await
            .expect("Failed to connect to test database")
    }

    async fn setup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
        // Create test tables
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS customers (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                email VARCHAR(100) NOT NULL,
                region VARCHAR(50)
            )
        "#)
        .execute(pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS products (
                id VARCHAR(36) PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                category VARCHAR(50),
                price INTEGER NOT NULL
            )
        "#)
        .execute(pool)
        .await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS orders (
                id VARCHAR(36) PRIMARY KEY,
                customer_id VARCHAR(36) NOT NULL,
                product_id VARCHAR(36) NOT NULL,
                amount INTEGER NOT NULL,
                status VARCHAR(20)
            )
        "#)
        .execute(pool)
        .await?;

        // Clear existing data
        sqlx::query("DELETE FROM orders").execute(pool).await?;
        sqlx::query("DELETE FROM products").execute(pool).await?;
        sqlx::query("DELETE FROM customers").execute(pool).await?;

        // Insert test customers
        sqlx::query(r#"
            INSERT INTO customers (id, name, email, region) VALUES
                ('cust-1', 'Alice Johnson', 'alice@example.com', 'north'),
                ('cust-2', 'Bob Smith', 'bob@example.com', 'south'),
                ('cust-3', 'Carol White', 'carol@example.com', 'east')
        "#)
        .execute(pool)
        .await?;

        // Insert test products
        sqlx::query(r#"
            INSERT INTO products (id, name, category, price) VALUES
                ('prod-1', 'Laptop', 'Electronics', 1200),
                ('prod-2', 'Mouse', 'Electronics', 25),
                ('prod-3', 'Desk', 'Furniture', 500)
        "#)
        .execute(pool)
        .await?;

        // Insert test orders
        sqlx::query(r#"
            INSERT INTO orders (id, customer_id, product_id, amount, status) VALUES
                ('order-1', 'cust-1', 'prod-1', 1200, 'completed'),
                ('order-2', 'cust-1', 'prod-2', 25, 'pending'),
                ('order-3', 'cust-2', 'prod-3', 500, 'shipped'),
                ('order-4', 'cust-2', 'prod-1', 1200, 'completed')
        "#)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DROP TABLE IF EXISTS orders").execute(pool).await?;
        sqlx::query("DROP TABLE IF EXISTS products").execute(pool).await?;
        sqlx::query("DROP TABLE IF EXISTS customers").execute(pool).await?;
        Ok(())
    }

    // ============================================================================
    // INNER JOIN Tests
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_inner_join_basic() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await.unwrap();

        // Should have 4 orders, all with matching customers
        assert_eq!(results.len(), 4);

        // All entities should be Some for INNER JOIN
        for result in &results {
            assert!(result.0.is_some(), "Order should always be Some for INNER JOIN");
            assert!(result.1.is_some(), "Customer should always be Some for INNER JOIN");
        }

        // Verify data integrity
        let result = &results[0];
        assert_eq!(result.0.as_ref().unwrap().id, "order-1");
        assert_eq!(result.1.as_ref().unwrap().name, "Alice Johnson");

        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_inner_join_with_where() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .where_("orders.status = {}", &["completed"])
        .fetch_all(&pool)
        .await.unwrap();

        // Should have 2 completed orders
        assert_eq!(results.len(), 2);

        for result in results {
            let order = result.0.as_ref().unwrap();
            assert_eq!(order.status, "completed");
        }

        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_inner_join_multiple_where_conditions() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .where_("orders.status = {} AND customers.region = {}", &["completed", "south"])
        .fetch_all(&pool)
        .await.unwrap();

        // Should have 1 order (completed, south region)
        assert_eq!(results.len(), 1);

        let result = &results[0];
        let order = result.0.as_ref().unwrap();
        let customer = result.1.as_ref().unwrap();

        assert_eq!(order.status, "completed");
        assert_eq!(customer.region, "south");

        cleanup_test_data(&pool).await.unwrap();
    }

    // ============================================================================
    // LEFT JOIN Tests
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_left_join_with_orphans() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        // Add an order with no matching customer (violates FK, but test uses direct SQL)
        sqlx::query("INSERT INTO orders (id, customer_id, product_id, amount, status) VALUES ('order-orphan', 'cust-999', 'prod-1', 100, 'pending')")
            .execute(&pool)
            .await
            .unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_left::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // Should have 5 orders (4 with customers + 1 orphan)
        assert_eq!(results.len(), 5);

        // Find the orphan order
        let orphan = results.iter()
            .find(|r| r.0.as_ref().map(|o| &o.id) == Some(&"order-orphan".to_string()))
            .expect("Should find orphan order");

        assert!(orphan.0.is_some(), "Order should be Some");
        assert!(orphan.1.is_none(), "Customer should be None for orphan");

        cleanup_test_data(&pool).await.unwrap();
    }

    // ============================================================================
    // Fetch Methods Tests
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_fetch_one_found() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let result: JoinTuple2<Order, Customer> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .where_("orders.id = {}", &["order-1"])
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_some());

        let order = result.0.as_ref().unwrap();
        assert_eq!(order.id, "order-1");

        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_fetch_optional_found() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let result: Option<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .where_("orders.id = {}", &["order-1"])
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(result.is_some());

        let tuple = result.unwrap();
        assert!(tuple.0.is_some());
        assert!(tuple.1.is_some());

        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_fetch_optional_not_found() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let result: Option<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .where_("orders.id = {}", &["non-existent"])
        .fetch_optional(&pool)
        .await
        .unwrap();

        assert!(result.is_none());

        cleanup_test_data(&pool).await.unwrap();
    }

    // ============================================================================
    // Column Conflict Resolution Tests
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_column_conflict_both_tables_have_id() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // Both tables have 'id' column, qualified columns should resolve conflict
        assert_eq!(results.len(), 4);

        for result in results {
            let order = result.0.as_ref().unwrap();
            let customer = result.1.as_ref().unwrap();

            // Should get correct IDs from both tables
            assert!(!order.id.is_empty());
            assert!(!customer.id.is_empty());

            // IDs should be different (order ID vs customer ID)
            assert_ne!(order.id, customer.id);
        }

        cleanup_test_data(&pool).await.unwrap();
    }

    // ============================================================================
    // NULL Handling Tests
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_inner_join_no_nulls() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // INNER JOIN should never have NULL entities
        for result in results {
            assert!(result.0.is_some(), "INNER JOIN: Order should never be NULL");
            assert!(result.1.is_some(), "INNER JOIN: Customer should never be NULL");
        }

        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_null_handling_in_join_result() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        for result in results {
            // Should be able to safely unwrap
            if let (Some(order), Some(customer)) = (result.0, result.1) {
                // Should have valid data
                assert!(!order.id.is_empty());
                assert!(!customer.name.is_empty());
            }
        }

        cleanup_test_data(&pool).await.unwrap();
    }

    // ============================================================================
    // Data Integrity Tests
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_join_data_integrity() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // Verify foreign key relationships are correct
        for result in results {
            let order = result.0.as_ref().unwrap();
            let customer = result.1.as_ref().unwrap();

            // customer_id in order should match customer id
            assert_eq!(order.customer_id, customer.id);
        }

        cleanup_test_data(&pool).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_join_all_fields_populated() {
        let pool = get_test_pool().await;
        setup_test_data(&pool).await.unwrap();

        let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
            "orders.customer_id = customers.id"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // Verify all fields are populated
        for result in results {
            let order = result.0.as_ref().unwrap();
            let customer = result.1.as_ref().unwrap();

            // Check Order fields
            assert!(!order.id.is_empty());
            assert!(!order.customer_id.is_empty());
            assert!(!order.product_id.is_empty());
            assert_ne!(order.amount, 0);
            assert!(!order.status.is_empty());

            // Check Customer fields
            assert!(!customer.id.is_empty());
            assert!(!customer.name.is_empty());
            assert!(!customer.email.is_empty());
            assert!(!customer.region.is_empty());
        }

        cleanup_test_data(&pool).await.unwrap();
    }
}

// ============================================================================
// Unit Tests (no database required)
// ============================================================================

#[cfg(test)]
mod join_tuple_unit_tests {
    use sqlx_struct_enhanced::{EnhancedCrud, join::{JoinTuple2, SchemeAccessor}};
    use sqlx::{FromRow, Postgres, Row};
    use sqlx::query::Query;
    use sqlx::query::QueryAs;
    use sqlx::database::HasArguments;

    // Import Row trait at module level for derive macro
    use sqlx::Row as _;

    // All fields must be pub for JOIN query deserialization
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
        pub email: String,
        pub region: String,
    }

    #[derive(Debug, Clone, FromRow, EnhancedCrud)]
    #[table_name = "products"]
    struct Product {
        pub id: String,
        pub name: String,
        pub category: String,
        pub price: i32,
    }

    // ============================================================================
    // SchemeAccessor Tests
    // ============================================================================

    #[test]
    fn test_scheme_accessor_generated() {
        // Verify that SchemeAccessor is implemented for our structs
        // This is a compile-time check - if it compiles, the trait is implemented

        // These will fail to compile if SchemeAccessor is not implemented
        let _order_scheme = Order::get_scheme();
        let _customer_scheme = Customer::get_scheme();
        let _product_scheme = Product::get_scheme();

        // Verify scheme metadata
        assert_eq!(_order_scheme.table_name(), "orders");
        assert_eq!(_customer_scheme.table_name(), "customers");
        assert_eq!(_product_scheme.table_name(), "products");

        // Verify column definitions exist
        assert!(!_order_scheme.column_definitions().is_empty());
        assert!(!_customer_scheme.column_definitions().is_empty());
    }

    // ============================================================================
    // JoinTuple2 Tests
    // ============================================================================

    #[test]
    fn test_join_tuple2_creation() {
        let tuple = JoinTuple2(
            Some(Order {
                id: "test-order".to_string(),
                customer_id: "cust-1".to_string(),
                product_id: "prod-1".to_string(),
                amount: 100,
                status: "pending".to_string(),
            }),
            Some(Customer {
                id: "cust-1".to_string(),
                name: "Test Customer".to_string(),
                email: "test@example.com".to_string(),
                region: "north".to_string(),
            })
        );

        assert!(tuple.0.is_some());
        assert!(tuple.1.is_some());

        let order = tuple.0.unwrap();
        let customer = tuple.1.unwrap();

        assert_eq!(order.id, "test-order");
        assert_eq!(customer.name, "Test Customer");
    }

    #[test]
    fn test_join_tuple2_with_nulls() {
        let tuple = JoinTuple2::<Order, Customer>(
            Some(Order {
                id: "test-order".to_string(),
                customer_id: "cust-1".to_string(),
                product_id: "prod-1".to_string(),
                amount: 100,
                status: "pending".to_string(),
            }),
            None
        );

        assert!(tuple.0.is_some());
        assert!(tuple.1.is_none());
    }

    #[test]
    fn test_join_tuple2_both_null() {
        let tuple = JoinTuple2::<Order, Customer>(None, None);

        assert!(tuple.0.is_none());
        assert!(tuple.1.is_none());
    }

    #[test]
    fn test_join_tuple2_pattern_matching() {
        let tuple = JoinTuple2(
            Some(Order {
                id: "test-order".to_string(),
                customer_id: "cust-1".to_string(),
                product_id: "prod-1".to_string(),
                amount: 100,
                status: "pending".to_string(),
            }),
            Some(Customer {
                id: "cust-1".to_string(),
                name: "Test Customer".to_string(),
                email: "test@example.com".to_string(),
                region: "north".to_string(),
            })
        );

        match (tuple.0, tuple.1) {
            (Some(order), Some(customer)) => {
                assert_eq!(order.id, "test-order");
                assert_eq!(customer.name, "Test Customer");
            }
            _ => panic!("Expected both entities to be Some"),
        }
    }
}
