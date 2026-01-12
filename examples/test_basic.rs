// Test with only basic types
use sqlx_struct_enhanced::EnhancedCrud;
use sqlx::{Postgres, Row, FromRow, query::Query, query::QueryAs, database::HasArguments};

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "test_basic"]
struct TestBasic {
    id: String,
    name: String,
    count: i32,
}

fn main() {
    println!("Test basic types");
}
