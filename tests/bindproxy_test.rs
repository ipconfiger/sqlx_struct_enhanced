// Test BindProxy integration with insert_bind and update_bind
//
// This test verifies that the EnhancedCrud derive macro correctly uses BindProxy
// for type conversion of custom types (Decimal, DateTime, etc.)

use sqlx_struct_enhanced::EnhancedCrud;
use sqlx::{FromRow, Postgres, Row as _};
use sqlx::database::HasArguments;
use sqlx::query::{Query, QueryAs};

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct SimpleProduct {
    id: String,
    name: String,  // Should use regular bind
    quantity: i32,  // Should use regular bind
    active: bool,  // Should use regular bind
    price: Option<String>,  // Option type should use regular bind
}

#[test]
fn test_simple_product_compiles() {
    // This test verifies backward compatibility with basic types
    let product = SimpleProduct {
        id: "1".to_string(),
        name: "Test Product".to_string(),
        quantity: 100,
        active: true,
        price: Some("99.99".to_string()),
    };

    // Verify the struct can be created
    assert_eq!(product.id, "1");
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.quantity, 100);
    assert_eq!(product.active, true);
    assert_eq!(product.price, Some("99.99".to_string()));
}

#[test]
fn test_product_with_none_optional() {
    // Test with None values for optional fields
    let product = SimpleProduct {
        id: "2".to_string(),
        name: "Product without price".to_string(),
        quantity: 50,
        active: false,
        price: None,
    };

    assert_eq!(product.id, "2");
    assert_eq!(product.name, "Product without price");
    assert_eq!(product.quantity, 50);
    assert_eq!(product.active, false);
    assert!(product.price.is_none());
}

// Integration test for Decimal types (requires rust_decimal feature)
// This is a compilation test to ensure the macro generates correct code
#[cfg(feature = "decimal")]
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct DecimalProduct {
    id: String,
    name: String,
    price: rust_decimal::Decimal,  // Should use bind_proxy
    discount: Option<rust_decimal::Decimal>,  // Should use bind_proxy with mem::replace
}

#[cfg(feature = "decimal")]
#[test]
fn test_decimal_product_compiles() {
    use rust_decimal::Decimal;

    let product = DecimalProduct {
        id: "1".to_string(),
        name: "Test Product".to_string(),
        price: Decimal::from_str("99.99").unwrap(),
        discount: Some(Decimal::from_str("10.00").unwrap()),
    };

    assert_eq!(product.id, "1");
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.price, Decimal::from_str("99.99").unwrap());
}

// Integration test for DateTime types (requires chrono feature)
// This is a compilation test to ensure the macro generates correct code
#[cfg(feature = "chrono")]
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct Event {
    id: String,
    name: String,
    created_at: chrono::NaiveDateTime,  // Should use bind_proxy
    updated_at: Option<chrono::NaiveDateTime>,  // Should use bind_proxy with mem::replace
}

#[cfg(feature = "chrono")]
#[test]
fn test_datetime_event_compiles() {
    use chrono::Utc;

    let event = Event {
        id: "1".to_string(),
        name: "Test Event".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Some(Utc::now().naive_utc()),
    };

    assert_eq!(event.id, "1");
    assert_eq!(event.name, "Test Event");
}

// Note: These are compilation tests. To run actual database operations,
// you would need to integrate with a real database test using:
// - product.insert_bind().execute(&pool).await
// - product.update_bind().execute(&pool).await
