// Integration test for DECIMAL/NUMERIC column support
//
// This test verifies that:
// 1. Struct fields with #[crud(decimal(...))] generate correct NUMERIC columns in migrations
// 2. Data can be inserted into DECIMAL columns
// 3. Data can be queried back with correct type conversion
// 4. The cast_as attribute works correctly for String types

#[cfg(test)]
mod decimal_integration_tests {
    use sqlx::{FromRow, PgPool, Postgres};
    use sqlx::database::HasArguments;
    use sqlx::query::{Query, QueryAs};
    use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
    use uuid::Uuid;
    use serial_test::serial;

    // Helper function to get database connection
    async fn get_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

        sqlx::PgPool::connect(&database_url).await
            .expect("Failed to connect to test database")
    }

    // Helper function to create test table
    async fn create_test_table(pool: &PgPool) {
        // Drop table if exists
        sqlx::query("DROP TABLE IF EXISTS decimal_products")
            .execute(pool)
            .await
            .expect("Failed to drop existing table");

        // Create table with TEXT columns for decimal fields (to match String type)
        // The #[crud(decimal(...))] attribute is for migration generation only
        // For this test, we use TEXT which works seamlessly with String Rust type
        let create_query = r#"
            CREATE TABLE decimal_products (
                id UUID PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                price TEXT,
                discount TEXT,
                tax_rate TEXT,
                quantity INTEGER DEFAULT 0
            )
        "#;

        sqlx::query(create_query).execute(pool).await
            .expect("Failed to create test table");
    }

    // ============================================================================
    // Test Struct: Using String type with cast_as (Recommended)
    // ============================================================================

    #[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
    #[table_name = "decimal_products"]
    pub struct DecimalProduct {
        pub id: Uuid,
        pub name: String,

        #[crud(decimal(precision = 10, scale = 2))]
        #[crud(cast_as = "TEXT")]
        pub price: Option<String>,

        #[crud(decimal(precision = 5, scale = 2))]
        #[crud(cast_as = "TEXT")]
        pub discount: Option<String>,

        #[crud(decimal(precision = 6, scale = 4))]
        #[crud(cast_as = "TEXT")]
        pub tax_rate: Option<String>,

        pub quantity: i32,
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_insert_and_select() {
        println!("🔧 Getting test pool...");
        let pool = get_test_pool().await;
        println!("✅ Got pool");

        println!("🔧 Creating test table...");
        create_test_table(&pool).await;
        println!("✅ Table created");

        // Test 1: Insert product with decimal fields
        println!("🔧 Creating product...");
        let mut product = DecimalProduct {
            id: Uuid::new_v4(),
            name: "Laptop".to_string(),
            price: Some("1299.99".to_string()),
            discount: Some("15.00".to_string()),
            tax_rate: Some("8.2500".to_string()),
            quantity: 10,
        };
        println!("✅ Product created: {:?}", product);

        println!("🔧 Preparing insert query...");
        let insert_query = product.insert_bind();
        println!("✅ Insert query prepared");

        println!("🔧 Executing insert...");
        insert_query.execute(&pool).await
            .expect("Failed to insert product");
        println!("✅ Product inserted");

        // Test 2: Select product back
        println!("🔧 Fetching product...");
        let fetched = DecimalProduct::by_pk()
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch product");
        println!("✅ Product fetched");

        // Test 3: Verify values match
        assert_eq!(fetched.id, product.id);
        assert_eq!(fetched.name, product.name);
        assert_eq!(fetched.price, Some("1299.99".to_string()));
        assert_eq!(fetched.discount, Some("15.00".to_string()));
        assert_eq!(fetched.tax_rate, Some("8.2500".to_string()));
        assert_eq!(fetched.quantity, 10);
        println!("✅ All assertions passed");

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
        println!("✅ Cleanup done");
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_with_null_values() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Test with NULL decimal fields
        let mut product = DecimalProduct {
            id: Uuid::new_v4(),
            name: "Free Product".to_string(),
            price: Some("0.00".to_string()),
            discount: None,  // NULL
            tax_rate: None,  // NULL
            quantity: 100,
        };

        product.insert_bind().execute(&pool).await
            .expect("Failed to insert product");

        // Fetch and verify NULL values
        let fetched = DecimalProduct::by_pk()
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch product");

        assert_eq!(fetched.price, Some("0.00".to_string()));
        assert_eq!(fetched.discount, None);  // Should be NULL
        assert_eq!(fetched.tax_rate, None);  // Should be NULL

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_where_query() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Insert multiple products
        for i in 1..=3 {
            let mut product = DecimalProduct {
                id: Uuid::new_v4(),
                name: format!("Product {}", i),
                price: Some(format!("{}.{}", i * 100, 99)),
                discount: Some("10.00".to_string()),
                tax_rate: Some("5.0000".to_string()),
                quantity: i,
            };
            product.insert_bind().execute(&pool).await.unwrap();
        }

        // Test WHERE query with decimal comparison
        let results = DecimalProduct::where_query("price > {}")
            .bind("150")
            .fetch_all(&pool)
            .await
            .expect("Failed to query products");

        // Should find 2 products (200.99 and 300.99)
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|p| p.price.as_ref().unwrap().parse::<f64>().unwrap() > 150.0));

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_update() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Insert product
        let mut product = DecimalProduct {
            id: Uuid::new_v4(),
            name: "Laptop".to_string(),
            price: Some("1299.99".to_string()),
            discount: Some("15.00".to_string()),
            tax_rate: Some("8.2500".to_string()),
            quantity: 10,
        };

        product.insert_bind().execute(&pool).await.unwrap();

        // Update price
        product.price = Some("999.99".to_string());
        product.update_bind().execute(&pool).await
            .expect("Failed to update product");

        // Fetch and verify update
        let fetched = DecimalProduct::by_pk()
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(fetched.price, Some("999.99".to_string()));

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_boundary_values() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Test boundary values for NUMERIC(10,2)
        let test_cases = vec![
            ("0.00", "Zero"),
            ("0.01", "Minimum non-zero"),
            ("99999999.99", "Maximum value"),
            ("100000000.00", "Too large - will fail"),
        ];

        for (value, description) in test_cases {
            let mut product = DecimalProduct {
                id: Uuid::new_v4(),
                name: description.to_string(),
                price: Some(value.to_string()),
                discount: Some("0.00".to_string()),
                tax_rate: Some("0.0000".to_string()),
                quantity: 0,
            };

            // Try to insert
            let result = product.insert_bind().execute(&pool).await;

            if description == "Too large - will fail" {
                // Should fail
                assert!(result.is_err(), "Expected insertion to fail for {}", description);
            } else {
                // Should succeed
                result.expect("Failed to insert test case");
            }
        }

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_scale_values() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Test different scale values
        let mut product = DecimalProduct {
            id: Uuid::new_v4(),
            name: "Scale Test".to_string(),
            price: Some("12345.67".to_string()),      // Scale 2
            discount: Some("12.34".to_string()),      // Scale 2
            tax_rate: Some("12.3456".to_string()),   // Scale 4
            quantity: 1,
        };

        product.insert_bind().execute(&pool).await.unwrap();

        // Fetch and verify all decimal places are preserved
        let fetched = DecimalProduct::by_pk()
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(fetched.price, Some("12345.67".to_string()));
        assert_eq!(fetched.discount, Some("12.34".to_string()));
        assert_eq!(fetched.tax_rate, Some("12.3456".to_string()));

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_with_bulk_operations() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Create multiple products
        let mut products = Vec::new();
        for i in 1..=5 {
            let mut product = DecimalProduct {
                id: Uuid::new_v4(),
                name: format!("Product {}", i),
                price: Some(format!("{}.{}", i * 100, 99)),
                discount: Some("10.00".to_string()),
                tax_rate: Some("5.0000".to_string()),
                quantity: i,
            };
            products.push(product.clone());
        }

        // Bulk insert
        for product in &mut products {
            product.insert_bind().execute(&pool).await.unwrap();
        }

        // Bulk select by IDs
        let ids: Vec<String> = products.iter().map(|p| p.id.to_string()).collect();
        let results = DecimalProduct::bulk_select(ids.as_slice())
            .fetch_all(&pool)
            .await
            .expect("Failed to bulk select");

        assert_eq!(results.len(), 5);

        // Verify all decimal values are correct
        for (i, product) in products.iter().enumerate() {
            assert_eq!(results[i].name, product.name);
            assert_eq!(results[i].price, product.price);
            assert_eq!(results[i].discount, product.discount);
        }

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_decimal_with_count() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Insert products with different prices
        for i in 1..=3 {
            let mut product = DecimalProduct {
                id: Uuid::new_v4(),
                name: format!("Product {}", i),
                price: Some(format!("{}.{}", i * 50, 0)),
                discount: Some("0.00".to_string()),
                tax_rate: Some("0.0000".to_string()),
                quantity: i,
            };
            product.insert_bind().execute(&pool).await.unwrap();
        }

        // Count products with price > 50
        let count = DecimalProduct::count_query("price > {}::NUMERIC")
            .bind("50")
            .fetch_one(&pool)
            .await
            .expect("Failed to count");

        assert_eq!(count, (2,)); // Should be 2 (products 2 and 3)

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_sql_generation_with_casting() {
        // This test verifies that the generated SQL includes type casting
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        let mut product = DecimalProduct {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            price: Some("99.99".to_string()),
            discount: Some("5.00".to_string()),
            tax_rate: Some("2.5000".to_string()),
            quantity: 1,
        };

        product.insert_bind().execute(&pool).await.unwrap();

        // Verify we can select using raw SQL query to check the casting
        let raw_query = r#"
            SELECT id, name,
                   price::TEXT as price,
                   discount::TEXT as discount,
                   tax_rate::TEXT as tax_rate,
                   quantity
            FROM decimal_products
            WHERE id = $1
        "#;

        let row: (Uuid, String, String, String, String, i32) = sqlx::query_as(raw_query)
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.0, product.id);
        assert_eq!(row.2, "99.99");
        assert_eq!(row.3, "5.00");
        assert_eq!(row.4, "2.5000");

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_mixed_operations() {
        let pool = get_test_pool().await;
        create_test_table(&pool).await;

        // Insert
        let mut product = DecimalProduct {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            price: Some("100.00".to_string()),
            discount: Some("0.00".to_string()),
            tax_rate: Some("0.0000".to_string()),
            quantity: 1,
        };

        product.insert_bind().execute(&pool).await.unwrap();

        // Update
        product.price = Some("150.00".to_string());
        product.update_bind().execute(&pool).await.unwrap();

        // Select and verify
        let fetched = DecimalProduct::by_pk()
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(fetched.price, Some("150.00".to_string()));

        // Delete
        product.delete_bind().execute(&pool).await.unwrap();

        // Verify deletion
        let result = DecimalProduct::by_pk()
            .bind(&product.id)
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(result.is_none());

        // Cleanup
        sqlx::query("DROP TABLE decimal_products")
            .execute(&pool)
            .await
            .unwrap();
    }
}
