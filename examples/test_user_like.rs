// Test with multiple field types like User
use sqlx_struct_enhanced::EnhancedCrud;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, FromRow, Decode, Type, query::Query, query::QueryAs, database::HasArguments};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, EnhancedCrud)]
#[table_name = "test_users"]
pub struct TestUser {
    pub id: Uuid,
    pub username: String,
    /* TEMPORARILY REMOVED
    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    pub commission_rate: Option<String>,
    */
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum TestRole {
    Owner,
    Admin,
}

fn main() {
    println!("Test");
}
