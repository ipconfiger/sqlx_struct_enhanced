//! JOIN query support for returning entity tuples.
//!
//! This module provides functionality for performing JOIN queries that return
//! type-safe tuples of entities like `Vec<(Option<A>, Option<B>)>`.
//!
//! # Example
//!
//! ```ignore
//! use sqlx_struct_enhanced::EnhancedCrud;
//!
//! #[derive(EnhancedCrud)]
//! struct Order {
//!     pub id: String,
//!     pub customer_id: String,
//!     pub amount: i32,
//! }
//!
//! #[derive(EnhancedCrud)]
//! struct Customer {
//!     pub id: String,
//!     pub name: String,
//!     pub email: String,
//! }
//!
//! // INNER JOIN
//! let results: Vec<(Option<Order>, Option<Customer>)> = Order::join_inner::<Customer>(
//!     "orders.customer_id = customers.id"
//! )
//! .fetch_all(&pool)
//! .await?;
//! ```

mod query_builder;
mod sql_generator;
mod tuple_decoder;

pub use query_builder::JoinQueryBuilder;
pub use sql_generator::{JoinSqlGenerator, JoinType, JoinClause, SchemeAccessor};
pub use tuple_decoder::{JoinTuple2, JoinTuple3, JoinTuple4, JoinTuple5};
