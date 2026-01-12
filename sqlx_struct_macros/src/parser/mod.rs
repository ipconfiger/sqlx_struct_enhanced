// Parser module - Simplified version for architecture validation
//
// This is a temporary simplified implementation to validate the architecture
// before integrating sqlparser-rs.

pub mod sql_parser;
pub mod column_extractor;
// pub mod ast_visitor;  // Temporarily disabled - requires sqlparser

// Re-export main types for convenience
pub use sql_parser::SqlParser;
pub use column_extractor::{JoinInfo, GroupByInfo};

// Database dialect support (simplified)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]  // All variants are used depending on feature flags
pub enum SqlDialect {
    Postgres,
    MySQL,
    SQLite,
}

impl SqlDialect {
    /// Check if this dialect supports INCLUDE clauses (covering indexes)
    #[allow(dead_code)]  // Used by IndexSyntax
    pub fn supports_include(&self) -> bool {
        matches!(self, SqlDialect::Postgres | SqlDialect::MySQL)
    }

    /// Check if this dialect supports partial indexes
    #[allow(dead_code)]  // Used by IndexSyntax
    pub fn supports_partial_indexes(&self) -> bool {
        matches!(self, SqlDialect::Postgres | SqlDialect::SQLite)
    }
}

/// Index syntax capabilities for different dialects
#[derive(Debug, Clone)]
pub struct IndexSyntax {
    pub include_supported: bool,
    pub partial_supported: bool,
    #[allow(dead_code)]  // Reserved for future use
    pub if_not_exists_supported: bool,
}

impl IndexSyntax {
    /// Get index syntax for a dialect
    pub fn for_dialect(dialect: SqlDialect) -> Self {
        match dialect {
            SqlDialect::Postgres => IndexSyntax {
                include_supported: true,
                partial_supported: true,
                if_not_exists_supported: true,
            },
            SqlDialect::MySQL => IndexSyntax {
                include_supported: true,   // MySQL 8.0+
                partial_supported: false,  // MySQL doesn't support partial indexes
                if_not_exists_supported: false,
            },
            SqlDialect::SQLite => IndexSyntax {
                include_supported: false,
                partial_supported: true,   // SQLite supports partial indexes with WHERE
                if_not_exists_supported: true,
            },
        }
    }
}
