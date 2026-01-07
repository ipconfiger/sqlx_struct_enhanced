// 编译期索引分析示例 - Day 2 增强版
//
// 运行方式:
// cargo build --example compile_time_analysis
//
// 查看编译器输出中的索引推荐
//
// Day 2 新功能:
// - 支持范围运算符 (>, <, >=, <=)
// - 支持 IN 子句
// - 支持 LIKE 子句
// - 智能优先级排序 (等值 > IN > 范围 > LIKE > ORDER BY)

fn main() {
    println!("=================================================================");
    println!("SQLx Struct Enhanced - Compile-Time Index Analysis (Day 2)");
    println!("=================================================================");
    println!();
    println!("Check the compiler output above!");
    println!("You should see index recommendations printed during compilation.");
    println!();
    println!("Day 2 New Features:");
    println!("  ✅ Range operators: >, <, >=, <=");
    println!("  ✅ IN clauses");
    println!("  ✅ LIKE clauses");
    println!("  ✅ Smart priority ordering: Equality > IN > Range > LIKE > ORDER BY");
    println!();
    println!("The #[analyze_queries] macro analyzed all queries in the module");
    println!("and recommended indexes based on the SQL patterns found.");
    println!();
    println!("=================================================================");
}

// 使用分析宏的模块 - Day 2 增强版
#[sqlx_struct_macros::analyze_queries]
mod enhanced_queries {
    /*
    Day 2 新功能演示 - 实际使用中的代码：

    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
        age: i32,
        name: String,
        category_id: String,
    }

    impl User {
        // 查询1: 等值条件
        fn find_by_email(email: &str) {
            let _ = User::where_query!("email = $1");
            // 推荐: idx_user_email (email)
        }

        // 查询2: IN 子句
        fn find_by_statuses(statuses: &[&str]) {
            let _ = User::where_query!("status IN ($1, $2, $3)");
            // 推荐: idx_user_status (status)
        }

        // 查询3: 范围条件 - 大于
        fn find_created_after(timestamp: i64) {
            let _ = User::where_query!("created_at > $1");
            // 推荐: idx_user_created_at (created_at)
        }

        // 查询4: 范围条件 - 区间查询
        fn find_by_age_range(min_age: i32, max_age: i32) {
            let _ = User::where_query!("age >= $1 AND age <= $2");
            // 推荐: idx_user_age (age)
        }

        // 查询5: LIKE 条件
        fn find_by_name_pattern(pattern: &str) {
            let _ = User::where_query!("name LIKE $1");
            // 推荐: idx_user_name (name)
        }

        // 查询6: 复杂混合查询 - 等值 + IN + 范围 + ORDER BY
        fn find_tasks(tenant_id: &str, statuses: &[&str], min_priority: i32) {
            let _ = User::where_query!(
                "tenant_id = $1 AND status IN ($2, $3) AND priority > $4 ORDER BY created_at DESC"
            );
            // 推荐: idx_user_tenant_id_status_priority_created_at
            //       (tenant_id, status, priority, created_at)
            // 优先级: 等值(tenant_id) > IN(status) > 范围(priority) > ORDER BY(created_at)
        }

        // 查询7: IN + LIKE 组合
        fn find_articles_by_category_and_title(categories: &[&str], title_pattern: &str) {
            let _ = User::where_query!("category IN ($1, $2) AND title LIKE $3");
            // 推荐: idx_user_category_title (category, title)
            // 优先级: IN(category) > LIKE(title)
        }
    }
    */

    // 模拟结构体定义（用于演示分析功能）
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
        title: String,
    }

    // 为了演示分析功能，这里模拟会被分析到的查询字符串
    // Day 2 新功能示例：

    // 等值条件
    const _QUERY1: &str = "User::where_query!(\"email = $1\")";

    // IN 子句
    const _QUERY2: &str = "User::where_query!(\"status IN ($1, $2, $3)\")";

    // 范围条件 - 大于
    const _QUERY3: &str = "User::where_query!(\"created_at > $1\")";

    // 范围条件 - 区间
    const _QUERY4: &str = "User::where_query!(\"age >= $1 AND age <= $2\")";

    // LIKE 条件
    const _QUERY5: &str = "User::where_query!(\"name LIKE $1\")";

    // 复杂混合查询：等值 + IN + 范围 + ORDER BY
    const _QUERY6: &str = "User::where_query!(\"tenant_id = $1 AND status IN ($2, $3) AND priority > $4 ORDER BY created_at DESC\")";

    // IN + LIKE 组合
    const _QUERY7: &str = "User::where_query!(\"category_id IN ($1, $2) AND title LIKE $3\")";
}
