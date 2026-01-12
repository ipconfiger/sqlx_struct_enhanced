// Integration tests for JSON type handling in bulk operations
//
// This test file verifies that JSON fields are properly converted
// during bulk_insert and bulk_update operations using the BindProxy trait.
//
// Run with:
//   cargo test --test bulk_json_test --features json -- --ignored
//
// Requires PostgreSQL running at postgres://postgres:@127.0.0.1/test-sqlx-tokio

use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use sqlx::Postgres;
use sqlx::Row;
use sqlx::database::HasArguments;
use sqlx::query::{Query, QueryAs};
use sqlx_struct_enhanced::EnhancedCrud;
use serial_test::serial;

#[cfg(feature = "json")]
#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct JsonDocument {
    id: String,
    title: String,
    metadata: serde_json::Value,  // JSON field
    tags: Option<serde_json::Value>,  // Optional JSON field
}

#[tokio::test]
#[cfg(feature = "json")]
#[serial]
#[ignore = "Requires PostgreSQL database"]
async fn test_bulk_insert_with_json() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    // Create test table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_documents (
            id VARCHAR(50) PRIMARY KEY,
            title VARCHAR(200) NOT NULL,
            metadata JSONB NOT NULL,
            tags JSONB
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up before test
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-bulk-%'")
        .execute(&pool)
        .await?;

    println!("=== Test: Bulk Insert with JSON fields ===");

    // Prepare test data with JSON fields
    let items = vec![
        JsonDocument {
            id: "json-bulk-1".to_string(),
            title: "Document 1".to_string(),
            metadata: json!({
                "author": "Alice",
                "views": 100,
                "published": true,
                "tags": ["rust", "database"]
            }),
            tags: Some(json!(["tech", "programming"])),
        },
        JsonDocument {
            id: "json-bulk-2".to_string(),
            title: "Document 2".to_string(),
            metadata: json!({
                "author": "Bob",
                "views": 250,
                "published": false,
                "category": "tutorial"
            }),
            tags: Some(json!(["database", "sql"])),
        },
        JsonDocument {
            id: "json-bulk-3".to_string(),
            title: "Document 3".to_string(),
            metadata: json!({
                "author": "Charlie",
                "views": 500,
                "published": true,
                "featured": true
            }),
            tags: None,  // Test NULL JSON field
        },
    ];

    println!("Inserting {} documents with JSON fields...", items.len());

    // Execute bulk insert
    JsonDocument::bulk_insert(&items).execute(&pool).await?;

    println!("✓ Bulk insert successful");

    // Verify the inserts
    let ids = vec![
        "json-bulk-1".to_string(),
        "json-bulk-2".to_string(),
        "json-bulk-3".to_string(),
    ];

    let results = JsonDocument::bulk_select(&ids).fetch_all(&pool).await?;

    println!("Retrieved {} documents from database", results.len());
    assert_eq!(results.len(), 3, "Should insert all 3 documents");

    // Verify JSON content
    for result in &results {
        println!("Document: {} - Metadata: {}",
            result.id,
            result.metadata.to_string()
        );

        // Check that metadata is a valid JSON object
        assert!(result.metadata.is_object(), "metadata should be a JSON object");

        // Check that views field exists and is a number
        if let Some(views) = result.metadata.get("views") {
            assert!(views.is_i64() || views.is_u64(), "views should be a number");
        }
    }

    // Verify specific document content
    let doc1 = results.iter().find(|d| d.id == "json-bulk-1").expect("Doc 1 should exist");
    assert_eq!(doc1.title, "Document 1");
    assert_eq!(doc1.metadata["author"], "Alice");
    assert_eq!(doc1.metadata["views"], 100);
    assert!(doc1.tags.is_some());
    assert_eq!(doc1.tags.as_ref().unwrap()[0], "tech");

    let doc3 = results.iter().find(|d| d.id == "json-bulk-3").expect("Doc 3 should exist");
    assert_eq!(doc3.title, "Document 3");
    assert!(doc3.tags.is_none(), "Doc 3 should have NULL tags");

    println!("✓ JSON data verified successfully");

    // Clean up
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-bulk-%'")
        .execute(&pool)
        .await?;

    Ok(())
}

#[tokio::test]
#[cfg(feature = "json")]
#[serial]
#[ignore = "Requires PostgreSQL database"]
async fn test_bulk_insert_with_complex_json() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    // Create test table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_documents (
            id VARCHAR(50) PRIMARY KEY,
            title VARCHAR(200) NOT NULL,
            metadata JSONB NOT NULL,
            tags JSONB
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up before test
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-complex-%'")
        .execute(&pool)
        .await?;

    println!("=== Test: Bulk Insert with complex JSON structures ===");

    let items = vec![
        JsonDocument {
            id: "json-complex-1".to_string(),
            title: "Complex Document 1".to_string(),
            metadata: json!({
                "nested": {
                    "level1": {
                        "level2": {
                            "value": "deep",
                            "count": 42
                        }
                    }
                },
                "array": [1, 2, 3, 4, 5],
                "mixed": ["string", 123, true, null],
                "empty_obj": {},
                "empty_arr": []
            }),
            tags: Some(json!(["nested", "array", "mixed"])),
        },
        JsonDocument {
            id: "json-complex-2".to_string(),
            title: "Complex Document 2".to_string(),
            metadata: json!({
                "unicode": "Hello 世界 🌍",
                "special_chars": "Line1\nLine2\tTabbed",
                "numbers": {
                    "decimal": 3.14159,
                    "negative": -42,
                    "scientific": 1.23e-4
                }
            }),
            tags: Some(json!([])),  // Empty array
        },
    ];

    println!("Inserting {} documents with complex JSON...", items.len());

    JsonDocument::bulk_insert(&items).execute(&pool).await?;

    println!("✓ Bulk insert successful");

    // Verify
    let ids = vec![
        "json-complex-1".to_string(),
        "json-complex-2".to_string(),
    ];

    let results = JsonDocument::bulk_select(&ids).fetch_all(&pool).await?;
    assert_eq!(results.len(), 2);

    // Verify complex structures
    let doc1 = results.iter().find(|d| d.id == "json-complex-1").expect("Doc 1 should exist");
    assert_eq!(doc1.metadata["nested"]["level1"]["level2"]["value"], "deep");
    assert_eq!(doc1.metadata["nested"]["level1"]["level2"]["count"], 42);
    assert_eq!(doc1.metadata["array"][2], 3);
    assert_eq!(doc1.metadata["mixed"][1], 123);

    let doc2 = results.iter().find(|d| d.id == "json-complex-2").expect("Doc 2 should exist");
    assert_eq!(doc2.metadata["unicode"], "Hello 世界 🌍");
    assert_eq!(doc2.metadata["numbers"]["decimal"], 3.14159);
    assert_eq!(doc2.tags.as_ref().unwrap().as_array().unwrap().len(), 0);

    println!("✓ Complex JSON structures verified");

    // Clean up
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-complex-%'")
        .execute(&pool)
        .await?;

    Ok(())
}

#[tokio::test]
#[cfg(feature = "json")]
#[serial]
#[ignore = "Requires PostgreSQL database"]
async fn test_bulk_insert_mixed_types() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    // Create test table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_documents (
            id VARCHAR(50) PRIMARY KEY,
            title VARCHAR(200) NOT NULL,
            metadata JSONB NOT NULL,
            tags JSONB
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up before test
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-mixed-%'")
        .execute(&pool)
        .await?;

    println!("=== Test: Bulk Insert with mixed types (JSON + regular fields) ===");

    let items = vec![
        JsonDocument {
            id: "json-mixed-1".to_string(),
            title: "Regular String Field".to_string(),
            metadata: json!({"count": 10}),
            tags: Some(json!(["tag1", "tag2"])),
        },
        JsonDocument {
            id: "json-mixed-2".to_string(),
            title: "Another String".to_string(),
            metadata: json!({"active": true}),
            tags: None,
        },
    ];

    println!("Inserting {} documents with mixed field types...", items.len());

    JsonDocument::bulk_insert(&items).execute(&pool).await?;

    println!("✓ Bulk insert successful");

    // Verify
    let ids = vec![
        "json-mixed-1".to_string(),
        "json-mixed-2".to_string(),
    ];

    let results = JsonDocument::bulk_select(&ids).fetch_all(&pool).await?;
    assert_eq!(results.len(), 2);

    let doc1 = results.iter().find(|d| d.id == "json-mixed-1").expect("Doc 1 should exist");
    assert_eq!(doc1.title, "Regular String Field");
    assert_eq!(doc1.metadata["count"], 10);

    let doc2 = results.iter().find(|d| d.id == "json-mixed-2").expect("Doc 2 should exist");
    assert_eq!(doc2.title, "Another String");
    assert_eq!(doc2.metadata["active"], true);

    println!("✓ Mixed types verified");

    // Clean up
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-mixed-%'")
        .execute(&pool)
        .await?;

    Ok(())
}

#[tokio::test]
#[cfg(feature = "json")]
#[serial]
#[ignore = "Requires PostgreSQL database"]
async fn test_bulk_insert_empty_list() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    // Create test table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_documents (
            id VARCHAR(50) PRIMARY KEY,
            title VARCHAR(200) NOT NULL,
            metadata JSONB NOT NULL,
            tags JSONB
        )
        "#
    )
    .execute(&pool)
    .await?;

    println!("=== Test: Bulk Insert with empty list ===");

    let items: Vec<JsonDocument> = vec![];

    // Should not error, just insert nothing
    JsonDocument::bulk_insert(&items).execute(&pool).await?;

    println!("✓ Empty list handled correctly");

    Ok(())
}

#[tokio::test]
#[cfg(feature = "json")]
#[serial]
#[ignore = "Requires PostgreSQL database"]
async fn test_json_serialization_format() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:@127.0.0.1/test-sqlx-tokio")
        .await?;

    // Create test table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS json_documents (
            id VARCHAR(50) PRIMARY KEY,
            title VARCHAR(200) NOT NULL,
            metadata JSONB NOT NULL,
            tags JSONB
        )
        "#
    )
    .execute(&pool)
    .await?;

    // Clean up before test
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-format-%'")
        .execute(&pool)
        .await?;

    println!("=== Test: JSON serialization format verification ===");

    let items = vec![
        JsonDocument {
            id: "json-format-1".to_string(),
            title: "Format Test".to_string(),
            metadata: json!({
                "string": "value",
                "number": 42,
                "float": 3.14,
                "bool": true,
                "null": null
            }),
            tags: Some(json!(["a", "b", "c"])),
        },
    ];

    JsonDocument::bulk_insert(&items).execute(&pool).await?;

    // Query the database directly to verify the stored JSON format
    let row: (String, String) = sqlx::query_as(
        "SELECT id, metadata::text FROM json_documents WHERE id = 'json-format-1'"
    )
    .bind("json-format-1")
    .fetch_one(&pool)
    .await?;

    let stored_json = row.1;
    println!("Stored JSON in database: {}", stored_json);

    // Verify JSON is properly serialized (no extra quotes, valid JSON format)
    assert!(stored_json.starts_with('{'), "JSON should start with {{");
    assert!(stored_json.ends_with('}'), "JSON should end with }}");
    assert!(stored_json.contains("\"string\":\"value\""), "String field should be properly quoted");
    assert!(stored_json.contains("\"number\":42"), "Number field should not be quoted");
    assert!(stored_json.contains("\"float\":3.14"), "Float should be preserved");

    println!("✓ JSON serialization format is correct");

    // Clean up
    sqlx::query("DELETE FROM json_documents WHERE id LIKE 'json-format-%'")
        .execute(&pool)
        .await?;

    Ok(())
}
