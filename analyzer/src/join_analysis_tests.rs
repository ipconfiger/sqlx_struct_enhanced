// Tests for JOIN query analysis functionality
// These tests verify that we can correctly extract table aliases,
// qualified columns, and generate proper index recommendations for JOIN queries

use std::collections::{HashMap, HashSet};

// Data structures that will be implemented

/// Maps table aliases to actual table names
pub struct TableAliasMap {
    pub aliases: HashMap<String, String>,
}

impl TableAliasMap {
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
        }
    }

    pub fn add_alias(&mut self, alias: String, table: String) {
        self.aliases.insert(alias, table);
    }

    /// Resolve an alias or table reference to the actual table name
    /// If the input is an alias, return the mapped table name
    /// If the input is already a table name, return it as-is
    pub fn resolve(&self, alias_or_table: &str) -> String {
        if let Some(table) = self.aliases.get(alias_or_table) {
            table.clone()
        } else {
            alias_or_table.to_string()
        }
    }
}

/// Index recommendation for a single table
#[derive(Debug, Clone)]
pub struct TableIndexRecommendation {
    pub table_name: String,
    pub columns: Vec<String>,
    pub reason: String,
}

/// Result of column extraction
pub enum ColumnExtractionResult {
    SingleTable(Vec<String>),
    MultiTable(Vec<TableIndexRecommendation>),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: TableAliasMap basic functionality
    #[test]
    fn test_table_alias_map_basic() {
        let mut map = TableAliasMap::new();

        // Add aliases
        map.add_alias("m".to_string(), "merchant".to_string());
        map.add_alias("mc".to_string(), "merchant_channel".to_string());

        // Resolve aliases
        assert_eq!(map.resolve("m"), "merchant");
        assert_eq!(map.resolve("mc"), "merchant_channel");

        // Unmapped name returns itself
        assert_eq!(map.resolve("orders"), "orders");
    }

    // Test 2: Extract table aliases from simple FROM clause
    #[test]
    fn test_extract_simple_from_alias() {
        let sql = "SELECT * FROM merchant AS m WHERE m.id = $1";
        let map = extract_table_aliases(sql);

        assert_eq!(map.resolve("m"), "merchant");
    }

    // Test 3: Extract table aliases from JOIN clause
    #[test]
    fn test_extract_join_alias() {
        let sql = "SELECT m.* FROM merchant AS m
                   INNER JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE mc.channel_id = $1";

        let map = extract_table_aliases(sql);

        assert_eq!(map.resolve("m"), "merchant");
        assert_eq!(map.resolve("mc"), "merchant_channel");
    }

    // Test 4: Extract qualified columns from WHERE clause
    #[test]
    fn test_extract_qualified_columns_from_where() {
        let sql = "SELECT m.* FROM merchant AS m
                   WHERE m.city_id = $1 AND m.status = $2";

        let columns = extract_qualified_columns(sql, "where");

        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0], ("m".to_string(), "city_id".to_string()));
        assert_eq!(columns[1], ("m".to_string(), "status".to_string()));
    }

    // Test 5: Extract qualified columns from ORDER BY clause
    #[test]
    fn test_extract_qualified_columns_from_order_by() {
        let sql = "SELECT m.* FROM merchant AS m ORDER BY m.created_at DESC";

        let columns = extract_qualified_columns(sql, "order by");

        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0], ("m".to_string(), "created_at".to_string()));
    }

    // Test 6: Complete JOIN query analysis (including ON clause)
    #[test]
    fn test_analyze_join_query() {
        let sql = "SELECT m.* FROM merchant AS m
                   INNER JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE mc.channel_id = $1 AND m.city_id = $2";

        let aliases = extract_table_aliases(sql);
        let recommendations = analyze_join_query_columns(sql, &aliases);

        assert_eq!(recommendations.len(), 2);

        // Find merchant recommendation
        let merchant_rec = recommendations.iter()
            .find(|r| r.table_name == "merchant")
            .expect("Should have merchant recommendation");
        // merchant should have: merchant_id (from ON) + city_id (from WHERE)
        assert_eq!(merchant_rec.columns, vec!["merchant_id", "city_id"]);

        // Find merchant_channel recommendation
        let mc_rec = recommendations.iter()
            .find(|r| r.table_name == "merchant_channel")
            .expect("Should have merchant_channel recommendation");
        // merchant_channel should have: merchant_id (from ON) + channel_id (from WHERE)
        assert_eq!(mc_rec.columns, vec!["merchant_id", "channel_id"]);
    }

    // Test 7: [Self] syntax handling
    #[test]
    fn test_self_syntax() {
        let sql = "SELECT m.* FROM [Self] AS m
                   INNER JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE mc.channel_id = $1";

        let map = extract_table_aliases(sql);

        // [Self] should be treated as literal table name
        assert_eq!(map.resolve("m"), "[Self]");
    }

    // Test 8: Tables without aliases
    #[test]
    fn test_table_without_alias() {
        let sql = "SELECT * FROM merchant JOIN merchant_channel ON merchant.id = merchant_channel.merchant_id
                   WHERE merchant_channel.status = $1";

        let map = extract_table_aliases(sql);

        // Table names should resolve to themselves
        assert_eq!(map.resolve("merchant_channel"), "merchant_channel");
    }

    // Test 9: Multi-table JOIN (3+ tables)
    #[test]
    fn test_multi_table_join() {
        let sql = "SELECT o.* FROM orders o
                   JOIN users u ON o.user_id = u.id
                   JOIN products p ON o.product_id = p.id
                   WHERE o.status = $1 AND u.category = $2";

        let aliases = extract_table_aliases(sql);
        let recommendations = analyze_join_query_columns(sql, &aliases);

        assert_eq!(recommendations.len(), 3);

        let orders_rec = recommendations.iter().find(|r| r.table_name == "orders").unwrap();
        // orders: user_id (ON), product_id (ON), status (WHERE)
        assert_eq!(orders_rec.columns, vec!["user_id", "product_id", "status"]);

        let users_rec = recommendations.iter().find(|r| r.table_name == "users").unwrap();
        // users: id (ON), category (WHERE)
        assert_eq!(users_rec.columns, vec!["id", "category"]);

        let products_rec = recommendations.iter().find(|r| r.table_name == "products").unwrap();
        // products: id (ON only)
        assert_eq!(products_rec.columns, vec!["id"]);
    }

    // Test 10: ORDER BY in JOIN query
    #[test]
    fn test_order_by_in_join() {
        let sql = "SELECT m.* FROM merchant AS m
                   INNER JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE mc.channel_id = $1
                   ORDER BY m.created_at DESC";

        let aliases = extract_table_aliases(sql);
        let recommendations = analyze_join_query_columns(sql, &aliases);

        assert_eq!(recommendations.len(), 2);

        // Check that both WHERE and ORDER BY columns are captured
        let mc_rec = recommendations.iter().find(|r| r.table_name == "merchant_channel").unwrap();
        assert_eq!(mc_rec.columns, vec!["merchant_id", "channel_id"]); // ON + WHERE

        let merchant_rec = recommendations.iter().find(|r| r.table_name == "merchant").unwrap();
        assert_eq!(merchant_rec.columns, vec!["merchant_id", "created_at"]); // ON + ORDER BY
    }

    // Test 11: ON clause extraction (the critical missing feature)
    #[test]
    fn test_on_clause_extraction() {
        let sql = "SELECT m.* FROM [Self] AS m
                   INNER JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE mc.channel_id = $1 AND m.city_id = $2";

        let aliases = extract_table_aliases(sql);
        let recommendations = analyze_join_query_columns(sql, &aliases);

        assert_eq!(recommendations.len(), 2);

        // merchant should have: merchant_id (from ON) + city_id (from WHERE)
        let merchant_rec = recommendations.iter()
            .find(|r| r.table_name == "[Self]")
            .expect("Should have [Self] recommendation");
        assert_eq!(merchant_rec.columns, vec!["merchant_id", "city_id"]);

        // merchant_channel should have: merchant_id (from ON) + channel_id (from WHERE)
        let mc_rec = recommendations.iter()
            .find(|r| r.table_name == "merchant_channel")
            .expect("Should have merchant_channel recommendation");
        assert_eq!(mc_rec.columns, vec!["merchant_id", "channel_id"]);
    }

    // Test 12: Multiple JOINs with ON clauses
    #[test]
    fn test_multiple_join_on_clauses() {
        let sql = "SELECT o.* FROM orders o
                   JOIN users u ON o.user_id = u.id
                   JOIN products p ON o.product_id = p.id
                   WHERE o.status = $1 AND u.category = $2";

        let aliases = extract_table_aliases(sql);
        let recommendations = analyze_join_query_columns(sql, &aliases);

        assert_eq!(recommendations.len(), 3);

        let orders_rec = recommendations.iter().find(|r| r.table_name == "orders").unwrap();
        assert_eq!(orders_rec.columns, vec!["user_id", "product_id", "status"]); // 2 ON + 1 WHERE

        let users_rec = recommendations.iter().find(|r| r.table_name == "users").unwrap();
        assert_eq!(users_rec.columns, vec!["id", "category"]); // 1 ON + 1 WHERE

        let products_rec = recommendations.iter().find(|r| r.table_name == "products").unwrap();
        assert_eq!(products_rec.columns, vec!["id"]); // 1 ON only
    }

    // Test 13: Subquery with alias - recursive alias extraction
    #[test]
    fn test_subquery_alias_extraction() {
        let sql = "SELECT m.* FROM merchant AS m
                   WHERE m.id in (SELECT m1.id FROM merchant_coupon_type as m1 WHERE m1.coupon_type_id = $1)";

        let map = extract_table_aliases(sql);

        // Should have aliases from both main query and subquery
        assert_eq!(map.resolve("m"), "merchant");
        assert_eq!(map.resolve("m1"), "merchant_coupon_type");
    }

    // Test 14: Nested subqueries - recursive alias extraction at multiple levels
    #[test]
    fn test_nested_subquery_alias_extraction() {
        let sql = "SELECT m.* FROM merchant AS m
                   WHERE m.id in (
                       SELECT m1.id FROM merchant_coupon_type as m1
                       WHERE m1.type_id in (SELECT t.id FROM type as t WHERE t.name = $1)
                   )";

        let map = extract_table_aliases(sql);

        // Should have aliases from all levels: main query, first subquery, nested subquery
        assert_eq!(map.resolve("m"), "merchant");
        assert_eq!(map.resolve("m1"), "merchant_coupon_type");
        assert_eq!(map.resolve("t"), "type");
    }

    // Test 15: Multiple subqueries in one query
    #[test]
    fn test_multiple_subquery_alias_extraction() {
        let sql = "SELECT m.* FROM merchant AS m
                   JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE m.id in (SELECT m1.id FROM merchant_coupon_type as m1 WHERE ...)
                     AND mc.channel_id in (SELECT c.id FROM channel as c WHERE ...)";

        let map = extract_table_aliases(sql);

        // Should have aliases from main query and both subqueries
        assert_eq!(map.resolve("m"), "merchant");
        assert_eq!(map.resolve("mc"), "merchant_channel");
        assert_eq!(map.resolve("m1"), "merchant_coupon_type");
        assert_eq!(map.resolve("c"), "channel");
    }

    // Test 16: Subquery alias resolution in JOIN query analysis
    #[test]
    fn test_subquery_alias_resolution_in_join_analysis() {
        let sql = "SELECT m.* FROM [Self] AS m
                   JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id
                   WHERE m.merchant_id in (
                       SELECT m1.merchant_id FROM merchant_coupon_type as m1
                       JOIN coupon_type as c ON m1.coupon_type_id = c.coupon_type_id
                       WHERE c.name = $1
                   )
                   AND m.city_id = $2";

        let aliases = extract_table_aliases(sql);

        // All aliases should be resolved correctly
        assert_eq!(aliases.resolve("m"), "[Self]");
        assert_eq!(aliases.resolve("mc"), "merchant_channel");
        assert_eq!(aliases.resolve("m1"), "merchant_coupon_type"); // Critical: was "m1" before fix
        assert_eq!(aliases.resolve("c"), "coupon_type");

        // Now verify column analysis works correctly
        let recommendations = analyze_join_query_columns(sql, &aliases);

        // Should have recommendations for all tables
        assert!(recommendations.iter().any(|r| r.table_name == "[Self]"));
        assert!(recommendations.iter().any(|r| r.table_name == "merchant_channel"));
        assert!(recommendations.iter().any(|r| r.table_name == "merchant_coupon_type"));
        assert!(recommendations.iter().any(|r| r.table_name == "coupon_type"));
    }

} // End of tests module

// Public implementations

/// Extract table name and alias mappings from FROM and JOIN clauses
/// Recursively extracts aliases from ALL levels of queries, including nested subqueries
pub fn extract_table_aliases(sql: &str) -> TableAliasMap {
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

            // Find the end of this JOIN clause (up to ON, WHERE, or end of string)
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
    // Use the extract_subqueries_from_sql function from main.rs
    let (_, subqueries) = crate::extract_subqueries_from_sql(sql);
    for subquery_sql in subqueries {
        // Recursively extract aliases from each subquery
        let subquery_aliases = extract_table_aliases(&subquery_sql);
        // Merge subquery aliases into main map
        for (alias, table) in subquery_aliases.aliases.iter() {
            map.add_alias(alias.clone(), table.clone());
        }
    }

    map
}

/// Find the end of a FROM clause (stops at WHERE, ORDER BY, GROUP BY, etc.)
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

/// Find the end of a JOIN clause (stops at WHERE, next JOIN, etc.)
/// NOTE: Should NOT stop at "on" - we need to extract the table name and alias
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
/// Supports: "table", "table AS alias", "table alias"
fn parse_table_clause(clause: &str, map: &mut TableAliasMap) {
    let trimmed = clause.trim();

    // Skip if empty
    if trimmed.is_empty() {
        return;
    }

    // Split by whitespace
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() {
        return;
    }

    let table_name = parts[0].trim();

    if parts.len() == 1 {
        // No alias: just the table name
        // Map table name to itself for easier resolution
        map.add_alias(table_name.to_string(), table_name.to_string());
    } else if parts.len() >= 2 {
        let second = parts[1].trim().to_uppercase();

        if second == "AS" && parts.len() >= 3 {
            // "table AS alias" pattern
            let alias = parts[2].trim();
            map.add_alias(alias.to_string(), table_name.to_string());
        } else if second != "ON" && second != "WHERE" && second != "," {
            // "table alias" pattern (second word is the alias)
            map.add_alias(second.to_lowercase(), table_name.to_string());
        } else {
            // No alias: map table to itself
            map.add_alias(table_name.to_string(), table_name.to_string());
        }
    }
}

/// Extract columns with table prefixes from WHERE or ORDER BY clauses
/// Returns Vec of (table_ref, column_name)
pub fn extract_qualified_columns(sql: &str, clause_keyword: &str) -> Vec<(String, String)> {
    let mut columns = Vec::new();
    let sql_lower = sql.to_lowercase();

    // Find the clause
    if let Some(clause_pos) = sql_lower.find(clause_keyword) {
        let clause_start = clause_pos + clause_keyword.len();
        let clause_end = find_clause_end(&sql_lower[clause_start..]);

        if clause_end > 0 {
            let clause_content = &sql[clause_start..clause_start + clause_end];

            // Patterns for qualified columns: table.column
            let patterns = [
                r"(\w+)\.(\w+)\s*=",   // table.column =
                r"(\w+)\.(\w+)\s*>",   // table.column >
                r"(\w+)\.(\w+)\s*<",   // table.column <
                r"(\w+)\.(\w+)\s*>=",  // table.column >=
                r"(\w+)\.(\w+)\s*<=",  // table.column <=
                r"(\w+)\.(\w+)\s+IN",  // table.column IN
                r"(\w+)\.(\w+)\s+LIKE", // table.column LIKE
            ];

            for pattern in &patterns {
                if let Ok(re) = regex::Regex::new(pattern) {
                    for caps in re.captures_iter(clause_content) {
                        if let (Some(table), Some(col)) = (caps.get(1), caps.get(2)) {
                            let table_ref = table.as_str().to_string();
                            let col_name = col.as_str().to_string();
                            if !columns.contains(&(table_ref.clone(), col_name.clone())) {
                                columns.push((table_ref, col_name));
                            }
                        }
                    }
                }
            }

            // For ORDER BY, also handle simple "table.column" pattern
            if clause_keyword.to_lowercase() == "order by" {
                let order_pattern = regex::Regex::new(r"(\w+)\.(\w+)").unwrap();
                for caps in order_pattern.captures_iter(clause_content) {
                    if let (Some(table), Some(col)) = (caps.get(1), caps.get(2)) {
                        let table_ref = table.as_str().to_string();
                        let col_name = col.as_str().to_string();
                        if !columns.contains(&(table_ref.clone(), col_name.clone())) {
                            columns.push((table_ref, col_name));
                        }
                    }
                }
            }
        }
    }

    columns
}

/// Find the end of a clause (stops at next major SQL keyword)
fn find_clause_end(clause: &str) -> usize {
    let keywords = ["order by", "group by", "having", "limit", "offset", "union"];
    let mut min_pos = clause.len();

    for keyword in &keywords {
        if let Some(pos) = clause.find(keyword) {
            min_pos = min_pos.min(pos);
        }
    }

    min_pos
}

/// Analyze JOIN query and extract columns per table
pub fn analyze_join_query_columns(
    sql: &str,
    aliases: &TableAliasMap,
) -> Vec<TableIndexRecommendation> {
    let mut recommendations: HashMap<String, TableIndexRecommendation> = HashMap::new();
    let sql_lower = sql.to_lowercase();

    // Check if query has ORDER BY
    let has_order_by = sql_lower.contains("order by");
    let has_on_clause = sql_lower.contains(" on ");

    // Extract ON clause columns (JOIN conditions)
    let on_columns = extract_on_columns(sql);
    for (table_ref, column) in on_columns {
        let table_name = aliases.resolve(&table_ref);
        recommendations
            .entry(table_name.clone())
            .or_insert_with(|| TableIndexRecommendation {
                table_name,
                columns: Vec::new(),
                reason: if has_order_by {
                    "ON/WHERE/ORDER BY in JOIN query".to_string()
                } else {
                    "ON/WHERE in JOIN query".to_string()
                },
            })
            .columns
            .push(column);
    }

    // Extract WHERE clause columns
    let where_columns = extract_qualified_columns(sql, "where");
    for (table_ref, column) in where_columns {
        let table_name = aliases.resolve(&table_ref);
        recommendations
            .entry(table_name.clone())
            .or_insert_with(|| TableIndexRecommendation {
                table_name,
                columns: Vec::new(),
                reason: if has_order_by {
                    "ON/WHERE/ORDER BY in JOIN query".to_string()
                } else if has_on_clause {
                    "ON/WHERE in JOIN query".to_string()
                } else {
                    "WHERE condition in JOIN query".to_string()
                },
            })
            .columns
            .push(column);
    }

    // Extract ORDER BY clause columns
    let order_columns = extract_qualified_columns(sql, "order by");
    for (table_ref, column) in order_columns {
        let table_name = aliases.resolve(&table_ref);
        recommendations
            .entry(table_name.clone())
            .or_insert_with(|| TableIndexRecommendation {
                table_name,
                columns: Vec::new(),
                reason: "ON/WHERE/ORDER BY in JOIN query".to_string(),
            })
            .columns
            .push(column);
    }

    // Deduplicate columns within each recommendation
    recommendations
        .into_values()
        .map(|mut rec| {
            // Use HashSet to deduplicate while preserving order
            let mut seen = HashSet::new();
            let mut unique_columns = Vec::new();
            for col in rec.columns {
                if seen.insert(col.clone()) {
                    unique_columns.push(col);
                }
            }
            rec.columns = unique_columns;
            rec
        })
        .collect()
}

/// Extract columns from ON clauses (JOIN conditions)
/// Returns Vec of (table_ref, column_name)
fn extract_on_columns(sql: &str) -> Vec<(String, String)> {
    let mut columns = Vec::new();
    let sql_lower = sql.to_lowercase();

    // Find all ON clauses
    let mut search_start = 0;
    while let Some(on_pos) = sql_lower[search_start..].find(" on ") {
        let actual_on_pos = search_start + on_pos + 4; // +4 for " on "

        // Find the end of ON clause (stops at WHERE, ORDER BY, GROUP BY, next JOIN, etc.)
        let on_end = find_on_clause_end(&sql_lower[actual_on_pos..]);

        if on_end > 0 {
            let on_content = &sql[actual_on_pos..actual_on_pos + on_end];

            // Match patterns like: table1.column1 = table2.column2
            // We need to extract both sides of the equality
            let on_pattern = regex::Regex::new(r"(\w+)\.(\w+)").unwrap();
            for caps in on_pattern.captures_iter(on_content) {
                if let (Some(table), Some(col)) = (caps.get(1), caps.get(2)) {
                    let table_ref = table.as_str().to_string();
                    let col_name = col.as_str().to_string();
                    if !columns.contains(&(table_ref.clone(), col_name.clone())) {
                        columns.push((table_ref, col_name));
                    }
                }
            }
        }

        search_start = actual_on_pos + on_end;
    }

    columns
}

/// Find the end of an ON clause
fn find_on_clause_end(clause: &str) -> usize {
    let keywords = ["where", "order by", "group by", "having", "limit", "inner join", "left join", "right join", "join"];
    let mut min_pos = clause.len();

    for keyword in &keywords {
        if let Some(pos) = clause.find(keyword) {
            min_pos = min_pos.min(pos);
        }
    }

    min_pos
}
