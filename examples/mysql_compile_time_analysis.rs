// MySQL编译期索引分析示例
//
// 运行方式:
// cargo build --example mysql_compile_time_analysis --features mysql
//
// 查看编译器输出中的索引推荐
//
// MySQL版本兼容性:
// - MySQL 8.0+: 支持INCLUDE索引（覆盖索引）
// - MySQL 5.7: 不支持INCLUDE，会显示注释提示

fn main() {
    println!("=================================================================");
    println!("SQLx Struct Enhanced - MySQL Compile-Time Index Analysis");
    println!("=================================================================");
    println!();
    println!("Check the compiler output above!");
    println!("You should see MySQL-specific index recommendations.");
    println!();
    println!("MySQL Features:");
    println!("  ✅ All WHERE conditions (=, >, <, IN, LIKE, etc.)");
    println!("  ✅ JOIN analysis");
    println!("  ✅ GROUP BY analysis");
    println!("  ✅ INCLUDE indexes (MySQL 8.0+ only)");
    println!("  ❌ Partial indexes (not supported by MySQL)");
    println!();
    println!("The #[analyze_queries] macro analyzed all queries");
    println!("and recommended MySQL-compatible indexes.");
    println!();
    println!("=================================================================");
}

// MySQL分析模块
#[sqlx_struct_macros::analyze_queries]
mod mysql_queries {
    // 模拟结构体定义（用于演示MySQL分析功能）
    #[allow(dead_code)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
        age: i32,
        name: String,
        category_id: String,
        tenant_id: String,
        priority: i32,
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

    // JOIN查询
    const _QUERY5: &str = "SELECT * FROM orders INNER JOIN users ON orders.user_id = users.id";
    // 推荐: CREATE INDEX idx_orders_user_id_join ON orders (user_id)

    // GROUP BY查询
    const _QUERY6: &str = "SELECT status, COUNT(*) FROM orders GROUP BY status";
    // 推荐: CREATE INDEX idx_orders_status_group ON orders (status)

    // 复杂查询：等值 + 范围 + ORDER BY
    const _QUERY7: &str = "User::where_query!(\"tenant_id = $1 AND status IN ($2, $3) AND priority > $4 ORDER BY created_at DESC\")";
    // 推荐 (MySQL 8.0+): CREATE INDEX idx_user_tenant_id_status_priority_created_at ON user (tenant_id, status, priority, created_at) INCLUDE (id)
    // 推荐 (MySQL 5.7): CREATE INDEX idx_user_tenant_id_status_priority_created_at ON user (tenant_id, status, priority, created_at) -- INCLUDE requires MySQL 8.0+ (consider including: id)
}
