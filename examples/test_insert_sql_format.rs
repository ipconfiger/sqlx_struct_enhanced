// Test to verify INSERT SQL format with explicit column list
use sqlx_struct_enhanced::EnhancedCrud;
use sqlx::{Row, FromRow, Postgres, query::{Query, QueryAs}};
use sqlx::database::HasArguments;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "users"]
struct User {
    id: String,
    username: String,
    email: String,
    created_at: String,
}

fn main() {
    println!("Check compilation output for generated INSERT SQL");
    // Should see: INSERT INTO "users" ("id", "username", "email", "created_at") VALUES ($1,$2,$3,$4)
}
