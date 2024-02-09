use sqlx::query::{Query, QueryAs};
use sqlx::database::HasArguments;
use sqlx::database::Database;
use sqlx::FromRow;
use sqlx::postgres::Postgres;
use sqlx::mysql::MySql;
use sqlx::sqlite::Sqlite;

#[cfg(feature = "postgres")]
pub trait EnhancedCrud {
    fn insert_bind(&self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn update_bind(&self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn delete_bind(&self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn select_by_id<'f, O>() -> QueryAs<'f, Postgres, O, <Postgres as HasArguments<'f>>::Arguments>
    where
        O: for<'r> FromRow<'r, <Postgres as Database>::Row>;
    fn select_where<'f, O>(w: &str) -> QueryAs<'f, Postgres, O, <Postgres as HasArguments<'f>>::Arguments>
    where
        O: for<'r> FromRow<'r, <Postgres as Database>::Row>;
    fn update_where(&self, w: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
    fn delete_where(&self, w: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>;
}

#[cfg(feature = "mysql")]
pub trait EnhancedCrud {
    fn insert_bind(&self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn update_bind(&self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn delete_bind(&self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn select_by_id<'f, O>() -> QueryAs<'f, MySql, O, <MySql as HasArguments<'f>>::Arguments>
    where
        O: for<'r> FromRow<'r, <MySql as Database>::Row>;
    fn select_where<'f, O>(w: &str) -> QueryAs<'f, MySql, O, <MySql as HasArguments<'f>>::Arguments>
    where
        O: for<'r> FromRow<'r, <MySql as Database>::Row>;
    fn update_where(&self, w: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
    fn delete_where(&self, w: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>;
}

#[cfg(feature = "sqlite")]
pub trait EnhancedCrud {
    fn insert_bind(&self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn update_bind(&self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn delete_bind(&self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn select_by_id<'f, O>() -> QueryAs<'f, Sqlite, O, <Sqlite as HasArguments<'f>>::Arguments>
    where
        O: for<'r> FromRow<'r, <Sqlite as Database>::Row>;
    fn select_where<'f, O>(w: &str) -> QueryAs<'f, Sqlite, O, <Sqlite as HasArguments<'f>>::Arguments>
    where
        O: for<'r> FromRow<'r, <Sqlite as Database>::Row>;
    fn update_where(&self, w: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
    fn delete_where(&self, w: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>;
}

