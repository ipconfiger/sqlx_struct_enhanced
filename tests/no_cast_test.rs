// Test without cast_as to isolate the issue
#[cfg(test)]
mod no_cast_tests {
    use sqlx::{FromRow, PgPool};
    use sqlx::query::{Query, QueryAs};
    use sqlx::database::HasArguments;
    use sqlx::postgres::Postgres;
    use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
    use uuid::Uuid;
    use serial_test::serial;

    #[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
    #[table_name = "no_cast_products"]
    pub struct NoCastProduct {
        pub id: Uuid,
        pub name: String,
        pub price: Option<String>,
        pub quantity: i32,
    }

    #[tokio::test]
    #[serial]
    async fn test_without_cast_as() {
        println!("🔧 Starting test without cast_as...");

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

        let pool = PgPool::connect(&database_url).await
            .expect("Failed to connect");
        println!("✅ Connected");

        // Clean up
        sqlx::query("DROP TABLE IF EXISTS no_cast_products")
            .execute(&pool)
            .await
            .ok();

        // Create table
        sqlx::query("CREATE TABLE no_cast_products (id UUID PRIMARY KEY, name TEXT NOT NULL, price TEXT, quantity INTEGER)")
            .execute(&pool)
            .await
            .expect("Failed to create table");
        println!("✅ Table created");

        let mut product = NoCastProduct {
            id: Uuid::new_v4(),
            name: "Test".to_string(),
            price: Some("99.99".to_string()),
            quantity: 1,
        };

        println!("🔧 Inserting...");
        product.insert_bind().execute(&pool).await
            .expect("Insert failed");
        println!("✅ Inserted");

        println!("🔧 Fetching...");
        let result = NoCastProduct::by_pk()
            .bind(&product.id)
            .fetch_one(&pool)
            .await
            .expect("Fetch failed");
        println!("✅ Fetched: {:?}", result);

        assert_eq!(result.price, Some("99.99".to_string()));
        println!("✅ Test passed!");

        sqlx::query("DROP TABLE no_cast_products")
            .execute(&pool)
            .await
            .ok();
    }
}
