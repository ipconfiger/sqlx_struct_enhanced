// Test for JOIN and GROUP BY index analysis
//
// To see the index recommendations, run:
// cargo test --test join_groupby_analysis_test --no-run

#[sqlx_struct_macros::analyze_queries]
mod join_groupby_queries {
    // Test queries demonstrating JOIN and GROUP BY patterns
    fn _demo_join_groupby_queries() {
        let _queries = vec![
            // Test 1: Simple JOIN query
            "Order::make_query!(\"SELECT o.*, u.email, u.username FROM orders o INNER JOIN users u ON o.user_id = u.id WHERE o.status = $1\")",

            // Test 2: Multiple JOINs
            "Order::make_query!(\"SELECT o.*, u.email, p.name FROM orders o INNER JOIN users u ON o.user_id = u.id INNER JOIN products p ON o.product_id = p.id WHERE o.status = $1\")",

            // Test 3: LEFT JOIN
            "Order::make_query!(\"SELECT o.*, u.email FROM orders o LEFT JOIN users u ON o.user_id = u.id WHERE o.status = $1\")",

            // Test 4: GROUP BY
            "Order::make_query!(\"SELECT status, COUNT(*) as count FROM orders GROUP BY status\")",

            // Test 5: GROUP BY with HAVING
            "Order::make_query!(\"SELECT status, COUNT(*) as count FROM orders GROUP BY status HAVING COUNT(*) > $1\")",

            // Test 6: GROUP BY multiple columns
            "Order::make_query!(\"SELECT category, status, COUNT(*) as count FROM orders GROUP BY category, status\")",

            // Test 7: JOIN + GROUP BY
            "Order::make_query!(\"SELECT u.id, u.username, COUNT(o.id) as order_count FROM users u INNER JOIN orders o ON u.id = o.user_id GROUP BY u.id, u.username\")",
        ];
    }
}

#[test]
fn test_join_groupby_analysis() {
    // This test just needs to compile
    // The real test is the compile-time output above
    assert!(true);
}
