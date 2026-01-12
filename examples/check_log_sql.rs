// 最简单的 log_sql 测试
// 将此文件复制到你的项目中，运行：cargo run --check_log_sql --features log_sql

use sqlx::FromRow;
use sqlx::Row;
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct SimpleTest {
    pub id: String,
    pub name: String,
}

fn main() {
    println!("========== 开始测试 log_sql feature ==========");
    println!("\n步骤1: 创建实例");
    let mut user = SimpleTest {
        id: "test-id".to_string(),
        name: "Test".to_string(),
    };

    println!("\n步骤2: 调用 insert_bind() - 应该在上方看到 [SQLxEnhanced] INSERT SQL");
    let _insert = user.insert_bind();

    println!("\n步骤3: 调用 SimpleTest::by_pk() - 应该在上方看到 [SQLxEnhanced] SELECT BY PK SQL");
    let _select = SimpleTest::by_pk();

    println!("\n========== 测试完成 ==========");
    println!("\n如果你没有看到任何 [SQLxEnhanced] 开头的日志，说明 feature 未生效");
    println!("\n请检查：");
    println!("1. Cargo.toml 中是否有: features = [\"log_sql\"]");
    println!("2. 是否完全重新编译: cargo clean && cargo build");
    println!("3. 是否正确运行: cargo run --features log_sql");
}
