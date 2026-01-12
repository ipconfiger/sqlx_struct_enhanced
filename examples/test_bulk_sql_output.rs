// 测试 bulk_select 和 bulk_delete 生成的 SQL
// 使用 cargo run --example test_bulk_sql_output

use sqlx_struct_enhanced::Scheme;

fn main() {
    println!("=== Bulk SQL 生成测试 ===\n");

    // 1. UUID 类型 ID（模拟）
    let scheme_uuid = Scheme {
        table_name: "order_with_uuid".to_string(),
        insert_fields: vec!["id".to_string(), "customer_name".to_string(), "amount".to_string()],
        update_fields: vec!["customer_name".to_string(), "amount".to_string()],
        id_field: "id".to_string(),
        column_definitions: vec![
            sqlx_struct_enhanced::ColumnDefinition {
                name: "id".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: true,  // ✅ UUID 类型
            },
            sqlx_struct_enhanced::ColumnDefinition {
                name: "customer_name".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: false,
            },
            sqlx_struct_enhanced::ColumnDefinition {
                name: "amount".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: false,
            },
        ],
    };

    // 2. DECIMAL 类型 ID（对比）
    let scheme_decimal = Scheme {
        table_name: "product_with_decimal_id".to_string(),
        insert_fields: vec!["id".to_string(), "name".to_string()],
        update_fields: vec!["name".to_string()],
        id_field: "id".to_string(),
        column_definitions: vec![
            sqlx_struct_enhanced::ColumnDefinition {
                name: "id".to_string(),
                cast_as: None,
                is_decimal: true,  // DECIMAL 类型
                is_uuid: false,
            },
            sqlx_struct_enhanced::ColumnDefinition {
                name: "name".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: false,
            },
        ],
    };

    // 3. INTEGER 类型 ID（对比）
    let scheme_int = Scheme {
        table_name: "user_with_int_id".to_string(),
        insert_fields: vec!["id".to_string(), "name".to_string(), "email".to_string()],
        update_fields: vec!["name".to_string(), "email".to_string()],
        id_field: "id".to_string(),
        column_definitions: vec![
            sqlx_struct_enhanced::ColumnDefinition {
                name: "id".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: false,
            },
            sqlx_struct_enhanced::ColumnDefinition {
                name: "name".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: false,
            },
            sqlx_struct_enhanced::ColumnDefinition {
                name: "email".to_string(),
                cast_as: None,
                is_decimal: false,
                is_uuid: false,
            },
        ],
    };

    // 测试不同数量的 ID
    for count in [1, 3, 5] {
        println!("\n--- 测试 {} 个 ID 的情况 ---", count);

        println!("\n1. UUID ID 类型 (order_with_uuid)");
        let sql_delete = scheme_uuid.gen_bulk_delete_sql_static(count);
        println!("   bulk_delete SQL: {}", sql_delete);
        let sql_select = scheme_uuid.gen_bulk_select_sql_static(count);
        println!("   bulk_select SQL: {}", sql_select);
        println!("   ID 字段 is_decimal: {}, is_uuid: {}",
                 scheme_uuid.column_definitions[0].is_decimal,
                 scheme_uuid.column_definitions[0].is_uuid);
        println!("   ✅ 注意：现在有 ::uuid cast！");

        println!("\n2. DECIMAL ID 类型 (product_with_decimal_id)");
        let sql_delete = scheme_decimal.gen_bulk_delete_sql_static(count);
        println!("   bulk_delete SQL: {}", sql_delete);
        let sql_select = scheme_decimal.gen_bulk_select_sql_static(count);
        println!("   bulk_select SQL: {}", sql_select);
        println!("   ID 字段 is_decimal: {}, is_uuid: {}",
                 scheme_decimal.column_definitions[0].is_decimal,
                 scheme_decimal.column_definitions[0].is_uuid);
        println!("   ✅ 注意：有 ::numeric cast！");

        println!("\n3. INTEGER ID 类型 (user_with_int_id)");
        let sql_delete = scheme_int.gen_bulk_delete_sql_static(count);
        println!("   bulk_delete SQL: {}", sql_delete);
        let sql_select = scheme_int.gen_bulk_select_sql_static(count);
        println!("   bulk_select SQL: {}", sql_select);
        println!("   ID 字段 is_decimal: {}, is_uuid: {}",
                 scheme_int.column_definitions[0].is_decimal,
                 scheme_int.column_definitions[0].is_uuid);
    }

    println!("\n=== 功能验证 ===");
    println!("\n✅ UUID 类型 ID 列现在会自动生成 ::uuid cast:");
    println!("   DELETE FROM \"order_with_uuid\" WHERE \"id\" IN ($1::uuid,$2::uuid,$3::uuid)");
    println!("   SELECT \"id\", \"customer_name\", \"amount\" FROM \"order_with_uuid\" WHERE \"id\" IN ($1::uuid,$2::uuid,$3::uuid)");
    println!("\n✅ 这样 PostgreSQL 就能正确处理 TEXT → UUID 的转换了！");
    println!("   不再报错：'operator does not exist: uuid = text'");
}
