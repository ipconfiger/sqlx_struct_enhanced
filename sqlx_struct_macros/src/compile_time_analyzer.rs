// 编译期查询分析器
//
// 提供编译时的索引分析和推荐功能

use proc_macro::TokenStream;
use crate::query_extractor::{QueryExtractor, ExtractedQuery};
use crate::simple_parser::SimpleSqlParser;
use crate::parser::{SqlParser, SqlDialect, IndexSyntax};
use std::collections::{HashMap, HashSet};

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

    // 分析并打印推荐
    print_recommendations(&queries);

    // 返回原代码，不做修改
    input
}

/// 打印索引推荐
fn print_recommendations(queries: &[ExtractedQuery]) {
    println!();
    println!("🔍 ======================================================");
    println!("🔍   SQLx Struct - Index Recommendations");
    println!("🔍 ======================================================");
    println!();

    // Phase C: 检测当前数据库方言
    let dialect = detect_dialect();
    let mysql_version = detect_mysql_version();
    let syntax = IndexSyntax::for_dialect(dialect);

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

                        // 生成 SQL 语句（根据数据库方言）
                        match dialect {
                            SqlDialect::Postgres => {
                                if !include_columns.is_empty() {
                                    // 覆盖索引 SQL
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) INCLUDE ({})",
                                        index_name, table_name, index_cols.join(", "), include_columns.join(", "));
                                } else if let Some(ref condition) = partial_condition {
                                    // 部分索引 SQL
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) WHERE {}",
                                        index_name, table_name, index_cols.join(", "), condition);
                                } else {
                                    // 普通索引 SQL
                                    println!("      SQL:    CREATE INDEX {} ON {} ({})",
                                        index_name, table_name, index_cols.join(", "));
                                }
                            },
                            SqlDialect::MySQL => {
                                // MySQL 8.0+支持INCLUDE，5.7不支持
                                let supports_include = mysql_version == Some(8);

                                if !include_columns.is_empty() && supports_include {
                                    // MySQL 8.0+ 覆盖索引
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) INCLUDE ({})",
                                        index_name, table_name, index_cols.join(", "), include_columns.join(", "));
                                } else if !include_columns.is_empty() && !supports_include {
                                    // MySQL 5.7：提示升级
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) -- INCLUDE requires MySQL 8.0+ (consider including: {})",
                                        index_name, table_name, index_cols.join(", "), include_columns.join(", "));
                                } else if let Some(ref _condition) = partial_condition {
                                    // MySQL不支持部分索引，添加注释
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) -- Note: Partial indexes not supported, consider filtering in WHERE clause",
                                        index_name, table_name, index_cols.join(", "));
                                } else {
                                    // 普通索引 SQL
                                    println!("      SQL:    CREATE INDEX {} ON {} ({})",
                                        index_name, table_name, index_cols.join(", "));
                                }
                            },
                            SqlDialect::SQLite => {
                                // SQLite不支持INCLUDE，但支持部分索引
                                if !include_columns.is_empty() {
                                    // SQLite不支持INCLUDE，添加注释
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) -- Note: INCLUDE not supported, consider adding these columns to the index",
                                        index_name, table_name, index_cols.join(", "));
                                } else if let Some(ref condition) = partial_condition {
                                    // SQLite支持部分索引
                                    println!("      SQL:    CREATE INDEX {} ON {} ({}) WHERE {}",
                                        index_name, table_name, index_cols.join(", "), condition);
                                } else {
                                    // 普通索引 SQL
                                    println!("      SQL:    CREATE INDEX {} ON {} ({})",
                                        index_name, table_name, index_cols.join(", "));
                                }
                            }
                        }
                        println!();
                    }
                }
            }

            // 生成 JOIN 索引推荐
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

                                // 检查是否是当前表的列
                                if is_current_table_column(table_alias, &query.sql) {
                                    let index_key = format!("JOIN_{}", join_col);
                                    if !seen_indexes.contains(&index_key) {
                                        seen_indexes.insert(index_key.clone());

                                        let index_name = format!("idx_{}_{}_join", table_name, column);
                                        println!("   ✨ Recommended: {}", index_name);
                                        println!("      Columns: {}", column);
                                        println!("      Reason: JOIN column ({} ON {})", join.join_type, condition);
                                        println!("      SQL:    CREATE INDEX {} ON {} ({})",
                                            index_name, table_name, column);
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
                        let index_key = format!("GROUP_BY_{}", column);

                        if !seen_indexes.contains(&index_key) {
                            seen_indexes.insert(index_key.clone());

                            let index_name = format!("idx_{}_{}_group", table_name, column);
                            println!("   ✨ Recommended: {}", index_name);
                            println!("      Columns: {}", column);
                            println!("      Reason: GROUP BY column{}", if group_by_info.has_having() {
                                format!(" with HAVING clause")
                            } else {
                                String::new()
                            });
                            println!("      SQL:    CREATE INDEX {} ON {} ({})",
                                index_name, table_name, column);
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

                            println!("   ✨ Recommended: {} (Subquery)", index_name);
                            println!("      Type: {} Subquery", subquery_type_name);
                            println!("      Columns: {}", subquery.columns.join(", "));
                            println!("      Reason: Index columns in subquery for better performance");
                            println!("      SQL:    CREATE INDEX {} ON {} ({})",
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
/// 通过检查 FROM 子句中的表别名
fn is_current_table_column(table_alias: &str, sql: &str) -> bool {
    let sql_lower = sql.to_lowercase();

    // 查找 FROM 子句
    if let Some(from_pos) = sql_lower.find("from") {
        let after_from = &sql[from_pos + 4..];

        // 提取 FROM 到第一个 JOIN 或 WHERE 之间的内容
        let from_clause = extract_until_keywords(after_from, &["join", "where", "group", "order", "limit"]);

        // 检查表别名是否在 FROM 子句中
        from_clause.contains(table_alias)
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
