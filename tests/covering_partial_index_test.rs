// Test for Covering Index (INCLUDE) and Partial Index support
//
// To see the index recommendations, run:
// cargo test -p sqlx_struct_enhanced --test covering_partial_index_test --no-run

#[sqlx_struct_macros::analyze_queries]
mod covering_partial_queries {
    // Test 1: Covering Index - SELECT extra columns
    // User struct to provide field information
    #[allow(dead_code)]
    struct User {
        id: String,
        email: String,
        username: String,
        status: String,
        created_at: i64,
        deleted_at: Option<i64>,
    }

    // Test 1: Covering Index example
    #[allow(dead_code)]
    fn find_active_users_with_details() {
        // This should recommend a covering index with INCLUDE
        let _ = "User::make_query!(\"SELECT id, email, username FROM users WHERE status = $1 ORDER BY created_at\")";
    }

    // Test 2: Partial Index - Soft delete pattern
    #[allow(dead_code)]
    fn find_active_user_by_email() {
        // This should recommend a partial index with WHERE deleted_at IS NULL
        let _ = "User::make_query!(\"SELECT id, email FROM users WHERE deleted_at IS NULL AND email = $1\")";
    }

    // Test 3: Partial Index - Status with literal value
    #[allow(dead_code)]
    struct Order {
        id: String,
        user_id: String,
        status: String,
        total: i32,
        created_at: i64,
    }

    #[allow(dead_code)]
    fn find_active_orders() {
        // This should recommend a partial index with WHERE status = 'active'
        let _ = "Order::make_query!(\"SELECT id, user_id, total FROM orders WHERE status = 'active' AND created_at > $1\")";
    }
}

#[test]
fn test_covering_partial_index() {
    // This test just needs to compile
    // The real test is the compile-time output above
    assert!(true);
}
