# 代理对象可行性研究报告

## 一、需求背景

当前 `where_query`, `by_pk`, `make_query` 等查询函数直接返回 SQLx 的 `QueryAs` 或 `Query` 类型。在实际使用中，存在以下痛点：

### 1.1 当前问题

**DECIMAL/NUMERIC 类型处理**
- PostgreSQL NUMERIC 列在 Rust 中需要映射为 `String` 类型
- 用户需要手动编写类型转换逻辑
- 已有 `#[crud(cast_as = "TEXT")]` 方案，但仅解决 SELECT 时的类型转换

**DateTime 类型处理**
- SQLx 支持 `chrono::DateTime`, `time::PrimitiveDateTime` 等
- 不同时间库之间需要手动转换
- 时区处理复杂

**JSON 类型处理**
- PostgreSQL JSONB 需要特殊处理
- 与 `serde_json::Value` 之间的转换

### 1.2 期望目标

通过代理对象实现：
1. **自动类型转换**：在 bind 时自动处理复杂类型转换
2. **统一接口**：简化用户代码，隐藏类型转换细节
3. **类型安全**：保持编译期类型检查
4. **向后兼容**：不破坏现有 API

## 二、技术可行性分析

### 2.1 当前 SQLx 类型分析

```rust
// 当前返回类型
pub fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments>

// SQLx 的核心类型
pub struct QueryAs<'q, DB, O, A> { ... }
pub struct Query<'q, DB, O, A> { ... }

// 主要方法
impl<'q, DB, O, A> QueryAs<'q, DB, O, A>
where
    DB: Database,
{
    // 绑定参数
    pub fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self { ... }

    // 执行查询
    pub fn fetch_one<'e, E>(self, executor: E) -> Result<O, Error>
    where
        E: Executor<'e, DB>;

    pub fn fetch_optional<'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        E: Executor<'e, DB>;

    pub fn fetch_all<'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        E: Executor<'e, DB>;
}

impl<'q, DB, A> Query<'q, DB, A>
where
    DB: Database,
{
    pub fn bind<T: 'q + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self { ... }

    pub fn execute<'e, E>(self, executor: E) -> Result<DB::QueryResult, Error>
    where
        E: Executor<'e, DB>;
}
```

### 2.2 技术挑战

#### 挑战 1: 复杂的类型参数

SQLx 的类型有多个生命周期和类型参数：
- `'q` - SQL 字符串的生命周期
- `DB` - 数据库类型 (Postgres/MySql/Sqlite)
- `O` - 输出类型 (QueryAs) 或输出参数类型 (Query)
- `A` - Arguments 类型

代理对象需要保留这些类型参数，同时提供友好的 API。

#### 挑战 2: 生命周期管理

`bind` 方法返回 `Self`，这意味着代理对象也必须支持链式调用：
```rust
query.bind(value1).bind(value2).fetch_one(&pool)
```

这要求代理对象的生命周期设计必须与 SQLx 的类型系统兼容。

#### 挑战 3: 类型转换的时机

类型转换可以在以下时机进行：
1. **编译期**：通过 derive macro 生成转换代码
2. **运行期**：在 bind 方法内部进行转换
3. **Decode 层**：实现 SQLx 的 `Decode` trait

每种方案都有不同的复杂度和性能影响。

### 2.3 可行性结论

✅ **技术可行**，但需要精心设计：

1. **类型系统兼容**：可以通过泛型和 trait 对象实现
2. **零性能开销**：使用内联和编译期优化
3. **类型安全**：保持编译期检查
4. **向后兼容**：可以提供渐进式迁移路径

## 三、设计方案

### 方案 A: 代理模式 (推荐)

#### 3.1.1 架构设计

```rust
// 1. 定义代理 trait
pub trait QueryProxy<'q, DB, O>: Sized
where
    DB: Database,
{
    // 增强的 bind 方法，支持类型转换
    fn bind_proxy<T>(self, value: T) -> Self
    where
        T: BindProxy<DB>;

    // 兼容 SQLx 的 bind 方法
    fn bind<T: Encode<'q, DB> + Type<DB>>(self, value: T) -> Self;

    // 执行方法
    fn fetch_one<'e, E>(self, executor: E) -> Result<O, Error>
    where
        E: Executor<'e, DB>;

    fn fetch_all<'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        E: Executor<'e, DB>;

    fn fetch_optional<'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        E: Executor<'e, DB>;
}

// 2. 定义类型转换 trait
pub trait BindProxy<DB: Database> {
    fn bind_proxy<'q>(self) -> BoundValue<'q, DB>;
}

// 3. 代理值包装
pub enum BoundValue<'q, DB: Database> {
    Raw(Box<dyn Encode<'q, DB> + Type<DB> + Send + Sync>),
    Converted(Box<dyn Encode<'q, DB> + Type<DB> + Send + Sync>),
}

// 4. 代理实现
pub struct EnhancedQueryAs<'q, DB, O>
where
    DB: Database,
{
    inner: QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>,
    phantom: PhantomData<&'q ()>,
}

impl<'q, DB, O> EnhancedQueryAs<'q, DB, O>
where
    DB: Database,
{
    pub fn from(inner: QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }
}
```

#### 3.1.2 具体实现

```rust
// 为 DECIMAL 类型实现转换
impl BindProxy<Postgres> for rust_decimal::Decimal {
    fn bind_proxy<'q>(self) -> BoundValue<'q, Postgres> {
        // rust_decimal -> String for PostgreSQL NUMERIC
        let s = self.to_string();
        BoundValue::Raw(Box::new(s))
    }
}

impl BindProxy<Postgres> for String {
    fn bind_proxy<'q>(self) -> BoundValue<'q, Postgres> {
        // 直接使用
        BoundValue::Raw(Box::new(self))
    }
}

// 为 DateTime 类型实现转换
impl BindProxy<Postgres> for chrono::DateTime<chrono::Utc> {
    fn bind_proxy<'q>(self) -> BoundValue<'q, Postgres> {
        // chrono::DateTime -> SQLx timestamp
        BoundValue::Raw(Box::new(self))
    }
}

impl BindProxy<Postgres> for time::PrimitiveDateTime {
    fn bind_proxy<'q>(self) -> BoundValue<'q, Postgres> {
        // time::PrimitiveDateTime -> chrono::DateTime (需要转换)
        let converted = chrono::DateTime::<chrono::Utc>::from(
            self.assume_utc()
        );
        BoundValue::Raw(Box::new(converted))
    }
}

// 实现代理的 bind 方法
impl<'q, DB, O> QueryProxy<'q, DB, O> for EnhancedQueryAs<'q, DB, O>
where
    DB: Database,
{
    fn bind_proxy<T>(mut self, value: T) -> Self
    where
        T: BindProxy<DB>,
    {
        let bound = value.bind_proxy();
        match bound {
            BoundValue::Raw(v) => {
                // 需要某种方式将 v 传递给 inner.bind()
                // 这里类型擦除带来挑战
                todo!()
            }
            BoundValue::Converted(v) => {
                todo!()
            }
        }
        self
    }

    fn bind<T: Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        self.inner = self.inner.bind(value);
        self
    }

    fn fetch_one<'e, E>(self, executor: E) -> Result<O, Error>
    where
        E: Executor<'e, DB>,
    {
        self.inner.fetch_one(executor)
    }

    fn fetch_all<'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        E: Executor<'e, DB>,
    {
        self.inner.fetch_all(executor)
    }

    fn fetch_optional<'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        E: Executor<'e, DB>,
    {
        self.inner.fetch_optional(executor)
    }
}
```

#### 3.1.3 修改 EnhancedCrud Trait

```rust
pub trait EnhancedCrud {
    // 修改返回类型为代理对象
    fn where_query(statement: &str) -> EnhancedQueryAs<'_, Postgres, Self>
    where
        Self: Sized;

    fn by_pk<'q>() -> EnhancedQueryAs<'q, Postgres, Self>
    where
        Self: Sized;

    fn make_query(sql: &str) -> EnhancedQueryAs<'_, Postgres, Self>
    where
        Self: Sized;
}
```

#### 3.1.4 使用示例

```rust
// 使用示例 1: DECIMAL 类型
#[derive(EnhancedCrud)]
struct Order {
    id: String,
    amount: String,  // PostgreSQL NUMERIC -> String
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Order {
    async fn find_by_amount_range(
        min: rust_decimal::Decimal,
        max: rust_decimal::Decimal
    ) -> Result<Vec<Self>, Error> {
        // 自动转换 rust_decimal -> String
        Order::where_query("amount BETWEEN {} AND {}")
            .bind_proxy(min)
            .bind_proxy(max)
            .fetch_all(&pool)
            .await
    }
}

// 使用示例 2: DateTime 类型
async fn find_orders_by_date(
    date: time::PrimitiveDateTime
) -> Result<Vec<Order>, Error> {
    // 自动转换 time::PrimitiveDateTime -> chrono::DateTime
    Order::where_query("created_at > {}")
        .bind_proxy(date)
        .fetch_all(&pool)
        .await
}
```

#### 3.1.5 优缺点分析

**优点：**
- ✅ 透明地处理类型转换
- ✅ 保持链式调用风格
- ✅ 向后兼容，可以混用 `bind` 和 `bind_proxy`
- ✅ 扩展性好，可以添加更多类型转换

**缺点：**
- ❌ 类型擦除导致 `bind_proxy` 实现复杂（需要 `dyn trait`）
- ❌ 每个 bind 调用都有类型转换开销
- ❌ API 表面面积增加（`bind` vs `bind_proxy`）

### 方案 B: 编译期类型转换

#### 3.2.1 设计思路

使用 derive macro 为每个结构体生成类型转换代码：

```rust
#[derive(EnhancedCrud)]
#[crud(auto_convert)]
struct Order {
    id: String,
    #[crud(convert = "decimal")]
    amount: rust_decimal::Decimal,
    #[crud(convert = "datetime_utc")]
    created_at: chrono::DateTime<chrono::Utc>,
}

// Macro 生成的辅助方法
impl Order {
    fn find_by_amount_auto(
        min: rust_decimal::Decimal,
        max: rust_decimal::Decimal
    ) -> EnhancedQueryAs<'_, Postgres, Self> {
        Order::where_query("amount BETWEEN {} AND {}")
            // Macro 生成: .bind(min.to_string())
            .bind(min.to_string())
            .bind(max.to_string())
    }
}
```

#### 3.2.2 优缺点分析

**优点：**
- ✅ 零运行时开销（编译期转换）
- ✅ 类型安全（编译期检查）
- ✅ 不需要代理对象

**缺点：**
- ❌ 需要修改 macro 逻辑
- ❌ 用户需要显式标注转换类型
- ❌ 不够灵活（每个类型都需要特定标注）

### 方案 C: Decode 层转换

#### 3.3.1 设计思路

为复杂类型实现 SQLx 的 `Decode` trait：

```rust
// 为 rust_decimal 实现 Decode for Postgres
impl<'r> Decode<'r, Postgres> for rust_decimal::Decimal {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // PostgreSQL NUMERIC 是 text format
        let s = <String as Decode<'r, Postgres>>::decode(value)?;
        rust_decimal::Decimal::from_str(&s)
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }
}

impl Encode<'_, Postgres> for rust_decimal::Decimal {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        // 编码为 string
        self.to_string().encode_by_ref(buf)
    }
}

impl Type<Postgres> for rust_decimal::Decimal {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        // PostgreSQL NUMERIC type
        PgTypeInfo::with_name("numeric")
    }
}
```

#### 3.3.2 优缺点分析

**优点：**
- ✅ 对用户完全透明
- ✅ 可以直接在 struct 字段中使用 `rust_decimal::Decimal`
- ✅ 最符合 Rust 惯用法

**缺点：**
- ❌ 需要修改 SQLx crate（不能在第三方 crate 中实现 Decode）
- ❌ 版本兼容性问题（SQLx 升级时需要重新实现）
- ❌ 需要维护所有数据库的实现

### 方案 D: 混合方案 (最优推荐)

#### 3.4.1 设计思路

结合方案 A 和 B：
1. 为常用类型提供 `BindProxy` 实现
2. 为特定字段提供编译期转换
3. 保留 SQLx 原生 `bind` 方法

```rust
pub trait EnhancedCrud {
    // 提供两种查询方式
    fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, ...>
    where
        Self: Sized;

    fn where_query_enhanced(statement: &str) -> EnhancedQueryAs<'_, Postgres, Self>
    where
        Self: Sized;
}

// 使用方式
// 方式 1: 原生 SQLx bind（需要手动转换）
Order::where_query("amount > {}")
    .bind(decimal.to_string())
    .fetch_all(&pool)

// 方式 2: 增强 bind（自动转换）
Order::where_query_enhanced("amount > {}")
    .bind_proxy(decimal)  // 自动转换
    .fetch_all(&pool)
```

#### 3.4.2 优缺点分析

**优点：**
- ✅ 向后兼容（不破坏现有 API）
- ✅ 渐进式采用（可以选择性使用）
- ✅ 最大化灵活性

**缺点：**
- ❌ API 有两种方式，可能造成混淆

## 四、实现难点与解决方案

### 4.1 类型擦除问题

**问题：** SQLx 的 `bind` 方法需要具体类型，但 `BindProxy` trait 是对象安全的，会导致类型擦除。

**解决方案：**

```rust
// 方案 1: 使用 Enum 列出所有支持的类型
pub enum BindValue<'q, DB: Database> {
    String(String),
    i32(i32),
    i64(i64),
    f64(f64),
    Decimal(String),  // DECIMAL 转换为 String
    DateTimeUtc(chrono::DateTime<chrono::Utc>),
    // ... 其他类型
}

impl<'q, DB: Database> BindValue<'q, DB> {
    fn apply_to_query<Q>(self, query: Q) -> Q
    where
        Q: Binder<'q, DB>,
    {
        match self {
            BindValue::String(s) => query.bind(s),
            BindValue::i32(i) => query.bind(i),
            BindValue::Decimal(s) => query.bind(s),
            // ... 其他分支
        }
    }
}

// 方案 2: 使用 macro 自动生成 match 分支
macro_rules! impl_bind_proxy {
    ($($type:ty => $converted:expr),*) => {
        pub trait BindProxy<DB: Database> {
            fn into_bind_value<'q>(self) -> BindValue<'q, DB>;
        }

        $(
            impl BindProxy<Postgres> for $type {
                fn into_bind_value<'q>(self) -> BindValue<'q, Postgres> {
                    BindValue::$converted
                }
            }
        )*
    };
}

impl_bind_proxy!(
    rust_decimal::Decimal => String(self.to_string()),
    time::PrimitiveDateTime => DateTimeUtc(chrono::DateTime::<chrono::Utc>::from(self.assume_utc()))
);
```

### 4.2 生命周期问题

**问题：** `bind` 返回 `Self`，要求代理对象的生命周期正确设计。

**解决方案：**

```rust
// 代理对象持有 SQLx 查询的所有权
pub struct EnhancedQueryAs<'q, DB, O>
where
    DB: Database,
{
    query: QueryAs<'q, DB, O, <DB as HasArguments<'q>>::Arguments>,
}

// bind 方法消耗 self 并返回 Self
impl<'q, DB, O> EnhancedQueryAs<'q, DB, O>
where
    DB: Database,
{
    pub fn bind_proxy<T: IntoBindValue<'q, DB>>(mut self, value: T) -> Self {
        let bind_value = value.into_bind_value();
        // 应用 bind 到 inner query
        self.query = bind_value.apply_to_query(self.query);
        self
    }
}
```

### 4.3 性能问题

**问题：** 类型转换可能带来运行时开销。

**解决方案：**

```rust
// 1. 使用 #[inline] 提示编译器内联
#[inline]
pub fn bind_proxy<T: IntoBindValue<'q, DB>>(mut self, value: T) -> Self {
    // ...
}

// 2. 提供零拷贝转换（如引用）
impl IntoBindValue<'_, Postgres> for &'_ str {
    fn into_bind_value(self) -> BindValue<'_, Postgres> {
        BindValue::String(self.to_string())  // 拷贝
        // 或
        BindValue::StrRef(self)  // 零拷贝，但需要生命周期标注
    }
}

// 3. 编译期优化
// 对于 DECIMAL -> String，Rust 编译器会优化掉临时 String
```

### 4.4 向后兼容性

**问题：** 不破坏现有 API。

**解决方案：**

```rust
// 1. 提供独立的 trait
pub trait EnhancedCrud {
    // 保留原有方法
    fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, ...>;

    // 新增增强方法
    fn where_query_ext(statement: &str) -> EnhancedQueryAs<'_, Postgres, Self>;
}

// 2. 使用 feature flag 控制启用
#[cfg(feature = "proxy")]
pub trait EnhancedCrudExt {
    fn where_query_proxy(statement: &str) -> EnhancedQueryAs<'_, Postgres, Self>;
}

// 3. 提供扩展 trait
pub trait EnhancedCrudProxy: EnhancedCrud {
    fn where_query_proxy(&self, statement: &str) -> EnhancedQueryAs<'_, Postgres, Self>;
}

// blanket implementation
impl<T: EnhancedCrud> EnhancedCrudProxy for T {
    fn where_query_proxy(&self, statement: &str) -> EnhancedQueryAs<'_, Postgres, Self> {
        EnhancedQueryAs::new(self.where_query(statement))
    }
}
```

## 五、推荐实现路径

### 阶段 1: MVP 实现 (1-2 周)

**目标：** 支持 DECIMAL 类型自动转换

```rust
// 1. 定义核心类型
pub struct EnhancedQueryAs<'q, DB, O>(QueryAs<'q, DB, O, ...>);

// 2. 实现 DECIMAL 转换
impl BindProxy<Postgres> for rust_decimal::Decimal {
    fn into_bind_value(self) -> BindValue<'_, Postgres> {
        BindValue::String(self.to_string())
    }
}

// 3. 提供扩展 trait
pub trait EnhancedCrudExt: EnhancedCrud {
    fn where_query_ext(&self, stmt: &str) -> EnhancedQueryAs<'_, Postgres, Self>;
}

// 4. 单元测试
#[test]
fn test_decimal_auto_convert() {
    // ...
}
```

### 阶段 2: 完善类型支持 (2-3 周)

**目标：** 支持 DateTime、JSON 等类型

```rust
// DateTime 支持
impl<Tz: TimeZone> BindProxy<Postgres> for DateTime<Tz> {
    fn into_bind_value(self) -> BindValue<'_, Postgres> {
        BindValue::DateTimeUtc(self.with_timezone(&Utc))
    }
}

// JSON 支持
impl BindProxy<Postgres> for serde_json::Value {
    fn into_bind_value(self) -> BindValue<'_, Postgres> {
        BindValue::Json(self)
    }
}
```

### 阶段 3: 性能优化 (1 周)

**目标：** 零开销抽象

```rust
// 1. 性能测试
#[bench]
fn bench_bind_proxy_overhead(b: &mut Bencher) {
    b.iter(|| {
        query.bind_proxy(decimal_value)
    });
}

// 2. 优化热点路径
// 3. 文档化性能特性
```

### 阶段 4: 文档与示例 (1 周)

**目标：** 完善文档和示例

```markdown
# 代理对象使用指南

## DECIMAL 类型
## DateTime 类型
## 性能考虑
## 迁移指南
```

## 六、最终推荐

### 推荐方案：**方案 D (混合方案) + 渐进式实现**

**理由：**

1. **向后兼容**：不破坏现有 API，降低迁移风险
2. **灵活性**：用户可以选择性使用增强功能
3. **可测试性**：MVP 可以快速验证核心假设
4. **可维护性**：清晰的模块边界，便于后续扩展

**实施建议：**

1. **第一阶段** (MVP):
   - 实现 `EnhancedQueryAs` 代理对象
   - 支持 `rust_decimal::Decimal` 自动转换
   - 提供扩展 trait `EnhancedCrudExt`
   - 完善单元测试

2. **第二阶段** (扩展):
   - 添加 DateTime 支持
   - 添加 JSON 支持
   - 性能基准测试

3. **第三阶段** (优化):
   - 优化类型转换开销
   - 完善文档和示例
   - 收集用户反馈

## 七、风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| 类型系统过于复杂 | 中 | 高 | MVP 验证，简化设计 |
| 性能开销过大 | 低 | 中 | 基准测试，内联优化 |
| API 混乱 | 中 | 中 | 清晰命名，文档说明 |
| SQLx 兼容性问题 | 低 | 低 | 版本锁定，兼容测试 |
| 用户接受度低 | 中 | 高 | 渐进式推出，可选使用 |

## 八、总结

✅ **可行性确认**：技术上完全可行

✅ **推荐实施**：建议采用混合方案，分阶段实施

✅ **优先级**：建议作为 medium priority feature，在核心功能稳定后实施

⚠️ **注意事项**：
- 需要仔细设计类型系统，避免过度复杂
- 性能测试必须跟上，确保零开销
- 文档和示例至关重要，降低学习成本

---

**报告完成时间：** 2026-01-08
**建议下一步：** 创建 MVP 实现的 proof-of-concept