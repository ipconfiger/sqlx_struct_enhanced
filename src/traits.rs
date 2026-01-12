use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;

#[cfg(feature = "postgres")]
use sqlx::postgres::Postgres;

#[cfg(feature = "mysql")]
#[allow(unused_imports)]  // May be unused when multiple features are enabled
use sqlx::mysql::MySql;

#[cfg(feature = "sqlite")]
#[allow(unused_imports)]  // May be unused when multiple features are enabled
use sqlx::sqlite::Sqlite;

// Re-export the concrete proxy types for PostgreSQL
#[cfg(feature = "postgres")]
pub use crate::proxy::{EnhancedQueryAsPostgres, BindProxy, BindValue};

// Re-export the concrete proxy types for MySQL
#[cfg(all(feature = "mysql", not(feature = "postgres")))]
pub use crate::proxy::{EnhancedQueryAsMySql, BindProxy, BindValue};

// Re-export the concrete proxy types for SQLite
#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub use crate::proxy::{EnhancedQueryAsSqlite, BindProxy, BindValue};

#[cfg(feature = "postgres")]
pub trait EnhancedCrud {
    fn insert_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn update_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn delete_bind(&mut self) ->  Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn by_pk<'q>() -> QueryAs<'q, Postgres, Self, <Postgres as HasArguments<'q>>::Arguments> where Self: Sized;
    fn make_query(sql: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn make_execute(sql: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn count_query(statement: &str) -> QueryAs<'_, Postgres, (i64,), <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn delete_where_query(statement: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_delete(ids: &[String]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_insert(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_update(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_select(ids: &[String]) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn agg_query() -> crate::aggregate::AggQueryBuilder<'static, Postgres> where Self: Sized;

    /// Start an INNER JOIN with another table, returning a query builder.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The other entity type to join with (must implement EnhancedCrud)
    ///
    /// # Returns
    ///
    /// A `JoinQueryBuilder` that can be used to execute the query.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use sqlx_struct_enhanced::{EnhancedCrud, join::JoinTuple2};
    ///
    /// let results: Vec<JoinTuple2<Order, Customer>> = Order::join_inner::<Customer>(
    ///     "orders.customer_id = customers.id"
    /// )
    /// .fetch_all(&pool)
    /// .await?;
    /// ```
    #[cfg(feature = "join_queries")]
    fn join_inner<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Postgres>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a LEFT JOIN with another table.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = Order::join_left::<Customer>("orders.customer_id = customers.id")
    ///     .fetch_all(&pool)
    ///     .await?;
    /// ```
    #[cfg(feature = "join_queries")]
    fn join_left<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Postgres>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a RIGHT JOIN with another table.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = Order::join_right::<Customer>("orders.customer_id = customers.id")
    ///     .fetch_all(&pool)
    ///     .await?;
    /// ```
    #[cfg(feature = "join_queries")]
    fn join_right<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Postgres>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a FULL JOIN with another table.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = Order::join_full::<Customer>("orders.customer_id = customers.id")
    ///     .fetch_all(&pool)
    ///     .await?;
    /// ```
    #[cfg(feature = "join_queries")]
    fn join_full<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Postgres>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;
}

#[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
pub trait EnhancedCrud {
    fn insert_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn update_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn delete_bind(&mut self) ->  Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn by_pk<'q>() -> QueryAs<'q, MySql, Self, <MySql as HasArguments<'q>>::Arguments> where Self: Sized;
    fn make_query(sql: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn make_execute(sql: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn where_query(statement: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn count_query(statement: &str) -> QueryAs<'_, MySql, (i64,), <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn delete_where_query(statement: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_delete(ids: &[String]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_insert(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_update(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_select(ids: &[String]) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn agg_query() -> crate::aggregate::AggQueryBuilder<'static, MySql> where Self: Sized;

    /// Start an INNER JOIN with another table.
    #[cfg(feature = "join_queries")]
    fn join_inner<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, MySql>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a LEFT JOIN with another table.
    #[cfg(feature = "join_queries")]
    fn join_left<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, MySql>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a RIGHT JOIN with another table.
    #[cfg(feature = "join_queries")]
    fn join_right<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, MySql>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a FULL JOIN with another table.
    ///
    /// Note: MySQL does not support FULL JOIN natively.
    /// This method is provided for API compatibility but will generate
    /// invalid SQL. Use LEFT JOIN UNION RIGHT JOIN pattern instead.
    #[cfg(feature = "join_queries")]
    fn join_full<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, MySql>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;
}

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub trait EnhancedCrud {
    fn insert_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn update_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn delete_bind(&mut self) ->  Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn by_pk<'q>() -> QueryAs<'q, Sqlite, Self, <Sqlite as HasArguments<'q>>::Arguments> where Self: Sized;
    fn make_query(sql: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn make_execute(sql: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn where_query(statement: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn count_query(statement: &str) -> QueryAs<'_, Sqlite, (i64,), <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn delete_where_query(statement: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_delete(ids: &[String]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_insert(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_update(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_select(ids: &[String]) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn agg_query() -> crate::aggregate::AggQueryBuilder<'static, Sqlite> where Self: Sized;

    /// Start an INNER JOIN with another table.
    #[cfg(feature = "join_queries")]
    fn join_inner<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Sqlite>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a LEFT JOIN with another table.
    #[cfg(feature = "join_queries")]
    fn join_left<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Sqlite>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a RIGHT JOIN with another table.
    ///
    /// Note: SQLite does not support RIGHT JOIN natively.
    /// This method is provided for API compatibility but will generate
    /// invalid SQL. Use LEFT JOIN with reversed table order instead.
    #[cfg(feature = "join_queries")]
    fn join_right<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Sqlite>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;

    /// Start a FULL JOIN with another table.
    ///
    /// Note: SQLite does not support FULL JOIN natively.
    /// This method is provided for API compatibility but will generate
    /// invalid SQL. Use LEFT JOIN UNION LEFT JOIN pattern instead.
    #[cfg(feature = "join_queries")]
    fn join_full<T>(condition: &str) -> crate::join::JoinQueryBuilder<'static, Self, T, Sqlite>
    where
        Self: Sized + crate::join::SchemeAccessor,
        T: Sized + crate::join::SchemeAccessor + Unpin + Send;
}

// ============================================================================
// Enhanced CRUD Extension Trait (PostgreSQL - Concrete Types)
// ============================================================================

/// Extension trait that provides enhanced query methods with automatic type conversion.
///
/// This trait provides `_ext` versions of query methods that return concrete
/// `EnhancedQueryAsPostgres` wrappers, which support the `bind_proxy` method
/// for automatic type conversion.
///
/// # Note
///
/// Only SELECT/FETCH methods are included because they return data that may
/// need type conversion (e.g., DECIMAL → String). Methods that use `execute()`
/// (INSERT/UPDATE/DELETE) are not included since they don't return data.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
/// use rust_decimal::Decimal;
///
/// // Use enhanced queries with automatic DECIMAL conversion
/// let orders = Order::where_query_ext("amount BETWEEN {} AND {}")
///     .bind_proxy(Decimal::from_str("100.00").unwrap())
///     .bind_proxy(Decimal::from_str("200.00").unwrap())
///     .fetch_all(&pool)
///     .await?;
/// ```
///
/// This trait is automatically implemented for all types that implement `EnhancedCrud`.
#[cfg(feature = "postgres")]
pub trait EnhancedCrudExt: EnhancedCrud {
    /// Enhanced version of `where_query` that returns a wrapper with `bind_proxy` support.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = MyTable::where_query_ext("status = {}")
    ///     .bind_proxy(MyStatus::Active)
    ///     .fetch_all(&pool)
    ///     .await?;
    /// ```
    fn where_query_ext(statement: &str) -> EnhancedQueryAsPostgres<'_, Self>
    where
        Self: Sized;

    /// Enhanced version of `by_pk` that returns a wrapper with `bind_proxy` support.
    fn by_pk_ext<'q>() -> EnhancedQueryAsPostgres<'q, Self>
    where
        Self: Sized;

    /// Enhanced version of `make_query` that returns a wrapper with `bind_proxy` support.
    fn make_query_ext(sql: &str) -> EnhancedQueryAsPostgres<'_, Self>
    where
        Self: Sized;

    /// Enhanced version of `count_query` that returns a wrapper with `bind_proxy` support.
    fn count_query_ext(statement: &str) -> EnhancedQueryAsPostgres<'_, (i64,)>
    where
        Self: Sized;
}

// Blanket implementation for all EnhancedCrud types (PostgreSQL only)
#[cfg(feature = "postgres")]
impl<T: EnhancedCrud + Unpin + Send> EnhancedCrudExt for T {
    fn where_query_ext(statement: &str) -> EnhancedQueryAsPostgres<'_, T>
    where
        T: Sized,
    {
        let query = T::where_query(statement);
        EnhancedQueryAsPostgres::from_query_as(query)
    }

    fn by_pk_ext<'q>() -> EnhancedQueryAsPostgres<'q, T>
    where
        T: Sized,
    {
        let query = T::by_pk();
        EnhancedQueryAsPostgres::from_query_as(query)
    }

    fn make_query_ext(sql: &str) -> EnhancedQueryAsPostgres<'_, T>
    where
        T: Sized,
    {
        let query = T::make_query(sql);
        EnhancedQueryAsPostgres::from_query_as(query)
    }

    fn count_query_ext(statement: &str) -> EnhancedQueryAsPostgres<'_, (i64,)>
    where
        T: Sized,
    {
        let query = T::count_query(statement);
        EnhancedQueryAsPostgres::from_query_as(query)
    }
}

// ============================================================================
// Enhanced CRUD Extension Trait (MySQL - Concrete Types)
// ============================================================================

/// Extension trait that provides enhanced query methods with automatic type conversion.
///
/// This trait provides `_ext` versions of query methods that return concrete
/// `EnhancedQueryAsMySql` wrappers, which support the `bind_proxy` method
/// for automatic type conversion.
///
/// # Note
///
/// Only SELECT/FETCH methods are included because they return data that may
/// need type conversion (e.g., DECIMAL → String). Methods that use `execute()`
/// (INSERT/UPDATE/DELETE) are not included since they don't return data.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
/// use rust_decimal::Decimal;
///
/// // Use enhanced queries with automatic DECIMAL conversion
/// let orders = Order::where_query_ext("amount BETWEEN {} AND {}")
///     .bind_proxy(Decimal::from_str("100.00").unwrap())
///     .bind_proxy(Decimal::from_str("200.00").unwrap())
///     .fetch_all(&pool)
///     .await?;
/// ```
///
/// This trait is automatically implemented for all types that implement `EnhancedCrud`.
#[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
pub trait EnhancedCrudExt: EnhancedCrud {
    /// Enhanced version of `where_query` that returns a wrapper with `bind_proxy` support.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = MyTable::where_query_ext("status = {}")
    ///     .bind_proxy(MyStatus::Active)
    ///     .fetch_all(&pool)
    ///     .await?;
    /// ```
    fn where_query_ext(statement: &str) -> EnhancedQueryAsMySql<'_, Self>
    where
        Self: Sized;

    /// Enhanced version of `by_pk` that returns a wrapper with `bind_proxy` support.
    fn by_pk_ext<'q>() -> EnhancedQueryAsMySql<'q, Self>
    where
        Self: Sized;

    /// Enhanced version of `make_query` that returns a wrapper with `bind_proxy` support.
    fn make_query_ext(sql: &str) -> EnhancedQueryAsMySql<'_, Self>
    where
        Self: Sized;

    /// Enhanced version of `count_query` that returns a wrapper with `bind_proxy` support.
    fn count_query_ext(statement: &str) -> EnhancedQueryAsMySql<'_, (i64,)>
    where
        Self: Sized;
}

// Blanket implementation for all EnhancedCrud types (MySQL only)
#[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
impl<T: EnhancedCrud + Unpin + Send> EnhancedCrudExt for T {
    fn where_query_ext(statement: &str) -> EnhancedQueryAsMySql<'_, T>
    where
        T: Sized,
    {
        let query = T::where_query(statement);
        EnhancedQueryAsMySql::from_query_as(query)
    }

    fn by_pk_ext<'q>() -> EnhancedQueryAsMySql<'q, T>
    where
        T: Sized,
    {
        let query = T::by_pk();
        EnhancedQueryAsMySql::from_query_as(query)
    }

    fn make_query_ext(sql: &str) -> EnhancedQueryAsMySql<'_, T>
    where
        T: Sized,
    {
        let query = T::make_query(sql);
        EnhancedQueryAsMySql::from_query_as(query)
    }

    fn count_query_ext(statement: &str) -> EnhancedQueryAsMySql<'_, (i64,)>
    where
        T: Sized,
    {
        let query = T::count_query(statement);
        EnhancedQueryAsMySql::from_query_as(query)
    }
}

// ============================================================================
// Enhanced CRUD Extension Trait (SQLite - Concrete Types)
// ============================================================================

/// Extension trait that provides enhanced query methods with automatic type conversion.
///
/// This trait provides `_ext` versions of query methods that return concrete
/// `EnhancedQueryAsSqlite` wrappers, which support the `bind_proxy` method
/// for automatic type conversion.
///
/// # Note
///
/// Only SELECT/FETCH methods are included because they return data that may
/// need type conversion (e.g., DECIMAL → String). Methods that use `execute()`
/// (INSERT/UPDATE/DELETE) are not included since they don't return data.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
/// use rust_decimal::Decimal;
///
/// // Use enhanced queries with automatic DECIMAL conversion
/// let orders = Order::where_query_ext("amount BETWEEN {} AND {}")
///     .bind_proxy(Decimal::from_str("100.00").unwrap())
///     .bind_proxy(Decimal::from_str("200.00").unwrap())
///     .fetch_all(&pool)
///     .await?;
/// ```
///
/// This trait is automatically implemented for all types that implement `EnhancedCrud`.
#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub trait EnhancedCrudExt: EnhancedCrud {
    /// Enhanced version of `where_query` that returns a wrapper with `bind_proxy` support.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = MyTable::where_query_ext("status = {}")
    ///     .bind_proxy(MyStatus::Active)
    ///     .fetch_all(&pool)
    ///     .await?;
    /// ```
    fn where_query_ext(statement: &str) -> EnhancedQueryAsSqlite<'_, Self>
    where
        Self: Sized;

    /// Enhanced version of `by_pk` that returns a wrapper with `bind_proxy` support.
    fn by_pk_ext<'q>() -> EnhancedQueryAsSqlite<'q, Self>
    where
        Self: Sized;

    /// Enhanced version of `make_query` that returns a wrapper with `bind_proxy` support.
    fn make_query_ext(sql: &str) -> EnhancedQueryAsSqlite<'_, Self>
    where
        Self: Sized;

    /// Enhanced version of `count_query` that returns a wrapper with `bind_proxy` support.
    fn count_query_ext(statement: &str) -> EnhancedQueryAsSqlite<'_, (i64,)>
    where
        Self: Sized;
}

// Blanket implementation for all EnhancedCrud types (SQLite only)
#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
impl<T: EnhancedCrud + Unpin + Send> EnhancedCrudExt for T {
    fn where_query_ext(statement: &str) -> EnhancedQueryAsSqlite<'_, T>
    where
        T: Sized,
    {
        let query = T::where_query(statement);
        EnhancedQueryAsSqlite::from_query_as(query)
    }

    fn by_pk_ext<'q>() -> EnhancedQueryAsSqlite<'q, T>
    where
        T: Sized,
    {
        let query = T::by_pk();
        EnhancedQueryAsSqlite::from_query_as(query)
    }

    fn make_query_ext(sql: &str) -> EnhancedQueryAsSqlite<'_, T>
    where
        T: Sized,
    {
        let query = T::make_query(sql);
        EnhancedQueryAsSqlite::from_query_as(query)
    }

    fn count_query_ext(statement: &str) -> EnhancedQueryAsSqlite<'_, (i64,)>
    where
        T: Sized,
    {
        let query = T::count_query(statement);
        EnhancedQueryAsSqlite::from_query_as(query)
    }
}
