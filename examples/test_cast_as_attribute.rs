// Test example to verify #[crud(cast_as = "TEXT")] attribute parsing
// This file demonstrates that Phase 1 implementation works correctly

use sqlx::FromRow;
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};

// Example 1: Basic usage with DECIMAL fields
#[derive(Debug, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "users"]
pub struct User {
    pub id: String,
    pub username: String,
    #[crud(cast_as = "TEXT")]
    pub commission_rate: Option<String>,
    #[crud(cast_as = "TEXT")]
    pub account_balance: Option<String>,
}

// Example 2: Mixed casted and non-casted fields
#[derive(Debug, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "orders"]
pub struct Order {
    pub id: String,
    pub customer_name: String,  // No casting needed
    #[crud(cast_as = "TEXT")]
    pub estimated_price: Option<String>,
    #[crud(cast_as = "TEXT")]
    pub final_price: Option<String>,
    pub created_at: String,  // No casting needed
}

// Example 3: Multiple different cast types (future enhancement)
#[derive(Debug, PartialEq, FromRow, EnhancedCrud)]
#[table_name = "products"]
pub struct Product {
    pub id: String,
    pub name: String,
    #[crud(cast_as = "TEXT")]
    pub price: Option<String>,
    // Future: #[crud(cast_as = "JSONB")]
    // pub metadata: Option<String>,
}

fn main() {
    println!("Phase 1 implementation successful!");
    println!("The #[crud(cast_as = \"TEXT\")] attribute is now parsed correctly.");
    println!("\nNext steps:");
    println!("1. Phase 2: Modify Schema to pass column metadata");
    println!("2. Phase 3: Modify Scheme to generate explicit column lists");
    println!("3. Phase 4: Add integration tests");
}
