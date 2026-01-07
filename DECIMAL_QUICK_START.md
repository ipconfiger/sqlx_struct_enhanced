# DECIMAL 类型支持 - 快速指南

## 概述

现在可以在 struct 中定义 DECIMAL 类型字段，migration 会自动生成正确的 NUMERIC 列定义！

---

## 使用方法

### 方法 1: String 类型 + 精度指定（推荐）

```rust
use sqlx::FromRow;
use sqlx_struct_enhanced::EnhancedCrud;
use uuid::Uuid;

#[derive(Debug, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // DECIMAL(10,2) - 价格字段
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,

    // DECIMAL(5,2) - 折扣率
    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub discount: Option<String>,
}
```

**Migration 自动生成**:
```sql
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    price NUMERIC(10,2),
    discount NUMERIC(5,2)
);
```

### 方法 2: rust_decimal 类型

```toml
# Cargo.toml
[dependencies]
rust_decimal = "1.32"
```

```rust
use rust_decimal::Decimal;

#[derive(Debug, FromRow, EnhancedCrud)]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // 使用 Decimal 类型
    #[crud(decimal(precision = 10, scale = 2))]
    pub price: Option<Decimal>,
}
```

---

## 精度说明

`#[crud(decimal(precision = X, scale = Y))]` 中：

- **precision**: 总位数（包括整数和小数部分）
- **scale**: 小数位数
- **整数位数** = precision - scale

### 常见精度选择

| 场景 | Precision | Scale | 示例 | 说明 |
|------|-----------|-------|------|------|
| 价格 | 10 | 2 | 12345678.99 | 最多8位整数 + 2位小数 |
| 百分比 | 5 | 2 | 100.00 | 最多3位整数 + 2位小数 |
| 税率 | 10 | 4 | 123456.7890 | 高精度税率 |
| 金融 | 30 | 10 | 大额金融交易 | 最高精度 |

---

## 完整示例

```rust
#[derive(EnhancedCrud)]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,  // NUMERIC(10,2) → String

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub discount: Option<String>,  // NUMERIC(5,2) → String
}

// 使用
let product = Product {
    id: Uuid::new_v4(),
    name: "Laptop".to_string(),
    price: Some("1299.99".to_string()),
    discount: Some("15.00".to_string()),
};

// 插入
product.insert_bind().execute(&pool).await.unwrap();

// 查询（自动转换 NUMERIC→TEXT）
let product = Product::by_pk()
    .bind(&id)
    .fetch_one(&pool)
    .await
    .unwrap();
```

---

## 自动类型映射

不需要指定精度时，使用默认值：

| Rust 类型 | 默认 SQL 类型 |
|----------|-------------|
| `Decimal` | `NUMERIC(18,6)` |
| `BigDecimal` | `NUMERIC(30,10)` |
| `BigInt` | `NUMERIC` |
| `String` + `#[crud(decimal(...))]` | `NUMERIC(precision, scale)` |

---

## Migration 生成

当你在 struct 中定义了字段：

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

或者对于 ALTER：

```sql
ALTER TABLE ... ALTER COLUMN price TYPE NUMERIC(10,2);
```

---

## 总结

✅ **3 步完成**：

1. 定义 struct 字段时添加 `#[crud(decimal(precision = X, scale = Y))]`
2. 添加 `#[crud(cast_as = "TEXT")]`（如果使用 String 类型）
3. 运行 migration，自动生成 NUMERIC 列

就这么简单！
