// Query Proxy Module
//
// Multi-database proxy implementation with automatic type conversion.
//
// This module provides enhanced query wrappers that support automatic type
// conversion for complex types (DECIMAL, DateTime, etc.) when binding parameters.

mod bind;
mod r#trait;
mod postgres;

#[cfg(feature = "postgres")]
pub use postgres::EnhancedQueryAsPostgres;

#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "mysql")]
pub use mysql::EnhancedQueryAsMySql;

#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "sqlite")]
pub use sqlite::EnhancedQueryAsSqlite;

// Re-export common types
pub use bind::{BindProxy, BindValue};
pub use r#trait::EnhancedQuery;
