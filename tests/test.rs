use std::boxed::Box;
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use serial_test::serial;



#[derive(Debug, Clone ,FromRow, EnhancedCrud)]
struct TestTb {
    id: String,
    name: String,
    ts: i32
}

#[tokio::test]
#[serial]
async fn test_something_async() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Clean up before test
    sqlx::query("DELETE FROM test_tb WHERE id = 'asdasd'").execute(&pool).await.ok();

    let mut bar = TestTb{id:"asdasd".to_string(), name:"asdasd".to_string(), ts:1231312};
    bar.insert_bind().execute(&pool).await?;
    let tb = TestTb::by_pk().bind("asdasd").fetch_one(&pool).await?;
    TestTb::make_query("select * from test_tb").fetch_all(&pool).await?;
    let (count,) = TestTb::count_query("1=1").fetch_one(&pool).await?;
    println!("{:?}", tb);
    println!("{:?}", count);

    // Clean up after test
    sqlx::query("DELETE FROM test_tb WHERE id = 'asdasd'").execute(&pool).await.ok();

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_bulk_operations() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Clean up before test
    sqlx::query("DELETE FROM test_tb WHERE id LIKE 'bulk-%'").execute(&pool).await.ok();

    // Test bulk insert
    let items = vec![
        TestTb { id: "bulk-1".to_string(), name: "Alice".to_string(), ts: 100 },
        TestTb { id: "bulk-2".to_string(), name: "Bob".to_string(), ts: 200 },
        TestTb { id: "bulk-3".to_string(), name: "Charlie".to_string(), ts: 300 },
    ];
    TestTb::bulk_insert(&items).execute(&pool).await?;

    // Test bulk select
    let ids = vec!["bulk-1".to_string(), "bulk-2".to_string(), "bulk-3".to_string()];
    let results = TestTb::bulk_select(&ids).fetch_all(&pool).await?;
    assert_eq!(results.len(), 3);
    // Note: Order is not guaranteed, so we just check that all items are present
    let result_ids: Vec<&str> = results.iter().map(|r| r.id.as_str()).collect();
    assert!(result_ids.contains(&"bulk-1"));
    assert!(result_ids.contains(&"bulk-2"));
    assert!(result_ids.contains(&"bulk-3"));

    // Test bulk select with different order (order not guaranteed)
    let ids_reordered = vec!["bulk-3".to_string(), "bulk-1".to_string(), "bulk-2".to_string()];
    let results_reordered = TestTb::bulk_select(&ids_reordered).fetch_all(&pool).await?;
    assert_eq!(results_reordered.len(), 3);
    // Verify all items are present (order may vary)
    let reordered_ids: Vec<&str> = results_reordered.iter().map(|r| r.id.as_str()).collect();
    assert!(reordered_ids.contains(&"bulk-1"));
    assert!(reordered_ids.contains(&"bulk-2"));
    assert!(reordered_ids.contains(&"bulk-3"));

    // Test bulk delete
    let ids_to_delete = vec!["bulk-1".to_string(), "bulk-2".to_string(), "bulk-3".to_string()];
    TestTb::bulk_delete(&ids_to_delete).execute(&pool).await?;

    // Verify deletion
    let remaining = TestTb::bulk_select(&ids_to_delete).fetch_all(&pool).await?;
    assert_eq!(remaining.len(), 0);

    // Clean up after test
    sqlx::query("DELETE FROM test_tb WHERE id LIKE 'bulk-%'").execute(&pool).await.ok();

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_bulk_select_empty_list() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    let empty_ids: Vec<String> = vec![];
    let results = TestTb::bulk_select(&empty_ids).fetch_all(&pool).await?;
    assert_eq!(results.len(), 0);

    Ok(())
}

#[tokio::test]
#[serial]
async fn test_where_and_delete_queries() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://alex:@127.0.0.1/test_sqlx_tokio")
        .await?;

    // Clean up before test - remove both old and new prefixes
    sqlx::query("DELETE FROM test_tb WHERE id LIKE 'test-3-where-test-%'").execute(&pool).await.ok();
    sqlx::query("DELETE FROM test_tb WHERE id LIKE 'where-test-%'").execute(&pool).await.ok();

    // Insert test data
    let items = vec![
        TestTb { id: "where-test-1".to_string(), name: "Test1".to_string(), ts: 10 },
        TestTb { id: "where-test-2".to_string(), name: "Test2".to_string(), ts: 20 },
        TestTb { id: "where-test-3".to_string(), name: "Test3".to_string(), ts: 30 },
    ];
    TestTb::bulk_insert(&items).execute(&pool).await?;

    // Test where_query
    let results = TestTb::where_query("ts >= 20").fetch_all(&pool).await?;
    println!("Results with ts >= 20: {:?}", results);
    assert_eq!(results.len(), 2);

    // Test count_query
    let (count,) = TestTb::count_query("ts >= 20").fetch_one(&pool).await?;
    assert_eq!(count, 2);

    // Test delete_where_query
    TestTb::delete_where_query("ts < 20").execute(&pool).await?;
    let (remaining_count,) = TestTb::count_query("id LIKE 'where-test-%'").fetch_one(&pool).await?;
    assert_eq!(remaining_count, 2);

    // Clean up
    sqlx::query("DELETE FROM test_tb WHERE id LIKE 'test-3-where-test-%'").execute(&pool).await.ok();
    sqlx::query("DELETE FROM test_tb WHERE id LIKE 'where-test-%'").execute(&pool).await.ok();

    Ok(())
}

