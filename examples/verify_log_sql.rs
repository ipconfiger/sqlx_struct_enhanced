// Simple test to verify log_sql feature is working
use sqlx::FromRow;
use sqlx::Row;
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct SimpleUser {
    pub id: String,
    pub name: String,
}

fn main() {
    println!("=== Verifying log_sql feature ===\n");

    // Create instance
    let mut user = SimpleUser {
        id: "test-id".to_string(),
        name: "Test User".to_string(),
    };

    println!("1. Calling insert_bind()...");
    let _insert = user.insert_bind();
    println!("   (Check above for [SQLxEnhanced] log)\n");

    println!("2. Calling update_bind()...");
    let _update = user.update_bind();
    println!("   (Check above for [SQLxEnhanced] log)\n");

    println!("3. Calling delete_bind()...");
    let _delete = user.delete_bind();
    println!("   (Check above for [SQLxEnhanced] log)\n");

    println!("4. Calling SimpleUser::by_pk()...");
    let _select = SimpleUser::by_pk();
    println!("   (Check above for [SQLxEnhanced] log)\n");

    println!("=== If you see [SQLxEnhanced] logs above, log_sql is working ===");
    println!("=== If NOT, check: cargo clean && cargo build --features log_sql ===");
}
