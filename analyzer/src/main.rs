use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

// Import JOIN analysis types and functions
// The test module provides public implementations that we use in main code
mod join_analysis_tests;
use join_analysis_tests::{ColumnExtractionResult, extract_table_aliases, analyze_join_query_columns};

/// Quote an identifier for PostgreSQL (double quotes)
/// This handles reserved keywords like "channel", "key", "user", etc.
fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier)
}

/// Converts a PascalCase or camelCase string to snake_case.
///
/// # Example
///
/// ```ignore
/// to_snake_case("MyTable");  // "my_table"
/// to_snake_case("userProfile");  // "user_profile"
/// ```
fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);

    for (i, c) in s.char_indices() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }

    result
}

/// Sanitize table name for index generation
/// Handles special cases like [Self] -> merchant
fn sanitize_table_name(table_name: &str, context_table: &str) -> String {
    // Handle [Self] syntax - replace with the context table name
    if table_name == "[Self]" || table_name == "[self]" {
        return to_snake_case(context_table);
    }

    // For other table names, just convert to snake_case
    // but preserve any special characters that might be valid
    table_name.to_string()
}

/// Extracted query from source code
#[derive(Debug, Clone)]
struct ExtractedQuery {
    table: String,
    query_type: String,
    sql: String,
    file: String,
    line: usize,
    is_from_subquery: bool,  // Track if this query was extracted from a subquery
}

/// Index recommendation with metadata for deduplication
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IndexRecommendation {
    table_name: String,
    columns: Vec<String>,
    source_file: String,
    source_line: usize,
    query_sql: String,
    reason: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let scan_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "src".to_string()
    };

    println!("🔍 SQLx Query Analyzer");
    println!("===================\n");
    println!("Scanning: {}\n", scan_path);

    // Collect all queries
    let mut all_queries = Vec::new();

    // Scan directory
    let path = Path::new(&scan_path);
    if path.is_dir() {
        scan_directory(path, &mut all_queries)?;
    } else if path.is_file() && path.extension().map_or(false, |e| e == "rs") {
        scan_file(path, &mut all_queries)?;
    } else {
        eprintln!("Error: Please specify a valid .rs file or directory");
        eprintln!("Usage: sqlx-analyze [path]");
        eprintln!("   or: sqlx-analyze (defaults to ./src)");
        std::process::exit(1);
    }

    if all_queries.is_empty() {
        println!("No queries found!");
        println!();
        println!("Make sure you're using:");
        println!("  User::where_query!(\"email = $1\")");
        println!("  User::count_query!(\"status = $1\")");
        println!("  etc.");
        return Ok(());
    }

    println!("Found {} queries\n", all_queries.len());

    // Group by table
    let mut by_table: HashMap<String, Vec<&ExtractedQuery>> = HashMap::new();
    for query in &all_queries {
        by_table
            .entry(query.table.clone())
            .or_insert_with(Vec::new)
            .push(query);
    }

    // Generate recommendations
    generate_index_recommendations(&by_table, &all_queries);

    // Save to file
    save_to_file(&all_queries, &scan_path)?;

    Ok(())
}

fn scan_directory(dir: &Path, queries: &mut Vec<ExtractedQuery>) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip target, node_modules, etc.
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if ["target", "node_modules", ".git", "dist"].contains(&file_name) {
                continue;
            }
            scan_directory(&path, queries)?;
        } else if path.extension().map_or(false, |e| e == "rs") {
            scan_file(&path, queries)?;
        }
    }

    Ok(())
}

fn scan_file(path: &Path, queries: &mut Vec<ExtractedQuery>) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let file_name = path.to_string_lossy().to_string();

    // Patterns to search for - support both macro (!) and runtime method (no !)
    let patterns: Vec<(&str, &str)> = vec![
        // Macro format: User::where_query!("...")
        (r#"(\w+)::where_query!\("([^"]+)"\)"#, "where_query"),
        (r#"(\w+)::count_query!\("([^"]+)"\)"#, "count_query"),
        (r#"(\w+)::delete_where_query!\("([^"]+)"\)"#, "delete_where_query"),
        (r#"(\w+)::make_query!\("([^"]+)"\)"#, "make_query"),
        (r#"(\w+)::make_execute!\("([^"]+)"\)"#, "make_execute"),
        // Runtime method format: User::where_query("...")
        (r#"(\w+)::where_query\("([^"]+)"\)"#, "where_query_runtime"),
        (r#"(\w+)::count_query\("([^"]+)"\)"#, "count_query_runtime"),
        (r#"(\w+)::delete_where_query\("([^"]+)"\)"#, "delete_where_query_runtime"),
        (r#"(\w+)::make_query\("([^"]+)"\)"#, "make_query_runtime"),
        (r#"(\w+)::make_execute\("([^"]+)"\)"#, "make_execute_runtime"),
    ];

    for (line_num, line) in content.lines().enumerate() {
        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        for (pattern, query_type) in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    let table = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                    let sql = caps.get(2).map(|m| m.as_str()).unwrap_or("").to_string();

                    // No longer skip JOIN queries - we can now analyze them!

                    if !table.is_empty() && !sql.is_empty() {
                        queries.push(ExtractedQuery {
                            table: table.clone(),
                            query_type: query_type.to_string(),
                            sql: sql.clone(),
                            file: file_name.clone(),
                            line: line_num + 1,
                            is_from_subquery: false,
                        });

                        // Extract subqueries and add them as separate queries
                        extract_subqueries(&sql, &table, query_type, &file_name, line_num + 1, queries);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Find the project root by looking for Cargo.toml
fn find_project_root(scan_path: &Path) -> PathBuf {
    // Start with the scan path or its parent if it's a file
    let mut current = if scan_path.is_dir() {
        scan_path.to_path_buf()
    } else {
        scan_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    // Walk up the directory tree looking for Cargo.toml
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return current;
        }

        // If we're at the root, return the scan path itself
        if !current.pop() {
            // No Cargo.toml found, use the original scan path (or its parent)
            return if scan_path.is_dir() {
                scan_path.to_path_buf()
            } else {
                scan_path.parent().unwrap_or(Path::new(".")).to_path_buf()
            };
        }
    }
}

fn generate_index_recommendations(
    by_table: &HashMap<String, Vec<&ExtractedQuery>>,
    _all_queries: &[ExtractedQuery],
) {
    println!();
    println!("🔍 ======================================================");
    println!("🔍   SQLx Struct - Index Recommendations");
    println!("🔍 ======================================================");
    println!();
    println!("🗄️  Database: Postgres");
    println!("   - INCLUDE indexes: ✅ Supported");
    println!("   - Partial indexes: ✅ Supported");
    println!();

    // Extract unique index requirements
    let mut seen_indexes = HashSet::new();

    for (table, queries) in by_table {
        println!("📊 Table: {}", table);
        println!();

        for query in queries {
            match extract_columns_from_sql(&query.sql) {
                Some(ColumnExtractionResult::SingleTable(cols)) => {
                    // Original logic for simple (non-JOIN) queries
                    let index_key = format!("{:?}_{:?}", table, cols);

                    if !seen_indexes.contains(&index_key) {
                        seen_indexes.insert(index_key.clone());

                        let table_snake = to_snake_case(&table);
                        let index_name = format!("idx_{}_{}", table_snake, cols.join("_"));
                        let quoted_table = quote_identifier(&table_snake);
                        let quoted_cols: Vec<String> = cols.iter().map(|c| quote_identifier(c)).collect();

                        println!("   ✨ Recommended: {}", index_name);
                        println!("      Columns: {}", cols.join(", "));
                        println!("      Source: {}:{}",
                            query.file.split('/').last().unwrap_or(&query.file),
                            query.line
                        );
                        println!("      Reason: {}", query.query_type);
                        println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                            index_name, quoted_table, quoted_cols.join(", "));
                        println!();
                    }
                }
                Some(ColumnExtractionResult::MultiTable(recommendations)) => {
                    // New logic for JOIN queries - one recommendation per table
                    for rec in recommendations {
                        // Sanitize table name (handle [Self] -> actual table name)
                        let sanitized_table = sanitize_table_name(&rec.table_name, table);
                        let table_snake = to_snake_case(&sanitized_table);
                        let index_name = format!("idx_{}_{}", table_snake, rec.columns.join("_"));
                        let quoted_table = quote_identifier(&table_snake);
                        let quoted_cols: Vec<String> = rec.columns.iter()
                            .map(|c| quote_identifier(c))
                            .collect();
                        let index_key = format!("{:?}_{:?}", sanitized_table, rec.columns);

                        if !seen_indexes.contains(&index_key) {
                            seen_indexes.insert(index_key.clone());

                            println!("   ✨ Recommended: {}", index_name);
                            println!("      Table: {}", sanitized_table);
                            println!("      Columns: {}", rec.columns.join(", "));
                            println!("      Source: {}:{}",
                                query.file.split('/').last().unwrap_or(&query.file),
                                query.line
                            );
                            println!("      Reason: {}", rec.reason);
                            println!("      SQL:    CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                                index_name, quoted_table, quoted_cols.join(", "));
                            println!();
                        }
                    }
                }
                None => {}
            }
        }
    }

    println!("🔍 ======================================================");
    println!("🔍   End of Recommendations");
    println!("🔍 ======================================================");
    println!();
}

fn extract_columns_from_sql(sql: &str) -> Option<ColumnExtractionResult> {
    let sql_upper = sql.to_uppercase();
    let is_join = sql_upper.contains("JOIN");

    if is_join {
        // JOIN query: extract columns per table
        let aliases = extract_table_aliases(sql);
        let recommendations = analyze_join_query_columns(sql, &aliases);

        if recommendations.is_empty() {
            None
        } else {
            Some(ColumnExtractionResult::MultiTable(recommendations))
        }
    } else {
        // Simple query (non-JOIN): use existing logic
        // First, remove subqueries to avoid extracting columns from them
        let sql_without_subqueries = remove_subqueries(sql);

        let mut columns = Vec::new();

        // Extract WHERE conditions (simplified)
        // Patterns: "column = $1", "column > $1", etc.
        let patterns = vec![
            r"(\w+)\s*=\s*\$\d+",
            r"(\w+)\s*>\s*\$\d+",
            r"(\w+)\s*<\s*\$\d+",
            r"(\w+)\s*>=\s*\$\d+",
            r"(\w+)\s*<=\s*\$\d+",
            r"(\w+)\s*IN\s*\(\$\d+",
            r"(\w+)\s*LIKE\s*\$\d+",
        ];

        for pattern in &patterns {
            let re = regex::Regex::new(pattern).unwrap();
            for caps in re.captures_iter(&sql_without_subqueries) {
                if let Some(col) = caps.get(1) {
                    let col_name = col.as_str().to_string();
                    if !columns.contains(&col_name) {
                        columns.push(col_name);
                    }
                }
            }
        }

        // Extract ORDER BY columns
        if let Some(order_pos) = sql_without_subqueries.to_uppercase().find("ORDER BY") {
            let after_order = &sql_without_subqueries[order_pos + 8..];
            if let Some(space_pos) = after_order.find(|c| c == ' ' || c == ',') {
                let order_col = after_order[..space_pos].trim();
                if !order_col.is_empty() && !columns.contains(&order_col.to_string()) {
                    columns.push(order_col.to_string());
                }
            }
        }

        if columns.is_empty() {
            None
        } else {
            Some(ColumnExtractionResult::SingleTable(columns))
        }
    }
}

/// Remove subqueries from SQL to avoid extracting columns from them
/// Returns (sql_without_subqueries, vec_of_subqueries)
pub fn extract_subqueries_from_sql(sql: &str) -> (String, Vec<String>) {
    let mut result = String::new();
    let mut subqueries = Vec::new();
    let mut depth = 0;
    let mut in_subquery = false;
    let mut subquery_start = 0;

    for (i, c) in sql.chars().enumerate() {
        if c == '(' {
            depth += 1;
            if depth == 1 && !in_subquery {
                // Check if this starts a SELECT subquery
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
                    // Extract the subquery SQL
                    let subquery_sql = &sql[subquery_start..i].trim();
                    subqueries.push(subquery_sql.to_string());
                    // Replace subquery with a placeholder
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

/// Extract subqueries from a query and add them as separate ExtractedQuery entries
fn extract_subqueries(
    sql: &str,
    parent_table: &str,
    parent_query_type: &str,
    file: &str,
    line: usize,
    queries: &mut Vec<ExtractedQuery>,
) {
    let (_, subqueries) = extract_subqueries_from_sql(sql);

    for subquery_sql in subqueries {
        // Extract table name from subquery
        if let Some(table_name) = extract_table_from_subquery(&subquery_sql) {
            queries.push(ExtractedQuery {
                table: table_name,
                query_type: format!("{}_subquery", parent_query_type),
                sql: subquery_sql,
                file: file.to_string(),
                line,
                is_from_subquery: true,
            });
        }
    }
}

/// Extract table name from a subquery
/// Pattern: SELECT ... FROM table_name ...
fn extract_table_from_subquery(subquery: &str) -> Option<String> {
    let subquery_lower = subquery.to_lowercase();

    // Find FROM clause
    if let Some(from_pos) = subquery_lower.find("from") {
        let after_from = &subquery[from_pos + 4..];

        // Extract the table name (first word after FROM)
        let table_name = after_from
            .split_whitespace()
            .next()?
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
            .to_string();

        Some(table_name)
    } else {
        None
    }
}

/// Remove subqueries from SQL to avoid extracting columns from them
fn remove_subqueries(sql: &str) -> String {
    let (result, _) = extract_subqueries_from_sql(sql);
    result
}

fn save_to_file(queries: &[ExtractedQuery], scan_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve output directory relative to the scanned project
    let scan_path_buf = PathBuf::from(scan_path);

    // Find the project root by looking for Cargo.toml
    let project_root = find_project_root(&scan_path_buf);

    let output_dir = project_root.join("target/sqlx_struct_indexes");
    fs::create_dir_all(&output_dir)?;

    // Generate CREATE INDEX SQL
    let mut create_sql = String::new();
    create_sql.push_str("-- Auto-generated by sqlx-analyze\n");
    create_sql.push_str("-- Scan all source files for queries\n\n");
    create_sql.push_str("BEGIN;\n\n");

    // Collect unique indexes
    let mut seen = HashSet::new();
    for query in queries {
        match extract_columns_from_sql(&query.sql) {
            Some(ColumnExtractionResult::SingleTable(cols)) => {
                // Use snake_case table name for deduplication key
                let table_snake = to_snake_case(&query.table);
                let index_key = format!("{}_{:?}", table_snake, cols);
                if !seen.contains(&index_key) {
                    seen.insert(index_key);
                    let index_name = format!("idx_{}_{}", table_snake, cols.join("_"));
                    let quoted_table = quote_identifier(&table_snake);
                    let quoted_cols: Vec<String> = cols.iter().map(|c| quote_identifier(c)).collect();

                    create_sql.push_str(&format!("-- {}: {}:{}\n",
                        query.query_type,
                        query.file.split('/').last().unwrap_or(&query.file),
                        query.line
                    ));
                    create_sql.push_str(&format!("-- Query: {}\n", query.sql.replace('\n', " ")));
                    create_sql.push_str(&format!("CREATE INDEX IF NOT EXISTS {} ON {} ({});\n\n",
                        index_name, quoted_table, quoted_cols.join(", ")));
                }
            }
            Some(ColumnExtractionResult::MultiTable(recommendations)) => {
                for rec in recommendations {
                    // Sanitize table name (handle [Self] -> actual table name)
                    let sanitized_table = sanitize_table_name(&rec.table_name, &query.table);
                    let table_snake = to_snake_case(&sanitized_table);
                    let index_name = format!("idx_{}_{}", table_snake, rec.columns.join("_"));
                    let quoted_table = quote_identifier(&table_snake);
                    let quoted_cols: Vec<String> = rec.columns.iter()
                        .map(|c| quote_identifier(c))
                        .collect();
                    // Use snake_case table name for deduplication key
                    let index_key = format!("{}_{:?}", table_snake, rec.columns);

                    if !seen.contains(&index_key) {
                        seen.insert(index_key);
                        create_sql.push_str(&format!("-- {}: {}:{}\n",
                            rec.reason,
                            query.file.split('/').last().unwrap_or(&query.file),
                            query.line
                        ));
                        create_sql.push_str(&format!("-- Query: {}\n", query.sql.replace('\n', " ")));
                        create_sql.push_str(&format!("CREATE INDEX IF NOT EXISTS {} ON {} ({});\n\n",
                            index_name, quoted_table, quoted_cols.join(", ")));
                    }
                }
            }
            None => {}
        }
    }

    create_sql.push_str("COMMIT;\n");

    let create_file = output_dir.join("indexes_postgres.sql");
    fs::write(&create_file, create_sql)?;

    // Generate DROP INDEX SQL
    let mut drop_sql = String::new();
    drop_sql.push_str("-- Auto-generated rollback script\n\n");
    drop_sql.push_str("BEGIN;\n\n");

    // Collect into a Vec first
    let keys: Vec<_> = seen.iter().collect();

    for key in keys.iter().rev() {
        if let Some(pos) = key.find('[') {
            // Format is "Table"_["col1", "col2"]
            // Extract table by removing leading " and trailing "_
            let table_raw = &key[1..pos]; // Skip opening "
            let table = table_raw.strip_suffix("\"_").unwrap_or(table_raw);

            let rest = &key[pos..];
            if let Some(end_pos) = rest.find(']') {
                // Format: ["col1"] for single column, ["col1", "col2"] for multiple
                // Need to find the content between [ and ]
                let cols_str = &rest[2..end_pos]; // Skip ["
                // Strip trailing quote if present (single column case)
                let cols_str = cols_str.strip_suffix('"').unwrap_or(cols_str);
                let cols: Vec<&str> = cols_str.split("\", \"").collect();
                let index_name = format!("idx_{}_{}", to_snake_case(table), cols.join("_"));

                drop_sql.push_str(&format!("DROP INDEX IF EXISTS {};\n\n", index_name));
            }
        }
    }

    drop_sql.push_str("COMMIT;\n");

    let drop_file = output_dir.join("drop_indexes_postgres.sql");
    fs::write(&drop_file, drop_sql)?;

    println!("   💾 Saved: {}", create_file.display());
    println!("   💾 Saved: {}", drop_file.display());
    println!();

    Ok(())
}
