// 编译期索引分析测试
//
// 这个文件展示了 analyze_queries 宏的实际使用场景
//
// 运行测试查看编译期的索引推荐:
// cargo test --test compile_time_analysis_test

// 使用分析宏的测试模块
#[sqlx_struct_macros::analyze_queries]
mod real_world_example {
    // 注意：这是一个演示模块，展示真实的查询模式
    // 在实际项目中，这些会是真实的数据库操作代码

    /*
    典型的用户管理查询模式:

    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        username: String,
        status: String,
        created_at: i64,
        updated_at: i64,
    }

    // 查询1: 按email查找用户 (登录场景)
    User::where_query!("email = $1")

    // 查询2: 按username查找用户 (注册检查)
    User::where_query!("username = $1")

    // 查询3: 查找活跃用户，按创建时间排序 (用户列表)
    User::where_query!("status = $1 ORDER BY created_at DESC")

    // 查询4: 查找特定时间后活跃的用户 (报表查询)
    User::where_query!("status = $1 AND created_at > $2 ORDER BY created_at DESC")
    */

    // 模拟上述查询，宏会分析这些模式
    // 在真实场景中，这些会是实际的宏调用
    fn _demo_queries() {
        // 这些字符串模拟会被分析到的查询
        let _queries = vec![
            "User::where_query!(\"email = $1\")",
            "User::where_query!(\"username = $1\")",
            "User::where_query!(\"status = $1 ORDER BY created_at DESC\")",
            "User::where_query!(\"status = $1 AND created_at > $2 ORDER BY created_at DESC\")",
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_analysis_runs() {
        // 这个测试的存在是为了触发编译
        // 编译期分析宏会在编译时输出索引推荐
        assert!(true);
    }
}
