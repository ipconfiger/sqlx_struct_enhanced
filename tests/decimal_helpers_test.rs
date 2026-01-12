// Integration tests for DECIMAL helper methods generation
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};
use sqlx_struct_enhanced::decimal_helpers::DecimalError;
use sqlx::{FromRow, Postgres, query::Query, query::QueryAs};
use sqlx::database::HasArguments;
use sqlx::Row;

#[derive(Debug, Clone, PartialEq, FromRow, EnhancedCrud)]
struct TestOrder {
    id: String,

    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    amount: Option<String>,

    #[crud(decimal(precision = 5, scale = 2))]
    #[crud(cast_as = "TEXT")]
    rate: Option<String>,
}

#[test]
fn test_type_conversion() {
    let order = TestOrder {
        id: "1".to_string(),
        amount: Some("1234.56".to_string()),
        rate: Some("8.25".to_string()),
    };

    // Test as_f64
    assert_eq!(order.amount_as_f64().unwrap(), Some(1234.56));
    assert_eq!(order.rate_as_f64().unwrap(), Some(8.25));

    // Test as_f64_or
    assert_eq!(order.amount_as_f64_or(0.0).unwrap(), 1234.56);
    assert_eq!(order.rate_as_f64_or(5.0).unwrap(), 8.25);

    // Test with None
    let order_none = TestOrder {
        id: "2".to_string(),
        amount: None,
        rate: None,
    };
    assert_eq!(order_none.amount_as_f64().unwrap(), None);
    assert_eq!(order_none.amount_as_f64_or(100.0).unwrap(), 100.0);
}

#[test]
fn test_invalid_format() {
    let order = TestOrder {
        id: "1".to_string(),
        amount: Some("invalid".to_string()),
        rate: Some("8.25".to_string()),
    };

    let result = order.amount_as_f64();
    assert!(matches!(result, Err(DecimalError::InvalidFormat(_))));
}

#[test]
fn test_chainable_arithmetic() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("100.00".to_string()),
        rate: Some("0.5".to_string()),
    };

    // Test string-based arithmetic
    order.amount_add("50.00").unwrap()
         .amount_mul("1.1").unwrap()
         .amount_round(2).unwrap();

    assert_eq!(order.amount, Some("165".to_string())); // (100+50)*1.1

    // Test f64-based arithmetic
    order.rate_add_f64(0.1).unwrap()
         .rate_mul_f64(2.0).unwrap()
         .rate_round(2).unwrap();

    assert_eq!(order.rate, Some("1.2".to_string())); // (0.5+0.1)*2.0
}

#[test]
fn test_arithmetic_on_none() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: None,
        rate: Some("0.5".to_string()),
    };

    let result = order.amount_add("100.00");
    assert!(matches!(result, Err(DecimalError::NullValue)));
}

#[test]
fn test_division_by_zero() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("100.00".to_string()),
        rate: Some("0.5".to_string()),
    };

    // String division by zero
    assert!(matches!(
        order.amount_div("0"),
        Err(DecimalError::DivisionByZero)
    ));

    assert!(matches!(
        order.amount_div("0.0"),
        Err(DecimalError::DivisionByZero)
    ));

    // f64 division by zero
    assert!(matches!(
        order.amount_div_f64(0.0),
        Err(DecimalError::DivisionByZero)
    ));
}

#[test]
fn test_round_and_abs() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("123.456".to_string()),
        rate: Some("-8.25".to_string()),
    };

    // Test round
    order.amount_round(2).unwrap();
    assert_eq!(order.amount, Some("123.46".to_string()));

    // Test abs
    order.rate_abs().unwrap();
    assert_eq!(order.rate, Some("8.25".to_string()));

    // Test neg
    order.amount_neg().unwrap();
    assert_eq!(order.amount, Some("-123.46".to_string()));
}

#[test]
fn test_validation() {
    // Valid value for NUMERIC(10,2)
    let order_valid = TestOrder {
        id: "1".to_string(),
        amount: Some("99999999.99".to_string()),
        rate: Some("8.25".to_string()),
    };

    assert!(order_valid.amount_validate().is_ok());

    // Invalid value - exceeds precision (too many integer digits)
    let order_invalid = TestOrder {
        id: "1".to_string(),
        amount: Some("999999999.99".to_string()), // 9 integer digits > 10-2=8
        rate: Some("8.25".to_string()),
    };

    assert!(matches!(
        order_invalid.amount_validate(),
        Err(DecimalError::Overflow { .. })
    ));
}

#[test]
fn test_value_checks() {
    let order = TestOrder {
        id: "1".to_string(),
        amount: Some("100.00".to_string()),
        rate: Some("-50.00".to_string()),
    };

    // Test is_positive
    assert_eq!(order.amount_is_positive(), Some(true));
    assert_eq!(order.rate_is_positive(), Some(false));

    // Test is_negative
    assert_eq!(order.amount_is_negative(), Some(false));
    assert_eq!(order.rate_is_negative(), Some(true));

    // Test is_zero
    assert_eq!(order.amount_is_zero(), Some(false));

    // Test with None
    let order_none = TestOrder {
        id: "2".to_string(),
        amount: None,
        rate: None,
    };
    assert_eq!(order_none.amount_is_positive(), None);
    assert_eq!(order_none.rate_is_negative(), None);
}

#[test]
fn test_formatting() {
    let order = TestOrder {
        id: "1".to_string(),
        amount: Some("1234.56".to_string()),
        rate: Some("0.0825".to_string()),
    };

    // Test format
    assert_eq!(order.amount_format().unwrap(), Some("1,234.56".to_string()));

    // Test format_currency
    assert_eq!(
        order.amount_format_currency("$").unwrap(),
        Some("$1,234.56".to_string())
    );

    // Test format_percent
    assert_eq!(
        order.rate_format_percent().unwrap(),
        Some("8.25%".to_string())
    );

    // Test with None
    let order_none = TestOrder {
        id: "2".to_string(),
        amount: None,
        rate: None,
    };
    assert_eq!(order_none.amount_format().unwrap(), None);
}

#[test]
fn test_truncate() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("123.456".to_string()),
        rate: Some("0.0825".to_string()),
    };

    // Truncate to 2 decimal places (should be 123.45, not 123.46)
    order.amount_truncate(2).unwrap();
    assert_eq!(order.amount, Some("123.45".to_string()));

    // Truncate to 3 decimal places (should be 0.082)
    order.rate_truncate(3).unwrap();
    assert_eq!(order.rate, Some("0.082".to_string()));
}

#[test]
fn test_precision_methods() {
    let order = TestOrder {
        id: "1".to_string(),
        amount: Some("100.00".to_string()),
        rate: Some("8.25".to_string()),
    };

    // Test precision
    assert_eq!(order.amount_precision(), 10);
    assert_eq!(order.rate_precision(), 5);

    // Test scale
    assert_eq!(order.amount_scale(), 2);
    assert_eq!(order.rate_scale(), 2);

    // Test max_value
    let max = order.amount_max_value().unwrap();
    assert_eq!(max, "99999999.99");

    // Test min_value
    let min = order.amount_min_value().unwrap();
    assert_eq!(min, "-99999999.99");
}

#[test]
fn test_clamp() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("100000000.00".to_string()), // Exceeds NUMERIC(10,2)
        rate: Some("8.25".to_string()),
    };

    // Clamp to max value
    order.amount_clamp().unwrap();
    assert_eq!(order.amount, Some("99999999.99".to_string()));
}

#[test]
fn test_multiple_operations_chained() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("100.00".to_string()),
        rate: Some("0.08".to_string()),
    };

    // Complex chain: add 50, multiply by 1.1, round to 2 decimal places
    order.amount_add("50.00")
         .unwrap()
         .amount_mul("1.1")
         .unwrap()
         .amount_round(2)
         .unwrap();

    assert_eq!(order.amount, Some("165".to_string()));

    // Another chain: calculate with percentage
    order.rate_add_f64(0.02)
         .unwrap()
         .rate_mul_f64(100.0)
         .unwrap()
         .rate_round(2)
         .unwrap();

    assert_eq!(order.rate, Some("10".to_string())); // (0.08+0.02)*100
}

#[test]
fn test_unwrap_methods() {
    let order = TestOrder {
        id: "1".to_string(),
        amount: Some("123.45".to_string()),
        rate: Some("5.0".to_string()),
    };

    // Test unwrap
    assert_eq!(order.amount_as_f64_unwrap(), 123.45);
    assert_eq!(order.rate_as_f64_unwrap(), 5.0);
}

#[test]
fn test_special_decimal_values() {
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("0.00".to_string()),
        rate: Some("-0.01".to_string()),
    };

    // Test zero value
    assert_eq!(order.amount_is_zero(), Some(true));

    // Test negative small value
    assert_eq!(order.rate_is_negative(), Some(true));

    // Test rounding edge cases
    order.amount = Some("0.005".to_string());
    order.amount_round(2).unwrap();
    // 0.005 * 100 = 0.5, round(0.5) = 1, /100 = 0.01
    assert_eq!(order.amount, Some("0.01".to_string()));
}

#[test]
fn test_precision_scale_validation() {
    // NUMERIC(10,2) means: max 8 integer digits, 2 fractional digits
    let mut order = TestOrder {
        id: "1".to_string(),
        amount: Some("99999999.99".to_string()), // 8 integer digits + 2 fractional = OK
        rate: Some("999.99".to_string()),        // 3 integer digits + 2 fractional = OK for NUMERIC(5,2)
    };

    // Both should be valid
    assert!(order.amount_validate().is_ok());
    assert!(order.rate_validate().is_ok());

    // Test overflow for amount (NUMERIC 10,2)
    order.amount = Some("999999999.99".to_string()); // 9 integer digits > 8
    assert!(matches!(
        order.amount_validate(),
        Err(DecimalError::Overflow { precision: 10, scale: 2, .. })
    ));

    // Test overflow for rate (NUMERIC 5,2)
    order.rate = Some("9999.99".to_string()); // 4 integer digits > 3
    assert!(matches!(
        order.rate_validate(),
        Err(DecimalError::Overflow { precision: 5, scale: 2, .. })
    ));
}
