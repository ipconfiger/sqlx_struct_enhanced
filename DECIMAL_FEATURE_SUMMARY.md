# DECIMAL 类型完整实现总结

## ✅ 功能已完成

现在你可以在 struct 中定义 DECIMAL 类型字段，migration 会自动生成正确的 NUMERIC 列！

---

## 实现的功能

### 1. ✅ 支持多种 Decimal 类型

```rust
// 方法1: String 类型（推荐）
#[crud(decimal(precision = 10, scale = 2))]
#[crud(cast_as = "TEXT")]
pub price: Option<String>,  // → NUMERIC(10,2)

// 方法2: rust_decimal
#[crud(decimal(precision = 10, scale = 2))]
pub price: Option<Decimal>,  // → NUMERIC(10,2)

// 方法3: bigdecimal
#[crud(decimal(precision = 30, scale = 10))]
pub amount: Option<BigDecimal>,  // → NUMERIC(30,10)
```

### 2. ✅ Migration 自动生成 NUMERIC 列

定义 struct 时：
```rust
#[crud(decimal(precision = 10, scale = 2))]
pub price: Option<String>,
```

Migration 自动生成：
```sql
CREATE TABLE ... (
    price NUMERIC(10,2)
);
```

### 3. ✅ 查询时自动类型转换

- **String 类型**: 使用 `#[crud(cast_as = "TEXT")]`，自动转换 NUMERIC→TEXT
- **Decimal 类型**: SQLx 自动处理，无需额外配置

---

## 完整示例

```rust
use sqlx::FromRow;
use sqlx_struct_enhanced::EnhancedCrud;
use uuid::Uuid;

#[derive(Debug, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    // DECIMAL(10,2) - 价格
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,

    // DECIMAL(5,2) - 折扣率
    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub discount: Option<String>,
}
```

**Migration 生成**:
```sql
CREATE TABLE products (
    id UUID PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    price NUMERIC(10,2),
    discount NUMERIC(5,2)
);
```

---

## 新增的属性

### `#[crud(decimal(precision = X, scale = Y))]`

指定 NUMERIC 类型的精度：

```rust
#[crud(decimal(precision = 10, scale = 2))]
pub price: Option<String>,
```

- `precision`: 总位数（默认 18）
- `scale`: 小数位数（默认 6）
- 可选：如果不指定，使用默认值

---

## 文件修改

### 1. `sqlx_struct_macros/src/struct_schema_parser.rs`

**新增字段**:
```rust
pub struct StructColumn {
    // ... 其他字段
    pub decimal_precision: Option<(u32, u32)>,  // (precision, scale)
}
```

**新增函数**:
- `map_rust_type_to_sql_with_precision()`: 使用自定义精度生成 SQL 类型

**更新函数**:
- `parse_field_attributes()`: 解析 `#[crud(decimal(...))]` 属性
- `parse_field()`: 传递 decimal_precision

**新增类型映射**:
```rust
"rust_decimal::Decimal" | "Decimal" => "NUMERIC(18,6)"
"bigdecimal::BigDecimal" | "BigDecimal" => "NUMERIC(30,10)"
"num_bigint::BigInt" | "BigInt" => "NUMERIC"
```

---

## 使用方式

### 方式 1: String + 精度（推荐）

```rust
#[derive(EnhancedCrud)]
pub struct Product {
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,
}
```

**优点**:
- ✅ 无需额外依赖
- ✅ 简单易用
- ✅ 所有数据库兼容

**缺点**:
- ⚠️ 需要手动解析/运算

### 方式 2: rust_decimal

```toml
[dependencies]
rust_decimal = "1.32"
```

```rust
use rust_decimal::Decimal;

#[derive(EnhancedCrud)]
pub struct Product {
    #[crud(decimal(precision = 10, scale = 2))]
    pub price: Option<Decimal>,
}
```

**优点**:
- ✅ 支持数学运算
- ✅ 类型安全
- ✅ 高精度计算

---

## 精度选择指南

| 场景 | Precision | Scale | 示例 | 说明 |
|------|-----------|-------|------|------|
| **价格** | 10 | 2 | 99999999.99 | 美元/人民币等货币 |
| **百分比** | 5 | 2 | 100.00 | 折扣率、增长率 |
| **精确百分比** | 6 | 4 | 99.9999 | 金融利率 |
| **税率** | 10 | 4 | 123456.7890 | 高精度税率 |
| **金融计算** | 30 | 10 | 大额交易 | 投资收益等 |

### 计算公式

```
NUMERIC(P, S)
├─ P: precision（总位数）
├─ S: scale（小数位数）
└─ 整数位数 = P - S

例如:
NUMERIC(10, 2)
├─ 总位数: 10
├─ 小数位: 2
├─ 整数位: 8
└─ 范围: -99999999.99 到 99999999.99
```

---

## 默认值

如果不指定精度，使用以下默认值：

| Rust 类型 | 默认 SQL 类型 |
|----------|-------------|
| `Decimal` | `NUMERIC(18,6)` |
| `BigDecimal` | `NUMERIC(30,10)` |
| `String` + `#[crud(decimal)]` | `NUMERIC(18,6)` |

---

## 文档

详细使用指南请查看：

1. **[DECIMAL_QUICK_START.md](DECIMAL_QUICK_START.md)** - 快速开始
2. **[DECIMAL_USAGE_GUIDE.md](DECIMAL_USAGE_GUIDE.md)** - 完整使用指南
3. **[examples/decimal_example.rs](examples/decimal_example.rs)** - 示例代码

---

## 兼容性

✅ **向后兼容**: 不使用 `#[crud(decimal(...))]` 的代码继续正常工作

✅ **可选功能**: 只在需要时添加精度定义

✅ **自动迁移**: 现有的 migration 系统自动支持新类型

---

## 总结

### 现在你可以：

1. ✅ 在 struct 中定义 DECIMAL 字段
2. ✅ 指定精度: `#[crud(decimal(precision = 10, scale = 2))]`
3. ✅ Migration 自动生成 NUMERIC 列
4. ✅ 查询时自动类型转换

### 示例：

```rust
#[derive(EnhancedCrud)]
pub struct Product {
    pub id: Uuid,
    pub name: String,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,
}

// Migration: CREATE TABLE ... (price NUMERIC(10,2))
// 查询: SELECT ... price::TEXT as price FROM ...
```

**就这么简单！** 🎉

---

**实现日期**: 2025-01-07
**状态**: ✅ 完成并可用
