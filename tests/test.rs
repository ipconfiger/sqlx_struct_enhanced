use std::boxed::Box;
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;



#[derive(Debug, Clone ,FromRow, EnhancedCrud)]
struct TestTb {
    id: String,
    name: String,
    ts: i32
}


#[tokio::test]
async fn test_something_async() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    let mut bar = TestTb{id:"asdasd".to_string(), name:"asdasd".to_string(), ts:1231312};
    bar.insert_bind().execute(&pool).await?;
    let tb = TestTb::by_pk().bind("asdasd").fetch_one(&pool).await?;
    TestTb::make_query("select * from test_tb").fetch_all(&pool).await?;
    let (count,) = TestTb::count_query("1=1").fetch_one(&pool).await?;
    println!("{:?}", tb);
    println!("{:?}", count);

    Ok(())
}

