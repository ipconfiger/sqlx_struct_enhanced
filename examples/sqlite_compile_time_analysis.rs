// SQLite编译期索引分析示例
//
// 运行方式:
// cargo build --example sqlite_compile_time_analysis --features sqlite
//
// 查看编译器输出中的索引推荐
//
// SQLite特性:
// - 支持部分索引（Partial Indexes with WHERE clause）
// - 不支持INCLUDE索引（覆盖索引）
// - 仅支持INNER JOIN和LEFT JOIN（不支持RIGHT JOIN）

fn main() {
    println!("=================================================================");
    println!("SQLx Struct Enhanced - SQLite Compile-Time Index Analysis");
    println!("=================================================================");
    println!();
    println!("Check the compiler output above!");
    println!("You should see SQLite-specific index recommendations.");
    println!();
    println!("SQLite Features:");
    println!("  ✅ All WHERE conditions (=, >, <, IN, LIKE, etc.)");
    println!("  ✅ JOIN analysis (INNER and LEFT only)");
    println!("  ✅ GROUP BY analysis");
    println!("  ✅ Partial indexes (WHERE clause)");
    println!("  ❌ INCLUDE indexes (not supported by SQLite)");
    println!();
    println!("The #[analyze_queries] macro analyzed all queries");
    println!("and recommended SQLite-compatible indexes.");
    println!();
    println!("=================================================================");
}

// SQLite分析模块
#[sqlx_struct_macros::analyze_queries]
mod sqlite_queries {
    // 模拟结构体定义（用于演示SQLite分析功能）
    #[allow(dead_code)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
        age: i32,
        name: String,
        active: bool,
    }

    #[allow(dead_code)]
    struct Order {
        id: String,
        user_id: String,
        status: String,
        amount: i32,
        created_at: i64,
    }

    // 基础查询示例
    const _QUERY1: &str = "User::where_query!(\"email = $1\")";
    // 推荐: CREATE INDEX idx_user_email ON user (email)

    // IN 子句
    const _QUERY2: &str = "User::where_query!(\"status IN ($1, $2, $3)\")";
    // 推荐: CREATE INDEX idx_user_status ON user (status)

    // 范围条件
    const _QUERY3: &str = "User::where_query!(\"created_at > $1\")";
    // 推荐: CREATE INDEX idx_user_created_at ON user (created_at)

    // LIKE 条件
    const _QUERY4: &str = "User::where_query!(\"name LIKE $1\")";
    // 推荐: CREATE INDEX idx_user_name ON user (name)

    // 部分索引示例（SQLite特有功能）
    const _QUERY5: &str = "User::where_query!(\"active = $1 AND created_at > $2\")";
    // 推荐: CREATE INDEX idx_user_active_created_at ON user (active, created_at) WHERE active = true

    // LEFT JOIN查询（SQLite支持）
    const _QUERY6: &str = "SELECT * FROM orders LEFT JOIN users ON orders.user_id = users.id";
    // 推荐: CREATE INDEX idx_orders_user_id_join ON orders (user_id)

    // GROUP BY查询
    const _QUERY7: &str = "SELECT status, COUNT(*) FROM orders GROUP BY status";
    // 推荐: CREATE INDEX idx_orders_status_group ON orders (status)

    // 复杂查询：等值 + 范围 + ORDER BY（会提示INCLUDE不支持）
    const _QUERY8: &str = "User::where_query!(\"status = $1 AND created_at > $2 ORDER BY name ASC\")";
    // 推荐: CREATE INDEX idx_user_status_created_at_name ON user (status, created_at, name)
    //       -- Note: INCLUDE not supported, consider adding these columns to the index
}
