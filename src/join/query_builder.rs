//! Fluent query builder for JOIN operations.
//!
//! Provides a type-safe builder pattern for constructing and executing
//! JOIN queries that return entity tuples.

use super::{JoinType, JoinSqlGenerator, JoinTuple2};
use super::sql_generator::SchemeAccessor;
use crate::{prepare_where, get_or_insert_sql};
use sqlx::{Database, Pool, Error};
use std::marker::PhantomData;

#[cfg(feature = "postgres")]
use sqlx::Postgres;

#[cfg(feature = "mysql")]
use sqlx::MySql;

#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

/// Fluent query builder for JOIN queries returning entity tuples.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::EnhancedCrud;
///
/// let results: Vec<(Option<Order>, Option<Customer>)> = Order::join_inner::<Customer>(
///     "orders.customer_id = customers.id"
/// )
/// .where_("orders.status = {}", &["completed"])
/// .fetch_all(&pool)
/// .await?;
/// ```
pub struct JoinQueryBuilder<'a, A, B, DB>
where
    A: SchemeAccessor,
    B: SchemeAccessor,
    DB: Database,
{
    join_type: JoinType,
    join_condition: String,
    where_clause: Option<String>,
    where_params: Vec<String>,
    _phantom_a: PhantomData<A>,
    _phantom_b: PhantomData<B>,
    _phantom_db: PhantomData<&'a DB>,
}

#[cfg(feature = "postgres")]
impl<'a, A, B> JoinQueryBuilder<'a, A, B, Postgres>
where
    A: SchemeAccessor + Unpin + Send,
    B: SchemeAccessor + Unpin + Send,
{
    /// Create a new JOIN query builder.
    pub fn new(join_type: JoinType, condition: &str) -> Self {
        Self {
            join_type,
            join_condition: condition.to_string(),
            where_clause: None,
            where_params: Vec::new(),
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
            _phantom_db: PhantomData,
        }
    }

    /// Add a WHERE clause with the given statement and parameters.
    ///
    /// The statement should use "{}" as parameter placeholders.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .where_("orders.status = {} AND customers.region = {}", &["completed", "north"])
    /// ```
    pub fn where_(mut self, clause: &str, params: &[&str]) -> Self {
        self.where_clause = Some(clause.to_string());
        self.where_params = params.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the SQL query and return a cached `&'static str`.
    fn build(&self) -> &'static str {
        let generator = JoinSqlGenerator::new::<A, B>(self.join_type, &self.join_condition);

        let where_clause = self.where_clause.as_ref().map(|clause| {
            format!("WHERE {}", prepare_where(clause, 1))
        });

        let sql = generator.gen_full_query(where_clause.as_deref());

        // Include join type in cache key to avoid reusing wrong JOIN type SQL
        let cache_key = format!(
            "join-{}-{}-{}-where-{}",
            self.join_type,
            A::get_scheme().table_name(),
            B::get_scheme().table_name(),
            self.where_clause.as_ref().unwrap_or(&String::new())
        );

        get_or_insert_sql(cache_key, || sql)
    }

    /// Execute the query and fetch all results.
    ///
    /// # Returns
    ///
    /// A `Vec` of JoinTuple2 where each element contains Option<T> to handle
    /// NULL values from LEFT/RIGHT/FULL joins.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = Order::join_inner::<Customer>("...")
    ///     .fetch_all(&pool)
    ///     .await?;
    ///
    /// for result in results {
    ///     if let (Some(order), Some(customer)) = (&result.0, &result.1) {
    ///         println!("Order {} by {}", order.id, customer.name);
    ///     }
    /// }
    /// ```
    pub async fn fetch_all(
        self,
        pool: &Pool<Postgres>
    ) -> Result<Vec<JoinTuple2<A, B>>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_all(pool).await
    }

    /// Execute the query and fetch exactly one result.
    ///
    /// # Returns
    ///
    /// An error if no results or more than one result is returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = Order::join_inner::<Customer>("orders.id = {} AND customers.id = {}")
    ///     .where_("orders.id = {}", &["order-123"])
    ///     .fetch_one(&pool)
    ///     .await?;
    ///
    /// if let (Some(order), Some(customer)) = (&result.0, &result.1) {
    ///     println!("Order {} by {}", order.id, customer.name);
    /// }
    /// ```
    pub async fn fetch_one(
        self,
        pool: &Pool<Postgres>
    ) -> Result<JoinTuple2<A, B>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_one(pool).await
    }

    /// Execute the query and fetch at most one result.
    ///
    /// # Returns
    ///
    /// `Ok(None)` if no results are found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = Order::join_left::<Customer>("orders.customer_id = customers.id")
    ///     .where_("orders.id = {}", &["order-123"])
    ///     .fetch_optional(&pool)
    ///     .await?;
    /// ```
    pub async fn fetch_optional(
        self,
        pool: &Pool<Postgres>
    ) -> Result<Option<JoinTuple2<A, B>>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_optional(pool).await
    }
}

// ============================================================================
// MySQL implementation
// ============================================================================

#[cfg(feature = "mysql")]
impl<'a, A, B> JoinQueryBuilder<'a, A, B, MySql>
where
    A: SchemeAccessor + Unpin + Send,
    B: SchemeAccessor + Unpin + Send,
{
    /// Create a new JOIN query builder for MySQL.
    pub fn new(join_type: JoinType, condition: &str) -> Self {
        Self {
            join_type,
            join_condition: condition.to_string(),
            where_clause: None,
            where_params: Vec::new(),
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
            _phantom_db: PhantomData,
        }
    }

    /// Add a WHERE clause with the given statement and parameters.
    pub fn where_(mut self, clause: &str, params: &[&str]) -> Self {
        self.where_clause = Some(clause.to_string());
        self.where_params = params.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the SQL query and return a cached `&'static str`.
    fn build(&self) -> &'static str {
        let generator = JoinSqlGenerator::new::<A, B>(self.join_type, &self.join_condition);

        let where_clause = self.where_clause.as_ref().map(|clause| {
            format!("WHERE {}", prepare_where(clause, 1))
        });

        let sql = generator.gen_full_query(where_clause.as_deref());

        // Include join type in cache key to avoid reusing wrong JOIN type SQL
        let cache_key = format!(
            "join-{}-{}-{}-where-{}",
            self.join_type,
            A::get_scheme().table_name(),
            B::get_scheme().table_name(),
            self.where_clause.as_ref().unwrap_or(&String::new())
        );

        get_or_insert_sql(cache_key, || sql)
    }

    /// Execute the query and fetch all results.
    pub async fn fetch_all(
        self,
        pool: &Pool<MySql>
    ) -> Result<Vec<JoinTuple2<A, B>>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_all(pool).await
    }

    /// Execute the query and fetch exactly one result.
    pub async fn fetch_one(
        self,
        pool: &Pool<MySql>
    ) -> Result<JoinTuple2<A, B>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_one(pool).await
    }

    /// Execute the query and fetch at most one result.
    pub async fn fetch_optional(
        self,
        pool: &Pool<MySql>
    ) -> Result<Option<JoinTuple2<A, B>>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_optional(pool).await
    }
}

// ============================================================================
// SQLite implementation
// ============================================================================

#[cfg(feature = "sqlite")]
impl<'a, A, B> JoinQueryBuilder<'a, A, B, Sqlite>
where
    A: SchemeAccessor + Unpin + Send,
    B: SchemeAccessor + Unpin + Send,
{
    /// Create a new JOIN query builder for SQLite.
    pub fn new(join_type: JoinType, condition: &str) -> Self {
        Self {
            join_type,
            join_condition: condition.to_string(),
            where_clause: None,
            where_params: Vec::new(),
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
            _phantom_db: PhantomData,
        }
    }

    /// Add a WHERE clause with the given statement and parameters.
    pub fn where_(mut self, clause: &str, params: &[&str]) -> Self {
        self.where_clause = Some(clause.to_string());
        self.where_params = params.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the SQL query and return a cached `&'static str`.
    fn build(&self) -> &'static str {
        let generator = JoinSqlGenerator::new::<A, B>(self.join_type, &self.join_condition);

        let where_clause = self.where_clause.as_ref().map(|clause| {
            format!("WHERE {}", prepare_where(clause, 1))
        });

        let sql = generator.gen_full_query(where_clause.as_deref());

        // Include join type in cache key to avoid reusing wrong JOIN type SQL
        let cache_key = format!(
            "join-{}-{}-{}-where-{}",
            self.join_type,
            A::get_scheme().table_name(),
            B::get_scheme().table_name(),
            self.where_clause.as_ref().unwrap_or(&String::new())
        );

        get_or_insert_sql(cache_key, || sql)
    }

    /// Execute the query and fetch all results.
    pub async fn fetch_all(
        self,
        pool: &Pool<Sqlite>
    ) -> Result<Vec<JoinTuple2<A, B>>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_all(pool).await
    }

    /// Execute the query and fetch exactly one result.
    pub async fn fetch_one(
        self,
        pool: &Pool<Sqlite>
    ) -> Result<JoinTuple2<A, B>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_one(pool).await
    }

    /// Execute the query and fetch at most one result.
    pub async fn fetch_optional(
        self,
        pool: &Pool<Sqlite>
    ) -> Result<Option<JoinTuple2<A, B>>, Error> {
        let sql = self.build();
        let mut query = sqlx::query_as::<_, JoinTuple2<A, B>>(sql);

        for param in &self.where_params {
            query = query.bind(param);
        }

        query.fetch_optional(pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests will be added once the derive macro generates SchemeAccessor implementations
    // for test structs. For now, the basic structure is in place.
}
