use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[cfg(feature = "postgres")]
fn get_db_type() -> Ident{
    Ident::new("Postgres", Span::call_site())
}

#[cfg(feature = "mysql")]
fn get_db_type() -> Ident{
    Ident::new("MySql", Span::call_site())
}

#[cfg(feature = "sqlite")]
fn get_db_type() -> Ident{
    Ident::new("Sqlite", Span::call_site())
}


// 定义一个派生宏
#[proc_macro_derive(EnhancedCrud)]
pub fn print_info_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let sql_builder = SqlBuilder::new(Schema::new(&input));
    // 获取结构体名字
    let name = input.ident;
    let db_type = get_db_type();
    let gen_scheme_code = sql_builder.gen_scheme_code();
    let gen_fill_insert = sql_builder.fill_insert_param();
    let gen_fill_update = sql_builder.fill_update_param();
    let gen_fill_id = sql_builder.fill_id_param();

    let output_token = quote! {
        impl EnhancedCrud for #name {
            fn insert_bind(&self) -> Query<'_, #db_type, <#db_type as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_insert_sql();
                println!("insert sql:{}", sql.clone());
                let query = sqlx::query::<#db_type>(Box::leak(sql.into_boxed_str()));
                #gen_fill_insert
                query
            }
            fn update_bind(&self) -> Query<'_, #db_type, <#db_type as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_update_by_id_sql();
                println!("update sql:{}", sql.clone());
                let query = sqlx::query::<#db_type>(Box::leak(sql.into_boxed_str()));
                #gen_fill_update
                #gen_fill_id
                query
            }
            fn delete_bind(&self) -> Query<'_, #db_type, <#db_type as HasArguments<'_>>::Arguments> {
                #gen_scheme_code
                let sql = scheme.gen_delete_sql();
                println!("delete sql:{}", sql.clone());
                let query = sqlx::query::<#db_type>(Box::leak(sql.into_boxed_str()));
                #gen_fill_id
                query
            }
            fn select_by_id<'f, O>() -> QueryAs<'f, #db_type, O, <#db_type as HasArguments<'f>>::Arguments>
            where
                O: for<'r> FromRow<'r, <#db_type as Database>::Row> {
                #gen_scheme_code
                let sql = scheme.gen_select_by_id_sql();
                println!("select by id sql:{}", sql.clone());
                let query = sqlx::query_as::<#db_type, O>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn select_where<'f, O>(w: &str) -> QueryAs<'f, #db_type, O, <#db_type as HasArguments<'f>>::Arguments>
            where
                O: for<'r> FromRow<'r, <#db_type as Database>::Row> {
                #gen_scheme_code
                let sql = scheme.gen_select_where_sql(w);
                println!("select where sql:{}", sql.clone());
                let query = sqlx::query_as::<#db_type, O>(Box::leak(sql.into_boxed_str()));
                query
            }
            fn update_where(&self, w: &str) -> Query<'_, #db_type, <#db_type as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_update_where_sql(w);
                println!("update where sql:{}", sql.clone());
                let query = sqlx::query::<#db_type>(Box::leak(sql.into_boxed_str()));
                #gen_fill_update
                query
            }
            fn delete_where(&self, w: &str) -> Query<'_, #db_type, <#db_type as HasArguments<'_>>::Arguments>{
                #gen_scheme_code
                let sql = scheme.gen_delete_where_sql(w);
                println!("delete where sql:{}", sql.clone());
                let query = sqlx::query::<#db_type>(Box::leak(sql.into_boxed_str()));
                query
            }
        }
    };
    output_token.into()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.char_indices() {
        if i > 0 && c.is_uppercase() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

#[allow(dead_code)]
struct Schema {
    table_name: String,
    fields: Vec<Ident>,
    id_field: Ident
}

impl Schema {
    fn new(input: &DeriveInput)->Self{
        let name = to_snake_case(input.ident.to_string().as_str());
        // 获取结构体字段
        let fields = match input.data.clone() {
            syn::Data::Struct(data) => data.fields,
            _ => panic!("Only structs are supported"),
        };
        let fields_name: Vec<Ident> = fields.iter().map(|field| {
            field.ident.as_ref().unwrap().clone()
        }).collect();
        let id_filed = fields_name.clone()[0].clone();
        Self{
            table_name: name,
            fields: fields_name,
            id_field: id_filed
        }
    }
}

#[allow(dead_code)]
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
        quote!{
            let scheme: Scheme = Scheme {
                table_name: #table_name.to_string(),
                insert_fields: vec![#(#append_insert_stmt),*],
                update_fields: vec![#(#append_update_stmt),*],
                id_field: stringify!(#id_field).to_string()
            };
        }.into()
    }

    fn fill_insert_param(&self) -> TokenStream2 {
        let bind_stmts = self.scheme.fields.iter().map(|field| {
            // 获取字段名字和类型
            quote! {
                let query = query.bind(&self.#field);
            }
        });
        quote!{
            #(#bind_stmts)*
        }.into()
    }

    fn fill_update_param(&self) -> TokenStream2 {
        let bind_stmts = self.scheme.fields[1..].iter().map(|field| {
            // 获取字段名字和类型
            quote! {
                let query = query.bind(&self.#field);
            }
        });
        quote!{
            #(#bind_stmts)*
        }.into()
    }

    fn fill_id_param(&self) -> TokenStream2 {
        let id_field = self.scheme.id_field.clone();
        quote! {
            let query = query.bind(&self.#id_field);
        }.into()
    }

}


