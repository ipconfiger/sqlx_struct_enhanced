// Unified Enhanced Query Trait
//
// This trait defines the interface for enhanced query wrappers across all databases.
// Each database (PostgreSQL, MySQL, SQLite) implements this trait for their
// specific query types.

use sqlx::{Executor, Encode, Type};
use std::future::Future;

use crate::proxy::bind::BindProxy;

/// Unified enhanced query trait for all database backends.
///
/// This trait provides a consistent interface for automatic type conversion
/// across PostgreSQL, MySQL, and SQLite.
///
/// # Type Parameters
///
/// * `'q` - Lifetime of the SQL query
/// * `DB` - Database type (Postgres, MySql, or Sqlite)
/// * `O` - Output type (the struct being selected)
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
/// use rust_decimal::Decimal;
///
/// // Works the same for all databases
/// let results = MyTable::where_query_ext("price > {}")
///     .bind_proxy(Decimal::from_str("10.00").unwrap())
///     .fetch_all(&pool)
///     .await?;
/// ```
pub trait EnhancedQuery<'q, DB, O>: Sized
where
    DB: sqlx::Database,
    O: Send + Unpin,
{
    /// Create an enhanced query from a SQLx QueryAs
    fn from_query_as(inner: sqlx::query::QueryAs<'q, DB, O, <DB as sqlx::database::HasArguments<'q>>::Arguments>) -> Self;

    /// Bind a value with automatic type conversion.
    ///
    /// This method accepts any type that implements `BindProxy` and automatically
    /// converts it to a database-compatible value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rust_decimal::Decimal;
    ///
    /// query.bind_proxy(Decimal::from_str("123.45").unwrap())
    ///     .fetch_one(&pool)
    ///     .await?;
    /// ```
    fn bind_proxy<T: BindProxy<DB>>(self, value: T) -> Self
    where
        T: Clone;

    /// Bind a value without conversion (standard SQLx behavior).
    ///
    /// This method is equivalent to SQLx's `bind` method and is provided for
    /// backward compatibility.
    fn bind<T: Encode<'q, DB> + Type<DB> + Send + 'q>(self, value: T) -> Self;

    /// Execute the query and return exactly one row.
    ///
    /// # Errors
    ///
    /// Returns an error if the query returns no rows or more than one row.
    fn fetch_one<'e, E>(self, executor: E) -> impl Future<Output = Result<O, sqlx::Error>>
    where
        'q: 'e,
        O: 'e,
        E: Executor<'e, Database = DB>;

    /// Execute the query and return at most one row.
    ///
    /// Returns `Ok(None)` if the query returns no rows.
    fn fetch_optional<'e, E>(self, executor: E) -> impl Future<Output = Result<Option<O>, sqlx::Error>>
    where
        'q: 'e,
        O: 'e,
        E: Executor<'e, Database = DB>;

    /// Execute the query and return all rows.
    fn fetch_all<'e, E>(self, executor: E) -> impl Future<Output = Result<Vec<O>, sqlx::Error>>
    where
        'q: 'e,
        O: 'e,
        E: Executor<'e, Database = DB>;
}
