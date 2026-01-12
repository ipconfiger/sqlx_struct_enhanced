use sqlx_struct_enhanced::{EnhancedCrud, join::JoinTuple2};
use sqlx::{FromRow, Postgres, Row};
use sqlx::query::Query;
use sqlx::query::QueryAs;
use sqlx::database::HasArguments;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "orders"]
struct Order {
    pub id: String,
    pub customer_id: String,
}

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "customers"]
struct Customer {
    pub id: String,
    pub name: String,
}

fn main() {
    use sqlx_struct_enhanced::join::{JoinSqlGenerator, JoinType, SchemeAccessor};

    let generator = JoinSqlGenerator::new::<Order, Customer>(
        JoinType::Inner,
        "orders.customer_id = customers.id"
    );

    let sql = generator.gen_full_query(None);
    println!("Generated SQL:");
    println!("{}", sql);
}
