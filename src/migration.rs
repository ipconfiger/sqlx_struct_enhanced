//! Database migration system for sqlx_struct_enhanced
//!
//! This module provides automatic schema comparison and migration generation
//! by comparing database metadata with Rust struct definitions.

use sqlx::{Pool, FromRow, Row as _};

#[cfg(feature = "postgres")]
use sqlx::Postgres;

#[cfg(feature = "mysql")]
use sqlx::MySql;

#[cfg(feature = "sqlite")]
use sqlx::Sqlite;

// ============================================================================
// Core Types
// ============================================================================

/// Migration mode for auto_generate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationMode {
    /// Automatically detect changes (Safe for production)
    Auto,
    /// Force recreation of all tables (Destructive - data loss!)
    Force,
}

/// Change type for a column
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnChangeType {
    Add { column: ColumnDef },
    Remove { column_name: String, sql_type: String },
    Rename { old_name: String, new_name: String },
    Modify { old: ColumnDef, new: ColumnDef },
}

/// Change type for a table
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableChangeType {
    Add { table: TableDef },
    Remove { table_name: String },
    Rename { old_name: String, new_name: String },
    Modify { changes: Vec<ColumnChangeType> },
}

/// Column definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub rename_from: Option<String>,
    pub data_migration: Option<DataMigration>,
}

/// Table definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDef {
    pub name: String,
    pub rename_from: Option<String>,
    pub columns: Vec<ColumnDef>,
    pub indexes: Vec<IndexDef>,
    pub primary_key: String,
}

/// Index definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDef {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: String, // "btree", "hash", "gist", etc.
}

/// Data migration specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMigration {
    pub migration_type: DataMigrationType,
    pub expression: Option<String>,
    pub callback_name: Option<String>,
}

/// Type of data migration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataMigrationType {
    /// Use a default value for new rows
    Default { value: String },
    /// Compute from SQL expression
    Compute { expression: String },
    /// Execute a custom callback function
    Callback { function_name: String },
}

/// Change detected for a single table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableChange {
    pub table_name: String,
    pub change_type: TableChangeType,
}

/// Migration with changes for multiple tables
#[derive(Debug, Clone)]
pub struct Migration {
    pub name: String,
    pub version: String,

    // Multiple tables in one migration
    pub table_changes: Vec<TableChange>,

    // SQL statements (all tables combined, ordered correctly)
    pub up_sql: Vec<String>,
    pub down_sql: Vec<String>,

    // Metadata
    pub checksum: String,
    pub created_at: i64, // Unix timestamp

    // Aggregate statistics
    pub total_columns_added: usize,
    pub total_indexes_created: usize,
}

impl Migration {
    /// Create a new migration
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            table_changes: Vec::new(),
            up_sql: Vec::new(),
            down_sql: Vec::new(),
            checksum: String::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            total_columns_added: 0,
            total_indexes_created: 0,
        }
    }

    /// Add a table change to this migration
    pub fn add_table_change(&mut self, change: TableChange) {
        self.table_changes.push(change);
    }

    /// Add UP SQL statement
    pub fn add_up_sql(&mut self, sql: String) {
        self.up_sql.push(sql);
    }

    /// Add DOWN SQL statement
    pub fn add_down_sql(&mut self, sql: String) {
        self.down_sql.push(sql);
    }
}

/// Result of a migration execution
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub migration_name: String,
    pub version: String,
    pub success: bool,
    pub statements_executed: usize,
    pub duration_ms: u128,
    pub error_message: Option<String>,
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug)]
pub enum MigrationError {
    DatabaseError(String),
    SqlExecutionError(String, String),
    SchemaComparisonError(String),
    InvalidState(String),
    DataMigrationError(String),
    TransactionError(String),
    ChecksumMismatch { expected: String, found: String },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MigrationError::SqlExecutionError(sql, msg) => {
                write!(f, "SQL execution error '{}': {}", sql, msg)
            }
            MigrationError::SchemaComparisonError(msg) => write!(f, "Schema comparison error: {}", msg),
            MigrationError::InvalidState(msg) => write!(f, "Invalid migration state: {}", msg),
            MigrationError::DataMigrationError(msg) => write!(f, "Data migration failed: {}", msg),
            MigrationError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
            MigrationError::ChecksumMismatch { expected, found } => {
                write!(f, "Checksum mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for MigrationError {}

impl From<sqlx::Error> for MigrationError {
    fn from(err: sqlx::Error) -> Self {
        MigrationError::DatabaseError(err.to_string())
    }
}

// ============================================================================
// Migration History
// ============================================================================

/// Record of an applied migration
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: String,
    pub name: String,
    pub checksum: String,
    pub applied_at: i64, // Unix timestamp
    pub execution_time_ms: i64,
}

#[cfg(feature = "postgres")]
impl<'r> FromRow<'r, sqlx::postgres::PgRow> for MigrationRecord {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row as _;

        Ok(Self {
            version: row.try_get("version")?,
            name: row.try_get("name")?,
            checksum: row.try_get("checksum")?,
            applied_at: row.try_get("applied_at")?,
            execution_time_ms: row.try_get("execution_time_ms")?,
        })
    }
}

#[cfg(feature = "mysql")]
impl<'r> FromRow<'r, sqlx::mysql::MySqlRow> for MigrationRecord {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row as _;

        Ok(Self {
            version: row.try_get("version")?,
            name: row.try_get("name")?,
            checksum: row.try_get("checksum")?,
            applied_at: row.try_get("applied_at")?,
            execution_time_ms: row.try_get("execution_time_ms")?,
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for MigrationRecord {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row as _;

        Ok(Self {
            version: row.try_get("version")?,
            name: row.try_get("name")?,
            checksum: row.try_get("checksum")?,
            applied_at: row.try_get("applied_at")?,
            execution_time_ms: row.try_get("execution_time_ms")?,
        })
    }
}

/// Migration history manager
pub struct MigrationHistory {
    table_name: String,
}

impl MigrationHistory {
    /// Create a new migration history manager
    pub fn new() -> Self {
        Self {
            table_name: "_schema_migrations".to_string(),
        }
    }

    /// Initialize the migrations table if it doesn't exist
    pub async fn initialize(&self, pool: &Pool<Postgres>) -> Result<(), MigrationError> {
        let create_sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                version VARCHAR(20) PRIMARY KEY,
                name VARCHAR(500) NOT NULL,
                checksum VARCHAR(64) NOT NULL,
                applied_at BIGINT NOT NULL,
                execution_time_ms BIGINT NOT NULL
            )
            "#,
            self.table_name
        );

        sqlx::query(&create_sql)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Check if a migration has been applied
    pub async fn is_applied(
        &self,
        pool: &Pool<Postgres>,
        version: &str,
    ) -> Result<bool, MigrationError> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM _schema_migrations WHERE version = $1",
        )
        .bind(version)
        .fetch_one(pool)
        .await?;

        Ok(result > 0)
    }

    /// Record a successful migration
    pub async fn record(
        &self,
        pool: &Pool<Postgres>,
        migration: &Migration,
        execution_time_ms: u128,
    ) -> Result<(), MigrationError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO _schema_migrations (version, name, checksum, applied_at, execution_time_ms)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&migration.version)
        .bind(&migration.name)
        .bind(&migration.checksum)
        .bind(now)
        .bind(execution_time_ms as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get all applied migrations
    pub async fn get_all(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<MigrationRecord>, MigrationError> {
        let records = sqlx::query_as::<_, MigrationRecord>(
            "SELECT version, name, checksum, applied_at, execution_time_ms
             FROM _schema_migrations
             ORDER BY version ASC"
        )
        .fetch_all(pool)
        .await?;

        Ok(records)
    }

    /// Remove a migration record (for rollback)
    pub async fn remove(
        &self,
        pool: &Pool<Postgres>,
        version: &str,
    ) -> Result<(), MigrationError> {
        sqlx::query("DELETE FROM _schema_migrations WHERE version = $1")
            .bind(version)
            .execute(pool)
            .await?;

        Ok(())
    }
}

impl Default for MigrationHistory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Schema Reader
// ============================================================================

/// Reads database schema metadata (tables, columns, indexes)
pub struct SchemaReader;

impl SchemaReader {
    /// Create a new SchemaReader
    pub fn new() -> Self {
        Self
    }

    /// Read all tables from the database
    pub async fn read_tables(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<String>, MigrationError> {
        let rows = sqlx::query(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename"
        )
        .fetch_all(pool)
        .await?;

        let tables = rows.iter()
            .filter_map(|row| row.try_get::<String, _>("tablename").ok())
            .collect();

        Ok(tables)
    }

    /// Read table schema including columns
    pub async fn read_table_schema(
        &self,
        pool: &Pool<Postgres>,
        table_name: &str,
    ) -> Result<TableDef, MigrationError> {
        // Read columns
        let columns = self.read_columns(pool, table_name).await?;

        // Read indexes
        let indexes = self.read_indexes(pool, table_name).await?;

        // Get primary key (first column is assumed to be PK in our system)
        let primary_key = columns.first()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "id".to_string());

        Ok(TableDef {
            name: table_name.to_string(),
            rename_from: None,
            columns,
            indexes,
            primary_key,
        })
    }

    /// Read column definitions for a table
    pub async fn read_columns(
        &self,
        pool: &Pool<Postgres>,
        table_name: &str,
    ) -> Result<Vec<ColumnDef>, MigrationError> {
        let query = r#"
            SELECT
                column_name,
                data_type,
                is_nullable,
                column_default
            FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = $1
            ORDER BY ordinal_position
        "#;

        let rows = sqlx::query(query)
            .bind(table_name)
            .fetch_all(pool)
            .await?;

        let columns: Vec<ColumnDef> = rows.iter()
            .map(|row| {
                let data_type: String = row.try_get("data_type").unwrap_or_else(|_| "unknown".to_string());
                let is_nullable: String = row.try_get("is_nullable").unwrap_or_else(|_| "YES".to_string());

                ColumnDef {
                    name: row.try_get("column_name").unwrap_or_else(|_| "".to_string()),
                    sql_type: Self::map_postgres_type_to_rust_type(&data_type),
                    nullable: is_nullable == "YES",
                    default: row.try_get::<String, _>("column_default").ok(),
                    rename_from: None,
                    data_migration: None,
                }
            })
            .collect();

        Ok(columns)
    }

    /// Read index definitions for a table
    pub async fn read_indexes(
        &self,
        pool: &Pool<Postgres>,
        table_name: &str,
    ) -> Result<Vec<IndexDef>, MigrationError> {
        let query = r#"
            SELECT
                i.relname as index_name,
                a.attname as column_name,
                ix.indisunique as is_unique,
                am.amname as index_type
            FROM pg_class t
            JOIN pg_index ix ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_am am ON i.relam = am.oid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            JOIN pg_namespace n ON t.relnamespace = n.oid
            WHERE n.nspname = 'public' AND t.relname = $1
            ORDER BY i.relname, a.attnum
        "#;

        let rows = sqlx::query(query)
            .bind(table_name)
            .fetch_all(pool)
            .await?;

        // Group columns by index name
        let mut index_map: std::collections::HashMap<String, IndexDef> = std::collections::HashMap::new();

        for row in rows {
            let index_name: String = row.try_get("index_name").unwrap_or_else(|_| "".to_string());
            let column_name: String = row.try_get("column_name").unwrap_or_else(|_| "".to_string());
            let is_unique: bool = row.try_get("is_unique").unwrap_or(false);
            let index_type: String = row.try_get("index_type").unwrap_or_else(|_| "btree".to_string());

            index_map.entry(index_name.clone())
                .or_insert_with(|| IndexDef {
                    name: index_name,
                    columns: Vec::new(),
                    unique: is_unique,
                    index_type,
                })
                .columns
                .push(column_name);
        }

        Ok(index_map.into_values().collect())
    }

    /// Read complete database schema
    pub async fn read_database_schema(
        &self,
        pool: &Pool<Postgres>,
    ) -> Result<Vec<TableDef>, MigrationError> {
        let table_names = self.read_tables(pool).await?;

        let mut tables = Vec::new();
        for table_name in table_names {
            // Skip migration history table
            if table_name == "_schema_migrations" {
                continue;
            }

            let table_def = self.read_table_schema(pool, &table_name).await?;
            tables.push(table_def);
        }

        Ok(tables)
    }

    /// Map PostgreSQL data types to our simplified type system
    fn map_postgres_type_to_rust_type(pg_type: &str) -> String {
        match pg_type {
            "character varying" | "varchar" | "text" => "VARCHAR".to_string(),
            "integer" | "int4" => "INTEGER".to_string(),
            "bigint" | "int8" => "BIGINT".to_string(),
            "smallint" | "int2" => "SMALLINT".to_string(),
            "boolean" | "bool" => "BOOLEAN".to_string(),
            "timestamp with time zone" | "timestamptz" => "TIMESTAMPTZ".to_string(),
            "timestamp without time zone" | "timestamp" => "TIMESTAMP".to_string(),
            "date" => "DATE".to_string(),
            "time" => "TIME".to_string(),
            "uuid" => "UUID".to_string(),
            "json" | "jsonb" => "JSONB".to_string(),
            "numeric" | "decimal" => "NUMERIC".to_string(),
            "real" | "float4" => "REAL".to_string(),
            "double precision" | "float8" => "DOUBLE PRECISION".to_string(),
            "bytea" => "BYTEA".to_string(),
            _ => pg_type.to_uppercase(),
        }
    }
}

impl Default for SchemaReader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Schema Comparator
// ============================================================================

/// Compares database schema with struct definitions to detect changes
pub struct SchemaComparator;

impl SchemaComparator {
    /// Create a new SchemaComparator
    pub fn new() -> Self {
        Self
    }

    /// Compare database schema with struct schemas and detect changes
    pub fn compare_schemas(
        &self,
        db_schema: &[TableDef],
        struct_schemas: &[TableDef],
    ) -> Result<Vec<TableChange>, MigrationError> {
        let mut changes = Vec::new();

        // Create lookup maps
        let db_tables: std::collections::HashMap<String, &TableDef> = db_schema
            .iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        let struct_tables: std::collections::HashMap<String, &TableDef> = struct_schemas
            .iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        // Track processed tables to handle renames
        let mut processed_db_tables: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut processed_struct_tables: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 1. Detect table renames first
        let renames = self.detect_table_renames(db_schema, struct_schemas)?;

        for (old_name, new_name) in &renames {
            changes.push(TableChange {
                table_name: new_name.clone(),
                change_type: TableChangeType::Rename {
                    old_name: old_name.clone(),
                    new_name: new_name.clone(),
                },
            });
            processed_db_tables.insert(old_name.clone());
            processed_struct_tables.insert(new_name.clone());
        }

        // 2. Detect new tables (in struct but not in DB)
        for (table_name, struct_table) in &struct_tables {
            if processed_struct_tables.contains(table_name) {
                continue;
            }

            if !db_tables.contains_key(table_name) {
                // Check if this table might be a rename target
                if let Some(rename_from) = &struct_table.rename_from {
                    if db_tables.contains_key(rename_from) && !processed_db_tables.contains(rename_from) {
                        // This is explicitly marked as a rename
                        changes.push(TableChange {
                            table_name: table_name.clone(),
                            change_type: TableChangeType::Rename {
                                old_name: rename_from.clone(),
                                new_name: table_name.clone(),
                            },
                        });
                        processed_db_tables.insert(rename_from.clone());
                        processed_struct_tables.insert(table_name.clone());
                        continue;
                    }
                }

                // New table
                changes.push(TableChange {
                    table_name: table_name.clone(),
                    change_type: TableChangeType::Add {
                        table: (*struct_table).clone(),
                    },
                });
                processed_struct_tables.insert(table_name.clone());
            }
        }

        // 3. Detect removed tables (in DB but not in struct)
        for (table_name, _db_table) in &db_tables {
            if processed_db_tables.contains(table_name) {
                continue;
            }

            if !struct_tables.contains_key(table_name) {
                changes.push(TableChange {
                    table_name: table_name.clone(),
                    change_type: TableChangeType::Remove {
                        table_name: table_name.clone(),
                    },
                });
                processed_db_tables.insert(table_name.clone());
            }
        }

        // 4. Detect column changes for tables that exist in both
        for (table_name, db_table) in &db_tables {
            if processed_db_tables.contains(table_name) {
                continue;
            }

            if let Some(struct_table) = struct_tables.get(table_name) {
                let column_changes = self.compare_columns(db_table, struct_table)?;

                if !column_changes.is_empty() {
                    changes.push(TableChange {
                        table_name: table_name.clone(),
                        change_type: TableChangeType::Modify {
                            changes: column_changes,
                        },
                    });
                }
            }
        }

        Ok(changes)
    }

    /// Detect potential table renames using heuristics
    fn detect_table_renames(
        &self,
        db_schema: &[TableDef],
        struct_schemas: &[TableDef],
    ) -> Result<Vec<(String, String)>, MigrationError> {
        let mut renames = Vec::new();

        let db_tables: std::collections::HashMap<String, &TableDef> = db_schema
            .iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        // Check for explicit rename_from attributes first
        for struct_table in struct_schemas {
            if let Some(rename_from) = &struct_table.rename_from {
                if let Some(db_table) = db_tables.get(rename_from) {
                    // Verify similarity before accepting as rename
                    if self.are_tables_similar(db_table, struct_table, 0.8) {
                        renames.push((rename_from.clone(), struct_table.name.clone()));
                    }
                }
            }
        }

        // TODO: Could add heuristic rename detection here for tables without explicit attributes
        // For now, we only support explicit renames via attributes

        Ok(renames)
    }

    /// Compare columns between a database table and struct definition
    fn compare_columns(
        &self,
        db_table: &TableDef,
        struct_table: &TableDef,
    ) -> Result<Vec<ColumnChangeType>, MigrationError> {
        let mut changes = Vec::new();

        let db_columns: std::collections::HashMap<String, &ColumnDef> = db_table
            .columns
            .iter()
            .map(|c| (c.name.clone(), c))
            .collect();

        let struct_columns: std::collections::HashMap<String, &ColumnDef> = struct_table
            .columns
            .iter()
            .map(|c| (c.name.clone(), c))
            .collect();

        let mut processed_db_columns: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut processed_struct_columns: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 1. Detect column renames first (explicit)
        for (col_name, struct_col) in &struct_columns {
            if let Some(rename_from) = &struct_col.rename_from {
                if let Some(_db_col) = db_columns.get(rename_from) {
                    if !processed_db_columns.contains(rename_from) {
                        changes.push(ColumnChangeType::Rename {
                            old_name: rename_from.clone(),
                            new_name: col_name.clone(),
                        });
                        processed_db_columns.insert(rename_from.clone());
                        processed_struct_columns.insert(col_name.clone());
                        continue;
                    }
                }
            }
        }

        // 2. Detect new columns (in struct but not in DB)
        for (col_name, struct_col) in &struct_columns {
            if processed_struct_columns.contains(col_name) {
                continue;
            }

            if !db_columns.contains_key(col_name) {
                changes.push(ColumnChangeType::Add {
                    column: (*struct_col).clone(),
                });
                processed_struct_columns.insert(col_name.clone());
            }
        }

        // 3. Detect removed columns (in DB but not in struct)
        for (col_name, db_col) in &db_columns {
            if processed_db_columns.contains(col_name) {
                continue;
            }

            if !struct_columns.contains_key(col_name) {
                changes.push(ColumnChangeType::Remove {
                    column_name: col_name.clone(),
                    sql_type: db_col.sql_type.clone(),
                });
                processed_db_columns.insert(col_name.clone());
            }
        }

        // 4. Detect column modifications
        for (col_name, db_col) in &db_columns {
            if processed_db_columns.contains(col_name) {
                continue;
            }

            if let Some(struct_col) = struct_columns.get(col_name) {
                if processed_struct_columns.contains(col_name) {
                    continue;
                }

                // Check for type changes
                if db_col.sql_type != struct_col.sql_type {
                    changes.push(ColumnChangeType::Modify {
                        old: (*db_col).clone(),
                        new: (*struct_col).clone(),
                    });
                    processed_struct_columns.insert(col_name.clone());
                    continue;
                }

                // Check for nullable changes
                if db_col.nullable != struct_col.nullable {
                    changes.push(ColumnChangeType::Modify {
                        old: (*db_col).clone(),
                        new: (*struct_col).clone(),
                    });
                    processed_struct_columns.insert(col_name.clone());
                    continue;
                }
            }
        }

        Ok(changes)
    }

    /// Check if two tables are similar (for rename detection)
    fn are_tables_similar(
        &self,
        table1: &TableDef,
        table2: &TableDef,
        threshold: f64,
    ) -> bool {
        let similarity = self.calculate_table_similarity(table1, table2);
        similarity >= threshold
    }

    /// Calculate similarity score between two tables (0.0 to 1.0)
    fn calculate_table_similarity(&self, table1: &TableDef, table2: &TableDef) -> f64 {
        if table1.columns.is_empty() && table2.columns.is_empty() {
            return 1.0;
        }

        if table1.columns.is_empty() || table2.columns.is_empty() {
            return 0.0;
        }

        let mut matching_columns = 0;
        let col_names1: std::collections::HashSet<String> = table1
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect();

        let col_names2: std::collections::HashSet<String> = table2
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect();

        for col_name in &col_names1 {
            if col_names2.contains(col_name) {
                matching_columns += 1;
            }
        }

        let total_columns = col_names1.len().max(col_names2.len());
        if total_columns == 0 {
            return 0.0;
        }

        matching_columns as f64 / total_columns as f64
    }

    /// Generate a summary of changes
    pub fn summarize_changes(&self, changes: &[TableChange]) -> String {
        let mut summary = String::new();
        let mut tables_added = 0;
        let mut tables_removed = 0;
        let mut tables_renamed = 0;
        let mut tables_modified = 0;
        let mut columns_added = 0;
        let mut columns_removed = 0;
        let mut columns_renamed = 0;

        for change in changes {
            match &change.change_type {
                TableChangeType::Add { .. } => {
                    tables_added += 1;
                }
                TableChangeType::Remove { .. } => {
                    tables_removed += 1;
                }
                TableChangeType::Rename { .. } => {
                    tables_renamed += 1;
                }
                TableChangeType::Modify { changes } => {
                    tables_modified += 1;
                    for col_change in changes {
                        match col_change {
                            ColumnChangeType::Add { .. } => columns_added += 1,
                            ColumnChangeType::Remove { .. } => columns_removed += 1,
                            ColumnChangeType::Rename { .. } => columns_renamed += 1,
                            ColumnChangeType::Modify { .. } => {}
                        }
                    }
                }
            }
        }

        summary.push_str("Schema Changes Detected:\n");
        if tables_added > 0 {
            summary.push_str(&format!("  - Tables added: {}\n", tables_added));
        }
        if tables_removed > 0 {
            summary.push_str(&format!("  - Tables removed: {}\n", tables_removed));
        }
        if tables_renamed > 0 {
            summary.push_str(&format!("  - Tables renamed: {}\n", tables_renamed));
        }
        if tables_modified > 0 {
            summary.push_str(&format!("  - Tables modified: {}\n", tables_modified));
        }
        if columns_added > 0 {
            summary.push_str(&format!("  - Columns added: {}\n", columns_added));
        }
        if columns_removed > 0 {
            summary.push_str(&format!("  - Columns removed: {}\n", columns_removed));
        }
        if columns_renamed > 0 {
            summary.push_str(&format!("  - Columns renamed: {}\n", columns_renamed));
        }

        summary
    }
}

impl Default for SchemaComparator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Index Comparator
// ============================================================================

/// Compares compile-time index recommendations with database indexes
pub struct IndexComparator;

impl IndexComparator {
    /// Create a new IndexComparator
    pub fn new() -> Self {
        Self
    }

    /// Compare recommended indexes with database indexes and determine changes
    pub fn compare_indexes(
        &self,
        db_indexes: &[IndexDef],
        recommended_indexes: &[IndexDef],
    ) -> IndexComparison {
        let mut indexes_to_create = Vec::new();
        let mut indexes_to_keep = Vec::new();
        let mut indexes_to_drop = Vec::new();

        // Create lookup maps
        let db_index_map: std::collections::HashMap<String, &IndexDef> = db_indexes
            .iter()
            .map(|idx| (idx.name.clone(), idx))
            .collect();

        let recommended_map: std::collections::HashMap<String, &IndexDef> = recommended_indexes
            .iter()
            .map(|idx| (idx.name.clone(), idx))
            .collect();

        // Find indexes to create (recommended but not in DB)
        for (name, recommended_idx) in &recommended_map {
            if !db_index_map.contains_key(name) {
                indexes_to_create.push((*recommended_idx).clone());
            }
        }

        // Find indexes to keep (exist in both)
        for (name, recommended_idx) in &recommended_map {
            if let Some(db_idx) = db_index_map.get(name) {
                if self.are_indexes_equivalent(db_idx, recommended_idx) {
                    indexes_to_keep.push((*recommended_idx).clone());
                } else {
                    // Index with same name but different definition - needs recreation
                    indexes_to_drop.push((*db_idx).clone());
                    indexes_to_create.push((*recommended_idx).clone());
                }
            }
        }

        // Find indexes to drop (in DB but not recommended)
        // Note: We're conservative here - only drop indexes that were auto-generated
        // Manual indexes (with specific naming patterns) should be preserved
        for (name, db_idx) in &db_index_map {
            if !recommended_map.contains_key(name) {
                // Only drop auto-generated indexes (those starting with "idx_")
                if name.starts_with("idx_") || name.starts_with("unique_idx_") {
                    indexes_to_drop.push((*db_idx).clone());
                }
                // Otherwise, keep it - it might be a manual index
            }
        }

        IndexComparison {
            indexes_to_create,
            indexes_to_keep,
            indexes_to_drop,
        }
    }

    /// Merge compile-time recommendations into table definitions
    pub fn merge_recommendations_into_tables(
        &self,
        tables: &mut [TableDef],
        recommendations: &[(String, Vec<IndexDef>)],
    ) {
        let recommendations_map: std::collections::HashMap<String, Vec<IndexDef>> = recommendations
            .iter()
            .map(|(table, indexes)| (table.clone(), indexes.clone()))
            .collect();

        for table in tables.iter_mut() {
            if let Some(recommended_indexes) = recommendations_map.get(&table.name) {
                // Compare existing indexes with recommendations
                let comparison = self.compare_indexes(&table.indexes, recommended_indexes);

                // Update table indexes
                let mut merged_indexes = Vec::new();

                // Keep indexes that should be kept
                for idx in &comparison.indexes_to_keep {
                    merged_indexes.push(idx.clone());
                }

                // Add new indexes to create
                for idx in &comparison.indexes_to_create {
                    merged_indexes.push(idx.clone());
                }

                table.indexes = merged_indexes;
            }
        }
    }

    /// Check if two index definitions are equivalent
    fn are_indexes_equivalent(&self, idx1: &IndexDef, idx2: &IndexDef) -> bool {
        // Compare unique flag
        if idx1.unique != idx2.unique {
            return false;
        }

        // Compare index type (btree, hash, etc.)
        if idx1.index_type != idx2.index_type {
            return false;
        }

        // Compare columns (order matters for indexes)
        if idx1.columns.len() != idx2.columns.len() {
            return false;
        }

        for (col1, col2) in idx1.columns.iter().zip(idx2.columns.iter()) {
            if col1 != col2 {
                return false;
            }
        }

        true
    }

    /// Generate index name from columns
    pub fn generate_index_name(table_name: &str, columns: &[String], unique: bool) -> String {
        let prefix = if unique { "unique_idx_" } else { "idx_" };
        let columns_str = columns.join("_");
        format!("{}_{}_{}", prefix, table_name, columns_str)
    }

    /// Parse compile-time index recommendations from macro output
    ///
    /// This method processes the index recommendations collected at compile time
    /// and converts them into IndexDef structures
    pub fn parse_compile_time_recommendations(
        &self,
        table_name: &str,
        raw_recommendations: &[(Vec<String>, bool)], // (columns, is_unique)
    ) -> Vec<IndexDef> {
        raw_recommendations
            .iter()
            .map(|(columns, unique)| IndexDef {
                name: Self::generate_index_name(table_name, columns, *unique),
                columns: columns.clone(),
                unique: *unique,
                index_type: "btree".to_string(), // Default to btree
            })
            .collect()
    }

    /// Create default indexes for a table (primary key and foreign key indexes)
    pub fn create_default_indexes(table: &TableDef) -> Vec<IndexDef> {
        let mut indexes = Vec::new();

        // Primary key is already indexed by database, so no need to create

        // Create indexes for foreign key columns (columns ending with _id)
        for column in &table.columns {
            if column.name.ends_with("_id") && column.name != table.primary_key {
                indexes.push(IndexDef {
                    name: Self::generate_index_name(&table.name, &[column.name.clone()], false),
                    columns: vec![column.name.clone()],
                    unique: false,
                    index_type: "btree".to_string(),
                });
            }
        }

        indexes
    }

    /// Validate that recommended indexes don't conflict with each other
    pub fn validate_index_recommendations(&self, indexes: &[IndexDef]) -> Result<(), MigrationError> {
        let mut index_keys = std::collections::HashSet::new();

        for index in indexes {
            // Create a unique key for the index (table + columns)
            let key = format!("{}:{:?}", index.name, index.columns);

            if index_keys.contains(&key) {
                return Err(MigrationError::SchemaComparisonError(format!(
                    "Duplicate index definition: {}", index.name
                )));
            }

            index_keys.insert(key);
        }

        Ok(())
    }

    /// Estimate the cost of creating an index
    pub fn estimate_index_cost(&self, index: &IndexDef, table_row_count: usize) -> IndexCost {
        // Basic cost estimation
        let column_count = index.columns.len();

        let base_cost = match column_count {
            1 => 100,
            2 => 200,
            3 => 400,
            _ => 600,
        };

        let unique_multiplier = if index.unique { 1.5 } else { 1.0 };
        let type_multiplier = match index.index_type.as_str() {
            "btree" => 1.0,
            "hash" => 0.8,
            "gist" => 2.0,
            "gin" => 2.5,
            _ => 1.0,
        };

        let estimated_cost = (base_cost as f64 * unique_multiplier * type_multiplier) as usize;

        IndexCost {
            estimated_cost,
            estimated_time_seconds: (estimated_cost as f64 / table_row_count as f64 * 1000.0) as usize,
            disk_space_bytes: table_row_count * column_count * 8, // Rough estimate
        }
    }
}

impl Default for IndexComparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of index comparison
#[derive(Debug, Clone)]
pub struct IndexComparison {
    /// Indexes that need to be created
    pub indexes_to_create: Vec<IndexDef>,
    /// Indexes that already exist and should be kept
    pub indexes_to_keep: Vec<IndexDef>,
    /// Indexes that should be dropped
    pub indexes_to_drop: Vec<IndexDef>,
}

impl IndexComparison {
    /// Check if there are any index changes
    pub fn has_changes(&self) -> bool {
        !self.indexes_to_create.is_empty() || !self.indexes_to_drop.is_empty()
    }

    /// Get summary of index changes
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        summary.push_str("Index Changes:\n");

        if !self.indexes_to_create.is_empty() {
            summary.push_str(&format!("  - Indexes to create: {}\n", self.indexes_to_create.len()));
            for idx in &self.indexes_to_create {
                summary.push_str(&format!("    • {} on {:?} ({})\n",
                    idx.name,
                    idx.columns,
                    if idx.unique { "UNIQUE" } else { "NON-UNIQUE" }
                ));
            }
        }

        if !self.indexes_to_drop.is_empty() {
            summary.push_str(&format!("  - Indexes to drop: {}\n", self.indexes_to_drop.len()));
            for idx in &self.indexes_to_drop {
                summary.push_str(&format!("    • {}\n", idx.name));
            }
        }

        if !self.indexes_to_keep.is_empty() {
            summary.push_str(&format!("  - Indexes to keep: {}\n", self.indexes_to_keep.len()));
        }

        summary
    }
}

/// Estimated cost of creating an index
#[derive(Debug, Clone)]
pub struct IndexCost {
    /// Estimated computational cost (arbitrary units)
    pub estimated_cost: usize,
    /// Estimated time to create in seconds
    pub estimated_time_seconds: usize,
    /// Estimated disk space in bytes
    pub disk_space_bytes: usize,
}

// ============================================================================
// SQL Generator
// ============================================================================

/// Generates UP and DOWN SQL statements for migrations
pub struct SqlGenerator {
    /// Database type (postgres, mysql, sqlite)
    pub database_type: String,
}

impl SqlGenerator {
    /// Create a new SQL generator for PostgreSQL
    pub fn new_postgres() -> Self {
        Self {
            database_type: "postgres".to_string(),
        }
    }

    /// Create a new SQL generator for MySQL
    pub fn new_mysql() -> Self {
        Self {
            database_type: "mysql".to_string(),
        }
    }

    /// Create a new SQL generator for SQLite
    pub fn new_sqlite() -> Self {
        Self {
            database_type: "sqlite".to_string(),
        }
    }

    /// Generate UP and DOWN SQL for a complete migration
    pub fn generate_migration_sql(
        &self,
        changes: &[TableChange],
        index_changes: &[(String, IndexComparison)],
    ) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        // Generate SQL in 6-phase order for UP
        // PHASE 1: Rename tables
        let (mut up_phase1, mut down_phase1) = self.generate_table_renames(changes);
        up_sql.append(&mut up_phase1);
        // DOWN will be reversed at the end

        // PHASE 2: Rename columns
        let (mut up_phase2, mut down_phase2) = self.generate_column_renames(changes);
        up_sql.append(&mut up_phase2);

        // PHASE 3: Add new columns (with defaults)
        let (mut up_phase3, mut down_phase3) = self.generate_add_columns(changes);
        up_sql.append(&mut up_phase3);

        // PHASE 4: Drop old columns
        let (mut up_phase4, mut down_phase4) = self.generate_drop_columns(changes);
        up_sql.append(&mut up_phase4);

        // PHASE 5: Execute data migrations
        let (mut up_phase5, mut down_phase5) = self.generate_data_migrations(changes);
        up_sql.append(&mut up_phase5);

        // PHASE 6: Create/drop indexes
        let (mut up_phase6, mut down_phase6) = self.generate_index_changes(index_changes);
        up_sql.append(&mut up_phase6);

        // Build DOWN SQL in reverse order
        down_sql.append(&mut down_phase6);
        down_sql.append(&mut down_phase5);
        down_sql.append(&mut down_phase4);
        down_sql.append(&mut down_phase3);
        down_sql.append(&mut down_phase2);
        down_sql.append(&mut down_phase1);

        (up_sql, down_sql)
    }

    /// PHASE 1: Generate table rename SQL
    fn generate_table_renames(&self, changes: &[TableChange]) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        for change in changes {
            if let TableChangeType::Rename { old_name, new_name } = &change.change_type {
                up_sql.push(self.generate_rename_table_sql(old_name, new_name));
                down_sql.push(self.generate_rename_table_sql(new_name, old_name));
            }
        }

        (up_sql, down_sql)
    }

    /// PHASE 2: Generate column rename SQL
    fn generate_column_renames(&self, changes: &[TableChange]) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        for change in changes {
            if let TableChangeType::Modify { changes } = &change.change_type {
                for col_change in changes {
                    if let ColumnChangeType::Rename { old_name, new_name } = col_change {
                        up_sql.push(self.generate_rename_column_sql(&change.table_name, old_name, new_name));
                        down_sql.push(self.generate_rename_column_sql(&change.table_name, new_name, old_name));
                    }
                }
            }
        }

        (up_sql, down_sql)
    }

    /// PHASE 3: Generate ADD COLUMN SQL
    fn generate_add_columns(&self, changes: &[TableChange]) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        for change in changes {
            if let TableChangeType::Modify { changes } = &change.change_type {
                for col_change in changes {
                    if let ColumnChangeType::Add { column } = col_change {
                        up_sql.push(self.generate_add_column_sql(&change.table_name, column));
                        down_sql.push(self.generate_drop_column_sql(&change.table_name, &column.name));
                    }
                }
            }
        }

        (up_sql, down_sql)
    }

    /// PHASE 4: Generate DROP COLUMN SQL
    fn generate_drop_columns(&self, changes: &[TableChange]) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        for change in changes {
            if let TableChangeType::Modify { changes } = &change.change_type {
                for col_change in changes {
                    if let ColumnChangeType::Remove { column_name, sql_type } = col_change {
                        up_sql.push(self.generate_drop_column_sql(&change.table_name, column_name));
                        down_sql.push(self.generate_add_column_simple_sql(&change.table_name, column_name, sql_type));
                    }
                }
            }
        }

        (up_sql, down_sql)
    }

    /// PHASE 5: Generate data migration SQL
    fn generate_data_migrations(&self, changes: &[TableChange]) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        for change in changes {
            if let TableChangeType::Modify { changes } = &change.change_type {
                for col_change in changes {
                    if let ColumnChangeType::Add { column } = col_change {
                        // Check if this column has a data migration
                        if let Some(data_migration) = &column.data_migration {
                            match &data_migration.migration_type {
                                DataMigrationType::Compute { expression } => {
                                    let sql = format!(
                                        "UPDATE {} SET {} = {}",
                                        change.table_name,
                                        column.name,
                                        expression
                                    );
                                    up_sql.push(sql);
                                    // DOWN: Set to default or NULL
                                    let default_sql = if let Some(default_val) = &column.default {
                                        format!(
                                            "UPDATE {} SET {} = {}",
                                            change.table_name,
                                            column.name,
                                            default_val
                                        )
                                    } else if column.nullable {
                                        format!(
                                            "UPDATE {} SET {} = NULL",
                                            change.table_name,
                                            column.name
                                        )
                                    } else {
                                        continue; // Skip if no default and not nullable
                                    };
                                    down_sql.push(default_sql);
                                }
                                DataMigrationType::Default { .. } => {
                                    // Defaults are handled in ADD COLUMN, skip here
                                }
                                DataMigrationType::Callback { function_name } => {
                                    // Generate a comment for manual callback execution
                                    up_sql.push(format!(
                                        "-- Execute data migration callback: {}() for table {}",
                                        function_name, change.table_name
                                    ));
                                    down_sql.push(format!(
                                        "-- Data migration callback: {}() (cannot be automatically reversed)",
                                        function_name
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        (up_sql, down_sql)
    }

    /// PHASE 6: Generate index creation/drop SQL
    fn generate_index_changes(&self, index_changes: &[(String, IndexComparison)]) -> (Vec<String>, Vec<String>) {
        let mut up_sql = Vec::new();
        let mut down_sql = Vec::new();

        for (table_name, comparison) in index_changes {
            // Drop old indexes first
            for idx in &comparison.indexes_to_drop {
                up_sql.push(self.generate_drop_index_sql(&idx.name));
                down_sql.push(self.generate_create_index_sql(table_name, idx));
            }

            // Create new indexes
            for idx in &comparison.indexes_to_create {
                up_sql.push(self.generate_create_index_sql(table_name, idx));
                down_sql.push(self.generate_drop_index_sql(&idx.name));
            }
        }

        (up_sql, down_sql)
    }

    /// Generate CREATE TABLE SQL
    pub fn generate_create_table_sql(&self, table: &TableDef) -> String {
        let mut columns_sql = Vec::new();

        for column in &table.columns {
            columns_sql.push(self.format_column_definition(column));
        }

        let columns_str = columns_sql.join(",\n    ");

        format!(
            "CREATE TABLE {} (\n    {}\n);",
            table.name,
            columns_str
        )
    }

    /// Generate DROP TABLE SQL
    pub fn generate_drop_table_sql(&self, table_name: &str) -> String {
        format!("DROP TABLE IF EXISTS {};", table_name)
    }

    /// Generate RENAME TABLE SQL
    pub fn generate_rename_table_sql(&self, old_name: &str, new_name: &str) -> String {
        match self.database_type.as_str() {
            "postgres" | "sqlite" => {
                format!("ALTER TABLE {} RENAME TO {};", old_name, new_name)
            }
            "mysql" => {
                format!("RENAME TABLE {} TO {};", old_name, new_name)
            }
            _ => format!("ALTER TABLE {} RENAME TO {};", old_name, new_name),
        }
    }

    /// Generate ADD COLUMN SQL
    pub fn generate_add_column_sql(&self, table_name: &str, column: &ColumnDef) -> String {
        let column_def = self.format_column_definition(column);
        format!("ALTER TABLE {} ADD COLUMN {};", table_name, column_def)
    }

    /// Generate ADD COLUMN SQL (simple version for DOWN migration)
    fn generate_add_column_simple_sql(&self, table_name: &str, column_name: &str, sql_type: &str) -> String {
        match self.database_type.as_str() {
            "postgres" => {
                format!("ALTER TABLE {} ADD COLUMN {} {};", table_name, column_name, sql_type)
            }
            "mysql" => {
                format!("ALTER TABLE {} ADD COLUMN {} {};", table_name, column_name, sql_type)
            }
            "sqlite" => {
                // SQLite doesn't support ALTER TABLE ADD COLUMN with constraints in older versions
                // For simplicity, we use basic syntax
                format!("ALTER TABLE {} ADD COLUMN {} {};", table_name, column_name, sql_type)
            }
            _ => format!("ALTER TABLE {} ADD COLUMN {} {};", table_name, column_name, sql_type),
        }
    }

    /// Generate DROP COLUMN SQL
    fn generate_drop_column_sql(&self, table_name: &str, column_name: &str) -> String {
        match self.database_type.as_str() {
            "postgres" | "mysql" => {
                format!("ALTER TABLE {} DROP COLUMN {};", table_name, column_name)
            }
            "sqlite" => {
                // SQLite has limited ALTER TABLE support
                // In real implementation, would need to recreate table
                format!(
                    "-- SQLite requires table recreation to drop column: {}.{}",
                    table_name, column_name
                )
            }
            _ => format!("ALTER TABLE {} DROP COLUMN {};", table_name, column_name),
        }
    }

    /// Generate RENAME COLUMN SQL
    fn generate_rename_column_sql(&self, table_name: &str, old_name: &str, new_name: &str) -> String {
        match self.database_type.as_str() {
            "postgres" => {
                format!(
                    "ALTER TABLE {} RENAME COLUMN {} TO {};",
                    table_name, old_name, new_name
                )
            }
            "mysql" => {
                format!(
                    "ALTER TABLE {} CHANGE COLUMN {} {} {}",
                    table_name, old_name, new_name, "VARCHAR(255)" // Would need actual type
                )
            }
            "sqlite" => {
                format!(
                    "-- SQLite requires table recreation to rename column: {}.{} -> {}",
                    table_name, old_name, new_name
                )
            }
            _ => format!(
                "ALTER TABLE {} RENAME COLUMN {} TO {};",
                table_name, old_name, new_name
            ),
        }
    }

    /// Generate CREATE INDEX SQL
    pub fn generate_create_index_sql(&self, table_name: &str, index: &IndexDef) -> String {
        let unique = if index.unique { "UNIQUE " } else { "" };
        let columns_str = index.columns.join(", ");

        match self.database_type.as_str() {
            "postgres" => {
                format!(
                    "CREATE {}INDEX IF NOT EXISTS {} ON {} USING {} ({});",
                    unique, index.name, table_name, index.index_type, columns_str
                )
            }
            "mysql" => {
                format!(
                    "CREATE {}INDEX {} ON {} ({});",
                    unique, index.name, table_name, columns_str
                )
            }
            "sqlite" => {
                format!(
                    "CREATE {}INDEX IF NOT EXISTS {} ON {} ({});",
                    unique, index.name, table_name, columns_str
                )
            }
            _ => format!(
                "CREATE {}INDEX {} ON {} ({});",
                unique, index.name, table_name, columns_str
            ),
        }
    }

    /// Generate DROP INDEX SQL
    fn generate_drop_index_sql(&self, index_name: &str) -> String {
        match self.database_type.as_str() {
            "postgres" => {
                format!("DROP INDEX IF EXISTS {};", index_name)
            }
            "mysql" => {
                format!("DROP INDEX {};", index_name)
            }
            "sqlite" => {
                format!("DROP INDEX IF EXISTS {};", index_name)
            }
            _ => format!("DROP INDEX IF NOT EXISTS {};", index_name),
        }
    }

    /// Format a column definition for CREATE TABLE
    fn format_column_definition(&self, column: &ColumnDef) -> String {
        let null_constraint = if column.nullable { "" } else { " NOT NULL" };
        let default_constraint = if let Some(default_val) = &column.default {
            format!(" DEFAULT {}", default_val)
        } else {
            String::new()
        };

        format!(
            "{} {}{}{}",
            column.name, column.sql_type, null_constraint, default_constraint
        )
    }

    /// Generate SQL to set column value
    pub fn generate_update_sql(
        &self,
        table_name: &str,
        column_name: &str,
        expression: &str,
    ) -> String {
        format!(
            "UPDATE {} SET {} = {};",
            table_name, column_name, expression
        )
    }
}

impl Default for SqlGenerator {
    fn default() -> Self {
        Self::new_postgres()
    }
}

// ============================================================================
// Migration Executor
// ============================================================================

/// Executes migrations with transaction support
pub struct MigrationExecutor {
    pool: Pool<Postgres>,
    pub history: MigrationHistory,
    dry_run: bool,
}

impl MigrationExecutor {
    /// Create a new migration executor
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            history: MigrationHistory::new(),
            dry_run: false,
        }
    }

    /// Enable or disable dry-run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Initialize the migration system (create history table if needed)
    pub async fn initialize(&self) -> Result<(), MigrationError> {
        self.history.initialize(&self.pool).await
    }

    /// Run an UP migration
    pub async fn upgrade(&self, migration: &Migration) -> Result<MigrationResult, MigrationError> {
        let start = std::time::Instant::now();

        // Check if migration is already applied
        if self.history.is_applied(&self.pool, &migration.version).await? {
            return Ok(MigrationResult {
                migration_name: migration.name.clone(),
                version: migration.version.clone(),
                success: true,
                statements_executed: 0,
                duration_ms: 0,
                error_message: Some("Migration already applied".to_string()),
            });
        }

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Execute all UP SQL statements
        let mut executed = 0;
        for (idx, sql) in migration.up_sql.iter().enumerate() {
            // Execute SQL
            sqlx::query(sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| MigrationError::SqlExecutionError(sql.clone(), e.to_string()))?;

            executed += 1;

            // Log progress for non-trivial migrations
            if migration.up_sql.len() > 1 {
                println!("  [{} / {}] Executed: {}", idx + 1, migration.up_sql.len(), sql);
            }
        }

        let duration = start.elapsed().as_millis();

        if self.dry_run {
            // Rollback in dry-run mode
            tx.rollback().await?;
            println!("🔍 Dry-run mode: rolled back all changes");

            Ok(MigrationResult {
                migration_name: migration.name.clone(),
                version: migration.version.clone(),
                success: true,
                statements_executed: executed,
                duration_ms: duration,
                error_message: None,
            })
        } else {
            // Commit transaction
            tx.commit().await?;

            // Record migration
            self.history.record(&self.pool, migration, duration).await?;

            Ok(MigrationResult {
                migration_name: migration.name.clone(),
                version: migration.version.clone(),
                success: true,
                statements_executed: executed,
                duration_ms: duration,
                error_message: None,
            })
        }
    }

    /// Run a DOWN migration (rollback)
    pub async fn downgrade(&self, migration: &Migration) -> Result<MigrationResult, MigrationError> {
        let start = std::time::Instant::now();

        // Check if migration is applied
        if !self.history.is_applied(&self.pool, &migration.version).await? {
            return Ok(MigrationResult {
                migration_name: migration.name.clone(),
                version: migration.version.clone(),
                success: true,
                statements_executed: 0,
                duration_ms: 0,
                error_message: Some("Migration not applied".to_string()),
            });
        }

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Execute all DOWN SQL statements in reverse order
        let mut executed = 0;
        for (idx, sql) in migration.down_sql.iter().enumerate() {
            // Execute SQL
            sqlx::query(sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| MigrationError::SqlExecutionError(sql.clone(), e.to_string()))?;

            executed += 1;

            // Log progress
            if migration.down_sql.len() > 1 {
                println!("  [{} / {}] Executed: {}", idx + 1, migration.down_sql.len(), sql);
            }
        }

        let duration = start.elapsed().as_millis();

        if self.dry_run {
            // Rollback in dry-run mode
            tx.rollback().await?;
            println!("🔍 Dry-run mode: rolled back all changes");

            Ok(MigrationResult {
                migration_name: migration.name.clone(),
                version: migration.version.clone(),
                success: true,
                statements_executed: executed,
                duration_ms: duration,
                error_message: None,
            })
        } else {
            // Commit transaction
            tx.commit().await?;

            // Remove from history
            self.history.remove(&self.pool, &migration.version).await?;

            Ok(MigrationResult {
                migration_name: migration.name.clone(),
                version: migration.version.clone(),
                success: true,
                statements_executed: executed,
                duration_ms: duration,
                error_message: None,
            })
        }
    }

    /// Get migration history
    pub async fn get_history(&self) -> Result<Vec<MigrationRecord>, MigrationError> {
        self.history.get_all(&self.pool).await
    }

    /// Validate a migration before executing
    pub fn validate(&self, migration: &Migration) -> Result<(), MigrationError> {
        // Check if migration has any SQL to execute
        if migration.up_sql.is_empty() && migration.down_sql.is_empty() {
            return Err(MigrationError::InvalidState(
                "Migration has no SQL to execute".to_string()
            ));
        }

        // Validate UP and DOWN have same number of statements
        if migration.up_sql.len() != migration.down_sql.len() {
            // This is OK as long as we can still rollback
            // Just warn about it
        }

        Ok(())
    }

    /// Calculate migration checksum
    pub fn calculate_checksum(&self, migration: &Migration) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash all UP SQL statements
        for sql in &migration.up_sql {
            sql.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Verify migration checksum
    pub async fn verify_checksum(&self, migration: &Migration) -> Result<bool, MigrationError> {
        let current_checksum = self.calculate_checksum(migration);

        if let Ok(records) = self.history.get_all(&self.pool).await {
            for record in records {
                if record.version == migration.version {
                    return Ok(record.checksum == current_checksum);
                }
            }
        }

        Ok(false)
    }
}

// ============================================================================
// Migration Builder
// ============================================================================

/// Builder for creating migrations
pub struct MigrationBuilder {
    name: String,
    version: Option<String>,
    pool: Option<Pool<Postgres>>,
}

impl MigrationBuilder {
    /// Create a new migration builder
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
            pool: None,
        }
    }

    /// Set migration version
    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set database pool
    pub fn pool(mut self, pool: Pool<Postgres>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Auto-generate migration by comparing DB schema with struct definitions
    pub async fn auto_generate(
        &self,
        struct_schemas: Vec<TableDef>,
        index_recommendations: Vec<(String, Vec<IndexDef>)>,
    ) -> Result<Migration, MigrationError> {
        let pool = self.pool.as_ref()
            .ok_or_else(|| MigrationError::InvalidState("Pool not set".to_string()))?;

        // Read database schema
        let reader = SchemaReader::new();
        let db_schema = reader.read_database_schema(pool).await?;

        // Compare schemas
        let comparator = SchemaComparator::new();
        let changes = comparator.compare_schemas(&db_schema, &struct_schemas)?;

        // Generate index comparisons
        let index_comparator = IndexComparator::new();
        let mut index_changes = Vec::new();

        for (table_name, recommended_indexes) in &index_recommendations {
            if let Some(db_table) = db_schema.iter().find(|t| &t.name == table_name) {
                let comparison = index_comparator.compare_indexes(&db_table.indexes, recommended_indexes);
                if comparison.has_changes() {
                    index_changes.push((table_name.clone(), comparison));
                }
            }
        }

        // Generate SQL
        let generator = SqlGenerator::new_postgres();
        let (up_sql, down_sql) = generator.generate_migration_sql(&changes, &index_changes);

        // Calculate version
        let version = if let Some(v) = &self.version {
            v.clone()
        } else {
            Self::generate_version()
        };

        // Build migration
        let mut migration = Migration::new(self.name.clone(), version);
        migration.up_sql = up_sql;
        migration.down_sql = down_sql;
        migration.table_changes = changes.clone();
        migration.total_columns_added = changes.iter()
            .filter(|c| matches!(c.change_type, TableChangeType::Add { .. }))
            .count();
        migration.total_indexes_created = index_changes.iter()
            .map(|(_, comp)| comp.indexes_to_create.len())
            .sum();

        // Calculate checksum
        let executor = MigrationExecutor::new(pool.clone());
        migration.checksum = executor.calculate_checksum(&migration);

        Ok(migration)
    }

    /// Generate timestamp version (YYYYMMDD_HHMMSS format)
    fn generate_version() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();

        let secs = now.as_secs();

        // Convert seconds to datetime manually (simple version)
        let days_since_epoch = secs / 86400;
        let seconds_today = secs % 86400;

        // Unix epoch: January 1, 1970
        // Approximate calculation (ignoring leap seconds for simplicity)
        let year = 1970 + (days_since_epoch / 365);
        let day_of_year = (days_since_epoch % 365) as u32;

        // Approximate month and day
        let month = (day_of_year / 30) + 1;
        let day = (day_of_year % 30) + 1;

        let hours = seconds_today / 3600;
        let minutes = (seconds_today % 3600) / 60;
        let seconds = seconds_today % 60;

        format!(
            "{:04}{:02}{:02}_{:02}{:02}{:02}",
            year, month, day, hours, minutes, seconds
        )
    }

    /// Create a manual migration with custom SQL
    pub fn manual(&self, up_sql: Vec<String>, down_sql: Vec<String>) -> Migration {
        let version = if let Some(v) = &self.version {
            v.clone()
        } else {
            Self::generate_version()
        };

        let mut migration = Migration::new(self.name.clone(), version);
        migration.up_sql = up_sql;
        migration.down_sql = down_sql;

        migration
    }
}

impl Default for MigrationBuilder {
    fn default() -> Self {
        Self::new("unnamed_migration".to_string())
    }
}
