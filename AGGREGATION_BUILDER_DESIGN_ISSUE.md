# Aggregation Query Builder Design Issue - 需要改进

## 问题描述

`AggregationQueryBuilder` 当前设计存在严重缺陷：需要先调用 `build()` 生成 SQL 字符串，再手动使用 `sqlx::query_as()` 执行，导致代码量翻倍且没有简化任何复杂度。

## 当前设计的缺陷

### 1. 代码量对比

#### 手写 SQL（原方案）
```rust
let (count,) = sqlx::query_as::<_, (i64,)>(
    "SELECT COUNT(*) FROM users WHERE operation_center_id = $1"
)
.bind(id)
.fetch_one(&pool)
.await?;
```
**4 行代码，清晰直接**

#### 使用 Aggregation Query Builder（当前方案）
```rust
let id_str = id.to_string();
let sql = User::agg_query()
    .where_("operation_center_id = {}", &[&id_str])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)
    .bind(id)
    .bind(false)
    .fetch_one(&pool)
    .await?;
```
**8 行代码 + 多余的类型转换**

**结论：代码量增加 1 倍，没有简化任何东西！**

### 2. 核心问题：类型指定并没有减少

```rust
// 手写 SQL - 需要指定类型
let (count,) = sqlx::query_as::<_, (i64,)>(  // ← 需要指定
    "SELECT COUNT(*) FROM users WHERE id = $1"
)
.bind(id)
.fetch_one(&pool)
.await?;

// 使用 Builder - 还是要指定类型！
let sql = User::agg_query()
    .where_("id = {}", &[&id])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)  // ← 还是要指定
    .bind(id)
    .fetch_one(&pool)
    .await?;
```

**两种方式都要手动指定返回类型 `(i64,)`，Builder 没有提供任何帮助！**

### 3. build() 是多余的步骤

```rust
// 当前设计流程：
agg_query() → where_() → count() → build() → query_as() → bind() → fetch_one()
              ↑                                         ↑
           开始构建                                  生成 SQL 字符串
                                                    ↓
                                              又要重新组装查询
```

**build() 只是为了生成一个字符串，然后又用这个字符串重新构建查询，完全多余！**

### 4. 对比 CRUD 操作

#### CRUD 操作 - 直接执行 ✅
```rust
// 插入
user.insert_bind().execute(&pool).await?;

// 查询
let user = User::by_pk().bind(&id).fetch_one(&pool).await?;

// 更新
user.update_bind().execute(&pool).await?;

// 删除
user.delete_bind().execute(&pool).await?;
```

#### 聚合查询 - 无法直接执行 ❌
```rust
// ❌ 当前不支持直接执行
let (count,) = User::agg_query()
    .where_("id = {}", &[&id])
    .count()
    .fetch_one(&pool)  // 这个方法不存在！
    .await?;

// ✅ 必须先 build()
let sql = User::agg_query()
    .where_("id = {}", &[&id])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)
    .bind(id)
    .fetch_one(&pool)
    .await?;
```

## 问题分析

### 错误的设计理念

原始设计理念："聚合查询的返回类型不固定（可能是 `(i64,)`、`(Option<f64>, i64)` 等），所以让用户手动指定类型更灵活。"

**这个理念是错误的**，因为：
1. 手写 SQL 也要手动指定类型
2. Builder 没有减少任何类型指定的复杂度
3. 反而增加了 `build()` 这个多余步骤

### 真正的问题所在

查看源码发现：

```rust
// src/aggregate/query_builder.rs:509
impl AggQueryBuilder {
    /// ❌ 只有 build() 方法
    pub fn build(&self) -> &'static str {
        // ... 生成 SQL 并缓存
    }

    // ❌ 缺少 fetch_one(), fetch_all() 等执行方法
}
```

**对比 CRUD 操作：**

```rust
// CRUD 操作有完整的执行方法
impl ExecuteQuery {
    pub async fn execute(&self, pool: &Pool) -> Result<u64> { ... }
}

impl SelectQuery<T> {
    pub async fn fetch_one(&self, pool: &Pool) -> Result<T> { ... }
    pub async fn fetch_all(&self, pool: &Pool) -> Result<Vec<T>> { ... }
}
```

## 改进方案

### 方案 1：添加泛型 fetch 方法（推荐）

为 `AggQueryBuilder` 添加 `fetch_one()` 和 `fetch_all()` 方法：

```rust
impl<'a> AggQueryBuilder<'a, Postgres> {
    /// 执行查询并返回单行结果
    pub async fn fetch_one<T>(
        self,
        pool: &Pool<Postgres>
    ) -> Result<T, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    {
        let sql = self.build();

        let mut query = sqlx::query_as::<_, T>(sql);

        // 绑定 WHERE 参数
        for param in &self.where_params {
            query = query.bind(param);
        }

        // 绑定 HAVING 参数
        for param in &self.having_params {
            query = query.bind(param);
        }

        query.fetch_one(pool).await
    }

    /// 执行查询并返回多行结果
    pub async fn fetch_all<T>(
        self,
        pool: &Pool<Postgres>
    ) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    {
        let sql = self.build();

        let mut query = sqlx::query_as::<_, T>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        for param in &self.having_params {
            query = query.bind(param);
        }

        query.fetch_all(pool).await
    }

    /// 执行查询并返回可选结果
    pub async fn fetch_optional<T>(
        self,
        pool: &Pool<Postgres>
    ) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    {
        let sql = self.build();

        let mut query = sqlx::query_as::<_, T>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        for param in &self.having_params {
            query = query.bind(param);
        }

        query.fetch_optional(pool).await
    }
}
```

**使用示例：**

```rust
// ✅ 简洁多了！
let (count,): (i64,) = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_one(&pool)
    .await?;

// ✅ AVG + COUNT
let (avg, count): (Option<f64>, i64) = OrderRating::agg_query()
    .where_("engineer_id = {}", &[&id])
    .avg("score")
    .count()
    .fetch_one(&pool)
    .await?;

// ✅ 多行结果（GROUP BY）
let results: Vec<(String, i64)> = Order::agg_query()
    .group_by("status")
    .count()
    .fetch_all(&pool)
    .await?;

// ✅ 可选结果
let maybe_count: Option<(i64,)> = User::agg_query()
    .where_("id = {}", &[&id])
    .count()
    .fetch_optional(&pool)
    .await?;
```

### 方案 2：为常见聚合提供特化方法

为最常用的聚合（如 COUNT）提供专门的方法：

```rust
impl<'a> AggQueryBuilder<'a, Postgres> {
    /// 专门用于 COUNT 查询，自动解包元组
    pub async fn fetch_count(
        self,
        pool: &Pool<Postgres>
    ) -> Result<i64, sqlx::Error> {
        let sql = self.build();

        let mut query = sqlx::query_as::<_, (i64,)>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        let (count,) = query.fetch_one(pool).await?;
        Ok(count)
    }

    /// 用于 AVG 查询
    pub async fn fetch_avg(
        self,
        pool: &Pool<Postgres>
    ) -> Result<Option<f64>, sqlx::Error> {
        let sql = self.build();

        let mut query = sqlx::query_as::<_, (Option<f64>,)>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        let (avg,) = query.fetch_one(pool).await?;
        Ok(avg)
    }
}
```

**使用示例：**

```rust
// ✅ 最简洁！不需要指定类型
let count = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_count(&pool)
    .await?;

// ✅ AVG 也很简洁
let avg = OrderRating::agg_query()
    .where_("engineer_id = {}", &[&id])
    .avg("score")
    .fetch_avg(&pool)
    .await?;
```

### 方案 3：混合方案（最佳）

同时提供泛型方法和特化方法：

```rust
impl<'a> AggQueryBuilder<'a, Postgres> {
    // 泛型方法（灵活）
    pub async fn fetch_one<T>(...) -> Result<T, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    { ... }

    pub async fn fetch_all<T>(...) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    { ... }

    // 特化方法（便捷）
    pub async fn fetch_count(self, pool: &Pool<Postgres>) -> Result<i64, sqlx::Error> { ... }

    pub async fn fetch_avg(self, pool: &Pool<Postgres>) -> Result<Option<f64>, sqlx::Error> { ... }

    pub async fn fetch_sum(self, pool: &Pool<Postgres>) -> Result<Option<f64>, sqlx::Error> { ... }
}
```

## 需要修改的文件

### 1. `/Users/alex/Projects/workspace/sqlx_struct_enhanced/src/aggregate/query_builder.rs`

**位置**: `impl<'a> AggQueryBuilder<'a, Postgres>` 块内（约第 509 行之后）

**需要添加的方法**:
- `fetch_one<T>()` - 泛型单行查询
- `fetch_all<T>()` - 泛型多行查询
- `fetch_optional<T>()` - 泛型可选结果
- `fetch_count()` - 特化 COUNT 查询
- `fetch_avg()` - 特化 AVG 查询
- `fetch_sum()` - 特化 SUM 查询

### 2. `/Users/alex/Projects/workspace/sqlx_struct_enhanced/src/aggregate/query_builder.rs` (MySQL 版本)

**位置**: `impl<'a> AggQueryBuilder<'a, MySql>` 块

添加相同的 MySQL 特定方法。

### 3. `/Users/alex/Projects/workspace/sqlx_struct_enhanced/src/aggregate/query_builder.rs` (SQLite 版本)

**位置**: `impl<'a> AggQueryBuilder<'a, Sqlite>` 块

添加相同的 SQLite 特定方法。

### 4. `/Users/alex/Projects/workspace/sqlx_struct_enhanced/tests/aggregate_tests.rs`

添加新的测试用例：

```rust
#[sqlx::test]
async fn test_fetch_count(pool: PgPool) -> Result<(), sqlx::Error> {
    // 创建测试数据...

    // 使用新的 fetch_count 方法
    let count = User::agg_query()
        .where_("role = {}", &[&"admin"])
        .count()
        .fetch_count(&pool)
        .await?;

    assert!(count >= 0);
    Ok(())
}

#[sqlx::test]
async fn test_fetch_one_with_tuple(pool: PgPool) -> Result<(), sqlx::Error> {
    let (avg, count): (Option<f64>, i64) = OrderRating::agg_query()
        .where_("engineer_id = {}", &[&test_id])
        .avg("score")
        .count()
        .fetch_one(&pool)
        .await?;

    assert!(count >= 0);
    Ok(())
}

#[sqlx::test]
async fn test_fetch_all_group_by(pool: PgPool) -> Result<(), sqlx::Error> {
    let results: Vec<(String, i64)> = Order::agg_query()
        .group_by("status")
        .count()
        .fetch_all(&pool)
        .await?;

    assert!(!results.is_empty());
    Ok(())
}
```

### 5. 更新迁移的代码（可选）

一旦新方法实现，可以简化已迁移的代码：

#### notification_service.rs
```rust
// 之前（8 行）
let user_id_str = user_id.to_string();
let sql = Notification::agg_query()
    .where_("user_id = {} AND read = {}", &[&user_id_str, "false"])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)
    .bind(user_id)
    .bind(false)
    .fetch_one(&self.pool)
    .await?;

// 之后（3 行）- 使用泛型方法
let (count,): (i64,) = Notification::agg_query()
    .where_("user_id = {} AND read = {}", &[&user_id, &false])
    .count()
    .fetch_one(&self.pool)
    .await?;

// 或者（2 行）- 使用特化方法
let count = Notification::agg_query()
    .where_("user_id = {} AND read = {}", &[&user_id, &false])
    .count()
    .fetch_count(&self.pool)
    .await?;
```

## 实现优先级

### 高优先级（必须实现）

1. **`fetch_one<T>()`** - 支持 `(T,)` 元组返回类型
2. **`fetch_all<T>()`** - 支持 GROUP BY 查询
3. **`fetch_count()`** - 最常用的聚合，应提供便捷方法

### 中优先级（建议实现）

4. **`fetch_optional<T>()`** - 支持可选结果
5. **`fetch_avg()`** - 常用聚合
6. **`fetch_sum()`** - 常用聚合

### 低优先级（可选）

7. **`fetch_first<T>()`** - 只返回第一行（忽略其他）
8. **`fetch_exists()`** - 返回 bool（是否存在记录）

## 实现注意事项

### 1. 参数绑定顺序

**当前问题**：
```rust
// Builder 中有两套参数
pub struct AggQueryBuilder {
    where_params: Vec<String>,  // WHERE 参数
    having_params: Vec<String>, // HAVING 参数
}
```

**解决方案**：
按照 SQL 语法顺序绑定：
1. WHERE 参数先绑定（$1, $2, ...）
2. HAVING 参数后绑定（继续 $N+1, $N+2, ...）

### 2. 类型转换问题

**当前问题**：
```rust
// 用户需要手动转换类型
let id_str = id.to_string();  // Uuid → String
.where_("id = {}", &[&id_str])
```

**解决方案**：
`where_()` 应该接受 `&[&dyn std::fmt::Display]` 而不是 `&[&str]`：

```rust
pub fn where_(mut self, clause: &str, params: &[&dyn Display]) -> Self {
    self.where_clause = Some(clause.to_string());
    self.where_params = params.iter().map(|p| p.to_string()).collect();
    self
}
```

**使用时**：
```rust
// ✅ 不需要手动转换
User::agg_query()
    .where_("id = {}", &[&id])  // Uuid 直接使用
    .count()
    .fetch_one(&pool)
    .await?;
```

### 3. 返回类型推导

**当前问题**：
```rust
// 还是要手动指定类型
let (count,): (i64,) = User::agg_query()...fetch_one(&pool).await?;
```

**可能的改进**（高级）：
使用 Rust 的类型别名或类型推导来简化：

```rust
type Count = (i64,);
type AvgCount = (Option<f64>, i64);

// ✅ 稍微简洁一点
let count: Count = User::agg_query()...fetch_one(&pool).await?;
let stats: AvgCount = OrderRating::agg_query()...fetch_one(&pool).await?;
```

但这仍然不够理想，所以特化方法（`fetch_count()`）更好。

## 预期效果

### 代码量对比

#### 当前（8 行）
```rust
let id_str = id.to_string();
let sql = User::agg_query()
    .where_("operation_center_id = {}", &[&id_str])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)
    .bind(id)
    .fetch_one(&pool)
    .await?;
```

#### 改进后（2 行）
```rust
let count = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_count(&pool)
    .await?;
```

**减少 75% 的代码量！**

### 与手写 SQL 对比

#### 手写 SQL（4 行）
```rust
let (count,) = sqlx::query_as::<_, (i64,)>(
    "SELECT COUNT(*) FROM users WHERE operation_center_id = $1"
)
.bind(id)
.fetch_one(&pool)
.await?;
```

#### 改进后的 Builder（2 行）
```rust
let count = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_count(&pool)
    .await?;
```

**Builder 终于比手写 SQL 更简洁了！**

## 总结

### 当前设计的核心问题

1. ❌ `build()` 是多余的步骤
2. ❌ 没有减少任何类型指定的复杂度
3. ❌ 代码量比手写 SQL 还多
4. ❌ 不符合 CRUD 操作的一致性

### 正确的设计应该

1. ✅ 支持直接执行（`fetch_one()`, `fetch_all()`）
2. ✅ 提供特化方法（`fetch_count()`, `fetch_avg()`）
3. ✅ 自动绑定参数
4. ✅ 保持与 CRUD 操作的一致性

### 实现建议

**优先实现**：
1. `fetch_one<T>()` - 泛型方法
2. `fetch_all<T>()` - 泛型方法
3. `fetch_count()` - 特化方法

**这样就能让 Aggregation Query Builder 真正有用，而不是现在的半吊子状态。**

---

**文档创建时间**: 2026-01-08
**问题发现者**: sdb_project 用户反馈
**优先级**: 高
**预估工作量**: 2-3 小时
