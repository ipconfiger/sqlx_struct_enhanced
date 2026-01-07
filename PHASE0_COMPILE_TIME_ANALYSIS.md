# Phase 0: 编译期索引分析 - MVP方案

## 🎯 目标

实现一个**最小可行产品（MVP）**，在**编译期**分析代码中的查询，直接输出索引推荐，无需运行时收集。

### 核心特点

- ✅ **编译时分析** - 无需运行程序
- ✅ **静态代码分析** - 分析宏展开的tokens
- ✅ **即时反馈** - 编译时直接打印推荐
- ✅ **零运行时开销** - 不影响生产代码
- ✅ **简单实现** - 2周即可完成

---

## 📋 实施计划

### Week 1: 基础设施

#### Day 1-2: SQL解析器（简化版）

```rust
// src/sql_parser.rs

/// 简化的SQL解析器 - 仅支持编译期分析
pub struct CompileTimeSqlParser {
    table_columns: Vec<String>,
}

impl CompileTimeSqlParser {
    /// 解析WHERE子句，提取列名
    pub fn parse_where_columns(&self, sql: &str) -> Vec<String> {
        let mut columns = Vec::new();

        // 简单的字符串匹配（不需要完整的SQL解析）
        for col in &self.table_columns {
            // 匹配模式: "col = ", "col>", "col <", "col IN", "col >=" 等
            let patterns = [
                &format!(" {} = ", col),
                &format!("{}>", col),
                &format!(" {}<", col),
                &format!("{}>=", col),
                &format!(" {}<=", col),
                &format!(" {} IN ", col),
                &format!("{}in", col),
            ];

            for pattern in &patterns {
                if sql.contains(pattern) {
                    columns.push(col.clone());
                    break;
                }
            }
        }

        columns
    }

    /// 解析ORDER BY子句，提取列名
    pub fn parse_order_by_columns(&self, sql: &str) -> Vec<(String, bool)> {
        let mut columns = Vec::new();

        // 查找 "ORDER BY"
        if let Some(order_pos) = sql.to_lowercase().find("order by") {
            let order_clause = &sql[order_pos + 9..];

            // 简单的列名提取
            for col in &self.table_columns {
                if order_clause.contains(col) {
                    let is_desc = order_clause
                        .to_lowercase()
                        .contains(&format!("{} desc", col));
                    columns.push((col.clone(), is_desc));
                }
            }
        }

        columns
    }

    /// 从SQL提取索引列（按顺序）
    pub fn extract_index_columns(&self, sql: &str) -> Vec<String> {
        let mut index_columns = Vec::new();

        // 1. 先添加WHERE中的等值列
        for col in self.parse_where_columns(sql) {
            if !index_columns.contains(&col) {
                index_columns.push(col);
            }
        }

        // 2. 再添加ORDER BY中的列
        for (col, _) in self.parse_order_by_columns(sql) {
            if !index_columns.contains(&col) {
                index_columns.push(col);
            }
        }

        index_columns
    }
}
```

#### Day 3-4: 宏层面的查询收集

```rust
// sqlx_struct_macros/src/lib.rs

/// 新增：辅助宏，用于标记和分析查询
#[proc_macro_attribute]
pub fn analyze_queries(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // 在宏中查找所有 where_query! 和 make_query! 调用
    let queries = extract_queries_from_code(&input_str);

    // 分析并打印推荐索引
    for query in queries {
        let table_name = query.table_name;
        let sql = &query.sql;

        // 生成索引推荐
        let parser = CompileTimeSqlParser::new(query.table_fields);
        let index_columns = parser.extract_index_columns(sql);

        if !index_columns.is_empty() {
            // 在编译时打印（使用 println! 在宏展开时）
            println!(
                "🔍 [sqlx-struct] Found query for '{}': {}",
                table_name, sql
            );
            println!(
                "   💡 Recommended index: idx_{}",
                table_name,
                index_columns.join("_")
            );
            println!(
                "   → CREATE INDEX idx_{}_{} ON {} ({})",
                table_name,
                index_columns.join("_"),
                table_name,
                index_columns.join(", ")
            );
        }
    }

    // 返回原始代码，不做修改
    input
}

/// 从代码中提取查询信息
struct QueryInfo {
    table_name: String,
    table_fields: Vec<String>,
    sql: String,
}

fn extract_queries_from_code(code: &str) -> Vec<QueryInfo> {
    let mut queries = Vec::new();

    // 正则匹配 where_query!("...") 或 make_query!("...")
    let re = regex::Regex::new(
        r#"(?m)\b( where_query!| make_query!)\s*\(\s*"([^"]+)""#
    ).unwrap();

    for cap in re.captures_iter(code) {
        let sql = cap[2].to_string();

        // 尝试推断表名（从上下文）
        // 这里简化处理：假设代码中包含 Table::where_query!() 模式
        let table_re = regex::Regex::new(r#"\b(\w+)::\s*where_query!"#).unwrap();
        let table_name = table_re
            .captures(code)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // 获取表的字段（这里需要从结构体定义中提取）
        let table_fields = vec![];  // 稍后实现

        queries.push(QueryInfo {
            table_name,
            table_fields,
            sql,
        });
    }

    queries
}
```

### Week 2: 完善和测试

#### Day 5-7: 实现字段提取

```rust
// sqlx_struct_macros/src/field_extractor.rs

use syn::{ItemStruct, Path};

/// 从结构体定义中提取字段名
pub fn extract_struct_fields(ast: &syn::DeriveInput) -> Vec<String> {
    match &ast.data {
        syn::Data::Struct(data_struct) => {
            data_struct
                .fields
                .iter()
                .filter_map(|field| {
                    field.ident.as_ref().map(|ident| ident.to_string())
                })
                .collect()
        }
        _ => vec![],
    }
}

/// 从代码中查找所有使用了 EnhancedCrud 的结构体
pub fn find_crud_structs(code: &str) -> Vec<(String, Vec<String>)> {
    let mut structs = Vec::new();

    // 解析为 syn::File
    let file = syn::parse_file(code).unwrap();

    for item in file.items {
        if let syn::Item::Struct(item_struct) = item {
            // 检查是否有 #[derive(EnhancedCrud)]
            let has_enhanced_crud = item_struct
                .attrs
                .iter()
                .any(|attr| {
                    attr.path()
                        .segments
                        .iter()
                        .any(|seg| seg.ident == "EnhancedCrud")
                });

            if has_enhanced_crud {
                let name = item_struct.ident.to_string();
                let fields = extract_struct_fields(&syn::DeriveInput {
                    attrs: vec![],
                    vis: item_struct.vis,
                    ident: item_struct.ident,
                    generics: item_struct.generics,
                    data: item_struct.data.clone(),
                });

                structs.push((name, fields));
            }
        }
    }

    structs
}
```

#### Day 8-10: 测试和文档

```rust
// tests/compile_time_analysis_test.rs

#[test]
fn test_simple_query_analysis() {
    let code = r#"
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        status: String,
    }

    impl User {
        async fn find_by_email(pool: &PgPool, email: &str) -> Result<Self> {
            User::where_query("email = $1")
                .bind(email)
                .fetch_one(pool)
                .await
        }
    }
    "#;

    // 分析代码
    let recommendations = analyze_code_for_indexes(code);

    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0].table_name, "User");
    assert_eq!(recommendations[0].index_columns, vec!["email"]);
}

#[test]
fn test_complex_query_analysis() {
    let code = r#"
    #[derive(EnhancedCrud)]
    struct User {
        id: String,
        email: String,
        status: String,
        created_at: i64,
    }

    impl User {
        async fn find_active_users(pool: &PgPool) -> Result<Vec<Self>> {
            User::where_query("status = $1 AND created_at > $2 ORDER BY created_at DESC")
                .bind("active")
                .bind(123456)
                .fetch_all(pool)
                .await
        }
    }
    "#;

    let recommendations = analyze_code_for_indexes(code);

    assert_eq!(recommendations.len(), 1);
    assert_eq!(recommendations[0].index_columns, vec!["status", "created_at"]);
}
```

---

## 🚀 使用方式

### 方式1: 使用辅助宏（推荐）

```rust
#[analyze_queries]  // 添加这个属性
mod user_queries {
    use super::*;

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User> {
        User::where_query("email = $1")
            .bind(email)
            .fetch_one(pool)
            .await
    }

    pub async fn find_active_users(pool: &PgPool) -> Result<Vec<User>> {
        User::where_query("status = $1 AND created_at > $2 ORDER BY created_at DESC")
            .bind("active")
            .bind(123456)
            .fetch_all(pool)
            .await
    }
}
```

**编译输出**：
```
   Compiling your_project v0.1.0

🔍 [sqlx-struct] Found query for 'User': email = $1
   💡 Recommended index: idx_User_email
   → CREATE INDEX idx_User_email ON User (email)

🔍 [sqlx-struct] Found query for 'User': status = $1 AND created_at > $2 ORDER BY created_at DESC
   💡 Recommended index: idx_User_status_created_at
   → CREATE INDEX idx_User_status_created_at ON User (status, created_at DESC)

    Finished dev [unoptimized + debuginfo] target(s) in 2.5s
```

### 方式2: 手动触发分析

```rust
// 在测试或专门的模块中
sqlx_struct_enhanced::analyze_current_module!();
```

这会：
1. 扫描当前模块的所有查询
2. 分析并打印推荐
3. 不影响正常编译

### 方式3: Cargo子命令

```bash
# 分析整个项目
cargo sqlx-struct-analyze

# 输出:
# Analyzing src/...
# Found 15 queries across 5 tables
#
# Recommendations for User:
#   1. CREATE INDEX idx_user_email ON user (email)
#      Reason: WHERE email = $1
#      Found in: src/user_queries.rs:10
#
#   2. CREATE INDEX idx_user_status_created_at ON user (status, created_at DESC)
#      Reason: WHERE status = $1 AND created_at > $2 ORDER BY created_at DESC
#      Found in: src/user_queries.rs:25
```

---

## 📦 文件结构

```
sqlx_struct_enhanced/
├── sqlx_struct_macros/
│   └── src/
│       ├── lib.rs                    # 主入口
│       ├── compile_time_analyzer.rs   # 编译期分析器 (新增)
│       ├── query_extractor.rs         # 查询提取器 (新增)
│       ├── field_extractor.rs         # 字段提取器 (新增)
│       └── simple_parser.rs          # 简化SQL解析器 (新增)
├── src/
│   └── analysis.rs                   # 运行时分析API (新增)
└── examples/
    └── compile_time_analysis.rs       # 使用示例 (新增)
```

---

## 🔧 核心实现

### 1. 编译期分析器

```rust
// sqlx_struct_macros/src/compile_time_analyzer.rs

use proc_macro::{TokenStream, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use crate::simple_parser::SimpleSqlParser;
use crate::query_extractor::QueryExtractor;

pub struct CompileTimeAnalyzer {
    table_name: String,
    table_fields: Vec<String>,
}

impl CompileTimeAnalyzer {
    pub fn new(input: &DeriveInput) -> Self {
        let table_name = input.ident.to_string();
        let table_fields = Self::extract_fields(&input);

        Self { table_name, table_fields }
    }

    /// 分析并生成推荐代码
    pub fn analyze_and_recommend(&self) -> proc_macro2::TokenStream {
        // 这里实际上不做任何事，推荐由独立的宏处理
        quote! {}
    }

    /// 提取结构体字段
    fn extract_fields(input: &DeriveInput) -> Vec<String> {
        match &input.data {
            syn::Data::Struct(struct_data) => {
                struct_data
                    .fields
                    .iter()
                    .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
                    .collect()
            }
            _ => vec![],
        }
    }
}

/// 分析器辅助宏 - 用于分析整个模块
#[proc_macro_attribute]
pub fn analyze_queries(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // 提取所有查询
    let extractor = QueryExtractor::new();
    let queries = extractor.extract_from_code(&input_str);

    // 打印推荐（编译时）
    for query in &queries {
        println!("🔍 Found query: {}", query.sql);

        // 解析SQL
        let parser = SimpleSqlParser::new(query.table_fields.clone());
        let index_cols = parser.extract_index_columns(&query.sql);

        if !index_cols.is_empty() {
            let index_name = format!("idx_{}_{}", query.table_name, index_cols.join("_"));

            println!("   💡 Recommended: {}", index_name);
            println!("   → CREATE INDEX {} ON {} ({})",
                index_name,
                query.table_name,
                index_cols.join(", ")
            );
        }
    }

    // 返回原始代码
    input
}
```

### 2. 查询提取器

```rust
// sqlx_struct_macros/src/query_extractor.rs

pub struct QueryExtractor;

impl QueryExtractor {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_from_code(&self, code: &str) -> Vec<ExtractedQuery> {
        let mut queries = Vec::new();

        // 查找 where_query!("...") 模式
        let where_query_re = regex::Regex::new(
            r#"(?m)\b(\w+)::\s*where_query!\s*\(\s*"((?:[^"\\]|\\.)*)""#
        ).unwrap();

        for cap in where_query_re.captures_iter(code) {
            let table_name = cap[1].to_string();
            let sql = cap[2].to_string();

            // 获取表的字段（这里简化处理）
            let table_fields = Self::find_table_fields(code, &table_name);

            queries.push(ExtractedQuery {
                table_name,
                table_fields,
                sql,
                query_type: QueryType::WhereQuery,
            });
        }

        // 查找 make_query!("...") 模式
        let make_query_re = regex::Regex::new(
            r#"(?m)\b(\w+)::\s*make_query!\s*\(\s*"((?:[^"\\]|\\.)*)""#
        ).unwrap();

        for cap in make_query_re.captures_iter(code) {
            let table_name = cap[1].to_string();
            let sql = cap[2].to_string();
            let table_fields = Self::find_table_fields(code, &table_name);

            queries.push(ExtractedQuery {
                table_name,
                table_fields,
                sql,
                query_type: QueryType::MakeQuery,
            });
        }

        queries
    }

    fn find_table_fields(code: &str, table_name: &str) -> Vec<String> {
        // 查找 struct TableName { ... } 定义
        let struct_re = regex::Regex::new(
            &format!(r#"struct\s+{}\s*\{{([^}]+)\}"#, table_name)
        ).unwrap();

        if let Some(cap) = struct_re.captures(code) {
            let fields_str = &cap[1];

            fields_str
                .split(',')
                .filter_map(|field| {
                    let parts: Vec<&str> = field.split(':').collect();
                    if parts.is_empty() {
                        return None;
                    }
                    Some(parts[0].trim().to_string())
                })
                .collect()
        } else {
            vec![]
        }
    }
}

pub struct ExtractedQuery {
    pub table_name: String,
    pub table_fields: Vec<String>,
    pub sql: String,
    pub query_type: QueryType,
}

pub enum QueryType {
    WhereQuery,
    MakeQuery,
}
```

### 3. 简化的SQL解析器

```rust
// sqlx_struct_macros/src/simple_parser.rs

pub struct SimpleSqlParser {
    table_columns: Vec<String>,
}

impl SimpleSqlParser {
    pub fn new(table_columns: Vec<String>) -> Self {
        Self { table_columns }
    }

    /// 从SQL提取索引列
    pub fn extract_index_columns(&self, sql: &str) -> Vec<String> {
        let mut columns = Vec::new();

        // 1. WHERE子句中的列（等值条件优先）
        for col in self.parse_where_columns(sql) {
            if !columns.contains(&col) {
                columns.push(col);
            }
        }

        // 2. ORDER BY子句中的列
        for col in self.parse_order_by_columns(sql) {
            if !columns.contains(&col) {
                columns.push(col);
            }
        }

        columns
    }

    fn parse_where_columns(&self, sql: &str) -> Vec<String> {
        let mut found_columns = Vec::new();
        let sql_lower = sql.to_lowercase();

        if let Some(where_pos) = sql_lower.find("where") {
            let where_clause = &sql_lower[where_pos + 5..];

            // 查找下一个关键字作为结束
            let where_end = where_clause
                .find(" group by")
                .or_else(|| where_clause.find(" order by"))
                .or_else(|| where_clause.find(" limit"))
                .unwrap_or(where_clause.len());

            let where_clause = &where_clause[..where_end];

            // 检查每个表字段
            for col in &self.table_columns {
                // 匹配: col =, col >=, col <=, col >, col <, col IN
                if where_clause.contains(&format!("{} =", col))
                    || where_clause.contains(&format!("{}>=", col))
                    || where_clause.contains(&format!("{}<=", col))
                    || where_clause.contains(&format!("{}>", col))
                    || where_clause.contains(&format!("{}<", col))
                    || where_clause.contains(&format!("{} in ", col))
                {
                    found_columns.push(col.clone());
                }
            }
        }

        found_columns
    }

    fn parse_order_by_columns(&self, sql: &str) -> Vec<String> {
        let mut found_columns = Vec::new();
        let sql_lower = sql.to_lowercase();

        if let Some(order_pos) = sql_lower.find("order by") {
            let order_clause = &sql_lower[order_pos + 9..];

            // 检查每个表字段
            for col in &self.table_columns {
                if order_clause.contains(col) {
                    found_columns.push(col.clone());
                }
            }
        }

        found_columns
    }
}
```

---

## ✅ 验收标准

### 功能验收

- [ ] 能够正确识别代码中的 `where_query!()` 和 `make_query!()` 调用
- [ ] 能够从SQL中提取 WHERE 和 ORDER BY 列
- [ ] 能够生成合理的索引推荐
- [ ] 编译时能够打印推荐信息
- [ ] 不影响正常的代码编译

### 测试验收

```rust
// tests/phase0_tests.rs

#[test]
fn test_simple_query() {
    let code = r#"
        User::where_query!("email = $1")
    "#;

    let parser = SimpleSqlParser::new(vec!["email".into(), "id".into()]);
    let cols = parser.extract_index_columns(code);

    assert_eq!(cols, vec!["email"]);
}

#[test]
fn test_where_and_order() {
    let code = r#"
        User::where_query!("status = $1 AND created_at > $2 ORDER BY created_at DESC")
    "#;

    let parser = SimpleSqlParser::new(vec![
        "id".into(), "email".into(), "status".into(), "created_at".into()
    ]);
    let cols = parser.extract_index_columns(code);

    assert_eq!(cols, vec!["status", "created_at"]);
}
```

### 文档验收

- [ ] API文档完整
- [ ] 使用示例清晰
- [ ] 已知限制说明

---

## ⚠️ 已知限制

1. **仅支持静态SQL字符串**
   - ❌ 不支持动态拼接的SQL
   - ❌ 不支持条件构建的SQL

2. **解析功能有限**
   - ⚠️ 简单的字符串匹配，不是完整的SQL解析
   - ⚠️ 可能误判复杂查询

3. **没有运行时验证**
   - ⚠️ 推荐的索引未经过实际查询验证
   - ⚠️ 无法分析查询频率

---

## 🎯 后续计划（Phase 1+）

完成Phase 0后，可以基于此扩展：

### Phase 1: 运行时收集（+4周）

- 运行时收集查询
- 分析查询频率
- 生成统计报告

### Phase 2: 智能推断（+4周）

- 更复杂的推断规则
- 索引优化
- 去重合并

### Phase 3: 自动应用（+2周）

- 自动生成迁移
- 集成到测试
- CI/CD支持

---

## 💰 投入产出

### 投入

- **开发时间**: 2周（10个工作日）
- **人力**: 1个工程师
- **复杂度**: ⭐⭐ (简单)

### 产出

- ✅ 编译时索引推荐
- ✅ 零运行时开销
- ✅ 立即可用的功能
- ✅ 后续扩展的基础

### 价值

- 🚀 **快速验证**: 2周即可展示价值
- 📈 **用户反馈**: 快速获得用户反馈
- 🎓 **学习曲线**: 团队容易理解
- 💡 **创新点**: 编译期分析是独特优势

---

## 📞 下一步

### 立即开始

如果你想实施Phase 0，我可以立即开始：

1. ✅ 创建基础文件结构
2. ✅ 实现简化SQL解析器
3. ✅ 实现查询提取器
4. ✅ 实现编译期分析宏
5. ✅ 编写测试用例
6. ✅ 提供使用示例

### 预期进度

- **Day 1-3**: 核心解析功能
- **Day 4-6**: 宏实现
- **Day 7-8**: 测试
- **Day 9-10**: 文档和示例

**2周后**，你将拥有一个可用的编译期索引分析工具！

需要我开始实现吗？
