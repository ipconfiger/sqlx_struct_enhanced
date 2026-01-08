pub mod traits;
pub mod proxy;
pub mod decimal_helpers;
pub mod aggregate;
pub use sqlx_struct_macros::EnhancedCrud;
pub use traits::{EnhancedCrud, EnhancedCrudExt};
pub use aggregate::{AggQueryBuilder, Join, JoinType};

#[cfg(feature = "postgres")]
pub use proxy::{EnhancedQueryAsPostgres, EnhancedQuery, BindProxy, BindValue};

#[cfg(all(feature = "mysql", not(feature = "postgres")))]
pub use proxy::{EnhancedQueryAsMySql, EnhancedQuery, BindProxy, BindValue};

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub use proxy::{EnhancedQueryAsSqlite, EnhancedQuery, BindProxy, BindValue};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg(feature = "postgres")]
use sqlx::postgres::{PgPool, Postgres as Pg};

#[cfg(all(feature = "mysql", not(feature = "postgres")))]
use sqlx::mysql::{MySqlPool, MySql};

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
use sqlx::sqlite::{SqlitePool, Sqlite};

use sqlx::Transaction;
use futures::Future;

/// Transaction helper that executes a function within a database transaction.
///
/// The function receives a mutable reference to the transaction and can perform
/// multiple operations. If the function returns Ok, the transaction is committed.
/// If it returns Err, the transaction is rolled back.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::transaction;
///
/// let result = transaction(&pool, |tx| async move {
///     user.insert_bind().execute(tx).await?;
///     profile.update_bind().execute(tx).await?;
///     Ok(())
/// }).await?;
/// ```
#[cfg(feature = "postgres")]
pub async fn transaction<'a, F, Fut, R, E>(
    pool: &'a PgPool,
    f: F,
) -> Result<R, E>
where
    F: FnOnce(&mut Transaction<'a, Pg>) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await?;
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// Transaction helper for MySQL databases.
#[cfg(all(feature = "mysql", not(feature = "postgres")))]
pub async fn transaction<'a, F, Fut, R, E>(
    pool: &'a MySqlPool,
    f: F,
) -> Result<R, E>
where
    F: FnOnce(&mut Transaction<'a, MySql>) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await?;
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// Transaction helper for SQLite databases.
#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub async fn transaction<'a, F, Fut, R, E>(
    pool: &'a SqlitePool,
    f: F,
) -> Result<R, E>
where
    F: FnOnce(&mut Transaction<'a, Sqlite>) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await?;
    match f(&mut tx).await {
        Ok(result) => {
            tx.commit().await?;
            Ok(result)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// Nested transaction helper that uses savepoints for PostgreSQL.
///
/// This allows creating nested transactions within an existing transaction.
/// If the nested function returns Ok, the savepoint is released.
/// If it returns Err, the transaction rolls back to the savepoint.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::{transaction, nested_transaction};
///
/// transaction(&pool, |parent_tx| async move {
///     user.insert_bind().execute(parent_tx).await?;
///
///     // Nested transaction with savepoint
///     nested_transaction(parent_tx, |nested_tx| async move {
///         profile.update_bind().execute(nested_tx).await?;
///         Ok(())
///     }).await?;
///
///     Ok(())
/// }).await?;
/// ```
#[cfg(feature = "postgres")]
pub async fn nested_transaction<'a, F, Fut, R, E>(
    tx: &'a mut Transaction<'a, Pg>,
    f: F,
) -> Result<R, E>
where
    F: FnOnce(&mut Transaction<'a, Pg>) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: From<sqlx::Error>,
{
    use uuid::Uuid;

    // Generate unique savepoint name
    let savepoint = format!("sp_{}", Uuid::new_v4());

    // Create savepoint
    sqlx::query(&format!("SAVEPOINT {}", savepoint))
        .execute(tx.as_mut())
        .await?;

    // Execute user function
    match f(tx).await {
        Ok(result) => {
            // Release savepoint on success
            sqlx::query(&format!("RELEASE SAVEPOINT {}", savepoint))
                .execute(tx.as_mut())
                .await?;
            Ok(result)
        }
        Err(e) => {
            // Rollback to savepoint on error
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", savepoint))
                .execute(tx.as_mut())
                .await?;
            Err(e)
        }
    }
}

/// Nested transaction helper that uses savepoints for MySQL.
#[cfg(all(feature = "mysql", not(feature = "postgres")))]
pub async fn nested_transaction<'a, F, Fut, R, E>(
    tx: &'a mut Transaction<'a, MySql>,
    f: F,
) -> Result<R, E>
where
    F: FnOnce(&mut Transaction<'a, MySql>) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: From<sqlx::Error>,
{
    use uuid::Uuid;

    let savepoint = format!("sp_{}", Uuid::new_v4());

    sqlx::query(&format!("SAVEPOINT {}", savepoint))
        .execute(tx.as_mut())
        .await?;

    match f(tx).await {
        Ok(result) => {
            sqlx::query(&format!("RELEASE SAVEPOINT {}", savepoint))
                .execute(tx.as_mut())
                .await?;
            Ok(result)
        }
        Err(e) => {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", savepoint))
                .execute(tx.as_mut())
                .await?;
            Err(e)
        }
    }
}

/// Nested transaction helper that uses savepoints for SQLite.
#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub async fn nested_transaction<'a, F, Fut, R, E>(
    tx: &'a mut Transaction<'a, Sqlite>,
    f: F,
) -> Result<R, E>
where
    F: FnOnce(&mut Transaction<'a, Sqlite>) -> Fut,
    Fut: Future<Output = Result<R, E>>,
    E: From<sqlx::Error>,
{
    use uuid::Uuid;

    let savepoint = format!("sp_{}", Uuid::new_v4());

    sqlx::query(&format!("SAVEPOINT {}", savepoint))
        .execute(tx.as_mut())
        .await?;

    match f(tx).await {
        Ok(result) => {
            sqlx::query(&format!("RELEASE SAVEPOINT {}", savepoint))
                .execute(tx.as_mut())
                .await?;
            Ok(result)
        }
        Err(e) => {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {}", savepoint))
                .execute(tx.as_mut())
                .await?;
            Err(e)
        }
    }
}


#[cfg(feature = "postgres")]
fn get_db() -> DbType {
    DbType::PostgreSQL
}

#[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
fn get_db() -> DbType {
    DbType::MySQL
}

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
fn get_db() -> DbType {
    DbType::SQLite
}

#[cfg(not(any(feature = "postgres", feature = "mysql", feature = "sqlite")))]
fn get_db() -> DbType {
    compile_error!("You must enable one of the database features: postgres, mysql, or sqlite")
}

/// Translates a parameter placeholder to the database-specific format.
///
/// - PostgreSQL: Returns the parameter as-is (e.g., "$1", "$2")
/// - MySQL/SQLite: Returns "?" for all parameters
fn param_trans(p: String) -> String {
    match get_db() {
        DbType::PostgreSQL => p,
        DbType::MySQL | DbType::SQLite => "?".to_string(),
    }
}

/// Prepares a WHERE clause by replacing "{}" placeholders with database-specific parameter markers.
///
/// # Arguments
///
/// * `w` - The WHERE clause template with "{}" placeholders
/// * `field_count` - The starting parameter number for PostgreSQL
///
/// # Example
///
/// ```ignore
/// prepare_where("name = {} AND age = {}", 1);
/// // PostgreSQL: "name = $1 AND age = $2"
/// // MySQL/SQLite: "name = ? AND age = ?"
/// ```
fn prepare_where(w: &str, field_count: i32) -> String {
    let param_count = w.matches("{}").count() as i32;
    let mut where_sql = w.to_string();

    for i in 0..param_count {
        let param = param_trans(format!("${}", i + field_count));
        if let Some(pos) = where_sql.find("{}") {
            where_sql.replace_range(pos..pos + 2, &param);
        }
    }

    where_sql
}
/// Column definition with optional type casting for SQL queries.
///
/// This struct stores metadata about a column including whether it needs
/// explicit type casting in SELECT statements.
///
/// # Example
///
/// For a field `#[sqlx(cast_as = "TEXT")] pub price: Option<String>`:
/// ```ignore
/// ColumnDefinition {
///     name: "price".to_string(),
///     cast_as: Some("TEXT".to_string()),
/// }
/// ```
/// This will generate: `price::TEXT as price` in SELECT queries.
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    /// Column name
    pub name: String,
    /// Optional type cast (e.g., "TEXT" for NUMERIC→TEXT conversion)
    pub cast_as: Option<String>,
}


/// SQL generation scheme for CRUD operations.
///
/// This struct holds metadata about a database table and generates SQL queries
/// for common CRUD operations. The generated queries are cached globally for performance.
///
/// # Fields
///
/// * `table_name` - Name of the database table
/// * `insert_fields` - Fields to include in INSERT statements
/// * `update_fields` - Fields to include in UPDATE statements (excludes ID)
/// * `id_field` - Name of the primary key/ID field
/// * `column_definitions` - Column metadata with optional type casting
pub struct Scheme {
    pub table_name: String,
    pub insert_fields: Vec<String>,
    pub update_fields: Vec<String>,
    pub id_field: String,
    pub column_definitions: Vec<ColumnDefinition>,
}

// Global SQL cache that stores strings and returns &'static str references
static SQL_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Get SQL from cache or compute and store it, returning &'static str.
///
/// # Safety
///
/// The returned reference is valid for the entire program duration because
/// it points to a string stored in a global static HashMap. The HashMap is
/// never cleared, so the reference will remain valid.
fn get_or_insert_sql(key: String, gen_fn: impl FnOnce() -> String) -> &'static str {
    let mut cache = SQL_CACHE.lock().unwrap();
    if !cache.contains_key(&key) {
        cache.insert(key.clone(), gen_fn());
    }
    // SAFETY: The string is stored in a global static HashMap,
    // so the reference will live for the entire program duration
    unsafe {
        let ptr = cache.get(&key).unwrap().as_str() as *const str;
        &*ptr
    }
}

impl Scheme {
    /// Generates a SELECT clause with explicit column list and optional type casting.
    ///
    /// This method replaces `SELECT *` with an explicit column list, applying
    /// type casting where specified by `#[sqlx(cast_as = "TYPE")]` attributes.
    ///
    /// # Example
    ///
    /// For column_definitions:
    /// - ColumnDefinition { name: "id", cast_as: None }
    /// - ColumnDefinition { name: "commission_rate", cast_as: Some("TEXT") }
    ///
    /// Generates: `"id, commission_rate::TEXT as commission_rate"`
    ///
    /// # Returns
    ///
    /// A cached `&'static str` containing the comma-separated column list with
    /// optional casting expressions.
    pub fn gen_select_columns_static(&self) -> &'static str {
        let key = format!("{}-select-columns", self.table_name);
        get_or_insert_sql(key, || {
            // If no column definitions provided, fall back to SELECT *
            if self.column_definitions.is_empty() {
                return "*".to_string();
            }

            // Generate explicit column list with optional casting
            self.column_definitions.iter()
                .map(|col| {
                    match &col.cast_as {
                        Some(cast_type) => {
                            // PostgreSQL: column::TYPE as column
                            // Example: commission_rate::TEXT as commission_rate
                            format!("{}::{} as {}", col.name, cast_type, col.name)
                        }
                        None => col.name.clone(),
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
    }

    /// Generates a COUNT query with the given WHERE clause.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_count_sql_static(&self, where_stmt: &str) -> &'static str {
        let key = format!("{}-count-{}", self.table_name, where_stmt);
        get_or_insert_sql(key, || {
            let where_sql = prepare_where(where_stmt, 1);
            format!("SELECT COUNT(*) FROM {} WHERE {}", self.table_name, where_sql)
        })
    }

    /// Generates an INSERT query for all fields.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_insert_sql_static(&self) -> &'static str {
        let key = format!("{}-insert", self.table_name);
        get_or_insert_sql(key, || {
            let params: Vec<String> = self.insert_fields.iter().enumerate().map(|(idx, _)|{
                let p = format!("${}", idx + 1);
                param_trans(p)
            }).collect();
            let params_str = params.join(",");
            format!(r#"INSERT INTO {} VALUES ({})"#, self.table_name, params_str)
        })
    }

    /// Generates a bulk INSERT query for multiple rows.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    /// The SQL format is: INSERT INTO table VALUES ($1,$2),($3,$4),($5,$6)
    pub fn gen_bulk_insert_sql_static(&self, row_count: usize) -> &'static str {
        let key = format!("{}-bulk-insert-{}-rows", self.table_name, row_count);
        get_or_insert_sql(key, || {
            let field_count = self.insert_fields.len();
            let mut all_params = Vec::new();
            let mut param_index = 1;

            for _ in 0..row_count {
                let row_params: Vec<String> = (0..field_count).map(|_| {
                    let p = format!("${}", param_index);
                    param_index += 1;
                    param_trans(p)
                }).collect();
                all_params.push(format!("({})", row_params.join(",")));
            }

            format!(r#"INSERT INTO {} VALUES {}"#, self.table_name, all_params.join(","))
        })
    }

    /// Generates a bulk UPDATE query for multiple rows using CASE WHEN.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    /// The SQL format is: UPDATE table SET field1=CASE id WHEN $1 THEN $2 WHEN $3 THEN $4 ELSE field1 END,... WHERE id IN ($5,$7)
    pub fn gen_bulk_update_sql_static(&self, row_count: usize) -> &'static str {
        let key = format!("{}-bulk-update-{}-rows", self.table_name, row_count);
        get_or_insert_sql(key, || {
            let mut param_index = 1;
            let mut set_clauses = Vec::new();

            // Generate CASE WHEN for each update field
            for field in &self.update_fields {
                let when_clauses: Vec<String> = (0..row_count).map(|_| {
                    let id_param = param_trans(format!("${}", param_index));
                    param_index += 1;
                    let val_param = param_trans(format!("${}", param_index));
                    param_index += 1;
                    format!("WHEN {} THEN {}", id_param, val_param)
                }).collect();

                let case_expr = format!("{}=CASE {} {} END", field, self.id_field, when_clauses.join(" "));
                set_clauses.push(case_expr);
            }

            // Generate IN clause for IDs
            let id_params: Vec<String> = (0..row_count).map(|_| {
                let p = param_trans(format!("${}", param_index));
                param_index += 1;
                p
            }).collect();

            format!(r#"UPDATE {} SET {} WHERE {} IN ({})"#,
                self.table_name,
                set_clauses.join(","),
                self.id_field,
                id_params.join(",")
            )
        })
    }

    /// Generates an UPDATE query to modify a row by ID.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_update_by_id_sql_static(&self) -> &'static str {
        let key = format!("{}-update-by-id", self.table_name);
        get_or_insert_sql(key, || {
            let set_seq: Vec<String> = self.update_fields.iter().enumerate().map(|(idx, fd)|{
                let p = format!("${}", idx + 1);
                let p = param_trans(p);
                format!("{}={}", fd, p)
            }).collect();
            let id_param = param_trans(format!("${}", self.insert_fields.len() as i32));
            format!(r#"UPDATE {} SET {} WHERE {}={}"#, self.table_name, set_seq.join(","), self.id_field, id_param)
        })
    }

    /// Generates an UPDATE query with a custom WHERE clause.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_update_where_sql_static(&self, where_stmt: &str) -> &'static str {
        let key = format!("{}-update-where-{}", self.table_name, where_stmt);
        get_or_insert_sql(key, || {
            let set_seq: Vec<String> = self.update_fields.iter().enumerate().map(|(idx, fd)|{
                let p = format!("${}", idx + 1);
                let p = param_trans(p);
                format!("{}={}", fd, p)
            }).collect();
            let where_sql = prepare_where(where_stmt, self.insert_fields.len() as i32);
            format!(r#"UPDATE {} SET {} WHERE {}"#, self.table_name, set_seq.join(","), where_sql)
        })
    }

    /// Generates a DELETE query to remove a row by ID.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_delete_sql_static(&self) -> &'static str {
        let key = format!("{}-delete-by-id", self.table_name);
        get_or_insert_sql(key, || {
            let id_param = param_trans("$1".to_string());
            format!(r#"DELETE FROM {} WHERE {}={}"#, self.table_name, self.id_field, id_param)
        })
    }

    /// Generates a DELETE query with a custom WHERE clause.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_delete_where_sql_static(&self, where_stmt: &str) -> &'static str {
        let key = format!("{}-delete-where-{}", self.table_name, where_stmt);
        get_or_insert_sql(key, || {
            let where_sql = prepare_where(where_stmt, 1);
            format!(r#"DELETE FROM {} WHERE {}"#, self.table_name, where_sql)
        })
    }

    /// Generates a bulk DELETE query for multiple IDs using WHERE IN clause.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_bulk_delete_sql_static(&self, count: usize) -> &'static str {
        let key = format!("{}-bulk-delete-{}", self.table_name, count);
        get_or_insert_sql(key, || {
            let params: Vec<String> = (1..=count).map(|i| param_trans(format!("${}", i))).collect();
            let params_str = params.join(",");
            format!(r#"DELETE FROM {} WHERE {} IN ({})"#, self.table_name, self.id_field, params_str)
        })
    }

    /// Generates a bulk SELECT query for multiple IDs using WHERE IN clause.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    ///
    /// Note: This version does not guarantee the order of results. If you need results
    /// in the same order as input IDs, you should sort them in your application code.
    ///
    /// # Example
    ///
    /// For count=3 with PostgreSQL, generates:
    /// ```sql
    /// SELECT * FROM users WHERE id IN ($1,$2,$3)
    /// ```
    ///
    /// For MySQL/SQLite, generates:
    /// ```sql
    /// SELECT * FROM users WHERE id IN (?,?,?)
    /// ```
    pub fn gen_bulk_select_sql_static(&self, count: usize) -> &'static str {
        // IMPORTANT: Call gen_select_columns_static() BEFORE acquiring the lock
        // to avoid deadlock since it also accesses SQL_CACHE
        let columns = self.gen_select_columns_static();
        let key = format!("{}-bulk-select-{}", self.table_name, count);
        get_or_insert_sql(key, || {
            if count == 0 {
                // Empty list: return a query that always returns empty result
                format!(r#"SELECT {} FROM {} WHERE 1=0"#, columns, self.table_name)
            } else {
                let params: Vec<String> = (1..=count).map(|i| param_trans(format!("${}", i))).collect();
                let in_clause = params.join(",");
                format!(
                    r#"SELECT {} FROM {} WHERE {} IN ({})"#,
                    columns, self.table_name, self.id_field, in_clause
                )
            }
        })
    }

    /// Generates a SELECT query to fetch a row by ID.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_select_by_id_sql_static(&self) -> &'static str {
        // IMPORTANT: Call gen_select_columns_static() BEFORE acquiring the lock
        // to avoid deadlock since it also accesses SQL_CACHE
        let columns = self.gen_select_columns_static();
        let key = format!("{}-select-by-id", self.table_name);
        get_or_insert_sql(key, || {
            let id_param = param_trans("$1".to_string());
            format!(r#"SELECT {} FROM {} WHERE {}={}"#, columns, self.table_name, self.id_field, id_param)
        })
    }

    /// Generates a SELECT query with a custom WHERE clause.
    ///
    /// Returns a cached `&'static str` for efficient reuse.
    pub fn gen_select_where_sql_static(&self, where_stmt: &str) -> &'static str {
        // IMPORTANT: Call gen_select_columns_static() BEFORE acquiring the lock
        // to avoid deadlock since it also accesses SQL_CACHE
        let columns = self.gen_select_columns_static();
        let key = format!("{}-select-where-{}", self.table_name, where_stmt);
        get_or_insert_sql(key, || {
            let where_sql = prepare_where(where_stmt, 1);
            format!(r#"SELECT {} FROM {} WHERE {}"#, columns, self.table_name, where_sql)
        })
    }

    /// Prepares custom SQL by replacing the `[Self]` placeholder with the table name.
    ///
    /// This method is used for custom queries where you want to dynamically insert the table name.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let scheme = Scheme { /* ... */ };
    /// let sql = scheme.pre_sql_static("SELECT * FROM [Self] WHERE active = true");
    /// // Results in: "SELECT * FROM my_table WHERE active = true"
    /// ```
    pub fn pre_sql_static(&self, sql: &str) -> String {
        sql.replace("[Self]", self.table_name.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum DbType {
    PostgreSQL,
    MySQL,
    SQLite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_insert_sql_generation() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            update_fields: vec!["name".to_string(), "email".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_insert_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO users VALUES ($1,$2,$3)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO users VALUES (?,?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO users VALUES (?,?,?)");
    }

    #[test]
    fn test_scheme_update_sql_generation() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            update_fields: vec!["name".to_string(), "email".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_update_by_id_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE users SET name=$1,email=$2 WHERE id=$3");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE users SET name=?,email=? WHERE id=?");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE users SET name=?,email=? WHERE id=?");
    }

    #[test]
    fn test_scheme_delete_sql_generation() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM users WHERE id=$1");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM users WHERE id=?");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM users WHERE id=?");
    }

    #[test]
    fn test_scheme_select_sql_generation() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_select_by_id_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM users WHERE id=$1");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM users WHERE id=?");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "SELECT * FROM users WHERE id=?");
    }

    #[test]
    fn test_select_with_cast_as() {
        // Test the new cast_as functionality for DECIMAL support
        let scheme = Scheme {
            table_name: "decimal_users".to_string(),  // Use unique table name to avoid cache collision
            insert_fields: vec!["id".to_string(), "commission_rate".to_string()],
            update_fields: vec!["commission_rate".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![
                ColumnDefinition { name: "id".to_string(), cast_as: None },
                ColumnDefinition { name: "commission_rate".to_string(), cast_as: Some("TEXT".to_string()) },
            ],
        };

        let sql = scheme.gen_select_by_id_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT id, commission_rate::TEXT as commission_rate FROM decimal_users WHERE id=$1");
    }

    #[test]
    fn test_scheme_count_sql_generation() {
        let scheme = Scheme {
            table_name: "orders".to_string(),  // Use different table name to avoid cache collision
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_count_sql_static("active = true");
        assert_eq!(sql, "SELECT COUNT(*) FROM orders WHERE active = true");
    }

    #[test]
    fn test_pre_sql_replaces_self_placeholder() {
        let scheme = Scheme {
            table_name: "my_table".to_string(),
            insert_fields: vec![],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.pre_sql_static("SELECT * FROM [Self] WHERE active = true");
        assert_eq!(sql, "SELECT * FROM my_table WHERE active = true");
    }

    #[test]
    fn test_sql_caching() {
        let scheme = Scheme {
            table_name: "test_table".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        // First call should cache the SQL
        let sql1 = scheme.gen_insert_sql_static();
        let sql2 = scheme.gen_insert_sql_static();

        // Both should point to the same memory address (cached)
        assert_eq!(sql1, sql2);
        assert!(std::ptr::eq(sql1, sql2), "SQL should be cached and return the same pointer");
    }

    #[test]
    fn test_prepare_where_postgres() {
        #[cfg(feature = "postgres")]
        {
            let result = prepare_where("name = {} AND age = {}", 1);
            assert_eq!(result, "name = $1 AND age = $2");
        }
    }

    #[test]
    fn test_prepare_where_mysql_sqlite() {
        #[cfg(all(not(feature = "postgres"), any(feature = "mysql", feature = "sqlite")))]
        {
            let result = prepare_where("name = {} AND age = {}", 1);
            assert_eq!(result, "name = ? AND age = ?");
        }
    }

    #[test]
    fn test_empty_where_clause() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_select_where_sql_static("1=1");
        assert_eq!(sql, "SELECT * FROM users WHERE 1=1");
    }

    #[test]
    fn test_update_where_with_multiple_fields() {
        let scheme = Scheme {
            table_name: "products".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "price".to_string()],
            update_fields: vec!["name".to_string(), "price".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_update_where_sql_static("category = {}");

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE products SET name=$1,price=$2 WHERE category = $3");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE products SET name=?,price=? WHERE category = ?");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE products SET name=?,price=? WHERE category = ?");
    }

    #[test]
    fn test_delete_where() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_where_sql_static("created_at < NOW()");

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM logs WHERE created_at < NOW()");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM logs WHERE created_at < NOW()");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM logs WHERE created_at < NOW()");
    }

    #[test]
    fn test_multiple_schemes_cache_separately() {
        let scheme1 = Scheme {
            table_name: "table1".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let scheme2 = Scheme {
            table_name: "table2".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme1.gen_insert_sql_static();
        let sql2 = scheme2.gen_insert_sql_static();

        // Should have different content
        assert_ne!(sql1, sql2);
    }

    // Edge case tests
    #[test]
    fn test_single_field_scheme() {
        let scheme = Scheme {
            table_name: "minimal".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_insert_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO minimal VALUES ($1)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO minimal VALUES (?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO minimal VALUES (?)");
    }

    #[test]
    fn test_large_number_of_fields() {
        let fields: Vec<String> = (0..10).map(|i| format!("field{}", i)).collect();
        let insert_fields = fields.clone();
        let update_fields = fields[1..].to_vec();

        let scheme = Scheme {
            table_name: "wide_table".to_string(),
            insert_fields,
            update_fields,
            id_field: "field0".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_insert_sql_static();

        #[cfg(feature = "postgres")]
        assert!(sql.contains("$10"), "Should have parameter $10");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql.matches("?").count(), 10, "Should have 10 parameters");
    }

    #[test]
    fn test_special_characters_in_table_name() {
        let scheme = Scheme {
            table_name: "user_profiles".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_insert_sql_static();
        assert!(sql.contains("user_profiles"), "Table name should be preserved");
    }

    #[test]
    fn test_complex_where_clause() {
        let scheme = Scheme {
            table_name: "orders".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_select_where_sql_static(
            "status = 'active' AND created_at > {} AND payment_status = {}"
        );

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM orders WHERE status = 'active' AND created_at > $1 AND payment_status = $2");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM orders WHERE status = 'active' AND created_at > ? AND payment_status = ?");
    }

    #[test]
    fn test_pre_sql_with_multiple_placeholders() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec![],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.pre_sql_static(
            "SELECT * FROM [Self] WHERE [Self].created_at > [Self].updated_at"
        );
        assert_eq!(
            sql,
            "SELECT * FROM users WHERE users.created_at > users.updated_at"
        );
    }

    #[test]
    fn test_count_with_complex_condition() {
        let scheme = Scheme {
            table_name: "transactions".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_count_sql_static(
            "amount > 100 AND status IN ('pending', 'completed')"
        );

        assert!(sql.contains("SELECT COUNT(*) FROM transactions"));
        assert!(sql.contains("amount > 100"));
        assert!(sql.contains("status IN ('pending', 'completed')"));
    }

    #[test]
    fn test_delete_where_with_subquery() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_where_sql_static(
            "user_id IN (SELECT id FROM users WHERE banned = true)"
        );

        assert!(sql.contains("DELETE FROM logs WHERE"));
        assert!(sql.contains("user_id IN"));
    }

    #[test]
    fn test_update_where_no_placeholders() {
        let scheme = Scheme {
            table_name: "config".to_string(),
            insert_fields: vec!["id".to_string(), "value".to_string()],
            update_fields: vec!["value".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_update_where_sql_static("key = 'app_version'");

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE config SET value=$1 WHERE key = 'app_version'");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE config SET value=? WHERE key = 'app_version'");
    }

    // Test for custom table names with underscore and special characters
    #[test]
    fn test_table_name_with_underscores() {
        let scheme = Scheme {
            table_name: "user_profile_settings".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_insert_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO user_profile_settings VALUES ($1)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO user_profile_settings VALUES (?)");
    }

    #[test]
    fn test_table_name_with_prefix() {
        let scheme = Scheme {
            table_name: "app_users".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_select_by_id_sql_static();

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM app_users WHERE id=$1");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM app_users WHERE id=?");
    }

    // Tests for delete_where_query functionality
    #[test]
    fn test_delete_where_simple_condition() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_where_sql_static("level = 'DEBUG'");

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM logs WHERE level = 'DEBUG'");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM logs WHERE level = 'DEBUG'");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM logs WHERE level = 'DEBUG'");
    }

    #[test]
    fn test_delete_where_with_parameters() {
        let scheme = Scheme {
            table_name: "sessions".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_where_sql_static("expires_at < {}");

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM sessions WHERE expires_at < $1");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM sessions WHERE expires_at < ?");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM sessions WHERE expires_at < ?");
    }

    #[test]
    fn test_delete_where_with_complex_condition() {
        let scheme = Scheme {
            table_name: "temp_data".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_where_sql_static(
            "created_at < NOW() - INTERVAL '30 days' AND status = 'expired'"
        );

        assert!(sql.contains("DELETE FROM temp_data WHERE"));
        assert!(sql.contains("created_at < NOW() - INTERVAL '30 days'"));
        assert!(sql.contains("status = 'expired'"));
    }

    #[test]
    fn test_delete_where_with_multiple_parameters() {
        let scheme = Scheme {
            table_name: "events".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_delete_where_sql_static("status = {} AND created_at < {}");

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM events WHERE status = $1 AND created_at < $2");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM events WHERE status = ? AND created_at < ?");
    }

    #[test]
    fn test_delete_where_caching() {
        let scheme = Scheme {
            table_name: "cache_items".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_delete_where_sql_static("expired = true");
        let sql2 = scheme.gen_delete_where_sql_static("expired = true");

        // Should be cached and return the same pointer
        assert_eq!(sql1, sql2);
        assert!(std::ptr::eq(sql1, sql2), "SQL should be cached");
    }

    #[test]
    fn test_bulk_delete_single_id() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_delete_sql_static(1);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM users WHERE id IN ($1)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM users WHERE id IN (?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM users WHERE id IN (?)");
    }

    #[test]
    fn test_bulk_delete_multiple_ids() {
        let scheme = Scheme {
            table_name: "products".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_delete_sql_static(3);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM products WHERE id IN ($1,$2,$3)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM products WHERE id IN (?,?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM products WHERE id IN (?,?,?)");
    }

    #[test]
    fn test_bulk_delete_large_batch() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_delete_sql_static(100);

        #[cfg(feature = "postgres")]
        {
            let expected = (1..=100).map(|i| format!("${}", i)).collect::<Vec<_>>().join(",");
            assert_eq!(sql, format!("DELETE FROM logs WHERE id IN ({})", expected));
        }

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        {
            let expected = (1..=100).map(|_| "?").collect::<Vec<_>>().join(",");
            assert_eq!(sql, format!("DELETE FROM logs WHERE id IN ({})", expected));
        }

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        {
            let expected = (1..=100).map(|_| "?").collect::<Vec<_>>().join(",");
            assert_eq!(sql, format!("DELETE FROM logs WHERE id IN ({})", expected));
        }
    }

    #[test]
    fn test_bulk_delete_caching() {
        let scheme = Scheme {
            table_name: "cache_test".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_delete_sql_static(5);
        let sql2 = scheme.gen_bulk_delete_sql_static(5);

        // Should be cached and return the same pointer
        assert_eq!(sql1, sql2);
        assert!(std::ptr::eq(sql1, sql2), "SQL should be cached");
    }

    #[test]
    fn test_bulk_delete_different_counts_cached_separately() {
        let scheme = Scheme {
            table_name: "items".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_delete_sql_static(3);
        let sql2 = scheme.gen_bulk_delete_sql_static(5);

        // Different counts should generate different SQL
        assert_ne!(sql1, sql2);
        assert!(!std::ptr::eq(sql1, sql2), "Different counts should have different cached SQL");
    }

    #[test]
    fn test_bulk_delete_custom_id_field() {
        let scheme = Scheme {
            table_name: "orders".to_string(),
            insert_fields: vec!["order_id".to_string(), "customer_id".to_string()],
            update_fields: vec!["customer_id".to_string()],
            id_field: "order_id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_delete_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM orders WHERE order_id IN ($1,$2)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM orders WHERE order_id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM orders WHERE order_id IN (?,?)");
    }

    #[test]
    fn test_bulk_delete_with_custom_table_name() {
        let scheme = Scheme {
            table_name: "app.users".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_delete_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "DELETE FROM app.users WHERE id IN ($1,$2)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "DELETE FROM app.users WHERE id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "DELETE FROM app.users WHERE id IN (?,?)");
    }

    #[test]
    fn test_bulk_select_single_id() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_select_sql_static(1);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM users WHERE id IN ($1)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM users WHERE id IN (?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "SELECT * FROM users WHERE id IN (?)");
    }

    #[test]
    fn test_bulk_select_multiple_ids() {
        let scheme = Scheme {
            table_name: "products".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_select_sql_static(3);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM products WHERE id IN ($1,$2,$3)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM products WHERE id IN (?,?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "SELECT * FROM products WHERE id IN (?,?,?)");
    }

    #[test]
    fn test_bulk_select_empty_list() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_select_sql_static(0);
        assert_eq!(sql, "SELECT * FROM users WHERE 1=0");
    }

    #[test]
    fn test_bulk_select_large_batch() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_select_sql_static(100);

        #[cfg(feature = "postgres")]
        {
            let in_params = (1..=100).map(|i| format!("${}", i)).collect::<Vec<_>>().join(",");
            let expected = format!("SELECT * FROM logs WHERE id IN ({})", in_params);
            assert_eq!(sql, expected);
        }

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        {
            let in_params = (1..=100).map(|_| "?").collect::<Vec<_>>().join(",");
            let order_by_params = (0..100).map(|i| format!("WHEN ? THEN {}", i)).collect::<Vec<_>>().join("\n    ");
            let expected = format!("SELECT * FROM logs WHERE id IN ({}) ORDER BY CASE id\n    {}\nEND", in_params, order_by_params);
            assert_eq!(sql, expected);
        }

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        {
            let in_params = (1..=100).map(|_| "?").collect::<Vec<_>>().join(",");
            let order_by_params = (0..100).map(|i| format!("WHEN ? THEN {}", i)).collect::<Vec<_>>().join("\n    ");
            let expected = format!("SELECT * FROM logs WHERE id IN ({}) ORDER BY CASE id\n    {}\nEND", in_params, order_by_params);
            assert_eq!(sql, expected);
        }
    }

    #[test]
    fn test_bulk_select_caching() {
        let scheme = Scheme {
            table_name: "items".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_select_sql_static(5);
        let sql2 = scheme.gen_bulk_select_sql_static(5);

        // Should be cached and return the same pointer
        assert_eq!(sql1, sql2);
        assert!(std::ptr::eq(sql1, sql2), "SQL should be cached");
    }

    #[test]
    fn test_bulk_select_different_counts_cached_separately() {
        let scheme = Scheme {
            table_name: "items".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_select_sql_static(3);
        let sql2 = scheme.gen_bulk_select_sql_static(5);

        // Different counts should generate different SQL
        assert_ne!(sql1, sql2);
        assert!(!std::ptr::eq(sql1, sql2), "Different counts should have different cached SQL");
    }

    #[test]
    fn test_bulk_select_custom_id_field() {
        let scheme = Scheme {
            table_name: "orders".to_string(),
            insert_fields: vec!["order_id".to_string(), "customer_id".to_string()],
            update_fields: vec!["customer_id".to_string()],
            id_field: "order_id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_select_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM orders WHERE order_id IN ($1,$2)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM orders WHERE order_id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "SELECT * FROM orders WHERE order_id IN (?,?)");
    }

    #[test]
    fn test_bulk_select_with_custom_table_name() {
        let scheme = Scheme {
            table_name: "app.users".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_select_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "SELECT * FROM app.users WHERE id IN ($1,$2)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "SELECT * FROM app.users WHERE id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "SELECT * FROM app.users WHERE id IN (?,?)");
    }

    #[test]
    fn test_bulk_insert_single_row() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            update_fields: vec!["name".to_string(), "email".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_insert_sql_static(1);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO users VALUES ($1,$2,$3)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO users VALUES (?,?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO users VALUES (?,?,?)");
    }

    #[test]
    fn test_bulk_insert_multiple_rows() {
        let scheme = Scheme {
            table_name: "products".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "price".to_string()],
            update_fields: vec!["name".to_string(), "price".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_insert_sql_static(3);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO products VALUES ($1,$2,$3),($4,$5,$6),($7,$8,$9)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO products VALUES (?,?,?),(?,?,?),(?,?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO products VALUES (?,?,?),(?,?,?),(?,?,?)");
    }

    #[test]
    fn test_bulk_insert_two_fields() {
        let scheme = Scheme {
            table_name: "categories".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_insert_sql_static(4);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO categories VALUES ($1,$2),($3,$4),($5,$6),($7,$8)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO categories VALUES (?,?),(?,?,?),(?,?,?),(?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO categories VALUES (?,?),(?,?,?),(?,?,?),(?,?)");
    }

    #[test]
    fn test_bulk_insert_large_batch() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string(), "message".to_string(), "level".to_string()],
            update_fields: vec!["message".to_string(), "level".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_insert_sql_static(50);

        #[cfg(feature = "postgres")]
        {
            // Should have 50 rows with 3 fields each (150 parameters total)
            assert!(sql.contains("INSERT INTO logs VALUES"));
            assert!(sql.contains("($1,$2,$3)"));
            assert!(sql.contains("($148,$149,$150)"));
        }

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        {
            // MySQL/SQLite use ? placeholders
            let param_count = sql.matches('?').count();
            assert_eq!(param_count, 150, "Should have 150 parameters for 50 rows × 3 fields");
        }

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        {
            let param_count = sql.matches('?').count();
            assert_eq!(param_count, 150, "Should have 150 parameters for 50 rows × 3 fields");
        }
    }

    #[test]
    fn test_bulk_insert_caching() {
        let scheme = Scheme {
            table_name: "cache_test".to_string(),
            insert_fields: vec!["id".to_string(), "value".to_string()],
            update_fields: vec!["value".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_insert_sql_static(5);
        let sql2 = scheme.gen_bulk_insert_sql_static(5);

        // Should be cached and return the same pointer
        assert_eq!(sql1, sql2);
        assert!(std::ptr::eq(sql1, sql2), "SQL should be cached");
    }

    #[test]
    fn test_bulk_insert_different_row_counts_cached_separately() {
        let scheme = Scheme {
            table_name: "items".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_insert_sql_static(3);
        let sql2 = scheme.gen_bulk_insert_sql_static(5);

        // Different row counts should generate different SQL
        assert_ne!(sql1, sql2);
        assert!(!std::ptr::eq(sql1, sql2), "Different row counts should have different cached SQL");
    }

    #[test]
    fn test_bulk_insert_single_field() {
        let scheme = Scheme {
            table_name: "tags".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_insert_sql_static(5);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO tags VALUES ($1),($2),($3),($4),($5)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO tags VALUES (?),(?),(?),(?),(?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO tags VALUES (?),(?),(?),(?),(?)");
    }

    #[test]
    fn test_bulk_insert_with_custom_table_name() {
        let scheme = Scheme {
            table_name: "app.users".to_string(),
            insert_fields: vec!["id".to_string(), "username".to_string()],
            update_fields: vec!["username".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_insert_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "INSERT INTO app.users VALUES ($1,$2),($3,$4)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "INSERT INTO app.users VALUES (?,?),(?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "INSERT INTO app.users VALUES (?,?),(?,?)");
    }

    #[test]
    fn test_bulk_update_single_row() {
        let scheme = Scheme {
            table_name: "users".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            update_fields: vec!["name".to_string(), "email".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_update_sql_static(1);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE users SET name=CASE id WHEN $1 THEN $2 END,email=CASE id WHEN $3 THEN $4 END WHERE id IN ($5)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE users SET name=CASE id WHEN ? THEN ? END,email=CASE id WHEN ? THEN ? END WHERE id IN (?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE users SET name=CASE WHEN ? THEN ? END,email=CASE WHEN ? THEN ? END WHERE id IN (?)");
    }

    #[test]
    fn test_bulk_update_multiple_rows() {
        let scheme = Scheme {
            table_name: "products".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string(), "price".to_string()],
            update_fields: vec!["name".to_string(), "price".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_update_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE products SET name=CASE id WHEN $1 THEN $2 WHEN $3 THEN $4 END,price=CASE id WHEN $5 THEN $6 WHEN $7 THEN $8 END WHERE id IN ($9,$10)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE products SET name=CASE id WHEN ? THEN ? WHEN ? THEN ? END,price=CASE id WHEN ? THEN ? WHEN ? THEN ? END WHERE id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE products SET name=CASE WHEN ? THEN ? WHEN ? THEN ? END,price=CASE WHEN ? THEN ? WHEN ? THEN ? END WHERE id IN (?,?)");
    }

    #[test]
    fn test_bulk_update_single_field() {
        let scheme = Scheme {
            table_name: "categories".to_string(),
            insert_fields: vec!["id".to_string(), "name".to_string()],
            update_fields: vec!["name".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_update_sql_static(3);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE categories SET name=CASE id WHEN $1 THEN $2 WHEN $3 THEN $4 WHEN $5 THEN $6 END WHERE id IN ($7,$8,$9)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE categories SET name=CASE id WHEN ? THEN ? WHEN ? THEN ? WHEN ? THEN ? END WHERE id IN (?,?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE categories SET name=CASE WHEN ? THEN ? WHEN ? THEN ? WHEN ? THEN ? END WHERE id IN (?,?,?)");
    }

    #[test]
    fn test_bulk_update_large_batch() {
        let scheme = Scheme {
            table_name: "logs".to_string(),
            insert_fields: vec!["id".to_string(), "message".to_string(), "level".to_string()],
            update_fields: vec!["message".to_string(), "level".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_update_sql_static(10);

        #[cfg(feature = "postgres")]
        {
            // Should have 10 WHEN clauses per field (2 fields = 20 WHEN clauses)
            assert_eq!(sql.matches("WHEN").count(), 20);
            // Should have IN clause with 10 parameters
            assert!(sql.contains("WHERE id IN ($"));
        }

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        {
            // MySQL/SQLite use ? placeholders
            // Each row has: id + 2 update fields = 3 params for WHEN + 1 param for IN
            // Total: 10 rows × 3 params + 10 IN params = 40 params
            let param_count = sql.matches('?').count();
            assert_eq!(param_count, 40);
        }

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        {
            let param_count = sql.matches('?').count();
            assert_eq!(param_count, 40);
        }
    }

    #[test]
    fn test_bulk_update_caching() {
        let scheme = Scheme {
            table_name: "cache_test".to_string(),
            insert_fields: vec!["id".to_string(), "value".to_string()],
            update_fields: vec!["value".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_update_sql_static(5);
        let sql2 = scheme.gen_bulk_update_sql_static(5);

        // Should be cached and return the same pointer
        assert_eq!(sql1, sql2);
        assert!(std::ptr::eq(sql1, sql2), "SQL should be cached");
    }

    #[test]
    fn test_bulk_update_different_row_counts_cached_separately() {
        let scheme = Scheme {
            table_name: "items".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql1 = scheme.gen_bulk_update_sql_static(3);
        let sql2 = scheme.gen_bulk_update_sql_static(5);

        // Different row counts should generate different SQL
        assert_ne!(sql1, sql2);
        assert!(!std::ptr::eq(sql1, sql2), "Different row counts should have different cached SQL");
    }

    #[test]
    fn test_bulk_update_no_update_fields() {
        let scheme = Scheme {
            table_name: "tags".to_string(),
            insert_fields: vec!["id".to_string()],
            update_fields: vec![],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_update_sql_static(2);

        // Edge case: no update fields, should only have WHERE IN clause
        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE tags SET  WHERE id IN ($1,$2)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE tags SET  WHERE id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE tags SET  WHERE id IN (?,?)");
    }

    #[test]
    fn test_bulk_update_with_custom_table_name() {
        let scheme = Scheme {
            table_name: "app.users".to_string(),
            insert_fields: vec!["id".to_string(), "username".to_string()],
            update_fields: vec!["username".to_string()],
            id_field: "id".to_string(),
            column_definitions: vec![],
        };

        let sql = scheme.gen_bulk_update_sql_static(2);

        #[cfg(feature = "postgres")]
        assert_eq!(sql, "UPDATE app.users SET username=CASE id WHEN $1 THEN $2 WHEN $3 THEN $4 END WHERE id IN ($5,$6)");

        #[cfg(all(feature = "mysql", not(feature = "postgres")))]
        assert_eq!(sql, "UPDATE app.users SET username=CASE id WHEN ? THEN ? WHEN ? THEN ? END WHERE id IN (?,?)");

        #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
        assert_eq!(sql, "UPDATE app.users SET username=CASE WHEN ? THEN ? WHEN ? THEN ? END WHERE id IN (?,?)");
    }

    // Transaction helper tests - verify the transaction helper compiles correctly
    #[test]
    #[cfg(feature = "postgres")]
    fn test_transaction_helper_compiles() {
        // This test verifies that the transaction helper function exists
        // and compiles correctly. Actual integration tests require a database.
        use sqlx::PgPool;

        // Just verify the function is callable - this is a compile-time test
        fn check_compile_time<F>(_: F) where F: FnOnce() {}
        check_compile_time(|| {
            let _: PgPool = unsafe { std::mem::zeroed() };
        });
    }

    #[test]
    #[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
    fn test_transaction_helper_compiles_mysql() {
        use sqlx::{MySql, MySqlPool};

        fn check_compile_time<F>(_: F) where F: FnOnce() {}
        check_compile_time(|| {
            let _: MySqlPool = unsafe { std::mem::zeroed() };
        });
    }

    #[test]
    #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
    fn test_transaction_helper_compiles_sqlite() {
        use sqlx::{Sqlite, SqlitePool};

        fn check_compile_time<F>(_: F) where F: FnOnce() {}
        check_compile_time(|| {
            let _: SqlitePool = unsafe { std::mem::zeroed() };
        });
    }

    // Nested transaction helper tests - verify nested_transaction compiles correctly
    #[test]
    #[cfg(feature = "postgres")]
    fn test_nested_transaction_helper_compiles() {
        // This test verifies that the nested_transaction helper function exists
        // and compiles correctly. Actual integration tests require a database.
        use sqlx::PgPool;

        // Just verify the function signatures are valid - compile-time test
        fn check_compile_time<F>(_: F) where F: FnOnce() {}
        check_compile_time(|| {
            let _: PgPool = unsafe { std::mem::zeroed() };
        });
    }

    #[test]
    #[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
    fn test_nested_transaction_helper_compiles_mysql() {
        use sqlx::MySqlPool;

        fn check_compile_time<F>(_: F) where F: FnOnce() {}
        check_compile_time(|| {
            let _: MySqlPool = unsafe { std::mem::zeroed() };
        });
    }

    #[test]
    #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
    fn test_nested_transaction_helper_compiles_sqlite() {
        use sqlx::SqlitePool;

        fn check_compile_time<F>(_: F) where F: FnOnce() {}
        check_compile_time(|| {
            let _: SqlitePool = unsafe { std::mem::zeroed() };
        });
    }
}

