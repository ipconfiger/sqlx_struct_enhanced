// Test decimal helper generation
use sqlx_struct_enhanced::EnhancedCrud;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Decode, Type, query::Query, query::QueryAs, database::HasArguments};

#[derive(Debug, Clone, Serialize, Deserialize, EnhancedCrud)]
#[table_name = "test_decimal"]
struct TestDecimal {
    id: String,
    #[crud(decimal(precision = 10, scale = 2))]
    amount: Option<String>,
}

fn main() {
    println!("Test decimal helpers");
}
