use std::boxed::Box;
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::FromRow;
use sqlx::Database;
use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;

#[derive(Debug, FromRow, EnhancedCrud)]
struct Bar {
    id: String,
    name: String,
    ts: i32
}


#[test]
fn it_works() {
    let scheme = Scheme{
        table_name: "test_table".to_string(),
        insert_fields: vec!["id".to_string(), "name".to_string(), "ts".to_string()],
        update_fields: vec!["name".to_string(), "ts".to_string()],
        id_field: "id".to_string()
    };
    println!("insert sql:{}", scheme.gen_insert_sql());
    println!("update sql:{}", scheme.gen_update_by_id_sql());
    println!("delete sql:{}", scheme.gen_delete_sql());
    println!("select sql:{}", scheme.gen_select_by_id_sql());
    println!("select where sql:{}", scheme.gen_select_where_sql("a={} and b={}"));
    println!("update where sql:{}", scheme.gen_update_where_sql("a={} and b={}"));
    println!("delete where sql:{}", scheme.gen_delete_where_sql("a={} and b={}"));
    println!("Done");
}

