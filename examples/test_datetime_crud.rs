// Minimal test case for DateTime with EnhancedCrud
// Compile with: cargo build --example test_datetime_crud --features postgres,chrono

use sqlx_struct_enhanced::EnhancedCrud;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Row, Decode, Type, query::Query, query::QueryAs, database::HasArguments};

#[derive(Debug, Clone, EnhancedCrud)]
#[table_name = "test_table"]
struct TestDateTime {
    id: String,
    created_at: DateTime<Utc>,
}

fn main() {
    println!("If this compiles, DateTime works with EnhancedCrud!");
}
