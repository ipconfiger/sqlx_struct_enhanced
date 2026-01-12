//! SQL generation for JOIN queries with qualified column names.
//!
//! This module handles the complex task of generating SELECT statements
//! for JOIN queries where column names might conflict between tables.
//! It uses table-qualified column aliases (e.g., "table.column") to ensure
//! uniqueness.

use crate::{ColumnDefinition, Scheme};

/// Type of SQL join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER JOIN"),
            JoinType::Left => write!(f, "LEFT JOIN"),
            JoinType::Right => write!(f, "RIGHT JOIN"),
            JoinType::Full => write!(f, "FULL JOIN"),
        }
    }
}

/// Represents a JOIN operation in the query.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    pub table_name: String,
    pub condition: String,
    pub join_type: JoinType,
}

/// SQL generator for JOIN queries that produces qualified column aliases.
///
/// # Example Output
///
/// ```sql
/// SELECT
///   orders.id AS "orders.id",
///   orders.customer_id AS "orders.customer_id",
///   customers.id AS "customers.id",
///   customers.name AS "customers.name"
/// FROM orders
/// INNER JOIN customers ON orders.customer_id = customers.id
/// ```
pub struct JoinSqlGenerator {
    table_a_name: String,
    table_a_fields: Vec<ColumnDefinition>,
    table_b_name: String,
    table_b_fields: Vec<ColumnDefinition>,
    join_type: JoinType,
    join_condition: String,
}

impl JoinSqlGenerator {
    /// Create a new SQL generator for a 2-table JOIN.
    ///
    /// # Arguments
    ///
    /// * `join_type` - Type of JOIN (INNER, LEFT, RIGHT, FULL)
    /// * `condition` - JOIN condition (e.g., "orders.customer_id = customers.id")
    pub fn new<A, B>(join_type: JoinType, condition: &str) -> Self
    where
        A: SchemeAccessor,
        B: SchemeAccessor,
    {
        let scheme_a = A::get_scheme();
        let scheme_b = B::get_scheme();

        Self {
            table_a_name: scheme_a.table_name().to_string(),
            table_a_fields: scheme_a.column_definitions().to_vec(),
            table_b_name: scheme_b.table_name().to_string(),
            table_b_fields: scheme_b.column_definitions().to_vec(),
            join_type,
            join_condition: condition.to_string(),
        }
    }

    /// Quote an identifier for the current database type.
    fn quote_identifier(&self, identifier: &str) -> String {
        #[cfg(feature = "postgres")]
        return format!("\"{}\"", identifier);

        #[cfg(feature = "mysql")]
        return format!("`{}`", identifier);

        #[cfg(feature = "sqlite")]
        return identifier.to_string();
    }

    /// Quote a qualified column name (table.column) for the current database type.
    fn quote_qualified_column(&self, table: &str, column: &str) -> String {
        #[cfg(feature = "postgres")]
        return format!("\"{}.{}\"", table, column);

        #[cfg(feature = "mysql")]
        return format!("`{}.{}`", table, column);

        #[cfg(feature = "sqlite")]
        return format!("{}.{}", table, column);
    }

    /// Generate SELECT clause with table-qualified column aliases.
    ///
    /// Each column is aliased as "table_name.column_name" to prevent
    /// conflicts when both tables have columns with the same name.
    pub fn gen_select_clause(&self) -> String {
        let mut columns = Vec::new();

        // Table A columns: orders.id AS "orders.id"
        for col in &self.table_a_fields {
            let quoted_table = self.quote_identifier(&self.table_a_name);
            let quoted_col = self.quote_identifier(&col.name);
            let qualified = format!("{}.{}", quoted_table, quoted_col);
            let alias = self.quote_qualified_column(&self.table_a_name, &col.name);
            columns.push(format!("{} AS {}", qualified, alias));
        }

        // Table B columns: customers.id AS "customers.id"
        for col in &self.table_b_fields {
            let quoted_table = self.quote_identifier(&self.table_b_name);
            let quoted_col = self.quote_identifier(&col.name);
            let qualified = format!("{}.{}", quoted_table, quoted_col);
            let alias = self.quote_qualified_column(&self.table_b_name, &col.name);
            columns.push(format!("{} AS {}", qualified, alias));
        }

        columns.join(", ")
    }

    /// Generate the FROM and JOIN clauses.
    pub fn gen_from_join(&self) -> String {
        let quoted_table_a = self.quote_identifier(&self.table_a_name);
        let quoted_table_b = self.quote_identifier(&self.table_b_name);

        format!(
            "FROM {} {} {} ON {}",
            quoted_table_a, self.join_type, quoted_table_b, self.join_condition
        )
    }

    /// Generate the full JOIN query with optional WHERE clause.
    ///
    /// # Arguments
    ///
    /// * `where_clause` - Optional WHERE clause (must include "WHERE" keyword if present)
    pub fn gen_full_query(&self, where_clause: Option<&str>) -> String {
        let select = self.gen_select_clause();
        let from_join = self.gen_from_join();
        let where_str = where_clause.unwrap_or("");

        format!("SELECT {} {} {}", select, from_join, where_str)
            .trim_end()
            .to_string()
    }
}

/// Trait for types that can provide their Scheme metadata and decode themselves from JOIN rows.
///
/// This is implemented by the EnhancedCrud derive macro.
pub trait SchemeAccessor {
    fn get_scheme() -> &'static Scheme;

    /// Decode this entity from a PostgreSQL row with qualified column names.
    ///
    /// Returns `Ok(Some(entity))` if successfully decoded,
    /// `Ok(None)` if all columns are NULL (for LEFT/RIGHT/FULL joins),
    /// `Err(Error)` if decoding fails.
    #[cfg(feature = "postgres")]
    fn decode_from_qualified_row_pg(row: &sqlx::postgres::PgRow) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;

    /// Decode this entity from a MySQL row with qualified column names.
    #[cfg(feature = "mysql")]
    fn decode_from_qualified_row_mysql(row: &sqlx::mysql::MySqlRow) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;

    /// Decode this entity from a SQLite row with qualified column names.
    #[cfg(feature = "sqlite")]
    fn decode_from_qualified_row_sqlite(row: &sqlx::sqlite::SqliteRow) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_type_display() {
        assert_eq!(format!("{}", JoinType::Inner), "INNER JOIN");
        assert_eq!(format!("{}", JoinType::Left), "LEFT JOIN");
        assert_eq!(format!("{}", JoinType::Right), "RIGHT JOIN");
        assert_eq!(format!("{}", JoinType::Full), "FULL JOIN");
    }

    // Note: More comprehensive tests require actual Scheme implementations
    // which will be added when the derive macro is updated
}
