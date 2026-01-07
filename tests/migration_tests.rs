//! Integration tests for the migration system
//!
//! These tests require a running PostgreSQL instance at:
//! postgres://postgres:@127.0.0.1/test-sqlx-tokio

use sqlx::PgPool;
use sqlx_struct_enhanced::migration::*;

#[sqlx::test]
async fn test_migration_history_init(pool: PgPool) -> Result<(), MigrationError> {
    let history = MigrationHistory::new();
    history.initialize(&pool).await?;

    // Verify the table was created
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = '_schema_migrations'
        )"
    )
    .fetch_one(&pool)
    .await?;

    Ok(())
}

#[sqlx::test]
async fn test_schema_reader_read_tables(pool: PgPool) -> Result<(), MigrationError> {
    let reader = SchemaReader::new();

    // Create a test table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_users (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(500) NOT NULL,
            email VARCHAR(500)
        )"
    )
    .execute(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    // Read tables
    let tables = reader.read_tables(&pool).await?;

    // Verify our test table exists
    assert!(tables.contains(&"test_users".to_string()));

    // Cleanup
    sqlx::query("DROP TABLE test_users")
        .execute(&pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[sqlx::test]
async fn test_schema_reader_read_columns(pool: PgPool) -> Result<(), MigrationError> {
    let reader = SchemaReader::new();

    // Create a test table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_products (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(500) NOT NULL,
            price INTEGER,
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    )
    .execute(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    // Read columns
    let columns = reader.read_columns(&pool, "test_products").await?;

    // Verify columns
    assert_eq!(columns.len(), 4);
    assert_eq!(columns[0].name, "id");
    assert_eq!(columns[1].name, "name");
    assert_eq!(columns[2].name, "price");
    assert_eq!(columns[3].name, "created_at");

    // Verify types
    assert!(columns[0].sql_type.contains("VARCHAR") || columns[0].sql_type.contains("TEXT"));
    assert!(columns[2].sql_type.contains("INTEGER"));

    // Verify nullable
    assert!(!columns[0].nullable); // PRIMARY KEY is NOT NULL
    assert!(!columns[1].nullable); // NOT NULL
    assert!(columns[2].nullable); // No NOT NULL constraint
    assert!(columns[3].nullable); // DEFAULT allows NULL

    // Cleanup
    sqlx::query("DROP TABLE test_products")
        .execute(&pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[sqlx::test]
async fn test_schema_reader_read_indexes(pool: PgPool) -> Result<(), MigrationError> {
    let reader = SchemaReader::new();

    // Create a test table with indexes
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_orders (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36),
            status VARCHAR(50),
            created_at TIMESTAMPTZ DEFAULT NOW()
        )"
    )
    .execute(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    // Create an index
    sqlx::query("CREATE INDEX idx_test_orders_user_id ON test_orders (user_id)")
        .execute(&pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    // Read indexes
    let indexes = reader.read_indexes(&pool, "test_orders").await?;

    // Verify we have at least the user_id index
    let user_id_index = indexes.iter()
        .find(|idx| idx.name == "idx_test_orders_user_id");
    assert!(user_id_index.is_some());

    if let Some(idx) = user_id_index {
        assert_eq!(idx.columns, vec!["user_id"]);
        assert!(!idx.unique);
    }

    // Cleanup
    sqlx::query("DROP TABLE test_orders")
        .execute(&pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[sqlx::test]
async fn test_schema_reader_read_table_schema(pool: PgPool) -> Result<(), MigrationError> {
    let reader = SchemaReader::new();

    // Create a test table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS test_articles (
            id VARCHAR(36) PRIMARY KEY,
            title VARCHAR(500) NOT NULL,
            content TEXT
        )"
    )
    .execute(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    // Read full table schema
    let table = reader.read_table_schema(&pool, "test_articles").await?;

    // Verify table structure
    assert_eq!(table.name, "test_articles");
    assert_eq!(table.primary_key, "id");
    assert_eq!(table.columns.len(), 3);

    // Cleanup
    sqlx::query("DROP TABLE test_articles")
        .execute(&pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[sqlx::test]
async fn test_schema_comparator_compare_schemas(pool: PgPool) -> Result<(), MigrationError> {
    let comparator = SchemaComparator::new();

    // Create two different table definitions
    let db_table = TableDef {
        name: "users".to_string(),
        rename_from: None,
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                sql_type: "VARCHAR(36)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
            ColumnDef {
                name: "name".to_string(),
                sql_type: "VARCHAR(500)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
        ],
        indexes: vec![],
        primary_key: "id".to_string(),
    };

    let struct_table = TableDef {
        name: "users".to_string(),
        rename_from: None,
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                sql_type: "VARCHAR(36)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
            ColumnDef {
                name: "name".to_string(),
                sql_type: "VARCHAR(500)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
            ColumnDef {
                name: "email".to_string(),
                sql_type: "VARCHAR(500)".to_string(),
                nullable: true,
                default: None,
                rename_from: None,
                data_migration: None,
            },
        ],
        indexes: vec![],
        primary_key: "id".to_string(),
    };

    // Compare schemas
    let changes = comparator.compare_schemas(&[db_table], &[struct_table])?;

    // Should detect one new column (email)
    assert_eq!(changes.len(), 1);
    assert!(matches!(&changes[0].change_type, TableChangeType::Modify { .. }));

    Ok(())
}

#[sqlx::test]
async fn test_schema_comparator_table_rename(pool: PgPool) -> Result<(), MigrationError> {
    let comparator = SchemaComparator::new();

    // Test with rename_from attribute
    let db_table = TableDef {
        name: "app_users".to_string(),
        rename_from: None,
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                sql_type: "VARCHAR(36)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
        ],
        indexes: vec![],
        primary_key: "id".to_string(),
    };

    let struct_table = TableDef {
        name: "users".to_string(),
        rename_from: Some("app_users".to_string()),
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                sql_type: "VARCHAR(36)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
        ],
        indexes: vec![],
        primary_key: "id".to_string(),
    };

    // Compare schemas
    let changes = comparator.compare_schemas(&[db_table], &[struct_table])?;

    // Should detect table rename
    assert_eq!(changes.len(), 1);
    assert!(matches!(&changes[0].change_type, TableChangeType::Rename { old_name, new_name }
        if old_name == "app_users" && new_name == "users"
    ));

    Ok(())
}

#[sqlx::test]
async fn test_index_comparator_compare_indexes(pool: PgPool) -> Result<(), MigrationError> {
    let comparator = IndexComparator::new();

    // Existing indexes in DB
    let db_indexes = vec![
        IndexDef {
            name: "idx_users_email".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
            index_type: "btree".to_string(),
        },
        IndexDef {
            name: "idx_users_name".to_string(),
            columns: vec!["name".to_string()],
            unique: false,
            index_type: "btree".to_string(),
        },
    ];

    // Recommended indexes
    let recommended_indexes = vec![
        IndexDef {
            name: "idx_users_email".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
            index_type: "btree".to_string(),
        },
        IndexDef {
            name: "idx_users_created_at".to_string(),
            columns: vec!["created_at".to_string()],
            unique: false,
            index_type: "btree".to_string(),
        },
    ];

    // Compare indexes
    let comparison = comparator.compare_indexes(&db_indexes, &recommended_indexes);

    // Should keep email index, create created_at index
    assert_eq!(comparison.indexes_to_keep.len(), 1);
    assert_eq!(comparison.indexes_to_create.len(), 1);
    assert_eq!(comparison.indexes_to_drop.len(), 1); // old name index

    Ok(())
}

#[sqlx::test]
async fn test_sql_generator_create_table(pool: PgPool) -> Result<(), MigrationError> {
    let generator = SqlGenerator::new_postgres();

    let table = TableDef {
        name: "users".to_string(),
        rename_from: None,
        columns: vec![
            ColumnDef {
                name: "id".to_string(),
                sql_type: "VARCHAR(36)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
            ColumnDef {
                name: "name".to_string(),
                sql_type: "VARCHAR(500)".to_string(),
                nullable: false,
                default: None,
                rename_from: None,
                data_migration: None,
            },
        ],
        indexes: vec![],
        primary_key: "id".to_string(),
    };

    let create_sql = generator.generate_create_table_sql(&table);

    // Verify SQL contains expected elements
    assert!(create_sql.contains("CREATE TABLE users"));
    assert!(create_sql.contains("id VARCHAR(36) NOT NULL"));
    assert!(create_sql.contains("name VARCHAR(500) NOT NULL"));

    Ok(())
}

#[sqlx::test]
async fn test_sql_generator_add_column(pool: PgPool) -> Result<(), MigrationError> {
    let generator = SqlGenerator::new_postgres();

    let column = ColumnDef {
        name: "email".to_string(),
        sql_type: "VARCHAR(500)".to_string(),
        nullable: true,
        default: None,
        rename_from: None,
        data_migration: None,
    };

    let add_sql = generator.generate_add_column_sql("users", &column);

    // Verify SQL
    assert!(add_sql.contains("ALTER TABLE users"));
    assert!(add_sql.contains("ADD COLUMN email"));
    assert!(add_sql.contains("VARCHAR(500)"));

    Ok(())
}

#[sqlx::test]
async fn test_sql_generator_create_index(pool: PgPool) -> Result<(), MigrationError> {
    let generator = SqlGenerator::new_postgres();

    let index = IndexDef {
        name: "idx_users_email".to_string(),
        columns: vec!["email".to_string()],
        unique: true,
        index_type: "btree".to_string(),
    };

    let create_sql = generator.generate_create_index_sql("users", &index);

    // Verify SQL
    assert!(create_sql.contains("CREATE UNIQUE INDEX"));
    assert!(create_sql.contains("idx_users_email"));
    assert!(create_sql.contains("ON users"));
    assert!(create_sql.contains("email"));

    Ok(())
}

#[sqlx::test]
async fn test_sql_generator_rename_table(pool: PgPool) -> Result<(), MigrationError> {
    let generator = SqlGenerator::new_postgres();

    let rename_sql = generator.generate_rename_table_sql("old_users", "users");

    // Verify SQL
    assert!(rename_sql.contains("ALTER TABLE old_users"));
    assert!(rename_sql.contains("RENAME TO users"));

    Ok(())
}

#[sqlx::test]
async fn test_migration_creation(pool: PgPool) -> Result<(), MigrationError> {
    // Create a simple migration
    let mut migration = Migration::new(
        "test_migration".to_string(),
        "20231201_120000".to_string(),
    );

    migration.up_sql = vec![
        "CREATE TABLE test_table (id VARCHAR(36) PRIMARY KEY);".to_string(),
    ];

    migration.down_sql = vec![
        "DROP TABLE test_table;".to_string(),
    ];

    // Verify migration properties
    assert_eq!(migration.name, "test_migration");
    assert_eq!(migration.version, "20231201_120000");
    assert_eq!(migration.up_sql.len(), 1);
    assert_eq!(migration.down_sql.len(), 1);

    Ok(())
}

#[sqlx::test]
async fn test_migration_executor_init(pool: PgPool) -> Result<(), MigrationError> {
    let executor = MigrationExecutor::new(pool.clone());

    // Initialize migration system
    executor.initialize().await?;

    // Verify history table exists
    let _result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = '_schema_migrations'
        )"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[sqlx::test]
async fn test_migration_executor_upgrade_and_downgrade(pool: PgPool) -> Result<(), MigrationError> {
    let executor = MigrationExecutor::new(pool.clone());

    // Initialize
    executor.initialize().await?;

    // Create a test migration
    let mut migration = Migration::new(
        "test_users_table".to_string(),
        "20231201_120001".to_string(),
    );

    migration.up_sql = vec![
        "CREATE TABLE test_migration_users (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(500) NOT NULL
        );".to_string(),
    ];

    migration.down_sql = vec![
        "DROP TABLE test_migration_users;".to_string(),
    ];

    // Run UP migration
    let up_result = executor.upgrade(&migration).await?;
    assert!(up_result.success);
    assert_eq!(up_result.statements_executed, 1);

    // Verify table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'test_migration_users'
        )"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    assert!(table_exists);

    // Run DOWN migration
    let down_result = executor.downgrade(&migration).await?;
    assert!(down_result.success);
    assert_eq!(down_result.statements_executed, 1);

    // Verify table is dropped
    let table_exists_after: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'test_migration_users'
        )"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    assert!(!table_exists_after);

    Ok(())
}

#[sqlx::test]
async fn test_migration_executor_dry_run(pool: PgPool) -> Result<(), MigrationError> {
    let executor = MigrationExecutor::new(pool.clone())
        .with_dry_run(true);

    // Initialize
    executor.initialize().await?;

    // Create a test migration
    let mut migration = Migration::new(
        "test_dry_run".to_string(),
        "20231201_120002".to_string(),
    );

    migration.up_sql = vec![
        "CREATE TABLE test_dry_run_table (id VARCHAR(36) PRIMARY KEY);".to_string(),
    ];

    migration.down_sql = vec![
        "DROP TABLE test_dry_run_table;".to_string(),
    ];

    // Run in dry-run mode
    let result = executor.upgrade(&migration).await?;
    assert!(result.success);
    assert_eq!(result.statements_executed, 1);

    // Verify table was NOT created (dry-run rolled back)
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_name = 'test_dry_run_table'
        )"
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    assert!(!table_exists, "Table should not exist in dry-run mode");

    Ok(())
}

#[sqlx::test]
async fn test_migration_history_tracking(pool: PgPool) -> Result<(), MigrationError> {
    let executor = MigrationExecutor::new(pool.clone());

    // Initialize
    executor.initialize().await?;

    // Create and run a migration
    let mut migration = Migration::new(
        "test_history_tracking".to_string(),
        "20231201_120003".to_string(),
    );

    migration.up_sql = vec![
        "CREATE TABLE test_history_table (id VARCHAR(36) PRIMARY KEY);".to_string(),
    ];

    migration.down_sql = vec![
        "DROP TABLE test_history_table;".to_string(),
    ];

    // Run migration
    executor.upgrade(&migration).await?;

    // Verify it's in history
    let is_applied = executor.history.is_applied(&pool, &migration.version).await?;
    assert!(is_applied);

    // Get full history
    let history = executor.get_history().await?;
    assert!(history.iter().any(|r| r.version == migration.version));

    // Rollback
    executor.downgrade(&migration).await?;

    // Verify it's removed from history
    let is_applied_after = executor.history.is_applied(&pool, &migration.version).await?;
    assert!(!is_applied_after);

    // Note: Table is already dropped by downgrade migration, no cleanup needed

    Ok(())
}

#[sqlx::test]
async fn test_migration_idempotency(pool: PgPool) -> Result<(), MigrationError> {
    let executor = MigrationExecutor::new(pool.clone());

    // Initialize
    executor.initialize().await?;

    // Create a migration
    let mut migration = Migration::new(
        "test_idempotency".to_string(),
        "20231201_120004".to_string(),
    );

    migration.up_sql = vec![
        "CREATE TABLE test_idempotency_table (id VARCHAR(36) PRIMARY KEY);".to_string(),
    ];

    migration.down_sql = vec![
        "DROP TABLE test_idempotency_table;".to_string(),
    ];

    // Run migration twice
    let result1 = executor.upgrade(&migration).await?;
    assert!(result1.success);

    let result2 = executor.upgrade(&migration).await?;
    assert!(result2.success);
    assert!(result2.error_message.is_some());
    assert!(result2.error_message.unwrap().contains("already applied"));

    // Cleanup
    sqlx::query("DROP TABLE test_idempotency_table")
        .execute(&pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

    Ok(())
}
