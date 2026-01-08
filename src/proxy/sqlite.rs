// SQLite Enhanced Query Implementation
//
// This module provides the SQLite-specific implementation of the EnhancedQuery trait,
// which wraps SQLx's QueryAs for SQLite and provides automatic type conversion for
// bind parameters (e.g., DECIMAL → String for NUMERIC columns).

use sqlx::{Sqlite, Encode, Type, Executor, query::QueryAs};
use sqlx::database::HasArguments;
use sqlx::sqlite::SqliteRow;
use std::future::Future;

use crate::proxy::{BindProxy, BindValue, EnhancedQuery};

/// Enhanced query wrapper for SQLite SELECT queries with automatic type conversion.
///
/// This type wraps SQLx's `QueryAs` for SQLite and provides the `bind_proxy` method,
/// which automatically converts complex types (like DECIMAL) to database-compatible values.
///
/// # Type Parameters
///
/// * `'q` - Lifetime of the SQL query
/// * `O` - Output type (the struct being selected)
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
/// use rust_decimal::Decimal;
///
/// // Automatically convert rust_decimal::Decimal to String
/// let orders = Order::where_query_ext("amount BETWEEN {} AND {}")
///     .bind_proxy(Decimal::from_str("100.00").unwrap())
///     .bind_proxy(Decimal::from_str("200.00").unwrap())
///     .fetch_all(&pool)
///     .await?;
/// ```
pub struct EnhancedQueryAsSqlite<'q, O> {
    inner: QueryAs<'q, Sqlite, O, <Sqlite as HasArguments<'q>>::Arguments>,
}

impl<'q, O> EnhancedQueryAsSqlite<'q, O>
where
    O: Send + Unpin,
{
    /// Create an enhanced query from a SQLx QueryAs
    pub fn from_query_as(inner: QueryAs<'q, Sqlite, O, <Sqlite as HasArguments<'q>>::Arguments>) -> Self {
        Self { inner }
    }

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
    /// let decimal = Decimal::from_str("123.456").unwrap();
    /// query.bind_proxy(decimal)  // auto-converts to String
    ///     .fetch_one(&pool)
    ///     .await?;
    /// ```
    pub fn bind_proxy<T: BindProxy<Sqlite>>(mut self, value: T) -> Self
    where
        T: Clone,
    {
        let bind_value = value.into_bind_value();
        self = match bind_value {
            // Existing variants
            BindValue::String(s) => self.bind(s),
            BindValue::I32(i) => self.bind(i),
            BindValue::I64(i) => self.bind(i),
            BindValue::F64(f) => self.bind(f),
            BindValue::Bool(b) => self.bind(b),
            BindValue::Decimal(s) => self.bind(s),

            // Additional numeric types
            BindValue::I8(i) => self.bind(i),
            BindValue::I16(i) => self.bind(i),
            BindValue::F32(f) => self.bind(f),

            // Date/time types (all bind as String)
            BindValue::NaiveDate(s) => self.bind(s),
            BindValue::NaiveTime(s) => self.bind(s),
            BindValue::NaiveDateTime(s) => self.bind(s),
            BindValue::DateTimeUtc(s) => self.bind(s),

            // JSON (bind as String)
            BindValue::Json(s) => self.bind(s),

            // Binary (bind as Vec<u8>)
            BindValue::Binary(bytes) => self.bind(bytes),

            // UUID (bind as String)
            BindValue::Uuid(s) => self.bind(s),

            BindValue::_Marker(_) => panic!("BindValue::_Marker should never be used"),
        };
        self
    }

    /// Bind a value without conversion (standard SQLx behavior).
    ///
    /// This method is equivalent to SQLx's `bind` method and is provided for
    /// backward compatibility.
    pub fn bind<T: Encode<'q, Sqlite> + Type<Sqlite> + Send + 'q>(mut self, value: T) -> Self {
        self.inner = self.inner.bind(value);
        self
    }
}

// ============================================================================
// Implement EnhancedQuery trait for SQLite
// ============================================================================

impl<'q, O> EnhancedQuery<'q, Sqlite, O> for EnhancedQueryAsSqlite<'q, O>
where
    O: Send + Unpin + for<'r> sqlx::FromRow<'r, SqliteRow> + sqlx::Decode<'q, Sqlite> + sqlx::Type<Sqlite>,
{
    fn from_query_as(inner: QueryAs<'q, Sqlite, O, <Sqlite as HasArguments<'q>>::Arguments>) -> Self {
        Self { inner }
    }

    fn bind_proxy<T: BindProxy<Sqlite>>(mut self, value: T) -> Self
    where
        T: Clone,
    {
        let bind_value = value.into_bind_value();
        match bind_value {
            // Existing variants
            BindValue::String(s) => {
                self.inner = self.inner.bind(s);
                self
            }
            BindValue::I32(i) => {
                self.inner = self.inner.bind(i);
                self
            }
            BindValue::I64(i) => {
                self.inner = self.inner.bind(i);
                self
            }
            BindValue::F64(f) => {
                self.inner = self.inner.bind(f);
                self
            }
            BindValue::Bool(b) => {
                self.inner = self.inner.bind(b);
                self
            }
            BindValue::Decimal(s) => {
                self.inner = self.inner.bind(s);
                self
            }

            // Additional numeric types
            BindValue::I8(i) => {
                self.inner = self.inner.bind(i);
                self
            }
            BindValue::I16(i) => {
                self.inner = self.inner.bind(i);
                self
            }
            BindValue::F32(f) => {
                self.inner = self.inner.bind(f);
                self
            }

            // Date/time types (all bind as String)
            BindValue::NaiveDate(s) => {
                self.inner = self.inner.bind(s);
                self
            }
            BindValue::NaiveTime(s) => {
                self.inner = self.inner.bind(s);
                self
            }
            BindValue::NaiveDateTime(s) => {
                self.inner = self.inner.bind(s);
                self
            }
            BindValue::DateTimeUtc(s) => {
                self.inner = self.inner.bind(s);
                self
            }

            // JSON (bind as String)
            BindValue::Json(s) => {
                self.inner = self.inner.bind(s);
                self
            }

            // Binary (bind as Vec<u8>)
            BindValue::Binary(bytes) => {
                self.inner = self.inner.bind(bytes);
                self
            }

            // UUID (bind as String)
            BindValue::Uuid(s) => {
                self.inner = self.inner.bind(s);
                self
            }

            BindValue::_Marker(_) => {
                panic!("BindValue::_Marker should never be used");
            }
        }
    }

    fn bind<T: Encode<'q, Sqlite> + Type<Sqlite> + Send + 'q>(mut self, value: T) -> Self {
        self.inner = self.inner.bind(value);
        self
    }

    fn fetch_one<'e, E>(self, executor: E) -> impl Future<Output = Result<O, sqlx::Error>>
    where
        'q: 'e,
        O: 'e,
        E: Executor<'e, Database = Sqlite>,
    {
        async move {
            self.inner.fetch_one(executor).await
        }
    }

    fn fetch_optional<'e, E>(self, executor: E) -> impl Future<Output = Result<Option<O>, sqlx::Error>>
    where
        'q: 'e,
        O: 'e,
        E: Executor<'e, Database = Sqlite>,
    {
        async move {
            self.inner.fetch_optional(executor).await
        }
    }

    fn fetch_all<'e, E>(self, executor: E) -> impl Future<Output = Result<Vec<O>, sqlx::Error>>
    where
        'q: 'e,
        O: 'e,
        E: Executor<'e, Database = Sqlite>,
    {
        async move {
            self.inner.fetch_all(executor).await
        }
    }
}
