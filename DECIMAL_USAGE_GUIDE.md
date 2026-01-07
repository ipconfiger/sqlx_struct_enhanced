# DECIMAL 类型使用指南

## 概述

本指南展示如何在 `sqlx_struct_enhanced` 中使用 DECIMAL/NUMERIC 类型，包括：

1. 在 struct 中定义 Decimal 字段
2. 在 migration 时生成正确的 NUMERIC 列
3. 在查询时正确处理 DECIMAL 类型

---

## 方法 1: 使用 String 类型（推荐）

最简单的方法是使用 `String` 类型存储 DECIMAL 值，并使用 `#[crud(cast_as = "TEXT")]` 属性。

### 依赖

```toml
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
sqlx-struct-enhanced = "0.1.0"
```

### 定义 Struct

```rust
use sqlx::FromRow;
use sqlx_struct_enhanced::EnhancedCrud;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // DECIMAL 类型字段 - 使用 String 存储
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,

    #[crud(cast_as = "TEXT")]
    pub discount: Option<String>,

    pub created_at: String,
}

// Migration 会生成: price NUMERIC(10,2), discount NUMERIC(5,2)
```

### 生成 Migration

```rust
// 运行 cargo test 或调用 migration 代码
// 会自动生成:
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    price NUMERIC(10,2),      -- ✅ 自动使用默认精度
    discount NUMERIC(5,2),    -- ✅ 自动使用默认精度
    created_at VARCHAR(500) NOT NULL
);
```

### 使用示例

```rust
// 插入数据
let product = Product {
    id: Uuid::new_v4(),
    name: "Laptop".to_string(),
    price: Some("999.99".to_string()),  // 作为字符串存储
    discount: Some("15.5".to_string()),
    created_at: chrono::Utc::now().to_string(),
};

product.insert_bind().execute(&pool).await.unwrap();

// 查询数据（自动转换 NUMERIC→TEXT）
let product = Product::by_pk()
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();

assert_eq!(product.price, Some("999.99".to_string()));
```

---

## 方法 2: 使用 rust_decimal crate

如果你需要真正的 Decimal 类型支持（可以进行数学运算），使用 `rust_decimal` crate。

### 依赖

```toml
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio"] }
sqlx-struct-enhanced = "0.1.0"
rust_decimal = "1.32"  # 或更新版本
```

### 定义 Struct（使用默认精度）

```rust
use rust_decimal::Decimal;
use sqlx::FromRow;
use sqlx_struct_enhanced::EnhancedCrud;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // Decimal 类型 - 使用默认精度 NUMERIC(18,6)
    pub price: Option<Decimal>,

    pub discount: Option<Decimal>,
}

// Migration 会自动生成: price NUMERIC(18,6), discount NUMERIC(18,6)
```

### 定义 Struct（使用自定义精度）

```rust
#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // 自定义精度: NUMERIC(10, 2) - 最多10位数字，其中2位小数
    #[crud(decimal(precision = 10, scale = 2))]
    pub price: Option<Decimal>,

    // 自定义精度: NUMERIC(5, 2) - 价格/折扣等
    #[crud(decimal(precision = 5, scale = 2))]
    pub discount: Option<Decimal>,

    // 高精度: NUMERIC(30, 10) - 金融计算等
    #[crud(decimal(precision = 30, scale = 10))]
    pub tax_rate: Option<Decimal>,
}

// Migration 会生成:
// price NUMERIC(10,2)
// discount NUMERIC(5,2)
// tax_rate NUMERIC(30,10)
```

### 使用示例

```rust
// 插入数据
let product = Product {
    id: Uuid::new_v4(),
    name: "Laptop".to_string(),
    price: Some(Decimal::from_str("999.99").unwrap()),
    discount: Some(Decimal::from_str("15.5").unwrap()),
};

product.insert_bind().execute(&pool).await.unwrap();

// 查询数据
let product = Product::by_pk()
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();

// 可以进行数学运算
let discounted_price = product.price.unwrap()
    * (Decimal::from(100) - product.discount.unwrap()) / Decimal::from(100);

println!("Discounted price: {}", discounted_price);  // 844.99
```

---

## 方法 3: 使用 BigDecimal

对于需要更高精度的场景，使用 `bigdecimal` crate。

### 依赖

```toml
[dependencies]
bigdecimal = "0.3"
```

### 定义 Struct

```rust
use bigdecimal::BigDecimal;
use sqlx::FromRow;
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "financial_records"]
pub struct FinancialRecord {
    pub id: String,

    // 高精度: NUMERIC(30,10) - 默认
    pub amount: Option<BigDecimal>,

    // 自定义精度: NUMERIC(40,20)
    #[crud(decimal(precision = 40, scale = 20))]
    pub high_precision_value: Option<BigDecimal>,
}
```

---

## 完整示例：电商产品表

```rust
use sqlx::FromRow;
use sqlx_struct_enhanced::{EnhancedCrud, TableDef};
use uuid::Uuid;

/// 产品表 - 使用 String 存储 DECIMAL（推荐）
#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,

    // 价格: 最多10位数字，2位小数 (例如: 12345678.99)
    #[crud(decimal(precision = 10, scale = 2))]
    pub price: Option<String>,

    // 折扣率: 最多5位数字，2位小数 (例如: 99.99)
    #[crud(decimal(precision = 5, scale = 2))]
    pub discount_percent: Option<String>,

    // 税率: 最多6位数字，4位小数 (例如: 12.3456)
    #[crud(decimal(precision = 6, scale = 4))]
    pub tax_rate: Option<String>,

    pub created_at: String,
}

// Migration 生成:
// CREATE TABLE products (
//     id UUID PRIMARY KEY,
//     name VARCHAR(500) NOT NULL,
//     description TEXT,
//     price NUMERIC(10,2),
//     discount_percent NUMERIC(5,2),
//     tax_rate NUMERIC(6,4),
//     created_at VARCHAR(500) NOT NULL
// );

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = sqlx::PgPool::connect("postgres://postgres:@127.0.0.1/test").await?;

    // 插入产品
    let product = Product {
        id: Uuid::new_v4(),
        name: "Laptop".to_string(),
        description: Some("High-end laptop".to_string()),
        price: Some("1299.99".to_string()),
        discount_percent: Some("15.00".to_string()),
        tax_rate: Some("8.2500".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    product.insert_bind().execute(&pool).await?;

    // 查询产品
    let product = Product::by_pk()
        .bind(&product.id)
        .fetch_one(&pool)
        .await?;

    println!("Product: {} - ${}", product.name, product.price.unwrap());

    Ok(())
}
```

---

## 精度选择指南

### 精度和比例的选择

| 场景 | 精度 | 比例 | 示例 | 说明 |
|------|------|------|------|------|
| 价格（美元） | 10 | 2 | 12345678.99 | 最多 8 位整数，2 位小数 |
| 百分比 | 5 | 2 | 100.00 | 最多 3 位整数，2 位小数 |
| 百分比（精确） | 6 | 4 | 100.0000 | 最多 2 位整数，4 位小数 |
| 税率 | 10 | 4 | 123456.7890 | 高精度税率 |
| 金融计算 | 30 | 10 | 大额金融交易 | 最高精度 |
| 默认 | 18 | 6 | - | 一般用途 |

### 计算公式

```rust
// precision: 总位数
// scale: 小数位数
// 整数位数 = precision - scale

#[crud(decimal(precision = 10, scale = 2))]
pub field: Option<String>;
// 表示: 最多8位整数 + 2位小数
// 范围: -99999999.99 到 99999999.99
```

---

## NUMERIC vs DECIMAL

在 PostgreSQL 中，`NUMERIC` 和 `DECIMAL` 是等价的。本库统一使用 `NUMERIC`。

```sql
-- 两者完全相同
CREATE TABLE (
    amount NUMERIC(10,2)   -- ✅ 推荐
    -- amount DECIMAL(10,2) -- 同样效果
);
```

---

## 常见问题

### Q1: 为什么推荐使用 String 而不是 Decimal？

**答**：
- ✅ 更简单：无需额外的依赖
- ✅ 兼容性：所有数据库都支持 TEXT→String 转换
- ✅ 灵活性：可以存储任意精度的数字
- ⚠️ 缺点：需要手动进行数学运算

如果需要进行复杂的数学运算，建议使用 `rust_decimal`。

### Q2: Migration 会自动生成吗？

**答**：是的！当你在 struct 中定义了字段：

```rust
#[crud(decimal(precision = 10, scale = 2))]
pub price: Option<String>,
```

Migration 系统会自动生成：
```sql
CREATE TABLE ... (
    price NUMERIC(10,2)
);
```

### Q3: 如何处理 NULL 值？

**答**：使用 `Option<T>`：

```rust
#[crud(decimal(precision = 10, scale = 2))]
pub price: Option<String>,  // ✅ 可以为 NULL
```

### Q4: 可以改变已有字段的精度吗？

**答**：可以！修改 struct 定义并重新运行 migration：

```rust
// 原来: NUMERIC(10,2)
#[crud(decimal(precision = 10, scale = 2))]
pub price: Option<String>,

// 改为: NUMERIC(20,4)
#[crud(decimal(precision = 20, scale = 4))]
pub price: Option<String>,

// Migration 会生成:
ALTER TABLE products ALTER COLUMN price TYPE NUMERIC(20,4);
```

⚠️ **注意**: 改变精度可能导致数据截断，建议先备份数据！

---

## 总结

### 快速开始

```rust
#[derive(EnhancedCrud)]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // 方法1: String + cast_as（推荐）
    #[crud(cast_as = "TEXT")]
    #[crud(decimal(precision = 10, scale = 2))]
    pub price: Option<String>,

    // 方法2: rust_decimal
    // #[crud(decimal(precision = 10, scale = 2))]
    // pub price: Option<Decimal>,
}

// ✅ Migration 自动生成正确的 NUMERIC 列
// ✅ 查询时自动转换类型
// ✅ 无需手动编写 SQL
```

---

**版本**: 1.0
**最后更新**: 2025-01-07
