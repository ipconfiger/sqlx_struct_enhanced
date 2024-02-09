pub mod traits;
pub use sqlx_struct_macros::EnhancedCrud;
pub use traits::EnhancedCrud;
use std::collections::HashMap;
use std::sync::RwLock;


#[cfg(feature = "postgres")]
fn get_db()->DbType{
    DbType::PostgreSQL
}

#[cfg(feature = "mysql")]
fn get_db()->DbType{
    DbType::MySQL
}


#[cfg(feature = "sqlite")]
fn get_db()->DbType{
    DbType::SQLite
}


fn param_trans(p: String) -> String{
    match get_db() {
        DbType::PostgreSQL=>p,
        DbType::MySQL=>"?".to_string(),
        DbType::SQLite=>"?".to_string()
    }
}

#[allow(dead_code)]
fn wrap_field(fd: String) -> String {
    match get_db() {
        DbType::PostgreSQL=>format!("\"{}\"", fd),
        DbType::MySQL=>format!("`{}`", fd),
        DbType::SQLite=>fd
    }
}

fn prepare_where(w: &str, field_count:i32) -> String {
    let param_count = w.matches("{}").count() as i32;
    let vc:Vec<String> = (0..param_count).map(|n|{format!("${:?}", n+field_count)}).collect();
    let mut where_sql = w.to_string();
    for param in vc.iter(){
        while let Some(i) = where_sql.find("{}") {
            where_sql.replace_range(i..i+2, param_trans(param.clone()).as_str());
            break;
        }
    }
    where_sql
}


#[allow(dead_code)]
pub struct Scheme {
    pub table_name: String,
    pub insert_fields: Vec<String>,
    pub update_fields: Vec<String>,
    pub id_field: String
}

struct Cache {
    map: RwLock<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            map: RwLock::new(HashMap::new()),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let map = self.map.read().unwrap();
        map.get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        let mut map = self.map.write().unwrap();
        map.insert(key, value);
    }
}

#[allow(dead_code)]
impl Scheme {
    pub fn gen_insert_sql(&self) -> String {
        let key = format!("{}-insert", self.table_name);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let params: Vec<String> = self.insert_fields.iter().enumerate().map(|(idx, _)|{
            let p = format!("${}", idx + 1);
            param_trans(p)
        }).collect();
        let params_str = params.join(",");
        let sql = format!(r#"INSERT INTO {} VALUES ({})"#, self.table_name, params_str);
        Cache::new().set(key, sql.clone());
        sql
    }
    pub fn gen_update_by_id_sql(&self) -> String {
        let key = format!("{}-update-by-id", self.table_name);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let set_seq: Vec<String> = self.update_fields.iter().enumerate().map(|(idx, fd)|{
            let p = format!("${}", idx + 1);
            let p = param_trans(p);
            format!("{}={}", fd, p)
        }).collect();
        let id_param = param_trans(format!("${}", self.insert_fields.len() as i32));
        let sql = format!(r#"UPDATE {} SET {} WHERE {}={}"#, self.table_name, set_seq.join(","), self.id_field, id_param);
        Cache::new().set(key, sql.clone());
        sql
    }
    pub fn gen_update_where_sql(&self, where_stmt: &str) -> String {
        let key = format!("{}-update-where-{}", self.table_name, where_stmt);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let set_seq: Vec<String> = self.update_fields.iter().enumerate().map(|(idx, fd)|{
            let p = format!("${}", idx + 1);
            let p = param_trans(p);
            format!("{}={}", fd, p)
        }).collect();
        let where_sql = prepare_where(where_stmt, self.insert_fields.len() as i32);
        let sql = format!(r#"UPDATE {} SET {} WHERE {}"#, self.table_name, set_seq.join(","), where_sql);
        Cache::new().set(key, sql.clone());
        sql
    }
    pub fn gen_delete_sql(&self) -> String {
        let key = format!("{}-delete-by-id", self.table_name);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let id_param = param_trans("$1".to_string());
        let sql = format!(r#"DELETE FROM {} WHERE {}={}"#, self.table_name, self.id_field, id_param);
        Cache::new().set(key, sql.clone());
        sql
    }
    pub fn gen_delete_where_sql(&self, where_stmt: &str) -> String {
        let key = format!("{}-delete-where-{}", self.table_name, where_stmt);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let where_sql = prepare_where(where_stmt, 1);
        let sql = format!(r#"DELETE FROM {} WHERE {}"#, self.table_name, where_sql);
        Cache::new().set(key, sql.clone());
        sql
    }
    pub fn gen_select_by_id_sql(&self) -> String {
        let key = format!("{}-select-by-id", self.table_name);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let id_param = param_trans("$1".to_string());
        let sql = format!(r#"SELECT * FROM {} WHERE {}={}"#, self.table_name, self.id_field, id_param);
        Cache::new().set(key, sql.clone());
        sql
    }
    pub fn gen_select_where_sql(&self, where_stmt: &str) -> String {
        let key = format!("{}-select-where-{}", self.table_name, where_stmt);
        if let Some(cached_sql) = Cache::new().get(key.as_str()){
            return cached_sql;
        }
        let where_sql = prepare_where(where_stmt, 1);
        let sql = format!(r#"SELECT * FROM {} WHERE {}"#, self.table_name, where_sql);
        Cache::new().set(key, sql.clone());
        sql
    }
}

#[allow(dead_code)]
enum DbType {
    PostgreSQL,
    MySQL,
    SQLite
}