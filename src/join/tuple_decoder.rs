//! Custom FromRow implementations for JOIN query result tuples.
//!
//! This module provides newtype wrappers that implement FromRow for entity tuples,
//! handling table-qualified column names like "orders.id".

use sqlx::Error;

#[cfg(feature = "postgres")]
use sqlx::postgres::PgRow;

#[cfg(feature = "mysql")]
use sqlx::mysql::MySqlRow;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqliteRow;

use super::SchemeAccessor;

/// Result of a 2-table JOIN query with both entities wrapped in Option.
///
/// # Type Parameters
///
/// * `A` - First entity type (must implement SchemeAccessor)
/// * `B` - Second entity type (must implement SchemeAccessor)
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::EnhancedCrud;
/// use sqlx_struct_enhanced::join::JoinTuple2;
///
/// let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>("...")
///     .fetch_all(&pool)
///     .await?;
///
/// for result in results {
///     if let (Some(order), Some(customer)) = (&result.0, &result.1) {
///         println!("Order {} by {}", order.id, customer.name);
///     }
/// }
/// ```
///
/// # Accessing Results
///
/// The tuple provides public access to both entities:
/// - `result.0` - First entity (Option<Order>)
/// - `result.1` - Second entity (Option<Customer>)
///
/// For INNER joins, both will always be `Some(value)`.
/// For LEFT/RIGHT joins, one may be `None`.
/// For FULL joins, either may be `None`.
pub struct JoinTuple2<A, B>(
    /// First entity (may be None for LEFT/RIGHT/FULL joins)
    pub Option<A>,
    /// Second entity (may be None for LEFT/RIGHT/FULL joins)
    pub Option<B>,
);

/// Result of a 3-table JOIN query.
pub struct JoinTuple3<A, B, C>(
    pub Option<A>,
    pub Option<B>,
    pub Option<C>,
);

/// Result of a 4-table JOIN query.
pub struct JoinTuple4<A, B, C, D>(
    pub Option<A>,
    pub Option<B>,
    pub Option<C>,
    pub Option<D>,
);

/// Result of a 5-table JOIN query.
pub struct JoinTuple5<A, B, C, D, E>(
    pub Option<A>,
    pub Option<B>,
    pub Option<C>,
    pub Option<D>,
    pub Option<E>,
);

// Implement FromRow for PostgreSQL rows for 2-table joins
//
// This implementation uses the SchemeAccessor trait to decode entities
// from qualified column names (e.g., "orders.id", "customers.name").
#[cfg(feature = "postgres")]
impl<'r, A, B> sqlx::FromRow<'r, PgRow> for JoinTuple2<A, B>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        // Decode entity A from its qualified columns
        // Returns Ok(None) if the entity has all NULL columns (LEFT/RIGHT/FULL join)
        let entity_a = match A::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        // Decode entity B from its qualified columns
        let entity_b = match B::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple2(entity_a, entity_b))
    }
}

// Implement FromRow for PostgreSQL rows for 3-table joins
#[cfg(feature = "postgres")]
impl<'r, A, B, C> sqlx::FromRow<'r, PgRow> for JoinTuple3<A, B, C>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple3(entity_a, entity_b, entity_c))
    }
}

// Implement FromRow for PostgreSQL rows for 4-table joins
#[cfg(feature = "postgres")]
impl<'r, A, B, C, D> sqlx::FromRow<'r, PgRow> for JoinTuple4<A, B, C, D>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
    D: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_d = match D::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple4(entity_a, entity_b, entity_c, entity_d))
    }
}

// Implement FromRow for PostgreSQL rows for 5-table joins
#[cfg(feature = "postgres")]
impl<'r, A, B, C, D, E> sqlx::FromRow<'r, PgRow> for JoinTuple5<A, B, C, D, E>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
    D: SchemeAccessor + Send + Unpin,
    E: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_d = match D::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_e = match E::decode_from_qualified_row_pg(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple5(entity_a, entity_b, entity_c, entity_d, entity_e))
    }
}

// ============================================================================
// MySQL implementations
// ============================================================================

// Implement FromRow for MySQL rows for 2-table joins
#[cfg(feature = "mysql")]
impl<'r, A, B> sqlx::FromRow<'r, MySqlRow> for JoinTuple2<A, B>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r MySqlRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple2(entity_a, entity_b))
    }
}

// Implement FromRow for MySQL rows for 3-table joins
#[cfg(feature = "mysql")]
impl<'r, A, B, C> sqlx::FromRow<'r, MySqlRow> for JoinTuple3<A, B, C>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r MySqlRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple3(entity_a, entity_b, entity_c))
    }
}

// Implement FromRow for MySQL rows for 4-table joins
#[cfg(feature = "mysql")]
impl<'r, A, B, C, D> sqlx::FromRow<'r, MySqlRow> for JoinTuple4<A, B, C, D>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
    D: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r MySqlRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_d = match D::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple4(entity_a, entity_b, entity_c, entity_d))
    }
}

// Implement FromRow for MySQL rows for 5-table joins
#[cfg(feature = "mysql")]
impl<'r, A, B, C, D, E> sqlx::FromRow<'r, MySqlRow> for JoinTuple5<A, B, C, D, E>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
    D: SchemeAccessor + Send + Unpin,
    E: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r MySqlRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_d = match D::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_e = match E::decode_from_qualified_row_mysql(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple5(entity_a, entity_b, entity_c, entity_d, entity_e))
    }
}

// ============================================================================
// SQLite implementations
// ============================================================================

// Implement FromRow for SQLite rows for 2-table joins
#[cfg(feature = "sqlite")]
impl<'r, A, B> sqlx::FromRow<'r, SqliteRow> for JoinTuple2<A, B>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r SqliteRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple2(entity_a, entity_b))
    }
}

// Implement FromRow for SQLite rows for 3-table joins
#[cfg(feature = "sqlite")]
impl<'r, A, B, C> sqlx::FromRow<'r, SqliteRow> for JoinTuple3<A, B, C>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r SqliteRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple3(entity_a, entity_b, entity_c))
    }
}

// Implement FromRow for SQLite rows for 4-table joins
#[cfg(feature = "sqlite")]
impl<'r, A, B, C, D> sqlx::FromRow<'r, SqliteRow> for JoinTuple4<A, B, C, D>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
    D: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r SqliteRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_d = match D::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple4(entity_a, entity_b, entity_c, entity_d))
    }
}

// Implement FromRow for SQLite rows for 5-table joins
#[cfg(feature = "sqlite")]
impl<'r, A, B, C, D, E> sqlx::FromRow<'r, SqliteRow> for JoinTuple5<A, B, C, D, E>
where
    A: SchemeAccessor + Send + Unpin,
    B: SchemeAccessor + Send + Unpin,
    C: SchemeAccessor + Send + Unpin,
    D: SchemeAccessor + Send + Unpin,
    E: SchemeAccessor + Send + Unpin,
{
    fn from_row(row: &'r SqliteRow) -> Result<Self, Error> {
        let entity_a = match A::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_b = match B::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_c = match C::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_d = match D::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        let entity_e = match E::decode_from_qualified_row_sqlite(row) {
            Ok(opt) => opt,
            Err(_) => None,
        };

        Ok(JoinTuple5(entity_a, entity_b, entity_c, entity_d, entity_e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_tuple2_creation() {
        let tuple = JoinTuple2(
            Some("value_a"),
            Some("value_b")
        );
        assert_eq!(tuple.0, Some("value_a"));
        assert_eq!(tuple.1, Some("value_b"));
    }

    #[test]
    fn test_join_tuple2_with_none() {
        let tuple = JoinTuple2::<String, String>(
            Some("value_a".to_string()),
            None
        );
        assert_eq!(tuple.0, Some("value_a".to_string()));
        assert_eq!(tuple.1, None);
    }
}
