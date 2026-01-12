// Test reserved keyword column names
use sqlx_struct_enhanced::Scheme;

fn main() {
    println!("Testing SQL reserved keyword support...\n");

    #[cfg(feature = "postgres")]
    println!("Testing with PostgreSQL quoting (double quotes)\n");

    #[cfg(feature = "mysql")]
    println!("Testing with MySQL quoting (backticks)\n");

    #[cfg(feature = "sqlite")]
    println!("Testing with SQLite (no quotes)\n");

    test_reserved_keywords_in_select();
    println!();
    test_reserved_keywords_in_insert();
    println!();
    test_reserved_keywords_in_update();
    println!();
    test_reserved_keywords_in_delete();
    println!();
    test_multiple_reserved_keywords();
    println!();
    test_bulk_operations_with_reserved_keywords();
    println!();

    println!("🎉 All reserved keyword tests passed!");
    println!("\n✨ SQL reserved keywords are now fully supported!");
}

fn test_reserved_keywords_in_select() {
    let scheme = Scheme {
        table_name: "notifications".to_string(),
        insert_fields: vec![
            "id".to_string(),
            "type".to_string(),
            "order".to_string(),
            "user_id".to_string(),
            "read".to_string(),
            "created_at".to_string(),
        ],
        update_fields: vec![
            "type".to_string(),
            "order".to_string(),
            "read".to_string(),
        ],
        id_field: "id".to_string(),
        column_definitions: vec![],
    };

    // Test SELECT by ID
    let sql = scheme.gen_select_by_id_sql_static();

    #[cfg(feature = "postgres")]
    assert_eq!(
        sql,
        r#"SELECT * FROM "notifications" WHERE "id"=$1"#
    );

    #[cfg(feature = "mysql")]
    assert_eq!(
        sql,
        "SELECT * FROM `notifications` WHERE `id`=?"
    );

    #[cfg(feature = "sqlite")]
    assert_eq!(
        sql,
        "SELECT * FROM notifications WHERE id=?"
    );

    println!("✅ Reserved keyword SELECT test passed!");
    println!("   Generated SQL: {}", sql);
}

fn test_reserved_keywords_in_insert() {
    let scheme = Scheme {
        table_name: "notifications".to_string(),
        insert_fields: vec![
            "id".to_string(),
            "type".to_string(),
            "order".to_string(),
        ],
        update_fields: vec![],
        id_field: "id".to_string(),
        column_definitions: vec![],
    };

    let sql = scheme.gen_insert_sql_static();

    #[cfg(feature = "postgres")]
    assert_eq!(
        sql,
        r#"INSERT INTO "notifications" VALUES ($1,$2,$3)"#
    );

    #[cfg(feature = "mysql")]
    assert_eq!(
        sql,
        "INSERT INTO `notifications` VALUES (?,?,?)"
    );

    #[cfg(feature = "sqlite")]
    assert_eq!(
        sql,
        "INSERT INTO notifications VALUES (?,?,?)"
    );

    println!("✅ Reserved keyword INSERT test passed!");
    println!("   Generated SQL: {}", sql);
}

fn test_reserved_keywords_in_update() {
    let scheme = Scheme {
        table_name: "notifications".to_string(),
        insert_fields: vec![
            "id".to_string(),
            "type".to_string(),
        ],
        update_fields: vec![
            "type".to_string(),
            "read".to_string(),
        ],
        id_field: "id".to_string(),
        column_definitions: vec![],
    };

    let sql = scheme.gen_update_by_id_sql_static();

    #[cfg(feature = "postgres")]
    assert_eq!(
        sql,
        r#"UPDATE "notifications" SET "type"=$1,"read"=$2 WHERE "id"=$3"#
    );

    #[cfg(feature = "mysql")]
    assert_eq!(
        sql,
        "UPDATE `notifications` SET `type`=?,`read`=? WHERE `id`=?"
    );

    #[cfg(feature = "sqlite")]
    assert_eq!(
        sql,
        "UPDATE notifications SET type=?,read=? WHERE id=?"
    );

    println!("✅ Reserved keyword UPDATE test passed!");
    println!("   Generated SQL: {}", sql);
}

fn test_reserved_keywords_in_delete() {
    let scheme = Scheme {
        table_name: "notifications".to_string(),
        insert_fields: vec!["id".to_string()],
        update_fields: vec![],
        id_field: "id".to_string(),
        column_definitions: vec![],
    };

    let sql = scheme.gen_delete_sql_static();

    #[cfg(feature = "postgres")]
    assert_eq!(
        sql,
        r#"DELETE FROM "notifications" WHERE "id"=$1"#
    );

    #[cfg(feature = "mysql")]
    assert_eq!(
        sql,
        "DELETE FROM `notifications` WHERE `id`=?"
    );

    #[cfg(feature = "sqlite")]
    assert_eq!(
        sql,
        "DELETE FROM notifications WHERE id=?"
    );

    println!("✅ Reserved keyword DELETE test passed!");
    println!("   Generated SQL: {}", sql);
}

fn test_multiple_reserved_keywords() {
    let scheme = Scheme {
        table_name: "test_table".to_string(),
        insert_fields: vec![
            "id".to_string(),
            "select".to_string(),
            "from".to_string(),
            "where".to_string(),
            "order".to_string(),
            "group".to_string(),
        ],
        update_fields: vec![
            "select".to_string(),
            "order".to_string(),
        ],
        id_field: "id".to_string(),
        column_definitions: vec![],
    };

    let sql = scheme.gen_update_by_id_sql_static();

    #[cfg(feature = "postgres")]
    {
        let expected = r#"UPDATE "test_table" SET "select"=$1,"order"=$2 WHERE "id"=$3"#;
        assert_eq!(sql, expected);
        println!("✅ Multiple reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "mysql")]
    {
        let expected = "UPDATE `test_table` SET `select`=?,`order`=? WHERE `id`=?";
        assert_eq!(sql, expected);
        println!("✅ Multiple reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "sqlite")]
    {
        let expected = "UPDATE test_table SET select=?,order=? WHERE id=?";
        assert_eq!(sql, expected);
        println!("✅ Multiple reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }
}

fn test_bulk_operations_with_reserved_keywords() {
    let scheme = Scheme {
        table_name: "notifications".to_string(),
        insert_fields: vec![
            "id".to_string(),
            "type".to_string(),
            "order".to_string(),
            "read".to_string(),
        ],
        update_fields: vec![
            "type".to_string(),
            "read".to_string(),
        ],
        id_field: "id".to_string(),
        column_definitions: vec![],
    };

    // Test bulk SELECT
    let sql = scheme.gen_bulk_select_sql_static(3);

    #[cfg(feature = "postgres")]
    {
        assert_eq!(
            sql,
            r#"SELECT * FROM "notifications" WHERE "id" IN ($1,$2,$3)"#
        );
        println!("✅ Bulk SELECT with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "mysql")]
    {
        assert_eq!(
            sql,
            "SELECT * FROM `notifications` WHERE `id` IN (?,?,?)"
        );
        println!("✅ Bulk SELECT with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "sqlite")]
    {
        assert_eq!(
            sql,
            "SELECT * FROM notifications WHERE id IN (?,?,?)"
        );
        println!("✅ Bulk SELECT with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    // Test bulk UPDATE
    let sql = scheme.gen_bulk_update_sql_static(2);

    #[cfg(feature = "postgres")]
    {
        assert!(sql.contains(r#"UPDATE "notifications""#));
        assert!(sql.contains(r#"SET "type"=CASE "id" WHEN"#));
        assert!(sql.contains(r#""read"=CASE "id" WHEN"#));
        assert!(sql.contains(r#"WHERE "id" IN ($"#));
        println!("✅ Bulk UPDATE with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "mysql")]
    {
        assert!(sql.contains("UPDATE `notifications`"));
        assert!(sql.contains("SET `type`=CASE `id` WHEN"));
        assert!(sql.contains("`read`=CASE `id` WHEN"));
        assert!(sql.contains("WHERE `id` IN (?"));
        println!("✅ Bulk UPDATE with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "sqlite")]
    {
        assert!(sql.contains("UPDATE notifications"));
        assert!(sql.contains("SET type=CASE WHEN"));
        assert!(sql.contains("read=CASE WHEN"));
        assert!(sql.contains("WHERE id IN (?"));
        println!("✅ Bulk UPDATE with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    // Test bulk DELETE
    let sql = scheme.gen_bulk_delete_sql_static(2);

    #[cfg(feature = "postgres")]
    {
        assert_eq!(
            sql,
            r#"DELETE FROM "notifications" WHERE "id" IN ($1,$2)"#
        );
        println!("✅ Bulk DELETE with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "mysql")]
    {
        assert_eq!(
            sql,
            "DELETE FROM `notifications` WHERE `id` IN (?,?)"
        );
        println!("✅ Bulk DELETE with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }

    #[cfg(feature = "sqlite")]
    {
        assert_eq!(
            sql,
            "DELETE FROM notifications WHERE id IN (?,?)"
        );
        println!("✅ Bulk DELETE with reserved keywords test passed!");
        println!("   Generated SQL: {}", sql);
    }
}
