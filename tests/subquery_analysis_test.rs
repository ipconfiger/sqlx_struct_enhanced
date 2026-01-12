// Test for Subquery Analysis support
//
// To see the index recommendations, run:
// cargo test -p sqlx_struct_enhanced --test subquery_analysis_test --no-run

#[sqlx_struct_macros::analyze_queries]
mod subquery_examples {
    // Test 1: WHERE IN subquery
    #[allow(dead_code)]
    struct Order {
        id: String,
        user_id: String,
        total: i32,
        status: String,
    }

    #[allow(dead_code)]
    fn find_orders_with_high_total_users() {
        // Should detect subquery and recommend index on user_id in users table
        let _ = "Order::make_query!(\"SELECT * FROM orders WHERE user_id IN (SELECT id FROM users WHERE total_orders > 100) AND status = $1\")";
    }

    // Test 2: EXISTS subquery
    #[allow(dead_code)]
    struct User {
        id: String,
        email: String,
        status: String,
    }

    #[allow(dead_code)]
    fn find_users_with_pending_orders() {
        // Should detect EXISTS subquery and recommend index on user_id in orders table
        let _ = "User::make_query!(\"SELECT * FROM users WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id AND orders.status = 'pending') AND status = $1\")";
    }

    // Test 3: FROM subquery
    #[allow(dead_code)]
    fn get_user_order_stats() {
        // Should detect FROM subquery and recommend indexes within the subquery
        let _ = "Order::make_query!(\"SELECT * FROM (SELECT user_id, COUNT(*) as order_count FROM orders GROUP BY user_id) AS stats WHERE stats.order_count > $1\")";
    }

    // Test 4: Multiple subqueries
    #[allow(dead_code)]
    fn complex_subquery_example() {
        // Should detect both WHERE and EXISTS subqueries
        let _ = "User::make_query!(\"SELECT * FROM users WHERE id IN (SELECT user_id FROM orders WHERE total > $1) AND EXISTS (SELECT 1 FROM payments WHERE payments.user_id = users.id AND payments.status = 'pending')\")";
    }
}

#[test]
fn test_subquery_analysis() {
    // This test just needs to compile
    // The real test is the compile-time output above
    assert!(true);
}
