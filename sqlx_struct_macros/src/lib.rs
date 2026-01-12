// 编译期索引分析模块
mod compile_time_analyzer;
mod query_extractor;
mod simple_parser;
mod struct_schema_parser;

// DECIMAL 辅助方法生成模块
mod decimal_helpers;

// Advanced SQL parser module (based on sqlparser-rs)
mod parser;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

// Prevent simultaneous activation of multiple database features
// This avoids compilation errors due to type ambiguity in JoinQueryBuilder
#[cfg(all(feature = "postgres", feature = "mysql"))]
compile_error!("Cannot enable both 'postgres' and 'mysql' features simultaneously. Please choose one database backend.");

#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("Cannot enable both 'postgres' and 'sqlite' features simultaneously. Please choose one database backend.");

#[cfg(all(feature = "mysql", feature = "sqlite"))]
compile_error!("Cannot enable both 'mysql' and 'sqlite' features simultaneously. Please choose one database backend.");

// Single derive macro that uses conditional compilation internally
#[proc_macro_derive(EnhancedCrud, attributes(table_name, crud))]
pub fn enhanced_crud_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    // Generate EnhancedCrud implementation
    let sql_builder = SqlBuilder::new(Schema::new(&input));
    let table_name = sql_builder.scheme.table_name.clone();
    let gen_scheme_code = sql_builder.gen_scheme_code();
    let gen_fill_id = sql_builder.fill_id_param();
    let gen_join_field_extraction = sql_builder.gen_join_field_extraction(&name);

    // Extract DECIMAL fields and generate helper methods
    use decimal_helpers;
    let decimal_fields = decimal_helpers::extract_decimal_fields(&input);
    let decimal_helpers_impl = if !decimal_fields.is_empty() {
        decimal_helpers::generate_decimal_helpers_impl(&name, &decimal_fields)
    } else {
        quote! {}
    };

    // Generate parameter binding code for each database type
    let gen_fill_insert_pg = sql_builder.fill_insert_param(quote!(::sqlx::Postgres));
    let gen_fill_update_pg = sql_builder.fill_update_param(quote!(::sqlx::Postgres));
    let gen_fill_bulk_insert_pg = sql_builder.fill_bulk_insert_param(&quote!(::sqlx::Postgres));
    let gen_fill_bulk_update_pg = sql_builder.fill_bulk_update_param(&quote!(::sqlx::Postgres));

    let gen_fill_insert_mysql = sql_builder.fill_insert_param(quote!(::sqlx::MySql));
    let gen_fill_update_mysql = sql_builder.fill_update_param(quote!(::sqlx::MySql));
    let gen_fill_bulk_insert_mysql = sql_builder.fill_bulk_insert_param(&quote!(::sqlx::MySql));
    let gen_fill_bulk_update_mysql = sql_builder.fill_bulk_update_param(&quote!(::sqlx::MySql));

    let gen_fill_insert_sqlite = sql_builder.fill_insert_param(quote!(::sqlx::Sqlite));
    let gen_fill_update_sqlite = sql_builder.fill_update_param(quote!(::sqlx::Sqlite));
    let gen_fill_bulk_insert_sqlite = sql_builder.fill_bulk_insert_param(&quote!(::sqlx::Sqlite));
    let gen_fill_bulk_update_sqlite = sql_builder.fill_bulk_update_param(&quote!(::sqlx::Sqlite));

    // Each database feature defines its own implementation function
    // Only the enabled feature's function will be compiled

    #[cfg(feature = "postgres")]
    let enhanced_crud_impl = postgres_impl(
        name.clone(),
        table_name.clone(),
        gen_scheme_code,
        gen_fill_insert_pg,
        gen_fill_update_pg,
        gen_fill_id,
        gen_fill_bulk_insert_pg,
        gen_fill_bulk_update_pg,
        gen_join_field_extraction.clone(),
    );

    #[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
    let enhanced_crud_impl = mysql_impl(
        name.clone(),
        table_name.clone(),
        gen_scheme_code,
        gen_fill_insert_mysql,
        gen_fill_update_mysql,
        gen_fill_id,
        gen_fill_bulk_insert_mysql,
        gen_fill_bulk_update_mysql,
        gen_join_field_extraction.clone(),
    );

    #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
    let enhanced_crud_impl = sqlite_impl(
        name.clone(),
        table_name.clone(),
        gen_scheme_code,
        gen_fill_insert_sqlite,
        gen_fill_update_sqlite,
        gen_fill_id,
        gen_fill_bulk_insert_sqlite,
        gen_fill_bulk_update_sqlite,
        gen_join_field_extraction.clone(),
    );

    #[cfg(not(any(feature = "postgres", feature = "mysql", feature = "sqlite")))]
    let enhanced_crud_impl = quote! {
        compile_error!("You must enable one of the database features: postgres, mysql, or sqlite");
    };

    // Combine EnhancedCrud impl with DECIMAL helpers impl
    let output_token = quote! {
        #enhanced_crud_impl
        #decimal_helpers_impl
    };

    output_token.into()
}

#[cfg(feature = "postgres")]
#[allow(dead_code)]  // Used conditionally based on feature flags
fn postgres_impl(
    name: Ident,
    table_name: String,
    gen_scheme_code: TokenStream2,
    gen_fill_insert: TokenStream2,
    gen_fill_update: TokenStream2,
    gen_fill_id: TokenStream2,
    gen_fill_bulk_insert: TokenStream2,
    gen_fill_bulk_update: TokenStream2,
    gen_join_field_extraction: TokenStream2,
) -> TokenStream2 {
    quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] INSERT SQL: {}", sql);
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_insert
                query
            }
            fn update_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] UPDATE SQL: {}", sql);
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] DELETE SQL: {}", sql);
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_id
                query
            }
            fn by_pk<'q>() -> QueryAs<'q, Postgres, Self, <Postgres as HasArguments<'q>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] SELECT BY PK SQL: {}", sql);
                sqlx::query_as::<Postgres, Self>(sql)
            }
            fn make_query(sql: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let leaked_sql = Box::leak(sql.into_boxed_str());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] MAKE QUERY SQL: {}", leaked_sql);
                let query = sqlx::query_as::<Postgres, Self>(leaked_sql);
                query
            }
            fn make_execute(sql: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let leaked_sql = Box::leak(sql.into_boxed_str());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] MAKE EXECUTE SQL: {}", leaked_sql);
                let query = sqlx::query::<Postgres>(leaked_sql);
                query
            }
            fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] WHERE QUERY SQL: {}", sql);
                let query = sqlx::query_as::<Postgres, Self>(sql);
                query
            }
            fn count_query(statement: &str) -> QueryAs<'_, Postgres, (i64,), <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_count_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] COUNT QUERY SQL: {}", sql);
                let query = sqlx::query_as::<Postgres, (i64,)>(sql);
                query
            }
            fn delete_where_query(statement: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] DELETE WHERE SQL: {}", sql);
                let query = sqlx::query::<Postgres>(sql);
                query
            }
            fn bulk_delete(ids: &[String]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_delete_sql_static(ids.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK DELETE SQL: {}", sql);
                let mut query = sqlx::query::<Postgres>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn bulk_insert(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_insert_sql_static(items.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK INSERT SQL: {}", sql);
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_bulk_insert
            }
            fn bulk_update(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_update_sql_static(items.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK UPDATE SQL: {}", sql);
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_bulk_update
            }
            fn bulk_select(ids: &[String]) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_select_sql_static(ids.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK SELECT SQL: {}", sql);
                let mut query = sqlx::query_as::<Postgres, Self>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn agg_query() -> ::sqlx_struct_enhanced::aggregate::AggQueryBuilder<'static, Postgres> where Self: Sized {
                ::sqlx_struct_enhanced::aggregate::AggQueryBuilder::new(#table_name.to_string())
            }

            #[cfg(feature = "join_queries")]
            fn join_inner<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Postgres>
            where
                Self: Sized,
                T: Sized + ::sqlx_struct_enhanced::join::SchemeAccessor + std::marker::Unpin + Send
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN INNER SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Inner,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_left<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Postgres>
            where
                Self: Sized,
                T: Sized + ::sqlx_struct_enhanced::join::SchemeAccessor + std::marker::Unpin + Send
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN LEFT SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Left,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_right<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Postgres>
            where
                Self: Sized,
                T: Sized + ::sqlx_struct_enhanced::join::SchemeAccessor + std::marker::Unpin + Send
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN RIGHT SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Right,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_full<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Postgres>
            where
                Self: Sized,
                T: Sized + ::sqlx_struct_enhanced::join::SchemeAccessor + std::marker::Unpin + Send
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN FULL SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Full,
                    condition
                )
            }
        }

        #[cfg(feature = "join_queries")]
        impl ::sqlx_struct_enhanced::join::SchemeAccessor for #name {
            fn get_scheme() -> &'static ::sqlx_struct_enhanced::Scheme {
                #gen_scheme_code
                &scheme
            }

            #[cfg(feature = "postgres")]
            fn decode_from_qualified_row_pg(row: &::sqlx::postgres::PgRow) -> Result<Option<Self>, ::sqlx::Error>
            where
                Self: Sized
            {
                #gen_join_field_extraction
            }
        }
    }
}

#[cfg(feature = "mysql")]
#[allow(dead_code)]  // Used conditionally based on feature flags
fn mysql_impl(
    name: Ident,
    table_name: String,
    gen_scheme_code: TokenStream2,
    gen_fill_insert: TokenStream2,
    gen_fill_update: TokenStream2,
    gen_fill_id: TokenStream2,
    gen_fill_bulk_insert: TokenStream2,
    gen_fill_bulk_update: TokenStream2,
    gen_join_field_extraction: TokenStream2,
) -> TokenStream2 {
    quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] INSERT SQL: {}", sql);
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_insert
                query
            }
            fn update_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] UPDATE SQL: {}", sql);
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] DELETE SQL: {}", sql);
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_id
                query
            }
            fn by_pk<'q>() -> QueryAs<'q, MySql, Self, <MySql as HasArguments<'q>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] SELECT BY PK SQL: {}", sql);
                sqlx::query_as::<MySql, Self>(sql)
            }
            fn make_query(sql: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let leaked_sql = Box::leak(sql.into_boxed_str());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] MAKE QUERY SQL: {}", leaked_sql);
                let query = sqlx::query_as::<MySql, Self>(leaked_sql);
                query
            }
            fn make_execute(sql: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let leaked_sql = Box::leak(sql.into_boxed_str());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] MAKE EXECUTE SQL: {}", leaked_sql);
                let query = sqlx::query::<MySql>(leaked_sql);
                query
            }
            fn where_query(statement: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] WHERE QUERY SQL: {}", sql);
                let query = sqlx::query_as::<MySql, Self>(sql);
                query
            }
            fn count_query(statement: &str) -> QueryAs<'_, MySql, (i64,), <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_count_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] COUNT QUERY SQL: {}", sql);
                let query = sqlx::query_as::<MySql, (i64,)>(sql);
                query
            }
            fn delete_where_query(statement: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] DELETE WHERE SQL: {}", sql);
                let query = sqlx::query::<MySql>(sql);
                query
            }
            fn bulk_delete(ids: &[String]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_delete_sql_static(ids.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK DELETE SQL: {}", sql);
                let mut query = sqlx::query::<MySql>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn bulk_insert(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_insert_sql_static(items.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK INSERT SQL: {}", sql);
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_bulk_insert
            }
            fn bulk_update(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_update_sql_static(items.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK UPDATE SQL: {}", sql);
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_bulk_update
            }
            fn bulk_select(ids: &[String]) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_select_sql_static(ids.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK SELECT SQL: {}", sql);
                let mut query = sqlx::query_as::<MySql, Self>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn agg_query() -> ::sqlx_struct_enhanced::aggregate::AggQueryBuilder<'static, MySql> where Self: Sized {
                ::sqlx_struct_enhanced::aggregate::AggQueryBuilder::new(#table_name.to_string())
            }

            #[cfg(feature = "join_queries")]
            fn join_inner<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, MySql>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN INNER SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Inner,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_left<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, MySql>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN LEFT SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Left,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_right<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, MySql>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN RIGHT SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Right,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_full<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, MySql>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN FULL SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Full,
                    condition
                )
            }
        }

        #[cfg(feature = "join_queries")]
        impl ::sqlx_struct_enhanced::join::SchemeAccessor for #name {
            fn get_scheme() -> &'static ::sqlx_struct_enhanced::Scheme {
                #gen_scheme_code
                &scheme
            }

            #[cfg(feature = "mysql")]
            fn decode_from_qualified_row_mysql(row: &::sqlx::mysql::MySqlRow) -> Result<Option<Self>, ::sqlx::Error>
            where
                Self: Sized
            {
                #gen_join_field_extraction
            }
        }
    }
}

#[cfg(feature = "sqlite")]
#[allow(dead_code)]  // Used conditionally based on feature flags
fn sqlite_impl(
    name: Ident,
    table_name: String,
    gen_scheme_code: TokenStream2,
    gen_fill_insert: TokenStream2,
    gen_fill_update: TokenStream2,
    gen_fill_id: TokenStream2,
    gen_fill_bulk_insert: TokenStream2,
    gen_fill_bulk_update: TokenStream2,
    gen_join_field_extraction: TokenStream2,
) -> TokenStream2 {
    quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] INSERT SQL: {}", sql);
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_insert
                query
            }
            fn update_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] UPDATE SQL: {}", sql);
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] DELETE SQL: {}", sql);
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_id
                query
            }
            fn by_pk<'q>() -> QueryAs<'q, Sqlite, Self, <Sqlite as HasArguments<'q>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql_static();
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] SELECT BY PK SQL: {}", sql);
                sqlx::query_as::<Sqlite, Self>(sql)
            }
            fn make_query(sql: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let leaked_sql = Box::leak(sql.into_boxed_str());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] MAKE QUERY SQL: {}", leaked_sql);
                let query = sqlx::query_as::<Sqlite, Self>(leaked_sql);
                query
            }
             fn make_execute(sql: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let leaked_sql = Box::leak(sql.into_boxed_str());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] MAKE EXECUTE SQL: {}", leaked_sql);
                let query = sqlx::query::<Sqlite>(leaked_sql);
                query
            }
            fn where_query(statement: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] WHERE QUERY SQL: {}", sql);
                let query = sqlx::query_as::<Sqlite, Self>(sql);
                query
            }
            fn count_query(statement: &str) -> QueryAs<'_, Sqlite, (i64,), <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_count_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] COUNT QUERY SQL: {}", sql);
                let query = sqlx::query_as::<Sqlite, (i64,)>(sql);
                query
            }
            fn delete_where_query(statement: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql_static(statement);
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] DELETE WHERE SQL: {}", sql);
                let query = sqlx::query::<Sqlite>(sql);
                query
            }
            fn bulk_delete(ids: &[String]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_delete_sql_static(ids.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK DELETE SQL: {}", sql);
                let mut query = sqlx::query::<Sqlite>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn bulk_insert(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_insert_sql_static(items.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK INSERT SQL: {}", sql);
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_bulk_insert
            }
            fn bulk_update(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_update_sql_static(items.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK UPDATE SQL: {}", sql);
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_bulk_update
            }
            fn bulk_select(ids: &[String]) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_select_sql_static(ids.len());
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] BULK SELECT SQL: {}", sql);
                let mut query = sqlx::query_as::<Sqlite, Self>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn agg_query() -> ::sqlx_struct_enhanced::aggregate::AggQueryBuilder<'static, Sqlite> where Self: Sized {
                ::sqlx_struct_enhanced::aggregate::AggQueryBuilder::new(#table_name.to_string())
            }

            #[cfg(feature = "join_queries")]
            fn join_inner<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Sqlite>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN INNER SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Inner,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_left<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Sqlite>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN LEFT SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Left,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_right<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Sqlite>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN RIGHT SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Right,
                    condition
                )
            }

            #[cfg(feature = "join_queries")]
            fn join_full<T>(condition: &str) -> ::sqlx_struct_enhanced::join::JoinQueryBuilder<'static, Self, T, Sqlite>
            where
                Self: Sized,
                T: Sized
            {
                #[cfg(feature = "log_sql")]
                eprintln!("[SQLxEnhanced] JOIN FULL SQL: condition={}", condition);
                ::sqlx_struct_enhanced::join::JoinQueryBuilder::new(
                    ::sqlx_struct_enhanced::join::JoinType::Full,
                    condition
                )
            }
        }

        #[cfg(feature = "join_queries")]
        impl ::sqlx_struct_enhanced::join::SchemeAccessor for #name {
            fn get_scheme() -> &'static ::sqlx_struct_enhanced::Scheme {
                #gen_scheme_code
                &scheme
            }

            #[cfg(feature = "sqlite")]
            fn decode_from_qualified_row_sqlite(row: &::sqlx::sqlite::SqliteRow) -> Result<Option<Self>, ::sqlx::Error>
            where
                Self: Sized
            {
                #gen_join_field_extraction
            }
        }
    }
}

/// Converts a PascalCase or camelCase string to snake_case.
///
/// # Example
///
/// ```ignore
/// to_snake_case("MyTable");  // "my_table"
/// to_snake_case("userProfile");  // "user_profile"
/// ```
fn to_snake_case(s: &str) -> String {
    // Pre-allocate with capacity: each char might need 2 bytes (char + underscore)
    let mut result = String::with_capacity(s.len() * 2);

    for (i, c) in s.char_indices() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }

    result
}

/// Types that require BindProxy conversion for database binding
///
/// NOTE: The following types are NOT in this list because they already implement
/// Encode<'q, Postgres> + Type<Postgres> for PostgreSQL and should be bound
/// directly without BindProxy conversion:
///
/// - Uuid: uuid::Uuid implements native PostgreSQL uuid encoding
/// - DateTime/NaiveDateTime: chrono types implement native PostgreSQL timestamp encoding
/// - NaiveDate/NaiveTime: chrono types implement native PostgreSQL date/time encoding
///
/// Converting these types to String causes type mismatch errors when binding to
/// PostgreSQL columns (uuid, timestamp, date, time).
const TYPE_NEEDS_PROXY: &[&str] = &[
    "Decimal",
    "Json",
];

/// Extract base type name, handling Option<T> wrapper
/// Returns the inner type name if wrapped in Option, otherwise the type name
fn get_base_type_name(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                // Option<T> - extract inner type
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                        return get_type_name_from_type(inner_type);
                    }
                }
            }
        }
        return get_type_name_from_type(ty);
    }
    get_type_name_from_type(ty)
}

/// Get the type name as a string from a syn::Type
fn get_type_name_from_type(ty: &syn::Type) -> String {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident.to_string();
        }
    }
    String::new()
}

/// Check if type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Generate BindProxy conversion code for a field in bulk operations.
///
/// This function creates the TokenStream for converting a field value using BindProxy,
/// handling both Option<T> and required T types.
///
/// # Arguments
///
/// * `field` - The field identifier (e.g., `field_name`)
/// * `ty` - The field type for checking if it needs proxy conversion
///
/// # Returns
///
/// TokenStream containing the conversion code
fn gen_bind_proxy_conversion_for_item(field: &Ident, ty: &syn::Type, db_type: &TokenStream2) -> TokenStream2 {
    let type_name = get_base_type_name(ty);
    let needs_proxy = TYPE_NEEDS_PROXY.contains(&type_name.as_str());

    if needs_proxy {
        if is_option_type(ty) {
            // Option<T> conversion with BindProxy
            quote! {
                if let Some(v) = &item.#field {
                    let bind_val = ::sqlx_struct_enhanced::proxy::BindProxy::<#db_type>::into_bind_value(v.clone());
                    match bind_val {
                        ::sqlx_struct_enhanced::proxy::BindValue::String(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::I32(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::I64(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::F64(f) => query.bind(f),
                        ::sqlx_struct_enhanced::proxy::BindValue::Bool(b) => query.bind(b),
                        ::sqlx_struct_enhanced::proxy::BindValue::Decimal(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::I8(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::I16(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::F32(f) => query.bind(f),
                        ::sqlx_struct_enhanced::proxy::BindValue::NaiveDate(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::NaiveTime(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::NaiveDateTime(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::DateTimeUtc(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::Json(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::Binary(bytes) => query.bind(bytes),
                        _ => query,
                    }
                } else {
                    query.bind::<Option<String>>(None)
                }
            }
        } else {
            // Required T conversion with BindProxy
            quote! {
                {
                    let bind_val = ::sqlx_struct_enhanced::proxy::BindProxy::<#db_type>::into_bind_value(item.#field.clone());
                    match bind_val {
                        ::sqlx_struct_enhanced::proxy::BindValue::String(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::I32(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::I64(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::F64(f) => query.bind(f),
                        ::sqlx_struct_enhanced::proxy::BindValue::Bool(b) => query.bind(b),
                        ::sqlx_struct_enhanced::proxy::BindValue::Decimal(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::I8(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::I16(i) => query.bind(i),
                        ::sqlx_struct_enhanced::proxy::BindValue::F32(f) => query.bind(f),
                        ::sqlx_struct_enhanced::proxy::BindValue::NaiveDate(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::NaiveTime(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::NaiveDateTime(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::DateTimeUtc(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::Json(s) => query.bind(s),
                        ::sqlx_struct_enhanced::proxy::BindValue::Binary(bytes) => query.bind(bytes),
                        _ => query,
                    }
                }
            }
        }
    } else {
        // Direct binding for simple types
        quote! {
            query.bind(&item.#field)
        }
    }
}

/// Column metadata for code generation with optional type casting
#[derive(Debug, Clone)]
struct ColumnDefinition {
    name: String,
    cast_as: Option<String>,
    is_decimal: bool,
    is_uuid: bool,
}

struct Schema {
    table_name: String,
    fields: Vec<Ident>,
    id_field: Ident,
    column_definitions: Vec<ColumnDefinition>,
    field_types: Vec<syn::Type>,  // Store field type information for BindProxy detection
}

impl Schema {
    fn new(input: &DeriveInput) -> Self {
        // Check for custom table_name attribute
        let table_name = input.attrs.iter()
            .find(|attr| {
                // Check if this is a table_name attribute
                let path_str = quote::quote!(#attr).to_string();
                path_str.contains("table_name")
            })
            .and_then(|attr| {
                // Parse the attribute value: table_name = "my_table"
                let tokens = attr.tokens.to_string();
                if let Some(eq_pos) = tokens.find('=') {
                    let value_str = &tokens[eq_pos + 1..];
                    // Remove quotes if present
                    let value = value_str.trim().trim_matches('"').trim_matches('\'');
                    Some(value.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| to_snake_case(input.ident.to_string().as_str()));

        // 获取结构体字段
        let fields = match input.data.clone() {
            syn::Data::Struct(data) => data.fields,
            _ => panic!("Only structs are supported"),
        };
        let fields_name: Vec<Ident> = fields.iter().map(|field| {
            field.ident.as_ref().unwrap().clone()
        }).collect();
        let id_field = fields_name.first().expect("Struct must have at least one field").clone();

        // Extract field types for BindProxy detection
        let field_types: Vec<syn::Type> = fields.iter()
            .map(|field| field.ty.clone())
            .collect();

        // Parse column definitions with cast_as
        let column_definitions = fields.iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap().to_string();
                let mut cast_as = None;
                let mut is_decimal = false;
                let mut is_uuid = false;

                // Detect field type for UUID
                let type_str = quote::quote!(#field.ty).to_string();
                // Check if field type is uuid::Uuid or Uuid
                if type_str.contains("uuid::Uuid") || type_str.contains("Uuid") {
                    is_uuid = true;
                }

                // Parse #[crud(...)] attributes
                for attr in &field.attrs {
                    let path_str = quote::quote!(#attr).to_string();
                    if path_str.contains("crud") {
                        let tokens = attr.tokens.to_string();

                        // First: Check if this is a decimal field
                        if tokens.contains("decimal") {
                            is_decimal = true;

                            // Extract cast_as from within decimal(...)
                            if let Some(decimal_pos) = tokens.find("decimal") {
                                let remaining = &tokens[decimal_pos..];
                                if let Some(open_paren) = remaining.find('(') {
                                    if let Some(close_paren) = remaining.find(')') {
                                        let params_str = &remaining[open_paren + 1..close_paren];

                                        // Parse cast_as parameter
                                        if let Some(cast_pos) = params_str.find("cast_as") {
                                            let cast_remaining = &params_str[cast_pos..];
                                            if let Some(eq_pos) = cast_remaining.find('=') {
                                                let value_str = &cast_remaining[eq_pos + 1..];
                                                let end_pos = value_str.find(',').unwrap_or(value_str.len());
                                                let value = value_str[..end_pos]
                                                    .trim()
                                                    .trim_matches('"')
                                                    .trim_matches('\'');
                                                if !value.is_empty() {
                                                    cast_as = Some(value.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Second: Parse separate #[crud(cast_as = "TYPE")] (takes precedence)
                        if let Some(cast_pos) = tokens.find("cast_as") {
                            let remaining = &tokens[cast_pos..];
                            if let Some(eq_pos) = remaining.find('=') {
                                let value_str = &remaining[eq_pos + 1..];
                                let end_pos = value_str.find(',').unwrap_or(value_str.len());
                                let value = value_str[..end_pos]
                                    .trim()
                                    .trim_matches(|c| c == '"' || c == '\'' || c == ')' || c == ' ');
                                if !value.is_empty() {
                                    cast_as = Some(value.to_string());  // Overrides decimal() cast_as
                                }
                            }
                        }
                    }
                }

                // Note: We NO longer auto-set cast_as for decimal fields
                // is_decimal is used for INSERT/UPDATE (::numeric cast)
                // is_uuid is used for bulk operations (::uuid cast in WHERE IN clauses)
                // cast_as is used for SELECT (output type conversion)

                ColumnDefinition { name, cast_as, is_decimal, is_uuid }
            })
            .collect();

        Self {
            table_name,
            fields: fields_name,
            id_field,
            column_definitions,
            field_types,
        }
    }
}

struct SqlBuilder {
    scheme: Schema
}

impl SqlBuilder {
    fn new(s: Schema)-> SqlBuilder{
        SqlBuilder { scheme: s }
    }

    fn gen_scheme_code(&self) -> TokenStream2 {
        let table_name = self.scheme.table_name.clone();
        let id_field = self.scheme.id_field.clone();
        let append_insert_stmt = self.scheme.fields.iter().map(|f|{
            quote!{
                stringify!(#f).to_string()
            }
        });
        let append_update_stmt = self.scheme.fields[1..].iter().map(|f|{
            quote!{
                stringify!(#f).to_string()
            }
        });

        // Generate column definitions with optional casting
        let column_definitions = self.scheme.column_definitions.iter().map(|col| {
            let name = &col.name;
            let cast_as = &col.cast_as;
            let is_decimal = &col.is_decimal;
            let is_uuid = &col.is_uuid;
            match cast_as {
                Some(cast_type) => {
                    quote! {
                        ::sqlx_struct_enhanced::ColumnDefinition {
                            name: #name.to_string(),
                            cast_as: Some(#cast_type.to_string()),
                            is_decimal: #is_decimal,
                            is_uuid: #is_uuid,
                        }
                    }
                }
                None => {
                    quote! {
                        ::sqlx_struct_enhanced::ColumnDefinition {
                            name: #name.to_string(),
                            cast_as: None,
                            is_decimal: #is_decimal,
                            is_uuid: #is_uuid,
                        }
                    }
                }
            }
        });

        quote!{
            static mut SCHEME: Option<::sqlx_struct_enhanced::Scheme> = None;
            let scheme = unsafe {
                if SCHEME.is_none() {
                    SCHEME = Some(::sqlx_struct_enhanced::Scheme {
                        table_name: #table_name.to_string(),
                        insert_fields: vec![#(#append_insert_stmt),*],
                        update_fields: vec![#(#append_update_stmt),*],
                        id_field: stringify!(#id_field).to_string(),
                        column_definitions: vec![#(#column_definitions),*],
                    });
                }
                SCHEME.as_ref().unwrap()
            };
        }
    }

    fn fill_insert_param(&self, db_type: TokenStream2) -> TokenStream2 {
        let bind_stmts = self.scheme.fields.iter().enumerate().map(|(i, field)| {
            let ty = &self.scheme.field_types[i];
            let type_name = get_base_type_name(ty);
            let needs_proxy = TYPE_NEEDS_PROXY.contains(&type_name.as_str());

            if needs_proxy {
                if is_option_type(ty) {
                    // Option<Decimal> -> convert using BindProxy trait
                    quote! {
                        let query = if let Some(v) = std::mem::replace(&mut self.#field, None) {
                            let bind_val = ::sqlx_struct_enhanced::proxy::BindProxy::<#db_type>::into_bind_value(v);
                            match bind_val {
                                ::sqlx_struct_enhanced::proxy::BindValue::String(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I32(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I64(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F64(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::Bool(b) => query.bind(b),
                                ::sqlx_struct_enhanced::proxy::BindValue::Decimal(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I8(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I16(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F32(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDate(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDateTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::DateTimeUtc(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Json(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Binary(bytes) => query.bind(bytes),
                                ::sqlx_struct_enhanced::proxy::BindValue::Uuid(s) => query.bind(s),
                                _ => query,
                            }
                        } else {
                            query.bind::<Option<String>>(None)
                        };
                    }
                } else {
                    // Decimal, DateTime, etc. -> convert using BindProxy trait
                    quote! {
                        let query = {
                            let bind_val = ::sqlx_struct_enhanced::proxy::BindProxy::<#db_type>::into_bind_value(self.#field);
                            match bind_val {
                                ::sqlx_struct_enhanced::proxy::BindValue::String(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I32(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I64(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F64(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::Bool(b) => query.bind(b),
                                ::sqlx_struct_enhanced::proxy::BindValue::Decimal(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I8(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I16(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F32(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDate(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDateTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::DateTimeUtc(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Json(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Binary(bytes) => query.bind(bytes),
                                ::sqlx_struct_enhanced::proxy::BindValue::Uuid(s) => query.bind(s),
                                _ => query,
                            }
                        };
                    }
                }
            } else {
                // Basic types (String, i32, etc.) -> use bind()
                quote! {
                    let query = query.bind(&self.#field);
                }
            }
        });
        quote! {
            #(#bind_stmts)*
        }
    }

    fn fill_update_param(&self, db_type: TokenStream2) -> TokenStream2 {
        let bind_stmts = self.scheme.fields[1..].iter().enumerate().map(|(i, field)| {
            let actual_index = i + 1; // Skip first field (id)
            let ty = &self.scheme.field_types[actual_index];
            let type_name = get_base_type_name(ty);
            let needs_proxy = TYPE_NEEDS_PROXY.contains(&type_name.as_str());

            if needs_proxy {
                if is_option_type(ty) {
                    // Option<Decimal> -> convert using BindProxy trait
                    quote! {
                        let query = if let Some(v) = std::mem::replace(&mut self.#field, None) {
                            let bind_val = ::sqlx_struct_enhanced::proxy::BindProxy::<#db_type>::into_bind_value(v);
                            match bind_val {
                                ::sqlx_struct_enhanced::proxy::BindValue::String(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I32(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I64(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F64(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::Bool(b) => query.bind(b),
                                ::sqlx_struct_enhanced::proxy::BindValue::Decimal(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I8(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I16(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F32(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDate(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDateTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::DateTimeUtc(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Json(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Binary(bytes) => query.bind(bytes),
                                ::sqlx_struct_enhanced::proxy::BindValue::Uuid(s) => query.bind(s),
                                _ => query,
                            }
                        } else {
                            query.bind::<Option<String>>(None)
                        };
                    }
                } else {
                    // Decimal, DateTime, etc. -> convert using BindProxy trait
                    quote! {
                        let query = {
                            let bind_val = ::sqlx_struct_enhanced::proxy::BindProxy::<#db_type>::into_bind_value(self.#field);
                            match bind_val {
                                ::sqlx_struct_enhanced::proxy::BindValue::String(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I32(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I64(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F64(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::Bool(b) => query.bind(b),
                                ::sqlx_struct_enhanced::proxy::BindValue::Decimal(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::I8(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::I16(i) => query.bind(i),
                                ::sqlx_struct_enhanced::proxy::BindValue::F32(f) => query.bind(f),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDate(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::NaiveDateTime(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::DateTimeUtc(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Json(s) => query.bind(s),
                                ::sqlx_struct_enhanced::proxy::BindValue::Binary(bytes) => query.bind(bytes),
                                ::sqlx_struct_enhanced::proxy::BindValue::Uuid(s) => query.bind(s),
                                _ => query,
                            }
                        };
                    }
                }
            } else {
                // Basic types (String, i32, etc.) -> use bind()
                quote! {
                    let query = query.bind(&self.#field);
                }
            }
        });
        quote! {
            #(#bind_stmts)*
        }
    }

    fn fill_id_param(&self) -> TokenStream2 {
        let id_field = self.scheme.id_field.clone();
        quote! {
            let query = query.bind(&self.#id_field);
        }
    }

    fn fill_bulk_insert_param(&self, db_type: &TokenStream2) -> TokenStream2 {
        let fields = &self.scheme.fields;
        let field_types = &self.scheme.field_types;

        let bind_conversions = fields.iter().enumerate().map(|(i, field)| {
            gen_bind_proxy_conversion_for_item(field, &field_types[i], db_type)
        });

        quote! {
            let mut query = query;
            for item in items {
                #(query = #bind_conversions;)*
            }
            query
        }
    }

    fn fill_bulk_update_param(&self, db_type: &TokenStream2) -> TokenStream2 {
        let id_field = &self.scheme.id_field;
        let update_fields = &self.scheme.fields[1..];
        let update_types = &self.scheme.field_types[1..];
        let id_type = &self.scheme.field_types[0];

        // Generate BindProxy conversion for id field
        let id_conversion = gen_bind_proxy_conversion_for_item(id_field, id_type, db_type);

        // Generate BindProxy conversions for update fields
        let update_conversions = update_fields.iter().enumerate().map(|(i, field)| {
            gen_bind_proxy_conversion_for_item(field, &update_types[i], db_type)
        });

        quote! {
            let mut query = query;
            // Bind CASE WHEN parameters: for each item, bind id and all update fields
            for item in items {
                query = #id_conversion;
                #(query = #update_conversions;)*
            }
            // Bind IN clause parameters: bind all ids again
            for item in items {
                query = #id_conversion;
            }
            query
        }
    }

    /// Generate code to extract fields from qualified column names for JOIN queries.
    fn gen_join_field_extraction(&self, name: &Ident) -> TokenStream2 {
        let table_name = &self.scheme.table_name;
        let fields = &self.scheme.fields;

        // Generate code to extract each field using qualified column names
        let field_extractions = fields.iter().map(|field| {
            quote! {
                let column_name = format!("{}.{}", #table_name, stringify!(#field));
                let #field: _ = match row.try_get(column_name.as_str()) {
                    Ok(val) => val,
                    Err(::sqlx::Error::ColumnNotFound(_)) => return Ok(None),
                    Err(::sqlx::Error::Decode(_)) => return Ok(None),
                    Err(e) => return Err(e),
                };
            }
        });

        // Generate code to construct the entity
        let field_names = fields.iter().clone();

        // FIXED: Added block wrapper to fix "expected expression, found `let` statement" error
        quote! {
            {
                #(#field_extractions)*

                Ok(Some(#name {
                    #(#field_names),*
                }))
            }
        }
    }
}

// 编译期查询分析属性宏
#[proc_macro_attribute]
pub fn analyze_queries(attr: TokenStream, input: TokenStream) -> TokenStream {
    compile_time_analyzer::analyze_queries(attr, input)
}

// ============================================================================
// Migration Macros
// ============================================================================

/// Generate a simple migration with manual SQL
///
/// # Syntax
///
/// ```ignore
/// let migration = sqlx_struct_macros::migration!("create_users",
///     "CREATE TABLE users (id VARCHAR(36) PRIMARY KEY);",
///     "DROP TABLE users;"
/// );
/// ```
#[proc_macro]
pub fn migration(input: TokenStream) -> TokenStream {
    use syn::{token::Comma, Expr};
    use syn::parse::{Parse, ParseStream};

    // Parse three comma-separated expressions
    struct MigrationInput {
        name: Expr,
        up_sql: Expr,
        down_sql: Expr,
    }

    impl Parse for MigrationInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let name = input.parse::<Expr>()?;
            input.parse::<Comma>()?;
            let up_sql = input.parse::<Expr>()?;
            input.parse::<Comma>()?;
            let down_sql = input.parse::<Expr>()?;

            Ok(MigrationInput { name, up_sql, down_sql })
        }
    }

    let MigrationInput { name, up_sql, down_sql } = parse_macro_input!(input as MigrationInput);

    let expanded = quote::quote! {
        {
            use sqlx_struct_enhanced::migration::Migration;

            let mut migration = Migration::new(#name.to_string(), "manual".to_string());
            migration.up_sql = vec![#up_sql.to_string()];
            migration.down_sql = vec![#down_sql.to_string()];
            migration.checksum = "".to_string();

            migration
        }
    };

    TokenStream::from(expanded)
}

