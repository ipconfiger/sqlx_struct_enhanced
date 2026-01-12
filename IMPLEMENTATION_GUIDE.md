# 快速实现指南：添加 fetch 方法到 Aggregation Query Builder

## 一分钟概览

**问题**: 当前 `AggQueryBuilder` 只有 `build()` 方法，无法直接执行查询
**解决**: 添加 `fetch_one()`, `fetch_all()`, `fetch_count()` 等方法
**文件**: 只需修改 1 个文件
**时间**: 约 30 分钟

## 修改文件清单

### 唯一需要修改的文件

```
/Users/alex/Projects/workspace/sqlx_struct_enhanced/src/aggregate/query_builder.rs
```

## 具体修改步骤

### Step 1: 找到正确的位置

打开 `src/aggregate/query_builder.rs`，找到这个位置：

```rust
// 约在第 509 行
impl<'a> AggQueryBuilder<'a, Postgres> {
    pub fn build(&self) -> &'static str {
        // ... 现有实现 ...
    }
}
```

### Step 2: 在 build() 方法之后添加新方法

在 `build()` 方法之后（约第 524 行）添加：

```rust
// ============ 添加以下代码 ============

/// Execute query and return a single row.
pub async fn fetch_one<T>(
    self,
    pool: &Pool<Postgres>
) -> Result<T, sqlx::Error>
where
    T: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let sql = self.build();
    let mut query = sqlx::query_as::<_, T>(sql);

    // Bind WHERE parameters
    for param in &self.where_params {
        query = query.bind(param);
    }

    // Bind HAVING parameters
    for param in &self.having_params {
        query = query.bind(param);
    }

    query.fetch_one(pool).await
}

/// Execute query and return all rows.
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

/// Execute query and return optional result.
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

/// Convenience method for COUNT queries.
pub async fn fetch_count(
    self,
    pool: &Pool<Postgres>
) -> Result<i64, sqlx::Error> {
    let sql = self.build();
    let mut query = sqlx::query_as::<_, (i64,)>(sql);

    for param in &self.where_params {
        query = query.bind(param);
    }

    for param in &self.having_params {
        query = query.bind(param);
    }

    let (count,) = query.fetch_one(pool).await?;
    Ok(count)
}

/// Convenience method for AVG queries.
pub async fn fetch_avg(
    self,
    pool: &Pool<Postgres>
) -> Result<Option<f64>, sqlx::Error> {
    let sql = self.build();
    let mut query = sqlx::query_as::<_, (Option<f64>,)>(sql);

    for param in &self.where_params {
        query = query.bind(param);
    }

    for param in &self.having_params {
        query = query.bind(param);
    }

    let (avg,) = query.fetch_one(pool).await?;
    Ok(avg)
}

/// Convenience method for SUM queries.
pub async fn fetch_sum(
    self,
    pool: &Pool<Postgres>
) -> Result<Option<f64>, sqlx::Error> {
    let sql = self.build();
    let mut query = sqlx::query_as::<_, (Option<f64>,)>(sql);

    for param in &self.where_params {
        query = query.bind(param);
    }

    for param in &self.having_params {
        query = query.bind(param);
    }

    let (sum,) = query.fetch_one(pool).await?;
    Ok(sum)
}

// ============ 添加结束 ============
```

### Step 3: 为 MySQL 和 SQLite 添加相同的方法

在同一文件中，找到 `impl<'a> AggQueryBuilder<'a, MySql>` 和 `impl<'a> AggQueryBuilder<'a, Sqlite>`，添加完全相同的方法（只是把 `Postgres` 换成 `MySql` 或 `Sqlite`）。

### Step 4: 添加测试

在 `tests/aggregate_tests.rs` 末尾添加：

```rust
#[sqlx::test]
async fn test_fetch_count(pool: PgPool) -> Result<(), sqlx::Error> {
    // Create test table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_users_count (
            id VARCHAR PRIMARY KEY,
            role VARCHAR NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    // Insert test data
    sqlx::query("INSERT INTO test_users_count (id, role) VALUES ($1, $2)")
        .bind("user1")
        .bind("admin")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO test_users_count (id, role) VALUES ($1, $2)")
        .bind("user2")
        .bind("admin")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO test_users_count (id, role) VALUES ($1, $2)")
        .bind("user3")
        .bind("user")
        .execute(&pool)
        .await?;

    // Test fetch_count
    let count = TestUsersCount::agg_query()
        .where_("role = {}", &[&"admin"])
        .count()
        .fetch_count(&pool)
        .await?;

    assert_eq!(count, 2);

    // Cleanup
    sqlx::query("DROP TABLE test_users_count")
        .execute(&pool)
        .await?;

    Ok(())
}

#[sqlx::test]
async fn test_fetch_one_with_tuple(pool: PgPool) -> Result<(), sqlx::Error> {
    // Similar test for fetch_one with custom return types
    // ...
    Ok(())
}

#[sqlx::test]
async fn test_fetch_all_group_by(pool: PgPool) -> Result<(), sqlx::Error> {
    // Test GROUP BY with fetch_all
    // ...
    Ok(())
}

// Add test model at top of file
#[derive(sqlx_struct_enhanced::EnhancedCrud)]
#[table_name = "test_users_count"]
struct TestUsersCount {
    id: String,
    role: String,
}
```

### Step 5: 验证

```bash
cd /Users/alex/Projects/workspace/sqlx_struct_enhanced
cargo build
cargo test
```

## 使用新方法

### 示例 1: 简单 COUNT

```rust
// 之前（8 行）
let id_str = id.to_string();
let sql = User::agg_query()
    .where_("operation_center_id = {}", &[&id_str])
    .count()
    .build();
let (count,) = sqlx::query_as::<_, (i64,)>(sql)
    .bind(id)
    .fetch_one(&pool)
    .await?;

// 之后（2 行）
let count = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_count(&pool)
    .await?;
```

### 示例 2: AVG + COUNT

```rust
// 之前（10 行）
let id_str = id.to_string();
let sql = OrderRating::agg_query()
    .where_("engineer_id = {}", &[&id_str])
    .avg("score")
    .count()
    .build();
let (avg, count): (Option<f64>, i64) = sqlx::query_as(sql)
    .bind(id)
    .fetch_one(&pool)
    .await?;

// 之后（5 行）
let (avg, count): (Option<f64>, i64) = OrderRating::agg_query()
    .where_("engineer_id = {}", &[&id])
    .avg("score")
    .count()
    .fetch_one(&pool)
    .await?;
```

### 示例 3: GROUP BY

```rust
// 之前（10 行）
let sql = Order::agg_query()
    .group_by("status")
    .count()
    .build();
let results: Vec<(String, i64)> = sqlx::query_as(sql)
    .fetch_all(&pool)
    .await?;

// 之后（5 行）
let results: Vec<(String, i64)> = Order::agg_query()
    .group_by("status")
    .count()
    .fetch_all(&pool)
    .await?;
```

## 更新已迁移的代码（可选）

实现新方法后，可以简化之前迁移的代码：

### notification_service.rs

```rust
// 找到约第 176-194 行
pub async fn get_unread_count(&self, user_id: &Uuid) -> Result<i64> {
    // 替换为：
    let count = Notification::agg_query()
        .where_("user_id = {} AND read = {}", &[&user_id, &false])
        .count()
        .fetch_count(&self.pool)
        .await?;

    Ok(count)
}
```

### user_service.rs

```rust
// 找到约第 561-570 行
// 在 delete_operation_center 方法中
let user_count = User::agg_query()
    .where_("operation_center_id = {}", &[&id])
    .count()
    .fetch_count(&self.pool)
    .await?;
```

### rating_service.rs

```rust
// 找到约第 159-170 行
let (avg, count): (Option<f64>, i64) = OrderRating::agg_query()
    .where_("engineer_id = {}", &[&engineer_id])
    .avg("score")
    .count()
    .fetch_one(&self.pool)
    .await?;
```

## 检查清单

- [ ] 在 PostgreSQL 实现中添加 6 个方法
- [ ] 在 MySQL 实现中添加 6 个方法
- [ ] 在 SQLite 实现中添加 6 个方法
- [ ] 添加至少 3 个测试用例
- [ ] 运行 `cargo build` 验证编译
- [ ] 运行 `cargo test` 验证测试
- [ ] 更新文档（如果需要）
- [ ] 更新已迁移的代码（可选）

## 预期效果

**代码量减少**: 60-75%
**可读性提升**: 显著
**与 CRUD 一致性**: ✅ 达成

---

**时间估算**: 30-60 分钟
**难度**: 中等
**影响**: 向后兼容（不影响现有代码）
