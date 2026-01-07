# 基于查询模式的自动索引推断方案

## 1. 核心思想

**问题**: 手动定义索引容易过度索引或遗漏索引，影响性能

**解决方案**: 通过分析代码中实际使用的 `where_query()` 和 `make_query()` 语句，自动推断需要的索引

**示例**:
```rust
// 用户代码
User::where_query("status = $1 AND created_at > $2 ORDER BY created_at DESC")
    .bind("active")
    .bind(123456)
    .fetch_all(&pool).await?;

// 系统自动推断需要索引: Index(status, created_at)
// 并生成创建索引的 SQL
```

## 2. 方案设计

### 2.1 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│  用户代码                                                    │
│  User::where_query("status = $1 AND created_at > $2 ...")   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  Query 收集层                                                │
│  • 编译时收集: 宏收集所有 where_query/make_query 调用        │
│  • 测试时收集: 运行测试时记录实际执行的 SQL                  │
│  • 运行时收集: 生产环境收集查询模式                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  SQL 解析层                                                  │
│  • 解析 WHERE 子句: 提取等值条件列                           │
│  • 解析 ORDER BY: 提取排序列                                 │
│  • 识别查询模式: 精确匹配、范围查询、排序等                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  索引推断层                                                  │
│  • 单列索引: WHERE col = $1                                  │
│  • 联合索引: WHERE a = $1 AND b = $2 ORDER BY c              │
│  • 覆盖索引: SELECT a, b WHERE c = $1                        │
│  • 去重和合并: 避免重复索引                                  │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  索引生成层                                                  │
│  • 生成 CREATE INDEX SQL                                    │
│  • 优化索引顺序: 基于选择性、查询频率                        │
│  • 生成迁移脚本                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. 实现方案

### 3.1 方案 A: 编译时宏收集（推荐 ⭐）

#### 工作原理

在 derive macro 中，为每个 `where_query()` 和 `make_query()` 调用添加元数据收集：

```rust
// 用户代码
User::where_query("status = $1 AND created_at > $2 ORDER BY created_at DESC")
    .bind("active")
    .bind(123456)
    .fetch_all(&pool).await?;

// 宏展开后的代码（简化）
User::where_query_with_metadata(
    "status = $1 AND created_at > $2 ORDER BY created_at DESC",
    QueryMetadata {
        where_columns: vec!["status", "created_at"],
        order_by_columns: vec!["created_at"],
        query_type: QueryType::SelectWithOrder,
    }
)
.bind("active")
.bind(123456)
.fetch_all(&pool).await?;
```

#### 实现步骤

**Step 1: 定义查询元数据结构**

```rust
// 在 src/lib.rs 中添加

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryMetadata {
    pub where_columns: Vec<String>,
    pub order_by_columns: Vec<String>,
    pub query_type: QueryType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    SimpleSelect,           // SELECT * FROM table WHERE col = $1
    SelectWithOrder,        // ... ORDER BY col
    RangeQuery,             // WHERE col > $1
    MultiColumnEquality,    // WHERE a = $1 AND b = $2
    InQuery,                // WHERE col IN ($1, $2, $3)
}

#[derive(Debug, Clone)]
pub struct InferredIndex {
    pub columns: Vec<String>,
    pub unique: bool,
    pub reason: String,  // 为什么需要这个索引
    pub queries: Vec<String>,  // 哪些查询需要这个索引
}
```

**Step 2: 修改 Trait 方法签名**

```rust
// 在 src/traits.rs 中修改

pub trait EnhancedCrud {
    // 原有方法保持不变
    fn where_query(statement: &str) -> QueryAs<'_, DB, Self, ...> {
        // 新增：内部调用带元数据的版本
        Self::where_query_with_metadata(statement)
    }

    // 新增：带元数据的方法（内部使用）
    fn where_query_with_metadata(statement: &str) -> QueryAs<'_, DB, Self, ...> {
        #gen_scheme_code
        let sql = scheme.gen_select_where_sql_static(statement);
        let metadata = scheme.parse_query_metadata(statement);  // 解析元数据
        scheme.record_query_metadata(metadata);  // 记录到全局存储
        sqlx::query_as::<DB, Self>(sql)
    }

    // 新增：获取推断的索引
    fn get_inferred_indexes() -> Vec<InferredIndex>
    where
        Self: Sized;
}
```

**Step 3: SQL 解析实现**

```rust
// 在 src/lib.rs 中添加

impl Scheme {
    /// 从 SQL 语句中提取查询元数据
    pub fn parse_query_metadata(&self, sql: &str) -> QueryMetadata {
        let sql_lower = sql.to_lowercase();

        // 提取 WHERE 子句中的列
        let where_columns = self.extract_where_columns(&sql_lower);

        // 提取 ORDER BY 子句中的列
        let order_by_columns = self.extract_order_by_columns(&sql_lower);

        // 推断查询类型
        let query_type = self.infer_query_type(&sql_lower, &where_columns, &order_by_columns);

        QueryMetadata {
            where_columns,
            order_by_columns,
            query_type,
        }
    }

    /// 提取 WHERE 子句中的列名
    fn extract_where_columns(&self, sql: &str) -> Vec<String> {
        if let Some(where_start) = sql.find("where") {
            let where_clause = &sql[where_start + 5..];

            // 简单的列名提取（寻找 "col =", "col >", "col <", "col >=" 等模式）
            let mut columns = Vec::new();

            for field in &self.insert_fields {
                let patterns = [
                    &format!("{} = ", field),
                    &format!("{}>", field),
                    &format!("{} <", field),
                    &format!("{}>=", field),
                    &format!("{}<=", field),
                    &format!("{} != ", field),
                    &format!("{} in ", field),
                ];

                for pattern in &patterns {
                    if where_clause.contains(pattern) {
                        columns.push(field.clone());
                        break;
                    }
                }
            }

            columns
        } else {
            Vec::new()
        }
    }

    /// 提取 ORDER BY 子句中的列名
    fn extract_order_by_columns(&self, sql: &str) -> Vec<String> {
        if let Some(order_by_start) = sql.find("order by") {
            let order_clause = &sql[order_by_start + 9..];

            // 提取到下一个关键字（LIMIT, OFFSET）或结尾
            let order_clause_end = order_clause
                .find(" limit")
                .or_else(|| order_clause.find(" offset"))
                .unwrap_or(order_clause.len());

            let order_clause = &order_clause[..order_clause_end];

            // 解析列名（可能包含 ASC/DESC）
            order_clause
                .split(',')
                .filter_map(|col| {
                    let col = col.trim();
                    // 移除 ASC/DESC
                    let col_name = col
                        .trim_end_matches(" asc")
                        .trim_end_matches(" desc")
                        .trim();

                    // 验证是否是表中的列
                    if self.insert_fields.contains(&col_name.to_string()) {
                        Some(col_name.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 推断查询类型
    fn infer_query_type(
        &self,
        _sql: &str,
        where_columns: &[String],
        order_by_columns: &[String],
    ) -> QueryType {
        match (where_columns.len(), order_by_columns.is_empty()) {
            (0, true) => QueryType::SimpleSelect,
            (1, false) => QueryType::SelectWithOrder,
            (n, _) if n > 1 => QueryType::MultiColumnEquality,
            _ => QueryType::SimpleSelect,
        }
    }
}
```

**Step 4: 全局查询元数据存储**

```rust
// 在 src/lib.rs 中添加

use once_cell::sync::Mutex;
use std::sync::Arc;

// 全局存储: 表名 -> 查询元数据列表
static QUERY_METADATA_STORE: Lazy<Arc<Mutex<HashMap<String, Vec<QueryMetadata>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

impl Scheme {
    /// 记录查询元数据到全局存储
    fn record_query_metadata(&self, metadata: QueryMetadata) {
        let mut store = QUERY_METADATA_STORE.lock().unwrap();
        store
            .entry(self.table_name.clone())
            .or_insert_with(Vec::new)
            .push(metadata);
    }

    /// 获取某个表的所有查询元数据
    pub fn get_query_metadata(table_name: &str) -> Vec<QueryMetadata> {
        let store = QUERY_METADATA_STORE.lock().unwrap();
        store.get(table_name).cloned().unwrap_or_default()
    }
}
```

**Step 5: 索引推断算法**

```rust
// 在 src/lib.rs 中添加

impl Scheme {
    /// 基于查询模式推断需要的索引
    pub fn infer_indexes(table_name: &str, all_fields: &[String]) -> Vec<InferredIndex> {
        let queries = Self::get_query_metadata(table_name);

        if queries.is_empty() {
            return Vec::new();
        }

        let mut indexes: Vec<InferredIndex> = Vec::new();

        // 分析每个查询
        for query in &queries {
            // 规则 1: WHERE 单列 = 值
            if query.where_columns.len() == 1 && query.order_by_columns.is_empty() {
                let col = &query.where_columns[0];
                indexes.push(InferredIndex {
                    columns: vec![col.clone()],
                    unique: false,
                    reason: format!("Single column equality: WHERE {} = $1", col),
                    queries: vec![format!("{:?}", query)],
                });
            }

            // 规则 2: WHERE 多列 AND ORDER BY
            if !query.where_columns.is_empty() || !query.order_by_columns.is_empty() {
                let mut index_columns = query.where_columns.clone();
                index_columns.extend(query.order_by_columns.clone());

                // 去重
                index_columns.sort();
                index_columns.dedup();

                indexes.push(InferredIndex {
                    columns: index_columns.clone(),
                    unique: false,
                    reason: format!(
                        "Multi-column query: WHERE {} ORDER BY {}",
                        query.where_columns.join(", "),
                        query.order_by_columns.join(", ")
                    ),
                    queries: vec![format!("{:?}", query)],
                });
            }

            // 规则 3: ORDER BY 单列（无 WHERE）
            if query.where_columns.is_empty() && query.order_by_columns.len() == 1 {
                indexes.push(InferredIndex {
                    columns: query.order_by_columns.clone(),
                    unique: false,
                    reason: format!("Order by: {}", query.order_by_columns[0]),
                    queries: vec![format!("{:?}", query)],
                });
            }
        }

        // 合并重复的索引
        Self::merge_indexes(indexes)
    }

    /// 合并重复或包含的索引
    fn merge_indexes(indexes: Vec<InferredIndex>) -> Vec<InferredIndex> {
        use std::collections::HashMap;

        let mut merged: HashMap<Vec<String>, InferredIndex> = HashMap::new();

        for index in indexes {
            let key = index.columns.clone();

            if let Some(existing) = merged.get_mut(&key) {
                // 合并原因和查询列表
                existing.reason.push_str(&format!("; {}", index.reason));
                existing.queries.extend(index.queries);
            } else {
                merged.insert(key, index);
            }
        }

        merged.into_values().collect()
    }
}
```

**Step 6: 生成索引 SQL**

```rust
impl Scheme {
    /// 为推断的索引生成 CREATE INDEX SQL
    pub fn gen_inferred_indexes_sql(
        table_name: &str,
        all_fields: &[String],
    ) -> Vec<(String, String)> {
        let indexes = Self::infer_indexes(table_name, all_fields);

        indexes
            .into_iter()
            .map(|index| {
                let index_name = Self::generate_index_name(table_name, &index.columns);
                let columns = index.columns.join(", ");
                let sql = format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                    index_name, table_name, columns
                );

                (sql, index.reason)
            })
            .collect()
    }

    fn generate_index_name(table_name: &str, columns: &[String]) -> String {
        format!(
            "idx_{}_{}",
            table_name,
            columns.join("_")
        )
    }
}
```

**Step 7: 添加 Trait 方法**

```rust
// 在 traits.rs 中添加

pub trait EnhancedCrud {
    // ... 现有方法 ...

    /// 获取推断的索引定义（基于查询模式）
    fn get_inferred_indexes() -> Vec<InferredIndex>
    where
        Self: Sized,
    {
        #gen_scheme_code
        let all_fields = scheme.insert_fields.clone();
        scheme.infer_indexes(&scheme.table_name, &all_fields)
    }

    /// 生成所有推断索引的创建 SQL
    fn generate_inferred_indexes_sql() -> Vec<(String, String)>
    where
        Self: Sized,
    {
        #gen_scheme_code
        let all_fields = scheme.insert_fields.clone();
        Scheme::gen_inferred_indexes_sql(&scheme.table_name, &all_fields)
    }
}
```

#### 使用示例

```rust
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(EnhancedCrud)]
struct User {
    id: String,
    email: String,
    status: String,
    created_at: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = connect_to_db().await?;

    // 业务代码中的查询
    let users = User::where_query("status = $1 AND created_at > $2 ORDER BY created_at DESC")
        .bind("active")
        .bind(123456)
        .fetch_all(&pool)
        .await?;

    // ... 更多查询 ...

    // 在测试或开发环境，生成推荐的索引
    let recommended_indexes = User::get_inferred_indexes();

    println!("Recommended indexes for User table:");
    for index in recommended_indexes {
        println!("  - Index on columns: {:?}", index.columns);
        println!("    Reason: {}", index.reason);
        println!("    Used by queries:");
        for query in &index.queries {
            println!("      - {}", query);
        }
    }

    // 生成 SQL
    let index_sqls = User::generate_inferred_indexes_sql();
    for (sql, reason) in index_sqls {
        println!("{} -- {}", sql, reason);
        // 可选: 自动执行
        // sqlx::query(&sql).execute(&pool).await.ok();
    }

    Ok(())
}
```

### 3.2 方案 B: 测试时收集

在运行集成测试时，自动收集所有查询并分析：

```rust
// 在测试中启用查询收集
#[sqlx_struct_enhanced::collect_queries]
#[tokio::test]
async fn test_user_queries() -> Result<(), sqlx::Error> {
    // 执行各种查询
    User::where_query("status = $1").bind("active").fetch_all(&pool).await?;

    // 测试结束后，自动生成索引推荐
    let indexes = User::get_inferred_indexes();
    assert_indexes_exist(indexes).await?;

    Ok(())
}
```

### 3.3 方案 C: 静态源代码分析

使用 Rust 的语法分析，直接找到所有 `where_query()` 和 `make_query()` 调用：

```rust
// 构建时工具
cargo sqlx-struct-analyze --bin my_app

// 输出:
// Analyzing queries...
// Found 15 queries for User table
// Recommended indexes:
//   - Index(status, created_at)
//     Used by: 5 queries
//   - Index(email)
//     Used by: 3 queries
```

## 4. 高级索引推断规则

### 4.1 索引列顺序优化

```rust
impl Scheme {
    /// 优化索引列顺序
    ///
    /// 规则:
    /// 1. 等值条件列在前 (WHERE col = $1)
    /// 2. 范围条件列在后 (WHERE col > $1)
    /// 3. 排序列最后 (ORDER BY col)
    fn optimize_index_order(columns: &QueryMetadata) -> Vec<String> {
        let mut ordered = Vec::new();

        // 等值条件
        for col in &columns.where_columns {
            // TODO: 检测是否是等值条件
            ordered.push(col.clone());
        }

        // 范围条件和排序列
        for col in &columns.order_by_columns {
            if !ordered.contains(col) {
                ordered.push(col.clone());
            }
        }

        ordered
    }
}
```

### 4.2 识别部分索引

```rust
impl Scheme {
    /// 识别可以优化为部分索引的查询
    ///
    /// 例如: WHERE status = 'active' AND created_at > $1
    /// 推荐索引: CREATE INDEX ... WHERE status = 'active'
    fn infer_partial_index(metadata: &QueryMetadata) -> Option<String> {
        // 检测是否有高选择性的常量条件
        // 例如: status = 'active', deleted = false 等
        if let Some(partial_condition) = self.extract_high_selectivity_condition(metadata) {
            Some(partial_condition)
        } else {
            None
        }
    }
}
```

### 4.3 识别覆盖索引

```rust
impl Scheme {
    /// 识别可以创建覆盖索引的查询
    ///
    /// 覆盖索引: 包含查询所有列的索引，避免回表
    fn infer_covering_index(
        sql: &str,
        metadata: &QueryMetadata,
        all_fields: &[String],
    ) -> Option<Vec<String>> {
        // 提取 SELECT 中的列
        let select_columns = self.extract_select_columns(sql);

        // 如果所有 SELECT 列都在索引中，可以作为覆盖索引
        let mut index_columns = metadata.where_columns.clone();
        index_columns.extend(metadata.order_by_columns.clone());
        index_columns.extend(select_columns);

        if index_columns.len() <= 5 {  // 限制索引列数
            Some(index_columns)
        } else {
            None
        }
    }
}
```

## 5. 工作流程

### 5.1 开发阶段

```bash
# 1. 编写代码和测试
cargo test

# 2. 运行索引分析工具
cargo run --bin analyze_indexes

# 3. 查看推荐的索引
# Recommended indexes for User:
#   - idx_user_status_created_at
#     Reason: WHERE status = $1 ORDER BY created_at
#     Used by: 5 queries

# 4. 生成迁移脚本
cargo run --bin analyze_indexes -- --migrate > migrations/create_indexes.sql

# 5. Review 并应用迁移
psql -d mydb -f migrations/create_indexes.sql
```

### 5.2 CI/CD 集成

```yaml
# .github/workflows/index-analysis.yml
name: Index Analysis

on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run tests
        run: cargo test
      - name: Analyze indexes
        run: cargo run --bin analyze_indexes
      - name: Check for missing indexes
        run: |
          if cargo run --bin analyze_indexes -- --check; then
            echo "All queries have proper indexes"
          else
            echo "::error::Some queries are missing indexes"
            exit 1
          fi
```

## 6. 实现计划

### Phase 1: 基础推断 (1周)

- ✅ SQL 解析基础实现
- ✅ WHERE 子句列提取
- ✅ ORDER BY 列提取
- ✅ 简单索引推断（单列、多列）
- ✅ 单元测试

### Phase 2: 高级推断 (1周)

- ✅ 索引列顺序优化
- ✅ 部分索引识别
- ✅ 覆盖索引识别
- ✅ 索引去重和合并

### Phase 3: 工具集成 (1周)

- ✅ CLI 工具实现
- ✅ 测试集成
- ✅ CI/CD 集成
- ✅ 文档和示例

## 7. 优势分析

### vs 手动定义索引

| 特性 | 手动定义 | 自动推断 |
|------|---------|---------|
| 维护成本 | 高 | 低 |
| 准确性 | 取决于经验 | 基于实际查询 |
| 过度索引 | 容易出现 | 自动避免 |
| 遗漏索引 | 可能遗漏 | 完整覆盖 |
| 性能优化 | 需要手动分析 | 自动优化 |

### vs 其他方案

- **SeaORM/Diesel**: 需要手动定义，或使用外部工具
- **pg_stat_statements**: 需要生产环境数据，滞后
- **EXPLAIN ANALYZE**: 手动分析每个查询，耗时

本方案:
- ✅ 自动化
- ✅ 基于实际代码
- ✅ 开发时即可获得
- ✅ 零运行时开销

## 8. 局限性

### 8.1 无法处理的情况

1. **动态 SQL**: 字符串拼接的查询
2. **ORM 生成的查询**: 其他工具生成的查询
3. **存储过程**: 数据库内部的逻辑
4. **复杂查询**: 子查询、JOIN 等

### 8.2 需要人工判断

1. **选择性**: 某些列选择性低，不适合索引
2. **写入频率**: 高写入频率的表需要权衡
3. **数据分布**: 数据倾斜影响索引效果
4. **业务特点**: 特殊业务逻辑需要特殊考虑

## 9. 最佳实践

### 9.1 使用流程

1. **正常开发**: 编写业务代码
2. **运行测试**: 执行所有测试
3. **分析索引**: 运行分析工具
4. **Review**: 人工审查推荐索引
5. **应用**: 创建索引
6. **验证**: 运行性能测试

### 9.2 何时使用

✅ **适合**:
- 新项目，从零开始
- 查询模式清晰且稳定
- 团队熟悉数据库索引
- 有完善的测试覆盖

❌ **不适合**:
- 遗留系统（已有索引）
- 高度动态的查询
- 复杂的多表关联
- 极端性能要求的场景

## 10. 与原方案的对比

| 维度 | 方案 A: 手动定义 | 方案 B: 自动推断 |
|------|----------------|----------------|
| API 设计 | `#[index(...)]` | 无需额外 API |
| 用户负担 | 需要理解索引 | 自动化 |
| 准确性 | 可能遗漏/过度 | 基于实际查询 |
| 维护成本 | 代码变更需更新索引 | 自动同步 |
| 学习曲线 | 需要索引知识 | 零学习 |
| 实现复杂度 | 中等 | 较高 |
| 灵活性 | 完全控制 | 自动推断 |
| 适用场景 | 所有场景 | 查询模式明确 |

## 11. 混合方案（最优解 ⭐⭐⭐）

结合两种方案的优点：

```rust
#[derive(EnhancedCrud)]
// 手动定义关键索引（如有特殊需求）
#[index(columns = ["email"], unique = true)]
struct User {
    id: String,
    email: String,
    status: String,
    created_at: i64,
}

// 自动推断其他索引
// 运行测试后，自动生成推荐的索引补充
```

**工作流程**:
1. 用户手动定义**必须的索引**（如唯一约束）
2. 运行测试，自动收集查询模式
3. 生成**推荐的索引**补充
4. Review 后选择性地应用

## 12. 总结

### 核心价值

1. **自动化**: 减少人工分析工作
2. **准确性**: 基于实际查询模式
3. **及时性**: 开发时即可获得
4. **智能化**: 自动优化索引设计

### 技术可行性

- ✅ SQL 解析可行（已有成熟库）
- ✅ 元数据收集可行（编译时/测试时）
- ✅ 索引推断算法成熟
- ✅ 不影响现有代码

### 推荐实施

采用 **混合方案**：
- **手动**: 定义核心索引（唯一约束等）
- **自动**: 推荐补充索引
- **Review**: 人工审查后应用
