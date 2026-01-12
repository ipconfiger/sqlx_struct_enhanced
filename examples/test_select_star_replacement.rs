// Test SELECT * replacement with cast_as
use sqlx::FromRow;
use sqlx::Row;
use sqlx::Execute;
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx::query::{Query, QueryAs};
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct UserWithDecimal {
    #[crud(decimal(precision = 10, scale = 2), cast_as = "TEXT")]
    pub commission_rate: Option<String>,

    pub id: String,
    pub name: String,
}

fn main() {
    println!("=== Testing SELECT * replacement with cast_as ===\n");

    println!("1. Using make_query with SELECT *:");
    let query1 = UserWithDecimal::make_query("SELECT * FROM UserWithDecimal ORDER BY id LIMIT 10");
    println!("   SQL: {}\n", query1.sql());

    println!("2. Using make_query with SELECT * and [Self]:");
    let query2 = UserWithDecimal::make_query("SELECT * FROM [Self] WHERE active = true");
    println!("   SQL: {}\n", query2.sql());

    println!("3. Using make_query without SELECT * (explicit columns):");
    let query3 = UserWithDecimal::make_query("SELECT id, name FROM UserWithDecimal");
    println!("   SQL: {}\n", query3.sql());

    println!("4. Using by_pk (should have cast_as):");
    let query4 = UserWithDecimal::by_pk();
    println!("   SQL: {}\n", query4.sql());

    println!("=== Expected behavior ===");
    println!("- Query 1 & 2: SELECT * should be replaced with explicit column list");
    println!("- commission_rate should have ::TEXT as \"commission_rate\" cast");
    println!("- Query 3: Should keep explicit columns as-is (no replacement)");
}
