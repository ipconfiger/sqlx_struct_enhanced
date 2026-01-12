// Column Extractor - Extract column information from SQL AST
//
// This module provides data structures to hold extracted information
// from SQL queries (JOINs, GROUP BY, etc.)
//
// Note: Some structures and methods are currently unused but reserved for future features.
#![allow(dead_code)]

use std::fmt;

/// Information extracted from a JOIN clause
#[derive(Debug, Clone, PartialEq)]
pub struct JoinInfo {
    /// The table/relation being joined (e.g., "users")
    pub relation: String,
    /// Type of JOIN (e.g., "INNER JOIN", "LEFT JOIN")
    pub join_type: String,
    /// Join conditions (e.g., ["user_id = id"])
    pub conditions: Vec<String>,
}

impl JoinInfo {
    /// Create a new JoinInfo
    pub fn new(relation: String, join_type: String, conditions: Vec<String>) -> Self {
        Self {
            relation,
            join_type,
            conditions,
        }
    }

    /// Check if this JOIN has conditions
    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }

    /// Get the first condition if any
    pub fn first_condition(&self) -> Option<&String> {
        self.conditions.first()
    }

    /// Format as a human-readable description
    pub fn describe(&self) -> String {
        format!("{} ON {}", self.join_type, self.conditions.join(" AND "))
    }
}

impl fmt::Display for JoinInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.join_type, self.relation)
    }
}

/// Information extracted from a GROUP BY clause
#[derive(Debug, Clone, PartialEq)]
pub struct GroupByInfo {
    /// Columns being grouped (e.g., ["category", "status"])
    pub columns: Vec<String>,
    /// HAVING clause expression if present
    pub having: Option<String>,
}

impl GroupByInfo {
    /// Create a new GroupByInfo
    pub fn new(columns: Vec<String>, having: Option<String>) -> Self {
        Self {
            columns,
            having,
        }
    }

    /// Check if GROUP BY has columns
    pub fn has_columns(&self) -> bool {
        !self.columns.is_empty()
    }

    /// Check if GROUP BY has HAVING clause
    pub fn has_having(&self) -> bool {
        self.having.is_some()
    }

    /// Get the number of group by columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Format as a human-readable description
    pub fn describe(&self) -> String {
        let mut desc = format!("GROUP BY {}", self.columns.join(", "));
        if let Some(having) = &self.having {
            desc.push_str(&format!(" HAVING {}", having));
        }
        desc
    }
}

impl fmt::Display for GroupByInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}

/// Information extracted from a subquery
#[derive(Debug, Clone, PartialEq)]
pub struct SubqueryInfo {
    /// The subquery SQL
    pub query: String,
    /// WHERE conditions in the subquery
    pub where_conditions: Vec<String>,
    /// JOINs in the subquery
    pub joins: Vec<JoinInfo>,
    /// GROUP BY in the subquery
    pub group_by: Option<GroupByInfo>,
}

impl SubqueryInfo {
    /// Create a new SubqueryInfo
    pub fn new(query: String) -> Self {
        Self {
            query,
            where_conditions: Vec::new(),
            joins: Vec::new(),
            group_by: None,
        }
    }

    /// Check if subquery has WHERE conditions
    pub fn has_where(&self) -> bool {
        !self.where_conditions.is_empty()
    }

    /// Check if subquery has JOINs
    pub fn has_joins(&self) -> bool {
        !self.joins.is_empty()
    }

    /// Check if subquery has GROUP BY
    pub fn has_group_by(&self) -> bool {
        self.group_by.is_some() && self.group_by.as_ref().unwrap().has_columns()
    }
}

/// Information about a potential covering index
#[derive(Debug, Clone, PartialEq)]
pub struct CoveringIndexInfo {
    /// WHERE columns (indexed columns)
    pub where_columns: Vec<String>,
    /// INCLUDE columns (non-indexed but in SELECT)
    pub include_columns: Vec<String>,
}

impl CoveringIndexInfo {
    /// Create a new CoveringIndexInfo
    pub fn new(where_columns: Vec<String>, include_columns: Vec<String>) -> Self {
        Self {
            where_columns,
            include_columns,
        }
    }

    /// Check if there are include columns
    pub fn has_includes(&self) -> bool {
        !self.include_columns.is_empty()
    }

    /// Format as CREATE INDEX statement
    pub fn format_create_index(&self, table: &str, index_name: &str, dialect: super::SqlDialect) -> String {
        let syntax = super::IndexSyntax::for_dialect(dialect);
        let columns = self.where_columns.join(", ");

        if self.has_includes() && syntax.include_supported {
            let includes = self.include_columns.join(", ");
            format!("CREATE INDEX {} ON {} ({}) INCLUDE ({})",
                index_name, table, columns, includes)
        } else {
            format!("CREATE INDEX {} ON {} ({})",
                index_name, table, columns)
        }
    }
}

/// Information about a potential partial index
#[derive(Debug, Clone, PartialEq)]
pub struct PartialIndexInfo {
    /// Column to index
    pub column: String,
    /// Filter condition (e.g., "status = 'active'")
    pub filter: String,
}

impl PartialIndexInfo {
    /// Create a new PartialIndexInfo
    pub fn new(column: String, filter: String) -> Self {
        Self {
            column,
            filter,
        }
    }

    /// Format as CREATE INDEX statement
    pub fn format_create_index(&self, table: &str, index_name: &str) -> String {
        format!("CREATE INDEX {} ON {} ({}) WHERE {}",
            index_name, table, self.column, self.filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::SqlDialect;

    #[test]
    fn test_join_info_creation() {
        let join = JoinInfo::new(
            "users".to_string(),
            "INNER JOIN".to_string(),
            vec!["user_id = id".to_string()],
        );

        assert_eq!(join.relation, "users");
        assert_eq!(join.join_type, "INNER JOIN");
        assert!(join.has_conditions());
        assert_eq!(join.first_condition(), Some(&"user_id = id".to_string()));
    }

    #[test]
    fn test_join_info_describe() {
        let join = JoinInfo::new(
            "users".to_string(),
            "LEFT JOIN".to_string(),
            vec!["o.user_id = u.id".to_string(), "o.status = u.active".to_string()],
        );

        let desc = join.describe();
        assert!(desc.contains("LEFT JOIN"));
        assert!(desc.contains("o.user_id = u.id"));
        assert!(desc.contains("AND"));
    }

    #[test]
    fn test_group_by_info_creation() {
        let group_by = GroupByInfo::new(
            vec!["category".to_string(), "status".to_string()],
            Some("COUNT(*) > 10".to_string()),
        );

        assert_eq!(group_by.column_count(), 2);
        assert!(group_by.has_columns());
        assert!(group_by.has_having());
    }

    #[test]
    fn test_group_by_info_describe() {
        let group_by = GroupByInfo::new(
            vec!["category".to_string()],
            None,
        );

        let desc = group_by.describe();
        assert_eq!(desc, "GROUP BY category");
    }

    #[test]
    fn test_covering_index_format_postgres() {
        let info = CoveringIndexInfo::new(
            vec!["user_id".to_string()],
            vec!["username".to_string(), "email".to_string()],
        );

        let sql = info.format_create_index("users", "idx_user_covering", SqlDialect::Postgres);
        assert!(sql.contains("INCLUDE"));
        assert!(sql.contains("username"));
        assert!(sql.contains("email"));
    }

    #[test]
    fn test_covering_index_format_sqlite() {
        let info = CoveringIndexInfo::new(
            vec!["user_id".to_string()],
            vec!["username".to_string()],
        );

        let sql = info.format_create_index("users", "idx_user", SqlDialect::SQLite);
        // SQLite doesn't support INCLUDE, so it should be a regular index
        assert!(!sql.contains("INCLUDE"));
    }

    #[test]
    fn test_partial_index_format() {
        let info = PartialIndexInfo::new(
            "email".to_string(),
            "status = 'active'".to_string(),
        );

        let sql = info.format_create_index("users", "idx_active_users_email");
        assert!(sql.contains("WHERE status = 'active'"));
    }
}
