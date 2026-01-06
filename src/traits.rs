use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;

#[cfg(feature = "postgres")]
use sqlx::postgres::Postgres;

#[cfg(feature = "mysql")]
use sqlx::mysql::MySql;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::Sqlite;

#[cfg(feature = "postgres")]
pub trait EnhancedCrud {
    fn insert_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn update_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn delete_bind(&mut self) ->  Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn by_pk<'q>() -> QueryAs<'q, Postgres, Self, <Postgres as HasArguments<'q>>::Arguments> where Self: Sized;
    fn make_query(sql: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn make_execute(sql: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn count_query(statement: &str) -> QueryAs<'_, Postgres, (i64,), <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn delete_where_query(statement: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_delete(ids: &[String]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_insert(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_update(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_select(ids: &[String]) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized;
}

#[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
pub trait EnhancedCrud {
    fn insert_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn update_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn delete_bind(&mut self) ->  Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn by_pk<'q>() -> QueryAs<'q, MySql, Self, <MySql as HasArguments<'q>>::Arguments> where Self: Sized;
    fn make_query(sql: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn make_execute(sql: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn where_query(statement: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn count_query(statement: &str) -> QueryAs<'_, MySql, (i64,), <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn delete_where_query(statement: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_delete(ids: &[String]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_insert(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_update(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_select(ids: &[String]) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized;
}

#[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
pub trait EnhancedCrud {
    fn insert_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn update_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn delete_bind(&mut self) ->  Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn by_pk<'q>() -> QueryAs<'q, Sqlite, Self, <Sqlite as HasArguments<'q>>::Arguments> where Self: Sized;
    fn make_query(sql: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn make_execute(sql: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn where_query(statement: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn count_query(statement: &str) -> QueryAs<'_, Sqlite, (i64,), <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn delete_where_query(statement: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_delete(ids: &[String]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_insert(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_update(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
    fn bulk_select(ids: &[String]) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized;
}
