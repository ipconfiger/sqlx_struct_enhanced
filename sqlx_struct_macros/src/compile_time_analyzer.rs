// 编译期查询分析器
//
// 提供编译时的索引分析和推荐功能

use proc_macro::TokenStream;
use crate::query_extractor::{QueryExtractor, ExtractedQuery};
use crate::simple_parser::SimpleSqlParser;
use crate::parser::{SqlParser, SqlDialect, IndexSyntax};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Maps table aliases to actual table names
struct TableAliasMap {
    aliases: HashMap<String, String>,
}

impl TableAliasMap {
    fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    fn add_alias(&mut self, alias: String, table: String) {
        self.aliases.insert(alias, table);
    }

    /// Resolve an alias or table reference to the actual table name
    fn resolve(&self, alias_or_table: &str) -> String {
        if let Some(table) = self.aliases.get(alias_or_table) {
            table.clone()
        } else {
            alias_or_table.to_string()
        }
    }
}

/// Extract table name and alias mappings from FROM and JOIN clauses
/// Recursively extracts aliases from ALL levels of queries, including nested subqueries
fn extract_table_aliases(sql: &str) -> TableAliasMap {
    let mut map = TableAliasMap::new();
    let sql_lower = sql.to_lowercase();

    // Extract FROM clause (main query)
    if let Some(from_pos) = sql_lower.find("from") {
        let from_end = find_from_end(&sql_lower[from_pos..]);
        if from_end > 0 {
            let from_clause = &sql[from_pos + 4..from_pos + from_end];
            parse_table_clause(from_clause, &mut map);
        }
    }

    // Extract JOIN clauses (main query)
    let join_keywords = ["inner join", "left join", "right join", "join"];
    for keyword in &join_keywords {
        let mut search_start = 0;
        while let Some(join_pos) = sql_lower[search_start..].find(keyword) {
            let actual_pos = search_start + join_pos;
            let keyword_len = keyword.len();

            let join_start = actual_pos + keyword_len;
            let join_end = find_join_end(&sql_lower[join_start..]);
            if join_end > 0 {
                let join_clause = &sql[join_start..join_start + join_end];
                parse_table_clause(join_clause, &mut map);
            }

            search_start = actual_pos + keyword_len;
        }
    }

    // Recursively extract aliases FROM SUBQUERIES
    let (_, subqueries) = extract_subqueries_from_sql(sql);
    for subquery_sql in subqueries {
        let subquery_aliases = extract_table_aliases(&subquery_sql);
        for (alias, table) in subquery_aliases.aliases.iter() {
            map.add_alias(alias.clone(), table.clone());
        }
    }

    map
}

/// Find the end of a FROM clause
fn find_from_end(clause: &str) -> usize {
    let keywords = ["where", "order by", "group by", "having", "limit"];
    let mut min_pos = clause.len();

    for keyword in &keywords {
        if let Some(pos) = clause.find(keyword) {
            min_pos = min_pos.min(pos);
        }
    }

    min_pos
}

/// Find the end of a JOIN clause
fn find_join_end(clause: &str) -> usize {
    let keywords = ["where", "order by", "group by", "inner join", "left join", "right join", "join"];
    let mut min_pos = clause.len();

    for keyword in &keywords {
        if let Some(pos) = clause.find(keyword) {
            min_pos = min_pos.min(pos);
        }
    }

    min_pos
}

/// Parse a table clause (FROM or JOIN) to extract table name and alias
fn parse_table_clause(clause: &str, map: &mut TableAliasMap) {
    let trimmed = clause.trim();

    if trimmed.is_empty() {
        return;
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    let table_name = parts[0].trim();

    if parts.len() == 1 {
        map.add_alias(table_name.to_string(), table_name.to_string());
    } else if parts.len() >= 2 {
        let second = parts[1].trim().to_uppercase();

        if second == "AS" && parts.len() >= 3 {
            let alias = parts[2].trim();
            map.add_alias(alias.to_string(), table_name.to_string());
        } else if second != "ON" && second != "WHERE" && second != "," {
            map.add_alias(second.to_lowercase(), table_name.to_string());
        } else {
            map.add_alias(table_name.to_string(), table_name.to_string());
        }
    }
}

/// Remove subqueries from SQL and return both cleaned SQL and list of subqueries
fn extract_subqueries_from_sql(sql: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut subqueries = Vec::new();
    let mut depth = 0;
    let mut in_subquery = false;
    let mut subquery_start = 0;

    for (i, c) in sql.chars().enumerate() {
        if c == '(' {
            depth += 1;
            if depth == 1 && !in_subquery {
                let after_paren = &sql[i+1..].to_uppercase();
                if after_paren.trim().starts_with("SELECT") {
                    in_subquery = true;
                    subquery_start = i + 1;
                    continue;
                }
            }
        } else if c == ')' {
            if depth > 0 {
                depth -= 1;
                if in_subquery && depth == 0 {
                    in_subquery = false;
                    let subquery_sql = &sql[subquery_start..i].trim();
                    subqueries.push(subquery_sql.to_string());
                    result.push_str("($1)");
                    continue;
                }
            }
        }

        if !in_subquery {
            result.push(c);
        }
    }

    (result, subqueries)
}

/// 索引信息
#[derive(Debug, Clone)]
struct IndexInfo {
    name: String,
    table_name: String,
    columns: Vec<String>,
    include_columns: Vec<String>,
    partial_condition: Option<String>,
    reason: String,
}

impl IndexInfo {
    /// 生成 CREATE INDEX 语句
    fn to_create_sql(&self, dialect: SqlDialect) -> String {
        let columns_str = self.columns.join(", ");
        let mut sql = format!("CREATE INDEX IF NOT EXISTS {} ON {} ({})",
            self.name, self.table_name, columns_str);

        // 处理 INCLUDE 子句（PostgreSQL 和 MySQL 8.0+）
        if !self.include_columns.is_empty() {
            match dialect {
                SqlDialect::Postgres => {
                    sql.push_str(&format!(" INCLUDE ({})",
                        self.include_columns.join(", ")));
                }
                SqlDialect::MySQL => {
                    sql.push_str(&format!(" INCLUDE ({})",
                        self.include_columns.join(", ")));
                }
                SqlDialect::SQLite => {
                    // SQLite 不支持 INCLUDE，添加注释
                    sql.push_str(&format!(" -- INCLUDE not supported (consider adding: {})",
                        self.include_columns.join(", ")));
                }
            }
        }

        // 处理部分索引的 WHERE 子句
        if let Some(ref condition) = self.partial_condition {
            match dialect {
                SqlDialect::Postgres | SqlDialect::SQLite => {
                    sql.push_str(&format!(" WHERE {}", condition));
                }
                SqlDialect::MySQL => {
                    // MySQL 不支持部分索引，添加注释
                    sql.push_str(&format!(" -- Partial indexes not supported (WHERE {})",
                        condition));
                }
            }
        }

        sql
    }

    /// 生成 DROP INDEX 语句
    fn to_drop_sql(&self, dialect: SqlDialect) -> String {
        match dialect {
            SqlDialect::Postgres => {
                format!("DROP INDEX IF EXISTS {}", self.name)
            }
            SqlDialect::MySQL => {
                format!("DROP INDEX IF EXISTS {} ON {}", self.name, self.table_name)
            }
            SqlDialect::SQLite => {
                format!("DROP INDEX IF EXISTS {}", self.name)
            }
        }
    }
}

/// 检测当前启用的数据库方言
///
/// 通过编译时的 feature flags 检测当前使用的数据库
fn detect_dialect() -> SqlDialect {
    // 按优先级检查 feature flags
    // PostgreSQL 优先级最高
    #[cfg(feature = "postgres")]
    {
        return SqlDialect::Postgres;
    }

    #[cfg(all(feature = "mysql", not(feature = "postgres")))]
    {
        return SqlDialect::MySQL;
    }

    #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
    {
        return SqlDialect::SQLite;
    }

    // 默认使用 PostgreSQL
    #[cfg(not(any(feature = "postgres", feature = "mysql", feature = "sqlite")))]
    {
        SqlDialect::Postgres
    }
}

/// 检测MySQL版本（仅在MySQL feature启用时有效）
///
/// 返回值:
/// - Some(8) 表示MySQL 8.0+，支持INCLUDE索引
/// - Some(5) 表示MySQL 5.x，不支持INCLUDE索引
/// - None 表示非MySQL数据库
///
/// 默认假设MySQL 8.0+，可以通过feature flag `mysql_5_7`指定5.7版本
fn detect_mysql_version() -> Option<u8> {
    #[cfg(feature = "mysql")]
    {
        // 检查是否明确指定了5.7版本
        #[cfg(feature = "mysql_5_7")]
        return Some(5);

        // 默认假设MySQL 8.0+
        #[cfg(not(feature = "mysql_5_7"))]
        return Some(8);
    }

    #[cfg(not(feature = "mysql"))]
    None
}

/// 转换参数占位符为数据库特定的语法
///
/// PostgreSQL: $1, $2, $3
/// MySQL/SQLite: ?
#[allow(dead_code)]
fn convert_placeholder(sql: &str, dialect: SqlDialect) -> String {
    match dialect {
        SqlDialect::Postgres => sql.to_string(),
        SqlDialect::MySQL | SqlDialect::SQLite => {
            // 将 $1, $2, $3 等替换为 ?
            // 这是一个简化的实现，假设参数占位符格式为 $<number>
            let mut result = sql.to_string();

            // 查找所有 $<number> 格式的占位符
            while let Some(pos) = result.find('$') {
                // 检查 $ 后面是否是数字
                if let Some(next_char) = result.chars().nth(pos + 1) {
                    if next_char.is_ascii_digit() {
                        result.remove(pos); // 移除 $
                        // 移除数字
                        while result.chars().nth(pos).map_or(false, |c| c.is_ascii_digit()) {
                            result.remove(pos);
                        }
                        result.insert_str(pos, "?");
                    }
                }
            }

            result
        }
    }
}

/// 编译期查询分析宏
///
/// 使用方式:
/// ```ignore
/// #[sqlx_struct_macros::analyze_queries]
/// mod my_module {
///     // 你的查询代码...
/// }
/// ```
pub fn analyze_queries(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_str = input.to_string();

    // 创建查询提取器
    let mut extractor = QueryExtractor::new();
    let queries = extractor.extract_from_code(&input_str);

    // 如果没有找到查询，直接返回原代码
    if queries.is_empty() {
        return input;
    }

    // 分析、打印并保存推荐
    print_and_save_recommendations(&queries);

    // 返回原代码，不做修改
    input
}

/// 打印索引推荐并保存到 SQL 文件
fn print_and_save_recommendations(queries: &[ExtractedQuery]) {
    println!();
    println!("🔍 ======================================================");
    println!("🔍   SQLx Struct - Index Recommendations");
    println!("🔍 ======================================================");
    println!();

    // Phase C: 检测当前数据库方言
    let dialect = detect_dialect();
    let mysql_version = detect_mysql_version();
    let syntax = IndexSyntax::for_dialect(dialect);

    // 收集所有索引推荐用于保存到文件
    let mut all_indexes: Vec<IndexInfo> = Vec::new();

    // 显示当前数据库方言
    println!("🗄️  Database: {}", format!("{:?}", dialect));

    // 对于MySQL，显示版本信息
    if dialect == SqlDialect::MySQL {
        if let Some(version) = mysql_version {
            println!("   - MySQL Version: {}.x", version);
            println!("   - INCLUDE indexes: {}",
                if version >= 8 { "✅ Supported (MySQL 8.0+)" } else { "❌ Not supported (requires 8.0+)" });
        }
    } else {
        println!("   - INCLUDE indexes: {}", if syntax.include_supported { "✅ Supported" } else { "❌ Not supported" });
    }

    println!("   - Partial indexes: {}", if syntax.partial_supported { "✅ Supported" } else { "❌ Not supported" });
    println!();

    // 按表名分组
    let mut by_table: HashMap<String, Vec<&ExtractedQuery>> = HashMap::new();

    for query in queries {
        by_table
            .entry(query.table_name.clone())
            .or_insert_with(Vec::new)
            .push(query);
    }

    // 为每个表生成推荐
    for (table_name, table_queries) in &by_table {
        println!("📊 Table: {}", table_name);
        println!();

        // 去重并分析
        let mut seen_indexes = HashSet::new();

        for query in table_queries {
            // 使用检测到的方言来解析 JOIN 和 GROUP BY
            let sql_parser = SqlParser::new(dialect);
            let joins = sql_parser.extract_joins(&query.sql);
            let group_by = sql_parser.extract_group_by(&query.sql);

            // 生成 WHERE/ORDER BY 索引推荐 (仅在有表字段时)
            if !query.table_fields.is_empty() {
                let simple_parser = SimpleSqlParser::new(query.table_fields.clone());
                let index_cols = simple_parser.extract_index_columns(&query.sql);

                if !index_cols.is_empty() {
                    let index_key = format!("{:?}", index_cols);

                    if !seen_indexes.contains(&index_key) {
                        seen_indexes.insert(index_key.clone());

                        let index_name = format!("idx_{}_{}", table_name, index_cols.join("_"));

                        // Phase B.4: 检测覆盖索引 (INCLUDE)
                        let include_columns = simple_parser.detect_include_columns(&query.sql, &index_cols);

                        // Phase B.5: 检测部分索引
                        let is_partial = simple_parser.should_be_partial_index(&query.sql);
                        let partial_condition = if is_partial {
                            simple_parser.extract_partial_condition(&query.sql)
                        } else {
                            None
                        };

                        println!("   ✨ Recommended: {}", index_name);
                        println!("      Columns: {}", index_cols.join(", "));

                        // 显示覆盖索引信息
                        if !include_columns.is_empty() {
                            println!("      INCLUDE: {}", include_columns.join(", "));
                        }

                        // 显示部分索引信息
                        if let Some(ref condition) = partial_condition {
                            println!("      WHERE: {}", condition);
                            println!("      Type: Partial Index");
                        }

                        println!("      Reason: {}", explain_reason(&index_cols, query));

                        // 收集索引信息用于保存
                        all_indexes.push(IndexInfo {
                            name: index_name.clone(),
                            table_name: table_name.clone(),
                            columns: index_cols.clone(),
                            include_columns: include_columns.clone(),
                            partial_condition: partial_condition.clone(),
                            reason: explain_reason(&index_cols, query),
                        });

                        // 生成 SQL 语句（根据数据库方言，使用 IF NOT EXISTS）
                        match dialect {
                            SqlDialect::Postgres => {
                                if !include_columns.is_empty() {
                                    // 覆盖索引 SQL
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) INCLUDE ({})",
                                        index_name, table_name, index_cols.join(", "), include_columns.join(", "));
                                } else if let Some(ref condition) = partial_condition {
                                    // 部分索引 SQL
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) WHERE {}",
                                        index_name, table_name, index_cols.join(", "), condition);
                                } else {
                                    // 普通索引 SQL
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                        index_name, table_name, index_cols.join(", "));
                                }
                            },
                            SqlDialect::MySQL => {
                                // MySQL 8.0+支持INCLUDE，5.7不支持
                                let supports_include = mysql_version == Some(8);

                                if !include_columns.is_empty() && supports_include {
                                    // MySQL 8.0+ 覆盖索引
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) INCLUDE ({})",
                                        index_name, table_name, index_cols.join(", "), include_columns.join(", "));
                                } else if !include_columns.is_empty() && !supports_include {
                                    // MySQL 5.7：提示升级
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) -- INCLUDE requires MySQL 8.0+ (consider including: {})",
                                        index_name, table_name, index_cols.join(", "), include_columns.join(", "));
                                } else if let Some(ref _condition) = partial_condition {
                                    // MySQL不支持部分索引，添加注释
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) -- Note: Partial indexes not supported, consider filtering in WHERE clause",
                                        index_name, table_name, index_cols.join(", "));
                                } else {
                                    // 普通索引 SQL
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                        index_name, table_name, index_cols.join(", "));
                                }
                            },
                            SqlDialect::SQLite => {
                                // SQLite不支持INCLUDE，但支持部分索引
                                if !include_columns.is_empty() {
                                    // SQLite不支持INCLUDE，添加注释
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) -- Note: INCLUDE not supported, consider adding these columns to the index",
                                        index_name, table_name, index_cols.join(", "));
                                } else if let Some(ref condition) = partial_condition {
                                    // SQLite支持部分索引
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({}) WHERE {}",
                                        index_name, table_name, index_cols.join(", "), condition);
                                } else {
                                    // 普通索引 SQL
                                    println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                        index_name, table_name, index_cols.join(", "));
                                }
                            }
                        }
                        println!();
                    }
                }
            }

            // 生成 JOIN 索引推荐
            // 首先提取所有表别名（包括子查询中的别名）
            let aliases = extract_table_aliases(&query.sql);

            for join in &joins {
                if let Some(condition) = join.first_condition() {
                    // 从 JOIN 条件中提取列名
                    let join_columns = extract_columns_from_condition(condition);

                    for join_col in join_columns {
                        // 只推荐主表上的索引
                        if join_col.contains('.') {
                            let parts: Vec<&str> = join_col.split('.').collect();
                            if parts.len() == 2 {
                                let table_alias = parts[0];
                                let column = parts[1];

                                // 解析别名为实际表名
                                let resolved_table = aliases.resolve(table_alias);

                                // 检查是否是当前表的列
                                if is_current_table_column(table_alias, &query.sql) {
                                    let index_key = format!("JOIN_{}", join_col);
                                    if !seen_indexes.contains(&index_key) {
                                        seen_indexes.insert(index_key.clone());

                                        // 使用解析后的表名来生成索引名
                                        let index_name = format!("idx_{}_{}_join", resolved_table, column);

                                        // 收集索引信息
                                        all_indexes.push(IndexInfo {
                                            name: index_name.clone(),
                                            table_name: resolved_table.clone(),
                                            columns: vec![column.to_string()],
                                            include_columns: vec![],
                                            partial_condition: None,
                                            reason: format!("JOIN column ({} ON {})", join.join_type, condition),
                                        });

                                        println!("   ✨ Recommended: {}", index_name);
                                        println!("      Table: {}", resolved_table);
                                        println!("      Columns: {}", column);
                                        println!("      Reason: JOIN column ({} ON {})", join.join_type, condition);
                                        println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                            index_name, resolved_table, column);
                                        println!();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 生成 GROUP BY 索引推荐
            if let Some(group_by_info) = &group_by {
                if group_by_info.has_columns() {
                    for column in &group_by_info.columns {
                        // Handle qualified column names (e.g., "m1.merchant_id" -> "merchant_id")
                        let (column_name, resolved_table) = if column.contains('.') {
                            let parts: Vec<&str> = column.split('.').collect();
                            if parts.len() == 2 {
                                let table_alias = parts[0];
                                let col = parts[1];
                                // Resolve the alias to actual table name
                                let resolved = aliases.resolve(table_alias);
                                (col.to_string(), resolved)
                            } else {
                                (column.clone(), table_name.clone())
                            }
                        } else {
                            (column.clone(), table_name.clone())
                        };

                        let index_key = format!("GROUP_BY_{}_{}", resolved_table, column_name);

                        if !seen_indexes.contains(&index_key) {
                            seen_indexes.insert(index_key.clone());

                            let index_name = format!("idx_{}_{}_group", resolved_table, column_name);

                            // 收集索引信息
                            all_indexes.push(IndexInfo {
                                name: index_name.clone(),
                                table_name: resolved_table.clone(),
                                columns: vec![column_name.clone()],
                                include_columns: vec![],
                                partial_condition: None,
                                reason: format!("GROUP BY column{}", if group_by_info.has_having() {
                                    " with HAVING clause"
                                } else {
                                    ""
                                }),
                            });

                            println!("   ✨ Recommended: {}", index_name);
                            println!("      Table: {}", resolved_table);
                            println!("      Columns: {}", column_name);
                            println!("      Reason: GROUP BY column{}", if group_by_info.has_having() {
                                format!(" with HAVING clause")
                            } else {
                                String::new()
                            });
                            println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                index_name, resolved_table, column_name);
                            println!();
                        }
                    }
                }
            }

            // Phase B.3: 生成子查询索引推荐
            if !query.table_fields.is_empty() {
                let simple_parser = SimpleSqlParser::new(query.table_fields.clone());
                let subqueries = simple_parser.extract_subqueries(&query.sql);

                for subquery in &subqueries {
                    if !subquery.columns.is_empty() {
                        // 为子查询生成唯一标识
                        let subquery_key = format!("SUBQUERY_{:?}_{:?}", subquery.subquery_type, subquery.columns);

                        if !seen_indexes.contains(&subquery_key) {
                            seen_indexes.insert(subquery_key.clone());

                            let subquery_type_name = format!("{:?}", subquery.subquery_type);
                            let index_name = format!("idx_{}_subquery_{}", table_name, subquery.columns.join("_"));

                            // 收集索引信息
                            all_indexes.push(IndexInfo {
                                name: index_name.clone(),
                                table_name: table_name.clone(),
                                columns: subquery.columns.clone(),
                                include_columns: vec![],
                                partial_condition: None,
                                reason: "Index columns in subquery for better performance".to_string(),
                            });

                            println!("   ✨ Recommended: {} (Subquery)", index_name);
                            println!("      Type: {} Subquery", subquery_type_name);
                            println!("      Columns: {}", subquery.columns.join(", "));
                            println!("      Reason: Index columns in subquery for better performance");
                            println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                index_name, table_name, subquery.columns.join(", "));

                            // 显示子查询的 SQL（格式化后）
                            let formatted_sql = subquery.sql.chars().take(80).collect::<String>();
                            println!("      Subquery: {}...", formatted_sql);
                            println!();
                        }
                    }
                }
            }
        }
    }

    println!("🔍 ======================================================");
    println!("🔍   End of Recommendations");
    println!("🔍 ======================================================");
    println!();

    // 保存索引推荐到 SQL 文件
    save_indexes_to_file(&all_indexes, dialect);
}

/// 保存索引推荐到 SQL 文件
fn save_indexes_to_file(indexes: &[IndexInfo], dialect: SqlDialect) {
    if indexes.is_empty() {
        return;
    }

    // 创建输出目录
    let output_dir = Path::new("target/sqlx_struct_indexes");
    if let Err(e) = fs::create_dir_all(output_dir) {
        println!("   ⚠️  Warning: Could not create output directory: {}", e);
        return;
    }

    // 根据数据库方言确定文件名
    let db_name = match dialect {
        SqlDialect::Postgres => "postgres",
        SqlDialect::MySQL => "mysql",
        SqlDialect::SQLite => "sqlite",
    };

    // 生成 CREATE INDEX 文件
    let create_file = output_dir.join(format!("indexes_{}.sql", db_name));
    let create_content = generate_create_indexes_sql(indexes, dialect);
    if let Err(e) = fs::write(&create_file, create_content) {
        println!("   ⚠️  Warning: Could not write CREATE INDEX file: {}", e);
    } else {
        println!("   💾 Saved: {}", create_file.display());
    }

    // 生成 DROP INDEX 文件（用于回滚）
    let drop_file = output_dir.join(format!("drop_indexes_{}.sql", db_name));
    let drop_content = generate_drop_indexes_sql(indexes, dialect);
    if let Err(e) = fs::write(&drop_file, drop_content) {
        println!("   ⚠️  Warning: Could not write DROP INDEX file: {}", e);
    } else {
        println!("   💾 Saved: {}", drop_file.display());
    }
}

/// 生成 CREATE INDEX SQL 文件内容
fn generate_create_indexes_sql(indexes: &[IndexInfo], dialect: SqlDialect) -> String {
    let mut content = String::new();

    content.push_str("-- Auto-generated by sqlx_struct_enhanced\n");
    content.push_str(&format!("-- Database: {:?}\n", dialect));
    content.push_str("-- This file contains recommended indexes based on query analysis\n");
    content.push_str("-- Generated at: ");
    // Note: We can't use chrono here to avoid extra dependencies
    content.push_str("[compile time]\n");
    content.push_str("\n");
    content.push_str("-- Usage: Run this file in your database to create recommended indexes\n");
    content.push_str("-- Example: psql -U username -d database -f indexes_postgres.sql\n");
    content.push_str("\n");
    content.push_str("BEGIN;\n\n");

    for index in indexes {
        content.push_str("-- Index: ");
        content.push_str(&index.name);
        content.push_str("\n");
        content.push_str("-- Table: ");
        content.push_str(&index.table_name);
        content.push_str("\n");
        content.push_str("-- Reason: ");
        content.push_str(&index.reason);
        content.push_str("\n");
        content.push_str(&index.to_create_sql(dialect));
        content.push_str(";\n\n");
    }

    content.push_str("COMMIT;\n");

    content
}

/// 生成 DROP INDEX SQL 文件内容（用于回滚）
fn generate_drop_indexes_sql(indexes: &[IndexInfo], dialect: SqlDialect) -> String {
    let mut content = String::new();

    content.push_str("-- Auto-generated rollback script for sqlx_struct_enhanced\n");
    content.push_str(&format!("-- Database: {:?}\n", dialect));
    content.push_str("-- This file will DROP all indexes created by the migration\n");
    content.push_str("-- ⚠️  WARNING: Use with caution!\n");
    content.push_str("\n");
    content.push_str("-- Usage: Run this file to rollback the indexes\n");
    content.push_str("-- Example: psql -U username -d database -f drop_indexes_postgres.sql\n");
    content.push_str("\n");
    content.push_str("BEGIN;\n\n");

    // 反向顺序删除（先删除最后创建的索引）
    for index in indexes.iter().rev() {
        content.push_str("-- Drop index: ");
        content.push_str(&index.name);
        content.push_str("\n");
        content.push_str(&index.to_drop_sql(dialect));
        content.push_str(";\n\n");
    }

    content.push_str("COMMIT;\n");

    content
}

/// 解释推荐原因
fn explain_reason(columns: &[String], _query: &ExtractedQuery) -> String {
    if columns.len() == 1 {
        format!("Single column: WHERE {} = $1", columns[0])
    } else if columns.len() == 2 {
        // 可能是 WHERE + ORDER BY 或两个 WHERE
        let order_col = &columns[1];
        format!("WHERE {} ORDER BY {}", columns[0], order_col)
    } else {
        format!("Multi-column: {}", columns.join(" AND "))
    }
}

/// 从 JOIN 条件中提取列名
/// 例如: "o.user_id = u.id" -> ["o.user_id", "u.id"]
fn extract_columns_from_condition(condition: &str) -> Vec<String> {
    condition
        .split(&['=', '&', '|'][..])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.contains('('))  // 排除函数调用
        .map(|s| {
            // 移除运算符周围的空格和比较符
            s.split_whitespace()
                .next()
                .unwrap_or(s)
                .to_string()
        })
        .collect()
}

/// 检查列是否属于当前表
/// 使用别名解析来检查列是否属于当前表
fn is_current_table_column(table_alias: &str, sql: &str) -> bool {
    let aliases = extract_table_aliases(sql);
    let resolved_table = aliases.resolve(table_alias);

    // 检查解析后的表名是否是主表（检查 FROM 子句）
    let sql_lower = sql.to_lowercase();

    if let Some(from_pos) = sql_lower.find("from") {
        let after_from = &sql[from_pos + 4..];
        let from_clause = extract_until_keywords(after_from, &["join", "where", "group", "order", "limit"]);
        from_clause.contains(&resolved_table)
    } else {
        false
    }
}

/// 提取文本直到遇到指定关键字
fn extract_until_keywords(text: &str, keywords: &[&str]) -> String {
    let mut result = text.to_string();
    let text_lower = text.to_lowercase();

    for keyword in keywords {
        if let Some(pos) = text_lower.find(keyword) {
            result = text[..pos].to_string();
            break;
        }
    }

    result.trim().to_string()
}
