// Simple database connection test
#[cfg(test)]
mod db_connection_tests {
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_database_connection() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

        println!("Connecting to: {}", database_url);

        let pool = PgPool::connect(&database_url).await;

        match pool {
            Ok(pool) => {
                println!("✅ Successfully connected to database!");

                // Test a simple query
                let result: (i32,) = sqlx::query_as("SELECT 1")
                    .fetch_one(&pool)
                    .await
                    .unwrap();

                println!("✅ Query executed successfully! Result: {}", result.0);
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to database: {}", e);
                panic!("Database connection failed");
            }
        }
    }

    #[tokio::test]
    async fn test_create_simple_table() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

        let pool = PgPool::connect(&database_url).await
            .expect("Failed to connect to database");

        // Clean up
        sqlx::query("DROP TABLE IF EXISTS test_table")
            .execute(&pool)
            .await
            .ok();

        // Create table
        sqlx::query("CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("Failed to create table");

        println!("✅ Table created successfully");

        // Insert data
        sqlx::query("INSERT INTO test_table (name) VALUES ($1)")
            .bind("test")
            .execute(&pool)
            .await
            .expect("Failed to insert");

        println!("✅ Data inserted successfully");

        // Query data
        let result: (String,) = sqlx::query_as("SELECT name FROM test_table WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to query");

        assert_eq!(result.0, "test");
        println!("✅ Data queried successfully: {}", result.0);

        // Clean up
        sqlx::query("DROP TABLE test_table")
            .execute(&pool)
            .await
            .ok();

        println!("✅ All database operations successful!");
    }
}
