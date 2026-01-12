// Tests for simplified DECIMAL syntax
// Tests the new simplified syntax for DECIMAL fields with optional cast_as parameter

use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx::{FromRow, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;
use sqlx::Row;

// Test 1: New simplified syntax with default cast_as = "TEXT"
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct TestProduct1 {
    id: String,
    #[crud(decimal(precision = 10, scale = 2))]  // Default cast_as = "TEXT"
    price: Option<String>,
}

// Test 2: New syntax with explicit cast_as
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct TestProduct2 {
    id: String,
    #[crud(decimal(precision = 10, scale = 2, cast_as = "TEXT"))]
    price: Option<String>,
}

// Test 3: New syntax with custom cast_as
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct TestProduct3 {
    id: String,
    #[crud(decimal(precision = 10, scale = 2, cast_as = "VARCHAR"))]
    price: Option<String>,
}

// Test 4: Old two-attribute syntax (backward compatibility)
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct TestProduct4 {
    id: String,
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    price: Option<String>,
}

// Test 5: Both inline parameter and separate attribute (parameter should win)
#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct TestProduct5 {
    id: String,
    #[crud(decimal(precision = 10, scale = 2, cast_as = "VARCHAR"))]
    #[crud(cast_as = "TEXT")]  // Should be overridden
    price: Option<String>,
}

#[test]
fn test_new_syntax_with_default_cast_as() {
    let product = TestProduct1 {
        id: "1".to_string(),
        price: Some("99.99".to_string()),
    };

    // Verify DECIMAL helpers work
    assert_eq!(product.price_as_f64().unwrap(), Some(99.99));

    // Test arithmetic operations
    let mut product = TestProduct1 {
        id: "1".to_string(),
        price: Some("100.00".to_string()),
    };
    product.price_add("50.00").unwrap();
    // The helper methods preserve the value without adding trailing zeros
    assert_eq!(product.price_as_f64().unwrap(), Some(150.0));
}

#[test]
fn test_new_syntax_with_explicit_cast_as() {
    let product = TestProduct2 {
        id: "1".to_string(),
        price: Some("99.99".to_string()),
    };

    // Verify DECIMAL helpers work
    assert_eq!(product.price_as_f64().unwrap(), Some(99.99));
}

#[test]
fn test_new_syntax_with_custom_cast_as() {
    let product = TestProduct3 {
        id: "1".to_string(),
        price: Some("99.99".to_string()),
    };

    // Verify DECIMAL helpers work
    assert_eq!(product.price_as_f64().unwrap(), Some(99.99));
}

#[test]
fn test_old_two_attribute_syntax() {
    let product = TestProduct4 {
        id: "1".to_string(),
        price: Some("99.99".to_string()),
    };

    // Verify DECIMAL helpers work
    assert_eq!(product.price_as_f64().unwrap(), Some(99.99));
}

#[test]
fn test_both_parameter_and_attribute() {
    let product = TestProduct5 {
        id: "1".to_string(),
        price: Some("99.99".to_string()),
    };

    // Verify DECIMAL helpers work
    assert_eq!(product.price_as_f64().unwrap(), Some(99.99));
}

#[test]
fn test_chainable_operations_with_new_syntax() {
    let mut product = TestProduct1 {
        id: "1".to_string(),
        price: Some("100.00".to_string()),
    };

    // Test chainable arithmetic
    product.price_add_f64(50.0).unwrap();
    assert_eq!(product.price_as_f64_unwrap(), 150.0);

    product.price_mul_f64(1.1).unwrap();
    assert!((product.price_as_f64_unwrap() - 165.0).abs() < 0.01);

    product.price_round(2).unwrap();
    // Check the rounded value
    assert!((product.price_as_f64_unwrap() - 165.0).abs() < 0.01);
}

#[test]
fn test_validation_with_new_syntax() {
    let product = TestProduct1 {
        id: "1".to_string(),
        price: Some("12345678.90".to_string()),  // 8 digits + 2 decimal = 10 total (within precision)
    };

    // Should validate successfully
    assert!(product.price_validate().is_ok());
}

#[test]
fn test_formatting_with_new_syntax() {
    let product = TestProduct1 {
        id: "1".to_string(),
        price: Some("1234.56".to_string()),
    };

    // Test formatting with thousands separator
    let formatted = product.price_format().unwrap();
    assert_eq!(formatted, Some("1,234.56".to_string()));

    // Test currency formatting
    let currency = product.price_format_currency("$").unwrap();
    assert_eq!(currency, Some("$1,234.56".to_string()));
}
