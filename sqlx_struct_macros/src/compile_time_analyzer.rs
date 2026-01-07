// 编译期查询分析器
//
// 提供编译时的索引分析和推荐功能

use proc_macro::TokenStream;
use crate::query_extractor::{QueryExtractor, ExtractedQuery};
use crate::simple_parser::SimpleSqlParser;
use std::collections::{HashMap, HashSet};

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
            if query.table_fields.is_empty() {
                continue;
            }

            let parser = SimpleSqlParser::new(query.table_fields.clone());
            let index_cols = parser.extract_index_columns(&query.sql);

            if index_cols.is_empty() {
                continue;
            }

            let index_key = format!("{:?}", index_cols);

            if !seen_indexes.contains(&index_key) {
                seen_indexes.insert(index_key);

                let index_name = format!("idx_{}_{}", table_name, index_cols.join("_"));

                println!("   ✨ Recommended: {}", index_name);
                println!("      Columns: {}", index_cols.join(", "));
                println!("      Reason: {}", explain_reason(&index_cols, query));
                println!("      SQL:    CREATE INDEX {} ON {} ({})",
                    index_name, table_name, index_cols.join(", "));
                println!();
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
