// Real-world scenario: E-commerce order management system
//
// This example demonstrates practical usage of extended BindProxy types in a
// realistic e-commerce application handling orders, customers, and products.

#[cfg(all(feature = "postgres", feature = "all-types"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sqlx::{FromRow, PgPool};
    use sqlx_struct_enhanced::{EnhancedCrud, EnhancedCrudExt};
    use chrono::{NaiveDate, NaiveDateTime, Utc, TimeZone};
    use serde_json::json;
    use uuid::Uuid;

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:@127.0.0.1/test-sqlx-tokio".to_string());

    let pool = PgPool::connect(&database_url).await?;

    println!("🏪 E-commerce Order Management System Demo");
    println!("========================================\n");

    // ============================================================================
    // Setup: Create tables
    // ============================================================================

    sqlx::query("DROP TABLE IF EXISTS orders CASCADE").execute(&pool).await?;
    sqlx::query("DROP TABLE IF EXISTS customers CASCADE").execute(&pool).await?;
    sqlx::query("DROP TABLE IF EXISTS products CASCADE").execute(&pool).await?;

    // Customers table
    sqlx::query(r#"
        CREATE TABLE customers (
            id UUID PRIMARY KEY,
            email VARCHAR(255) NOT NULL UNIQUE,
            name VARCHAR(100) NOT NULL,
            birth_date TEXT,
            registration_date TEXT,
            metadata TEXT,
            total_orders INTEGER DEFAULT 0
        )
    "#).execute(&pool).await?;

    // Products table
    sqlx::query(r#"
        CREATE TABLE products (
            id VARCHAR(36) PRIMARY KEY,
            sku VARCHAR(50) UNIQUE NOT NULL,
            name VARCHAR(200) NOT NULL,
            price TEXT NOT NULL,
            stock_count SMALLINT,
            rating REAL,
            weight_kg REAL,
            release_date TEXT,
            tags TEXT,
            created_at TEXT
        )
    "#).execute(&pool).await?;

    // Orders table
    sqlx::query(r#"
        CREATE TABLE orders (
            id UUID PRIMARY KEY,
            customer_id UUID NOT NULL,
            order_date TEXT NOT NULL,
            total_amount TEXT NOT NULL,
            status VARCHAR(20) NOT NULL,
            payment_method TEXT,
            shipping_address TEXT,
            metadata TEXT,
            CONSTRAINT fk_customer FOREIGN KEY (customer_id) REFERENCES customers(id)
        )
    "#).execute(&pool).await?;

    // ============================================================================
    // Scenario 1: Customer Registration with Date of Birth
    // ============================================================================

    println!("👤 Scenario 1: Customer Registration");
    println!("--------------------------------------");

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "customers"]
    struct Customer {
        pub id: String,
        pub email: String,
        pub name: String,
        pub birth_date: Option<String>,
        pub registration_date: Option<String>,
        pub metadata: Option<String>,
        pub total_orders: i32,
    }

    let customer_id = Uuid::new_v4();
    let birth_date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
    let registration_date = Utc.now();

    let customer_metadata = json!({
        "tier": "gold",
        "preferences": {
            "newsletter": true,
            "sms_notifications": false
        },
        "tags": ["vip", "early-adopter"]
    });

    let mut customer = Customer {
        id: customer_id.to_string(),
        email: "john.doe@example.com".to_string(),
        name: "John Doe".to_string(),
        birth_date: Some(birth_date.format("%Y-%m-%d").to_string()),
        registration_date: Some(registration_date.format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string()),
        metadata: Some(customer_metadata.to_string()),
        total_orders: 0,
    };

    customer.insert_bind(&pool).await?;
    println!("✅ Customer registered: {} (ID: {})", customer.name, customer.id);
    println!("   Date of Birth: {}", customer.birth_date.unwrap());
    println!("   Registration Date: {}", customer.registration_date.unwrap());
    println!("   Metadata: {}", customer.metadata.unwrap());

    // ============================================================================
    // Scenario 2: Product Catalog with Multiple Type Conversions
    // ============================================================================

    println!("\n📦 Scenario 2: Product Catalog Management");
    println!("-----------------------------------------");

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "products"]
    struct Product {
        pub id: String,
        pub sku: String,
        pub name: String,
        pub price: String,
        pub stock_count: Option<i16>,
        pub rating: Option<f32>,
        pub weight_kg: Option<f32>,
        pub release_date: Option<String>,
        pub tags: Option<String>,
        pub created_at: Option<String>,
    }

    // Add multiple products
    let products = vec![
        (
            "LAPTOP-PRO-15",
            "ProBook 15\" Laptop",
            "1299.99",
            50i16,
            4.7f32,
            2.3f32,
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        ),
        (
            "PHONE-GALAXY",
            "Galaxy Smartphone",
            "799.99",
            100i16,
            4.5f32,
            0.2f32,
            NaiveDate::from_ymd_opt(2023, 11, 20).unwrap(),
        ),
        (
            "TABLET-AIR",
            "AirPad Tablet",
            "499.99",
            75i16,
            4.8f32,
            0.5f32,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        ),
    ];

    for (sku, name, price, stock, rating, weight, release_date) in products {
        let product_id = Uuid::new_v4();
        let created_at = Utc.now();

        let tags = json!(["electronics", "new-arrival", "bestseller"]);

        let mut product = Product {
            id: product_id.to_string(),
            sku: sku.to_string(),
            name: name.to_string(),
            price: price.to_string(),
            stock_count: Some(stock),
            rating: Some(rating),
            weight_kg: Some(weight),
            release_date: Some(release_date.format("%Y-%m-%d").to_string()),
            tags: Some(tags.to_string()),
            created_at: Some(created_at.format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string()),
        };

        product.insert_bind(&pool).await?;
        println!("✅ Product added: {} (SKU: {}, Stock: {}, Rating: {})",
            product.name, product.sku, product.stock_count.unwrap(), product.rating.unwrap());
    }

    // ============================================================================
    // Scenario 3: Order Placement with Type Conversions
    // ============================================================================

    println!("\n🛒 Scenario 3: Order Placement");
    println!("-----------------------------");

    #[derive(FromRow, EnhancedCrud)]
    #[table_name = "orders"]
    struct Order {
        pub id: String,
        pub customer_id: String,
        pub order_date: String,
        pub total_amount: String,
        pub status: String,
        pub payment_method: Option<String>,
        pub shipping_address: Option<String>,
        pub metadata: Option<String>,
    }

    let order_id = Uuid::new_v4();
    let order_date = NaiveDateTime::from_timestamp_opt(1704067200, 0).unwrap();

    let shipping_address = json!({
        "street": "123 Main St",
        "city": "San Francisco",
        "state": "CA",
        "zip": "94102",
        "country": "USA"
    });

    let order_metadata = json!({
        "source": "web",
        "campaign": "summer_sale_2024",
        "items": [
            {"product_id": "laptop-pro-15", "quantity": 1},
            {"product_id": "phone-galaxy", "quantity": 2}
        ]
    });

    let mut order = Order {
        id: order_id.to_string(),
        customer_id: customer.id.clone(),
        order_date: order_date.format("%Y-%m-%d %H:%M:%S%.9f").to_string(),
        total_amount: "2899.97".to_string(),
        status: "pending".to_string(),
        payment_method: Some("credit_card".to_string()),
        shipping_address: Some(shipping_address.to_string()),
        metadata: Some(order_metadata.to_string()),
    };

    order.insert_bind(&pool).await?;
    println!("✅ Order placed: ID {}", order.id);
    println!("   Customer ID: {}", order.customer_id);
    println!("   Order Date: {}", order.order_date);
    println!("   Total Amount: ${}", order.total_amount);
    println!("   Shipping Address: {}", order.shipping_address.unwrap());

    // ============================================================================
    // Scenario 4: Advanced Queries with Multiple Type Bindings
    // ============================================================================

    println!("\n🔍 Scenario 4: Advanced Product Queries");
    println!("---------------------------------------");

    // Query 1: Find products released after a certain date with minimum rating
    let search_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let min_rating = 4.5f32;

    let products = Product::where_query("release_date >= {} AND rating >= {}")
        .bind_proxy(search_date)       // NaiveDate → String
        .bind_proxy(min_rating)        // f32 direct binding
        .fetch_all(&pool)
        .await?;

    println!("✅ Query 1: Products released after 2024-01-01 with rating >= 4.5");
    for product in products {
        println!("   - {} (Rating: {}, Release: {})",
            product.name, product.rating.unwrap(), product.release_date.unwrap());
    }

    // Query 2: Find products in stock within price range
    let min_price = "500.00";
    let max_price = "1500.00";
    let min_stock = 20i16;

    let products = Product::where_query("price BETWEEN {} AND {} AND stock_count >= {}")
        .bind_proxy(min_price)        // String
        .bind_proxy(max_price)        // String
        .bind_proxy(min_stock)        // i16 direct binding
        .fetch_all(&pool)
        .await?;

    println!("\n✅ Query 2: Products in stock ($500-$1500, stock >= 20)");
    for product in products {
        println!("   - {} (Price: ${}, Stock: {})",
            product.name, product.price, product.stock_count.unwrap());
    }

    // Query 3: Find products by weight range using f32
    let min_weight = 0.3f32;
    let max_weight = 3.0f32;

    let products = Product::where_query("weight_kg BETWEEN {} AND {}")
        .bind_proxy(min_weight)       // f32 direct binding
        .bind_proxy(max_weight)       // f32 direct binding
        .fetch_all(&pool)
        .await?;

    println!("\n✅ Query 3: Products weighing between 0.3kg and 3.0kg");
    for product in products {
        println!("   - {} (Weight: {}kg)",
            product.name, product.weight_kg.unwrap());
    }

    // ============================================================================
    // Scenario 5: Customer Analytics with Date Range Queries
    // ============================================================================

    println!("\n📊 Scenario 5: Customer Analytics");
    println!("---------------------------------");

    // Update customer's total orders count
    sqlx::query("UPDATE customers SET total_orders = total_orders + 1 WHERE id = {}")
        .bind(customer.id)
        .execute(&pool)
        .await?;

    // Find customers registered in a specific date range
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let customers = Customer::where_query("registration_date BETWEEN {} AND {}")
        .bind_proxy(start_date)       // NaiveDate → String
        .bind_proxy(end_date)         // NaiveDate → String
        .fetch_all(&pool)
        .await?;

    println!("✅ Customers registered in 2024: {}", customers.len());
    for customer in customers {
        println!("   - {} (Email: {}, Orders: {})",
            customer.name, customer.email, customer.total_orders);
    }

    // ============================================================================
    // Scenario 6: Complex Order Search with JSON Metadata
    // ============================================================================

    println!("\n📋 Scenario 6: Complex Order Search");
    println!("----------------------------------");

    // Find orders by customer within a date range
    let search_date_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let search_date_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let orders = Order::where_query("customer_id = {} AND order_date BETWEEN {} AND {}")
        .bind_proxy(customer.id)     // Uuid → String
        .bind_proxy(search_date_start) // NaiveDate → String
        .bind_proxy(search_date_end)  // NaiveDate → String
        .fetch_all(&pool)
        .await?;

    println!("✅ Found {} orders for customer in 2024", orders.len());
    for order in orders {
        println!("   - Order {} (Date: {}, Status: {}, Total: ${})",
            order.id, order.order_date, order.status, order.total_amount);
    }

    // ============================================================================
    // Scenario 7: Inventory Management with Unsigned Integers
    // ============================================================================

    println!("\n📦 Scenario 7: Inventory Management");
    println!("---------------------------------");

    // Query products with low stock threshold using unsigned integer
    let low_stock_threshold = 30u8;  // u8 → String

    let products = Product::where_query("stock_count < {}")
        .bind_proxy(low_stock_threshold)  // u8 → String
        .fetch_all(&pool)
        .await?;

    println!("✅ Products with low stock (< 30 units):");
    for product in products {
        println!("   - {} (SKU: {}, Stock: {})",
            product.name, product.sku, product.stock_count.unwrap());
    }

    // ============================================================================
    // Cleanup
    // ============================================================================

    println!("\n🧹 Cleaning up test data...");
    sqlx::query("DROP TABLE orders CASCADE").execute(&pool).await?;
    sqlx::query("DROP TABLE customers CASCADE").execute(&pool).await?;
    sqlx::query("DROP TABLE products CASCADE").execute(&pool).await?;
    println!("✅ Cleanup complete");

    println!("\n✅ All real-world scenarios completed successfully!");
    println!("\n📌 Key Takeaways:");
    println!("   • Chrono types (NaiveDate, NaiveDateTime) → ISO 8601 strings");
    println!("   • UUID → String for database compatibility");
    println!("   • JSON → JSON string for metadata storage");
    println!("   • f32 → Direct binding for floating-point numbers");
    println!("   • Unsigned integers → String conversion (SQLx limitation)");
    println!("   • All conversions are automatic and type-safe");

    Ok(())
}

// ============================================================================
// Feature-Gated Main for Other Databases
// ============================================================================

#[cfg(all(feature = "mysql", feature = "all-types", not(feature = "postgres")))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏪 MySQL E-commerce Example");
    println!("Run with: cargo run --example extended_types_real_world --features 'mysql,all-types'");
    Ok(())
}

#[cfg(all(feature = "sqlite", feature = "all-types", not(feature = "postgres"), not(feature = "mysql")))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏪 SQLite E-commerce Example");
    println!("Run with: cargo run --example extended_types_real_world --features 'sqlite,all-types'");
    Ok(())
}

#[cfg(not(feature = "all-types"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("This example requires the 'all-types' feature");
    println!("Run with: cargo run --example extended_types_real_world --features 'postgres,all-types'");
    Ok(())
}
