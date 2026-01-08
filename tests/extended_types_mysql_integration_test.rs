// Integration test for extended BindProxy data types - MySQL
//
// This test verifies that all extended types work correctly with MySQL

#[cfg(feature = "mysql")]
#[cfg(test)]
mod extended_types_mysql_integration_tests {
    use sqlx::{FromRow, MySqlPool};
    use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt, EnhancedQuery};
    use serial_test::serial;
    use chrono::{NaiveDate, NaiveTime, NaiveDateTime, Utc, TimeZone};

    // Helper function to get database connection
    async fn get_test_pool() -> MySqlPool {
        let database_url = std::env::var("MYSQL_DATABASE_URL")
            .unwrap_or_else(|_| "mysql://root:test@127.0.0.1:3306/test_sqlx".to_string());

        // Wait a bit for MySQL to be ready
        for _ in 0..10 {
            match sqlx::MySqlPool::connect(&database_url).await {
                Ok(pool) => return pool,
                Err(_) => tokio::time::sleep(tokio::time::Duration::from_millis(500)).await,
            }
        }

        panic!("Failed to connect to MySQL test database after multiple attempts");
    }

    // Helper function to create test table with all supported types
    async fn create_extended_types_table(pool: &MySqlPool) {
        let drop_query = "DROP TABLE IF EXISTS extended_types_test";
        sqlx::query(drop_query).execute(pool).await
            .expect("Failed to drop existing table");

        let create_query = r#"
            CREATE TABLE extended_types_test (
                id VARCHAR(36) PRIMARY KEY,

                -- Numeric types
                tiny_int SMALLINT,
                small_int SMALLINT,
                float_val FLOAT,

                -- Unsigned integers (stored as TEXT)
                tiny_uint TEXT,
                small_uint TEXT,
                medium_uint TEXT,
                big_uint TEXT,

                -- Date/time types (stored as TEXT)
                birth_date TEXT,
                wake_time TEXT,
                created_at TEXT,
                timestamp TEXT,

                -- Binary
                data LONGBLOB,

                -- UUID (stored as TEXT)
                parent_id TEXT,

                -- JSON (stored as TEXT)
                metadata TEXT
            )
        "#;

        sqlx::query(create_query).execute(pool).await
            .expect("Failed to create test table");
    }

    // ============================================================================
    // Test Struct with all extended types
    // ============================================================================

    #[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
    #[table_name = "extended_types_test"]
    pub struct ExtendedTypesTest {
        pub id: String,

        // Signed numeric types (native database types)
        pub tiny_int: Option<i16>,
        pub small_int: Option<i16>,
        pub float_val: Option<f32>,

        // Unsigned integers (stored as TEXT)
        pub tiny_uint: Option<String>,
        pub small_uint: Option<String>,
        pub medium_uint: Option<String>,
        pub big_uint: Option<String>,

        // Date/time types (stored as TEXT)
        pub birth_date: Option<String>,
        pub wake_time: Option<String>,
        pub created_at: Option<String>,
        pub timestamp: Option<String>,

        // Binary
        pub data: Option<Vec<u8>>,

        // UUID (stored as TEXT)
        pub parent_id: Option<String>,

        // JSON (stored as TEXT)
        pub metadata: Option<String>,
    }

    // ============================================================================
    // Test 1: Insert and select with all numeric types
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_insert_select_numeric() {
        println!("🔧 Starting test_mysql_extended_types_insert_select_numeric...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;
        println!("✅ Table created");

        // Create test record with all numeric types
        let mut record = ExtendedTypesTest {
            id: "test-numeric-1".to_string(),
            tiny_int: Some(127),       // i8 -> stored as i16
            small_int: Some(32767),    // i16
            float_val: Some(3.14159),  // f32

            tiny_uint: Some("255".to_string()),      // u8 -> String
            small_uint: Some("65535".to_string()),   // u16 -> String
            medium_uint: Some("4294967295".to_string()), // u32 -> String
            big_uint: Some("18446744073709551615".to_string()), // u64 -> String

            birth_date: None,
            wake_time: None,
            created_at: None,
            timestamp: None,
            data: None,
            parent_id: None,
            metadata: None,
        };

        // Insert using bind_proxy for various types
        record.insert_bind().execute(&pool).await
            .expect("Failed to insert record");

        println!("✅ Inserted record");

        // Select using bind_proxy
        let selected = ExtendedTypesTest::where_query_ext("id = ?")
            .bind_proxy("test-numeric-1")
            .fetch_optional(&pool)
            .await
            .expect("Failed to select record");

        assert!(selected.is_some());
        let selected = selected.unwrap();

        assert_eq!(selected.tiny_int, Some(127));
        assert_eq!(selected.small_int, Some(32767));
        assert!(selected.float_val.is_some());
        assert!((selected.float_val.unwrap() - 3.14159).abs() < 0.0001);

        assert_eq!(selected.tiny_uint, Some("255".to_string()));
        assert_eq!(selected.small_uint, Some("65535".to_string()));
        assert_eq!(selected.medium_uint, Some("4294967295".to_string()));
        assert_eq!(selected.big_uint, Some("18446744073709551615".to_string()));

        println!("✅ Numeric types test passed");
    }

    // ============================================================================
    // Test 2: Chrono date/time types
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_chrono_datetime() {
        println!("🔧 Starting test_mysql_extended_types_chrono_datetime...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;

        let birth_date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
        let wake_time = NaiveTime::from_hms_opt(7, 30, 0).unwrap();
        let created_at = NaiveDateTime::from_timestamp_opt(1704067200, 0).unwrap();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 45).unwrap();

        let mut record = ExtendedTypesTest {
            id: "test-datetime-1".to_string(),
            tiny_int: None,
            small_int: None,
            float_val: None,
            tiny_uint: None,
            small_uint: None,
            medium_uint: None,
            big_uint: None,

            birth_date: Some(birth_date.format("%Y-%m-%d").to_string()),
            wake_time: Some(wake_time.format("%H:%M:%S%.9f").to_string()),
            created_at: Some(created_at.format("%Y-%m-%d %H:%M:%S%.9f").to_string()),
            timestamp: Some(timestamp.format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string()),

            data: None,
            parent_id: None,
            metadata: None,
        };

        record.insert_bind().execute(&pool).await
            .expect("Failed to insert record");

        // Test using bind_proxy with chrono types
        let selected = ExtendedTypesTest::where_query_ext("id = ?")
            .bind_proxy("test-datetime-1")
            .fetch_one(&pool)
            .await
            .expect("Failed to select record");

        assert!(selected.birth_date.is_some());
        assert!(selected.birth_date.unwrap().contains("1990-05-15"));

        assert!(selected.wake_time.is_some());
        assert!(selected.wake_time.unwrap().starts_with("07:30:00"));

        assert!(selected.created_at.is_some());
        assert!(selected.created_at.unwrap().starts_with("2024-01-01"));

        assert!(selected.timestamp.is_some());
        assert!(selected.timestamp.unwrap().contains("2024-01-15"));

        println!("✅ Chrono date/time types test passed");
    }

    // ============================================================================
    // Test 3: Binary types
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_binary() {
        println!("🔧 Starting test_mysql_extended_types_binary...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;

        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 255, 254, 253];

        let mut record = ExtendedTypesTest {
            id: "test-binary-1".to_string(),
            tiny_int: None,
            small_int: None,
            float_val: None,
            tiny_uint: None,
            small_uint: None,
            medium_uint: None,
            big_uint: None,
            birth_date: None,
            wake_time: None,
            created_at: None,
            timestamp: None,
            data: Some(data.clone()),
            parent_id: None,
            metadata: None,
        };

        record.insert_bind().execute(&pool).await
            .expect("Failed to insert record");

        let selected = ExtendedTypesTest::where_query_ext("id = ?")
            .bind_proxy("test-binary-1")
            .fetch_one(&pool)
            .await
            .expect("Failed to select record");

        assert!(selected.data.is_some());
        let selected_data = selected.data.unwrap();
        assert_eq!(selected_data, data);

        println!("✅ Binary types test passed");
    }

    // ============================================================================
    // Test 4: UUID types
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_uuid() {
        println!("🔧 Starting test_mysql_extended_types_uuid...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;

        use uuid::Uuid;
        let parent_id = Uuid::new_v4();
        let parent_id_str = parent_id.to_string();

        let mut record = ExtendedTypesTest {
            id: "test-uuid-1".to_string(),
            tiny_int: None,
            small_int: None,
            float_val: None,
            tiny_uint: None,
            small_uint: None,
            medium_uint: None,
            big_uint: None,
            birth_date: None,
            wake_time: None,
            created_at: None,
            timestamp: None,
            data: None,
            parent_id: Some(parent_id_str.clone()),
            metadata: None,
        };

        record.insert_bind().execute(&pool).await
            .expect("Failed to insert record");

        // Query using bind_proxy with UUID
        let selected = ExtendedTypesTest::where_query_ext("id = ?")
            .bind_proxy("test-uuid-1")
            .fetch_one(&pool)
            .await
            .expect("Failed to select record");

        assert!(selected.parent_id.is_some());
        assert_eq!(selected.parent_id.unwrap(), parent_id_str);

        println!("✅ UUID types test passed");
    }

    // ============================================================================
    // Test 5: JSON types
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_json() {
        println!("🔧 Starting test_mysql_extended_types_json...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;

        use serde_json::json;
        let metadata = json!({
            "name": "test",
            "value": 42,
            "tags": ["a", "b", "c"]
        });
        let metadata_str = metadata.to_string();

        let mut record = ExtendedTypesTest {
            id: "test-json-1".to_string(),
            tiny_int: None,
            small_int: None,
            float_val: None,
            tiny_uint: None,
            small_uint: None,
            medium_uint: None,
            big_uint: None,
            birth_date: None,
            wake_time: None,
            created_at: None,
            timestamp: None,
            data: None,
            parent_id: None,
            metadata: Some(metadata_str),
        };

        record.insert_bind().execute(&pool).await
            .expect("Failed to insert record");

        let selected = ExtendedTypesTest::where_query_ext("id = ?")
            .bind_proxy("test-json-1")
            .fetch_one(&pool)
            .await
            .expect("Failed to select record");

        assert!(selected.metadata.is_some());
        let selected_metadata = selected.metadata.unwrap();
        assert!(selected_metadata.contains("test"));
        assert!(selected_metadata.contains("42"));

        println!("✅ JSON types test passed");
    }

    // ============================================================================
    // Test 6: Complex WHERE query with multiple bind_proxy calls
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_complex_where() {
        println!("🔧 Starting test_mysql_extended_types_complex_where...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;

        // Insert multiple records
        for i in 0..5 {
            let mut record = ExtendedTypesTest {
                id: format!("test-complex-{}", i),
                tiny_int: Some(i as i16),
                small_int: Some(1000 + i as i16),
                float_val: Some(1.0 + i as f32),
                tiny_uint: Some(format!("{}", i * 10)),
                small_uint: None,
                medium_uint: None,
                big_uint: None,
                birth_date: None,
                wake_time: None,
                created_at: None,
                timestamp: None,
                data: None,
                parent_id: None,
                metadata: None,
            };
            record.insert_bind().execute(&pool).await
                .expect("Failed to insert record");
        }

        // Query using bind_proxy with multiple types
        let results = ExtendedTypesTest::where_query_ext("tiny_int >= ? AND small_int > ?")
            .bind_proxy(3i16)
            .bind_proxy(1002i16)
            .fetch_all(&pool)
            .await
            .expect("Failed to query records");

        assert_eq!(results.len(), 2); // Records with tiny_int >= 3

        // Test with f32
        let results = ExtendedTypesTest::where_query_ext("float_val BETWEEN ? AND ?")
            .bind_proxy(2.0f32)
            .bind_proxy(4.0f32)
            .fetch_all(&pool)
            .await
            .expect("Failed to query records");

        assert!(results.len() >= 2);

        println!("✅ Complex WHERE query test passed");
    }

    // ============================================================================
    // Test 7: Test unsigned integers in WHERE clause
    // ============================================================================

    #[tokio::test]
    #[serial]
    async fn test_mysql_extended_types_unsigned_where() {
        println!("🔧 Starting test_mysql_extended_types_unsigned_where...");
        let pool = get_test_pool().await;
        create_extended_types_table(&pool).await;

        let mut record = ExtendedTypesTest {
            id: "test-unsigned-1".to_string(),
            tiny_int: None,
            small_int: None,
            float_val: None,
            tiny_uint: Some("255".to_string()),
            small_uint: Some("65535".to_string()),
            medium_uint: Some("4294967295".to_string()),
            big_uint: Some("18446744073709551615".to_string()),
            birth_date: None,
            wake_time: None,
            created_at: None,
            timestamp: None,
            data: None,
            parent_id: None,
            metadata: None,
        };

        record.insert_bind().execute(&pool).await
            .expect("Failed to insert record");

        // Query using bind_proxy with unsigned integers (converts to String)
        let selected = ExtendedTypesTest::where_query_ext("tiny_uint = ?")
            .bind_proxy(255u8)
            .fetch_optional(&pool)
            .await
            .expect("Failed to query record");

        assert!(selected.is_some());

        // Test with u16
        let selected = ExtendedTypesTest::where_query_ext("small_uint = ?")
            .bind_proxy(65535u16)
            .fetch_optional(&pool)
            .await
            .expect("Failed to query record");

        assert!(selected.is_some());

        // Test with u32
        let selected = ExtendedTypesTest::where_query_ext("medium_uint = ?")
            .bind_proxy(4294967295u32)
            .fetch_optional(&pool)
            .await
            .expect("Failed to query record");

        assert!(selected.is_some());

        println!("✅ Unsigned integers WHERE clause test passed");
    }
}
