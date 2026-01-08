//! Aggregation query support for sqlx_struct_enhanced
//!
//! This module provides a fluent query builder for SQL aggregation operations
//! including SUM, AVG, COUNT, MIN, MAX with GROUP BY, HAVING, ORDER BY,
//! LIMIT/OFFSET, and JOIN support.

mod query_builder;

pub use query_builder::{AggQueryBuilder, Join, JoinType};
