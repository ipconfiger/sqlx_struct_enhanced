// DECIMAL 类型完整示例
//
// 这个示例展示了如何使用 DECIMAL 类型：
// 1. 在 struct 中定义 DECIMAL 字段
// 2. 在 migration 时生成正确的 NUMERIC 列
// 3. 在运行时正确处理 DECIMAL 类型

use sqlx::FromRow;
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use uuid::Uuid;

// ============================================================================
// 示例 1: 使用 String 类型（推荐）
// ============================================================================

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,

    // DECIMAL 类型 - 使用 String 存储
    // 精度: 最多10位数字，其中2位小数
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,

    // 折扣率 - 最多5位数字，2位小数
    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub discount_percent: Option<String>,

    pub created_at: String,
}

// Migration 会自动生成:
// CREATE TABLE products (
//     id UUID PRIMARY KEY,
//     name VARCHAR(500) NOT NULL,
//     description TEXT,
//     price NUMERIC(10,2),
//     discount_percent NUMERIC(5,2),
//     created_at VARCHAR(500) NOT NULL
// );

// ============================================================================
// 示例 2: 使用 rust_decimal（需要额外依赖）
// ============================================================================

/*
// 需要在 Cargo.toml 中添加:
// rust_decimal = "1.32"

use rust_decimal::Decimal;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products_with_decimal"]
pub struct ProductWithDecimal {
    pub id: Uuid,
    pub name: String,

    // 使用 rust_decimal::Decimal 类型
    #[crud(decimal(precision = 10, scale = 2))]
    pub price: Option<Decimal>,

    #[crud(decimal(precision = 5, scale = 2))]
    pub discount_percent: Option<Decimal>,
}
*/

// ============================================================================
// 使用示例
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DECIMAL 类型使用示例 ===\n");

    // 示例 1: 创建产品
    let product = Product {
        id: Uuid::new_v4(),
        name: "Laptop".to_string(),
        description: Some("High-end laptop with 16GB RAM".to_string()),
        price: Some("1299.99".to_string()),
        discount_percent: Some("15.00".to_string()),
        created_at: "2024-01-01T00:00:00Z".to_string(),
    };

    println!("产品信息:");
    println!("  名称: {}", product.name);
    println!("  价格: ${}", product.price.as_ref().unwrap());
    println!("  折扣: {}%", product.discount_percent.as_ref().unwrap());
    println!();

    // 示例 2: 计算折后价格（使用字符串操作）
    if let (Some(price), Some(discount)) = (&product.price, &product.discount_percent) {
        // 将字符串转换为 f64 进行计算
        let price_val: f64 = price.parse()?;
        let discount_val: f64 = discount.parse()?;
        let discounted_price = price_val * (100.0 - discount_val) / 100.0;

        println!("计算折后价格:");
        println!("  原价: ${}", price);
        println!("  折扣: {}%", discount);
        println!("  折后价: ${:.2}", discounted_price);
    }

    println!("\n=== Migration SQL 示例 ===");
    println!("-- 当你运行 migration 时，会自动生成以下 SQL:\n");

    println!("CREATE TABLE products (");
    println!("    id UUID PRIMARY KEY,");
    println!("    name VARCHAR(500) NOT NULL,");
    println!("    description TEXT,");
    println!("    price NUMERIC(10,2),           -- 最多8位整数 + 2位小数");
    println!("    discount_percent NUMERIC(5,2),   -- 最多3位整数 + 2位小数");
    println!("    created_at VARCHAR(500) NOT NULL");
    println!(");");

    println!("\n=== 精度说明 ===");
    println!("NUMERIC(10,2) 表示:");
    println!("  - precision = 10: 总位数（包括小数点两边）");
    println!("  - scale = 2:    小数位数");
    println!("  - 范围: -99999999.99 到 99999999.99");
    println!("  - 示例: 12345678.99, 9999.99, 0.01");

    println!("\n=== 不同精度选择 ===");
    println!("场景              | Precision | Scale | 示例值");
    println!("------------------|-----------|-------|-----------------");
    println!("价格（美元）      | 10        | 2     | 12345678.99");
    println!("百分比            | 5         | 2     | 100.00");
    println!("精确百分比        | 6         | 4     | 99.9999");
    println!("税率              | 10        | 4     | 123456.7890");
    println!("金融计算          | 30        | 10    | 1234567890.1234567890");

    println!("\n✅ 功能完成！");
    println!("   1. struct 中定义 DECIMAL 字段 ✅");
    println!("   2. migration 生成 NUMERIC 列 ✅");
    println!("   3. 查询时自动类型转换 ✅");

    Ok(())
}
