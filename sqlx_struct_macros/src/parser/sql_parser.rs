// SQL Parser - Simplified version for architecture validation
//
// This is a temporary implementation using string matching to validate
// the architecture before integrating sqlparser-rs.

use super::{SqlDialect, JoinInfo, GroupByInfo};

/// Simplified SQL parser using string matching
pub struct SqlParser {
    #[allow(dead_code)]  // Reserved for dialect-specific parsing logic
    dialect: SqlDialect,
}

impl SqlParser {
    /// Create a new SQL parser for the specified dialect
    pub fn new(dialect: SqlDialect) -> Self {
        Self { dialect }
    }

    /// Parse SQL and extract JOIN information
    ///
    /// This is a simplified implementation that uses string matching
    /// to detect JOIN keywords and basic structure.
    pub fn extract_joins(&self, sql: &str) -> Vec<JoinInfo> {
        let mut joins = Vec::new();
        let sql_lower = sql.to_lowercase();

        // Detect INNER JOIN
        if sql_lower.contains("inner join") {
            let table = self.extract_join_table(sql, "inner join");
            let conditions = self.extract_join_on_conditions(sql);
            joins.push(JoinInfo::new(
                table,
                "INNER JOIN".to_string(),
                conditions,
            ));
        }

        // Detect LEFT JOIN
        if sql_lower.contains("left join") {
            let table = self.extract_join_table(sql, "left join");
            let conditions = self.extract_join_on_conditions(sql);
            joins.push(JoinInfo::new(
                table,
                "LEFT JOIN".to_string(),
                conditions,
            ));
        }

        // Detect RIGHT JOIN
        if sql_lower.contains("right join") {
            let table = self.extract_join_table(sql, "right join");
            let conditions = self.extract_join_on_conditions(sql);
            joins.push(JoinInfo::new(
                table,
                "RIGHT JOIN".to_string(),
                conditions,
            ));
        }

        joins
    }

    /// Parse SQL and extract GROUP BY information
    pub fn extract_group_by(&self, sql: &str) -> Option<GroupByInfo> {
        let sql_lower = sql.to_lowercase();

        // Find GROUP BY clause
        if let Some(group_by_pos) = sql_lower.find("group by") {
            let after_group_by = &sql[group_by_pos + 8..];

            // Extract columns until ORDER BY, HAVING, LIMIT, or end of string
            let columns_part = self.extract_until_keywords(
                after_group_by,
                &["order by", "having", "limit", "offset", "for", "window"]
            );

            // Parse column names
            let columns: Vec<String> = columns_part
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    // Remove quotes if present
                    if (s.starts_with('"') && s.ends_with('"')) ||
                       (s.starts_with('\'') && s.ends_with('\'')) {
                        s[1..s.len()-1].to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect();

            // Check for HAVING clause
            let having = sql_lower.find("having")
                .map(|pos| {
                    let after_having = &sql[pos + 6..];
                    self.extract_until_keywords(after_having, &["order by", "limit", "offset"])
                });

            Some(GroupByInfo::new(columns, having))
        } else {
            None
        }
    }

    /// Extract the table name from a JOIN clause
    fn extract_join_table(&self, sql: &str, join_keyword: &str) -> String {
        let sql_lower = sql.to_lowercase();
        let keyword_len = join_keyword.len();

        if let Some(pos) = sql_lower.find(join_keyword) {
            let after_join = &sql[pos + keyword_len..];

            // Extract the first word (table name) after JOIN
            let table_name = after_join
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string();

            table_name
        } else {
            "unknown".to_string()
        }
    }

    /// Extract conditions from ON clause
    fn extract_join_on_conditions(&self, sql: &str) -> Vec<String> {
        let mut conditions = Vec::new();
        let sql_lower = sql.to_lowercase();

        // Find ON keyword after JOIN
        let mut search_start = 0;
        while let Some(on_pos) = sql_lower[search_start..].find(" on ") {
            let abs_pos = search_start + on_pos;

            // Extract from ON to next JOIN or end
            let after_on = &sql[abs_pos + 4..];
            let condition_part = self.extract_until_keywords(
                after_on,
                &["inner join", "left join", "right join", "where", "group by", "order by", "limit"]
            );

            if !condition_part.trim().is_empty() {
                conditions.push(condition_part.trim().to_string());
            }

            search_start = abs_pos + 4;
        }

        conditions
    }

    /// Extract text until one of the keywords is found
    fn extract_until_keywords(&self, text: &str, keywords: &[&str]) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_inner_join() {
        let parser = SqlParser::new(SqlDialect::Postgres);
        let sql = "SELECT * FROM orders o INNER JOIN users u ON o.user_id = u.id";
        let joins = parser.extract_joins(sql);

        assert_eq!(joins.len(), 1);
        assert_eq!(joins[0].relation, "users");
        assert_eq!(joins[0].join_type, "INNER JOIN");
    }

    #[test]
    fn test_extract_left_join() {
        let parser = SqlParser::new(SqlDialect::Postgres);
        let sql = "SELECT * FROM orders o LEFT JOIN users u ON o.user_id = u.id";
        let joins = parser.extract_joins(sql);

        assert_eq!(joins.len(), 1);
        assert_eq!(joins[0].join_type, "LEFT JOIN");
    }

    #[test]
    fn test_extract_multiple_joins() {
        let parser = SqlParser::new(SqlDialect::Postgres);
        let sql = "SELECT * FROM orders o
                   INNER JOIN users u ON o.user_id = u.id
                   LEFT JOIN products p ON o.product_id = p.id";
        let joins = parser.extract_joins(sql);

        assert_eq!(joins.len(), 2);
        assert_eq!(joins[0].join_type, "INNER JOIN");
        assert_eq!(joins[1].join_type, "LEFT JOIN");
    }

    #[test]
    fn test_extract_group_by() {
        let parser = SqlParser::new(SqlDialect::Postgres);
        let sql = "SELECT category, COUNT(*) FROM products GROUP BY category";
        let group_by = parser.extract_group_by(sql);

        assert!(group_by.is_some());
        assert_eq!(group_by.unwrap().columns, vec!["category"]);
    }

    #[test]
    fn test_extract_group_by_multiple_columns() {
        let parser = SqlParser::new(SqlDialect::Postgres);
        let sql = "SELECT category, status, COUNT(*) FROM products GROUP BY category, status";
        let group_by = parser.extract_group_by(sql);

        assert!(group_by.is_some());
        let info = group_by.unwrap();
        assert_eq!(info.columns, vec!["category", "status"]);
    }

    #[test]
    fn test_extract_group_by_with_having() {
        let parser = SqlParser::new(SqlDialect::Postgres);
        let sql = "SELECT category, COUNT(*) FROM products GROUP BY category HAVING COUNT(*) > 10";
        let group_by = parser.extract_group_by(sql);

        assert!(group_by.is_some());
        let info = group_by.unwrap();
        assert!(info.has_having());
    }
}
