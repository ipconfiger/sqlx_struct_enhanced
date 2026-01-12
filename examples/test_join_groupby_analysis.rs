// Test example for JOIN and GROUP BY index analysis
//
// To see the index recommendations, run:
// cargo build --example test_join_groupby_analysis

#[sqlx_struct_macros::analyze_queries]
mod join_groupby_queries {
    #[derive(sqlx_struct_enhanced::EnhancedCrud)]
    struct Order {
        id: String,
        user_id: String,
        product_id: String,
        status: String,
        total: i32,
        created_at: i64,
    }

    #[derive(sqlx_struct_enhanced::EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        username: String,
        category: String,
        status: String,
    }

    #[derive(sqlx_struct_enhanced::EnhancedCrud)]
    struct Product {
        id: String,
        name: String,
        price: i32,
        category: String,
    }

    impl Order {
        // Test 1: Simple JOIN query - should recommend index on user_id
        fn find_orders_with_user() {
            let _ = Order::make_query(
                "SELECT o.*, u.email, u.username
                 FROM orders o
                 INNER JOIN users u ON o.user_id = u.id
                 WHERE o.status = $1"
            );
        }

        // Test 2: Multiple JOINs - should recommend indexes on user_id and product_id
        fn find_orders_with_user_and_product() {
            let _ = Order::make_query(
                "SELECT o.*, u.email, p.name
                 FROM orders o
                 INNER JOIN users u ON o.user_id = u.id
                 INNER JOIN products p ON o.product_id = p.id
                 WHERE o.status = $1"
            );
        }

        // Test 3: LEFT JOIN - should recommend index on user_id
        fn find_orders_with_optional_user() {
            let _ = Order::make_query(
                "SELECT o.*, u.email
                 FROM orders o
                 LEFT JOIN users u ON o.user_id = u.id
                 WHERE o.status = $1"
            );
        }

        // Test 4: GROUP BY - should recommend index on category
        fn count_orders_by_status() {
            let _ = Order::make_query(
                "SELECT status, COUNT(*) as count
                 FROM orders
                 GROUP BY status"
            );
        }

        // Test 5: GROUP BY with HAVING - should recommend index on status
        fn find_frequent_statuses() {
            let _ = Order::make_query(
                "SELECT status, COUNT(*) as count
                 FROM orders
                 GROUP BY status
                 HAVING COUNT(*) > $1"
            );
        }

        // Test 6: GROUP BY multiple columns - should recommend indexes on category and status
        fn count_orders_by_category_and_status() {
            let _ = Order::make_query(
                "SELECT category, status, COUNT(*) as count
                 FROM orders
                 GROUP BY category, status"
            );
        }

        // Test 7: JOIN + GROUP BY - combination test
        fn count_orders_per_user() {
            let _ = Order::make_query(
                "SELECT u.id, u.username, COUNT(o.id) as order_count
                 FROM users u
                 INNER JOIN orders o ON u.id = o.user_id
                 GROUP BY u.id, u.username"
            );
        }
    }
}

fn main() {
    println!("This example demonstrates compile-time index analysis for JOIN and GROUP BY queries.");
    println!("Check the build output above for index recommendations!");
}
