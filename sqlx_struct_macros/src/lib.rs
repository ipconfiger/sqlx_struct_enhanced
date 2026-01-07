// 编译期索引分析模块
mod compile_time_analyzer;
mod query_extractor;
mod simple_parser;
mod struct_schema_parser;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Expr};

// Single derive macro that uses conditional compilation internally
#[proc_macro_derive(EnhancedCrud, attributes(table_name, crud))]
pub fn enhanced_crud_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let sql_builder = SqlBuilder::new(Schema::new(&input));
    let name = input.ident;
    let gen_scheme_code = sql_builder.gen_scheme_code();
    let gen_fill_insert = sql_builder.fill_insert_param();
    let gen_fill_update = sql_builder.fill_update_param();
    let gen_fill_id = sql_builder.fill_id_param();
    let gen_fill_bulk_insert = sql_builder.fill_bulk_insert_param();
    let gen_fill_bulk_update = sql_builder.fill_bulk_update_param();

    // Each database feature defines its own implementation function
    // Only the enabled feature's function will be compiled

    #[cfg(feature = "postgres")]
    let output_token = postgres_impl(name, gen_scheme_code, gen_fill_insert, gen_fill_update, gen_fill_id, gen_fill_bulk_insert, gen_fill_bulk_update);

    #[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "sqlite")))]
    let output_token = mysql_impl(name, gen_scheme_code, gen_fill_insert, gen_fill_update, gen_fill_id, gen_fill_bulk_insert, gen_fill_bulk_update);

    #[cfg(all(feature = "sqlite", not(feature = "postgres"), not(feature = "mysql")))]
    let output_token = sqlite_impl(name, gen_scheme_code, gen_fill_insert, gen_fill_update, gen_fill_id, gen_fill_bulk_insert, gen_fill_bulk_update);

    #[cfg(not(any(feature = "postgres", feature = "mysql", feature = "sqlite")))]
    let output_token = quote! {
        compile_error!("You must enable one of the database features: postgres, mysql, or sqlite");
    };

    output_token.into()
}

#[cfg(feature = "postgres")]
fn postgres_impl(
    name: Ident,
    gen_scheme_code: TokenStream2,
    gen_fill_insert: TokenStream2,
    gen_fill_update: TokenStream2,
    gen_fill_id: TokenStream2,
    gen_fill_bulk_insert: TokenStream2,
    gen_fill_bulk_update: TokenStream2,
) -> TokenStream2 {
    quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql_static();
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_insert
                query
            }
            fn update_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql_static();
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&mut self) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql_static();
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_id
                query
            }
            fn by_pk<'q>() -> QueryAs<'q, Postgres, Self, <Postgres as HasArguments<'q>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql_static();
                sqlx::query_as::<Postgres, Self>(sql)
            }
            fn make_query(sql: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let query = sqlx::query_as::<Postgres, Self>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn make_execute(sql: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let query = sqlx::query::<Postgres>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn where_query(statement: &str) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql_static(statement);
                let query = sqlx::query_as::<Postgres, Self>(sql);
                query
            }
            fn count_query(statement: &str) -> QueryAs<'_, Postgres, (i64,), <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_count_sql_static(statement);
                let query = sqlx::query_as::<Postgres, (i64,)>(sql);
                query
            }
            fn delete_where_query(statement: &str) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql_static(statement);
                let query = sqlx::query::<Postgres>(sql);
                query
            }
            fn bulk_delete(ids: &[String]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_delete_sql_static(ids.len());
                let mut query = sqlx::query::<Postgres>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn bulk_insert(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_insert_sql_static(items.len());
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_bulk_insert
            }
            fn bulk_update(items: &[Self]) -> Query<'_, Postgres, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_update_sql_static(items.len());
                let query = sqlx::query::<Postgres>(sql);
                #gen_fill_bulk_update
            }
            fn bulk_select(ids: &[String]) -> QueryAs<'_, Postgres, Self, <Postgres as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_select_sql_static(ids.len());
                let mut query = sqlx::query_as::<Postgres, Self>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
        }
    }
}

#[cfg(feature = "mysql")]
fn mysql_impl(
    name: Ident,
    gen_scheme_code: TokenStream2,
    gen_fill_insert: TokenStream2,
    gen_fill_update: TokenStream2,
    gen_fill_id: TokenStream2,
    gen_fill_bulk_insert: TokenStream2,
    gen_fill_bulk_update: TokenStream2,
) -> TokenStream2 {
    quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql_static();
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_insert
                query
            }
            fn update_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql_static();
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&mut self) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql_static();
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_id
                query
            }
            fn by_pk<'q>() -> QueryAs<'q, MySql, Self, <MySql as HasArguments<'q>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql_static();
                sqlx::query_as::<MySql, Self>(sql)
            }
            fn make_query(sql: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let query = sqlx::query_as::<MySql, Self>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn make_execute(sql: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let query = sqlx::query::<MySql>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn where_query(statement: &str) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql_static(statement);
                let query = sqlx::query_as::<MySql, Self>(sql);
                query
            }
            fn count_query(statement: &str) -> QueryAs<'_, MySql, (i64,), <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_count_sql_static(statement);
                let query = sqlx::query_as::<MySql, (i64,)>(sql);
                query
            }
            fn delete_where_query(statement: &str) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql_static(statement);
                let query = sqlx::query::<MySql>(sql);
                query
            }
            fn bulk_delete(ids: &[String]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_delete_sql_static(ids.len());
                let mut query = sqlx::query::<MySql>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn bulk_insert(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_insert_sql_static(items.len());
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_bulk_insert
            }
            fn bulk_update(items: &[Self]) -> Query<'_, MySql, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_update_sql_static(items.len());
                let query = sqlx::query::<MySql>(sql);
                #gen_fill_bulk_update
            }
            fn bulk_select(ids: &[String]) -> QueryAs<'_, MySql, Self, <MySql as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_select_sql_static(ids.len());
                let mut query = sqlx::query_as::<MySql, Self>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
        }
    }
}

#[cfg(feature = "sqlite")]
fn sqlite_impl(
    name: Ident,
    gen_scheme_code: TokenStream2,
    gen_fill_insert: TokenStream2,
    gen_fill_update: TokenStream2,
    gen_fill_id: TokenStream2,
    gen_fill_bulk_insert: TokenStream2,
    gen_fill_bulk_update: TokenStream2,
) -> TokenStream2 {
    quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql_static();
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_insert
                query
            }
            fn update_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql_static();
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&mut self) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql_static();
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_id
                query
            }
            fn by_pk<'q>() -> QueryAs<'q, Sqlite, Self, <Sqlite as HasArguments<'q>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql_static();
                sqlx::query_as::<Sqlite, Self>(sql)
            }
            fn make_query(sql: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let query = sqlx::query_as::<Sqlite, Self>(Box::leak(sql.into_boxed_str()));
                query
            }
             fn make_execute(sql: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.pre_sql_static(sql);
                let query = sqlx::query::<Sqlite>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn where_query(statement: &str) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql_static(statement);
                let query = sqlx::query_as::<Sqlite, Self>(sql);
                query
            }
            fn count_query(statement: &str) -> QueryAs<'_, Sqlite, (i64,), <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_count_sql_static(statement);
                let query = sqlx::query_as::<Sqlite, (i64,)>(sql);
                query
            }
            fn delete_where_query(statement: &str) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql_static(statement);
                let query = sqlx::query::<Sqlite>(sql);
                query
            }
            fn bulk_delete(ids: &[String]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_delete_sql_static(ids.len());
                let mut query = sqlx::query::<Sqlite>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
            }
            fn bulk_insert(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_insert_sql_static(items.len());
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_bulk_insert
            }
            fn bulk_update(items: &[Self]) -> Query<'_, Sqlite, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_update_sql_static(items.len());
                let query = sqlx::query::<Sqlite>(sql);
                #gen_fill_bulk_update
            }
            fn bulk_select(ids: &[String]) -> QueryAs<'_, Sqlite, Self, <Sqlite as HasArguments<'_>>::Arguments> where Self: Sized {
                #gen_scheme_code
                let sql = scheme.gen_bulk_select_sql_static(ids.len());
                let mut query = sqlx::query_as::<Sqlite, Self>(sql);
                for id in ids {
                    query = query.bind(id.as_str());
                }
                query
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

/// Column metadata for code generation with optional type casting
#[derive(Debug, Clone)]
struct ColumnDefinition {
    name: String,
    cast_as: Option<String>,
}

struct Schema {
    table_name: String,
    fields: Vec<Ident>,
    id_field: Ident,
    column_definitions: Vec<ColumnDefinition>,
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

        // Parse column definitions with cast_as
        let column_definitions = fields.iter()
            .map(|field| {
                let name = field.ident.as_ref().unwrap().to_string();
                let mut cast_as = None;

                // Parse #[crud(cast_as = "TYPE")]
                for attr in &field.attrs {
                    let path_str = quote::quote!(#attr).to_string();
                    // Parse #[crud(...)] attributes
                    if path_str.contains("crud") {
                        let tokens = attr.tokens.to_string();
                        if let Some(cast_pos) = tokens.find("cast_as") {
                            let remaining = &tokens[cast_pos..];
                            if let Some(eq_pos) = remaining.find('=') {
                                let value_str = &remaining[eq_pos + 1..];
                                let end_pos = value_str.find(',').unwrap_or(value_str.len());
                                let value = value_str[..end_pos]
                                    .trim()
                                    .trim_matches(|c| c == '"' || c == '\'' || c == ')' || c == ' ');
                                if !value.is_empty() {
                                    cast_as = Some(value.to_string());
                                }
                            }
                        }
                    }
                }

                ColumnDefinition { name, cast_as }
            })
            .collect();

        Self {
            table_name,
            fields: fields_name,
            id_field,
            column_definitions,
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
            match cast_as {
                Some(cast_type) => {
                    quote! {
                        ::sqlx_struct_enhanced::ColumnDefinition {
                            name: #name.to_string(),
                            cast_as: Some(#cast_type.to_string()),
                        }
                    }
                }
                None => {
                    quote! {
                        ::sqlx_struct_enhanced::ColumnDefinition {
                            name: #name.to_string(),
                            cast_as: None,
                        }
                    }
                }
            }
        });

        quote!{
            let scheme: Scheme = Scheme {
                table_name: #table_name.to_string(),
                insert_fields: vec![#(#append_insert_stmt),*],
                update_fields: vec![#(#append_update_stmt),*],
                id_field: stringify!(#id_field).to_string(),
                column_definitions: vec![#(#column_definitions),*],
            };
        }
    }

    fn fill_insert_param(&self) -> TokenStream2 {
        let bind_stmts = self.scheme.fields.iter().map(|field| {
            quote! {
                let query = query.bind(&self.#field);
            }
        });
        quote!{
            #(#bind_stmts)*
        }
    }

    fn fill_update_param(&self) -> TokenStream2 {
        let bind_stmts = self.scheme.fields[1..].iter().map(|field| {
            quote! {
                let query = query.bind(&self.#field);
            }
        });
        quote!{
            #(#bind_stmts)*
        }
    }

    fn fill_id_param(&self) -> TokenStream2 {
        let id_field = self.scheme.id_field.clone();
        quote! {
            let query = query.bind(&self.#id_field);
        }
    }

    fn fill_bulk_insert_param(&self) -> TokenStream2 {
        let fields = self.scheme.fields.clone();
        quote! {
            let mut query = query;
            for item in items {
                #(query = query.bind(&item.#fields);)*
            }
            query
        }
    }

    fn fill_bulk_update_param(&self) -> TokenStream2 {
        let id_field = self.scheme.id_field.clone();
        let update_fields = self.scheme.fields[1..].to_vec();
        quote! {
            let mut query = query;
            // Bind CASE WHEN parameters: for each item, bind id and all update fields
            for item in items {
                query = query.bind(&item.#id_field);
                #(query = query.bind(&item.#update_fields);)*
            }
            // Bind IN clause parameters: bind all ids again
            for item in items {
                query = query.bind(&item.#id_field);
            }
            query
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

