// Minimal DateTime test
use sqlx_struct_enhanced::EnhancedCrud;
use sqlx::{Postgres, Row, FromRow, query::Query, query::QueryAs, database::HasArguments};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "test_dt"]
struct TestDateTime {
    id: String,
    created_at: DateTime<Utc>,
}

fn main() {
    println!("Test");
}
