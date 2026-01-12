// Comprehensive example for Covering Index (INCLUDE) and Partial Index support
//
// Build this example to see the index recommendations:
// cargo build --example covering_partial_indexes

fn main() {
    println!("This example demonstrates covering index and partial index recommendations.");
    println!("Check the build output above for index recommendations!");

    // The analyze_queries macro below will generate recommendations during compilation

    // Example 1: Covering Index
    // Query: SELECT id, email, username FROM users WHERE status = $1 ORDER BY created_at
    // Expected: Index on (status, created_at) INCLUDE (id, email, username)

    // Example 2: Partial Index (Soft Delete)
    // Query: SELECT id, email FROM users WHERE deleted_at IS NULL AND email = $1
    // Expected: Index on (email) WHERE deleted_at IS NULL

    // Example 3: Partial Index (Status Filter)
    // Query: SELECT * FROM orders WHERE status = 'active' AND created_at > $1
    // Expected: Index on (created_at) WHERE status = 'active'
}

#[sqlx_struct_macros::analyze_queries]
mod examples {
    // User struct with common fields
    #[allow(dead_code)]
    struct User {
        id: String,
        email: String,
        username: String,
        status: String,
        created_at: i64,
        deleted_at: Option<i64>,
    }

    // Example 1: Covering Index
    // SELECT includes columns (id, email, username) that are not in WHERE/ORDER BY
    // These can be INCLUDED in the index to avoid table lookups
    #[allow(dead_code)]
    fn example_1_covering_index() {
        // This should detect:
        // - WHERE columns: status
        // - ORDER BY columns: created_at
        // - SELECT columns (not in WHERE/ORDER BY): id, email, username
        // Recommendation: CREATE INDEX ... (status, created_at) INCLUDE (id, email, username)
        let _ = "User::make_query!(\"SELECT id, email, username FROM users WHERE status = $1 ORDER BY created_at\")";
    }

    // Example 2: Partial Index (Soft Delete Pattern)
    // This is a common pattern where you only want to index non-deleted records
    #[allow(dead_code)]
    fn example_2_partial_soft_delete() {
        // This should detect:
        // - WHERE columns: email, deleted_at IS NULL
        // - Partial index condition: deleted_at IS NULL
        // Recommendation: CREATE INDEX ... (email) WHERE deleted_at IS NULL
        let _ = "User::make_query!(\"SELECT id, email FROM users WHERE deleted_at IS NULL AND email = $1\")";
    }

    // Example 3: Partial Index (Status Filter)
    // Index only active orders to reduce index size
    #[allow(dead_code)]
    struct Order {
        id: String,
        user_id: String,
        status: String,
        total: i32,
        created_at: i64,
    }

    #[allow(dead_code)]
    fn example_3_partial_status() {
        // This should detect:
        // - WHERE columns: created_at, status = 'active'
        // - Partial index condition: status = 'active'
        // Recommendation: CREATE INDEX ... (created_at) WHERE status = 'active'
        let _ = "Order::make_query!(\"SELECT id, user_id, total FROM orders WHERE status = 'active' AND created_at > $1\")";
    }

    // Example 4: Combined Covering + Partial Index
    // A complex query that benefits from both optimizations
    #[allow(dead_code)]
    struct Task {
        id: String,
        title: String,
        description: String,
        user_id: String,
        status: String,
        priority: i32,
        created_at: i64,
    }

    #[allow(dead_code)]
    fn example_4_combined() {
        // This should detect:
        // - WHERE columns: user_id, priority
        // - ORDER BY columns: priority
        // - SELECT columns to INCLUDE: title, description
        // - Partial condition: status = 'pending'
        // Recommendation: CREATE INDEX ... (user_id, priority) INCLUDE (title, description) WHERE status = 'pending'
        let _ = "Task::make_query!(\"SELECT id, title, description, priority FROM tasks WHERE user_id = $1 AND status = 'pending' ORDER BY priority\")";
    }
}
