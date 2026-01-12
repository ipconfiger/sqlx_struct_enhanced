//! DECIMAL helper methods for enhancedCrud models.
//!
//! This module provides automatic helper method generation for fields annotated
//! with `#[crud(decimal(precision = N, scale = M))]`. The helper methods include
//! type conversion, arithmetic operations, validation, formatting, and precision control.
//!
//! # Example
//!
//! ```ignore
//! #[derive(EnhancedCrud)]
//! struct Order {
//!     id: String,
//!
//!     #[crud(decimal(precision = 10, scale = 2))]
//!     #[crud(cast_as = "TEXT")]
//!     total_amount: Option<String>,
//! }
//!
//! // Use the generated helper methods
//! let mut order = Order { /* ... */ };
//!
//! // Type conversion
//! let amount = order.total_amount_as_f64()?;
//!
//! // Chainable arithmetic
//! order.total_amount_add("100.00")?
//!      .total_amount_mul("1.1")?;
//!
//! // Validation
//! order.total_amount_validate()?;
//!
//! // Formatting
//! let formatted = order.total_amount_format_currency("$")?;
//! ```

use std::fmt;

/// Format a number with thousands separator.
///
/// # Example
///
/// ```
/// assert_eq!(format_with_thousands_separator(1234.56, 2), "1,234.56");
/// assert_eq!(format_with_thousands_separator(1234567.89, 2), "1,234,567.89");
/// ```
pub fn format_with_thousands_separator(value: f64, decimal_places: i32) -> String {
    let abs_value = value.abs();

    // Round to specified decimal places
    let multiplier = 10_f64.powi(decimal_places);
    let rounded = (abs_value * multiplier).round() / multiplier;

    // Split into integer and fractional parts
    let int_part = rounded.floor() as i64;
    let frac_part = ((rounded.fract() * multiplier).round() as i64).abs();

    // Format integer part with thousands separator
    let int_str = int_part.to_string();
    let mut formatted_int = String::new();
    for (i, c) in int_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            formatted_int.push(',');
        }
        formatted_int.push(c);
    }
    let formatted_int = formatted_int.chars().rev().collect();

    // Add fractional part
    if decimal_places > 0 {
        let prefix = if value < 0.0 { "-" } else { "" };
        format!("{}{}.{:0width$}", prefix, formatted_int, frac_part, width = decimal_places as usize)
    } else {
        if value < 0.0 {
            format!("-{}", formatted_int)
        } else {
            formatted_int
        }
    }
}

/// Custom error type for decimal operations.
#[derive(Debug, Clone, PartialEq)]
pub enum DecimalError {
    /// Invalid decimal string format (e.g., "abc", "12.34.56")
    InvalidFormat(String),

    /// Value exceeds the precision/scale constraints defined in #[crud(decimal(...))]
    Overflow {
        /// The value that caused the overflow
        value: String,
        /// Total number of digits (integer + fractional)
        precision: u8,
        /// Number of digits after decimal point
        scale: u8,
    },

    /// Division by zero attempted
    DivisionByZero,

    /// Operation attempted on a NULL/None field
    NullValue,
}

impl fmt::Display for DecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecimalError::InvalidFormat(s) => {
                write!(f, "Invalid decimal format: '{}'", s)
            }
            DecimalError::Overflow {
                value,
                precision,
                scale,
            } => {
                write!(
                    f,
                    "Value '{}' exceeds NUMERIC({}, {}) constraints",
                    value, precision, scale
                )
            }
            DecimalError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            DecimalError::NullValue => {
                write!(f, "Attempted operation on NULL field")
            }
        }
    }
}

impl std::error::Error for DecimalError {}

/// Result type for decimal operations.
pub type DecimalResult<T> = Result<T, DecimalError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            format!("{}", DecimalError::InvalidFormat("abc".to_string())),
            "Invalid decimal format: 'abc'"
        );

        assert_eq!(
            format!(
                "{}",
                DecimalError::Overflow {
                    value: "123.456".to_string(),
                    precision: 5,
                    scale: 2
                }
            ),
            "Value '123.456' exceeds NUMERIC(5, 2) constraints"
        );

        assert_eq!(
            format!("{}", DecimalError::DivisionByZero),
            "Division by zero"
        );

        assert_eq!(
            format!("{}", DecimalError::NullValue),
            "Attempted operation on NULL field"
        );
    }

    #[test]
    fn test_error_equality() {
        let err1 = DecimalError::InvalidFormat("test".to_string());
        let err2 = DecimalError::InvalidFormat("test".to_string());
        assert_eq!(err1, err2);

        let err3 = DecimalError::InvalidFormat("other".to_string());
        assert_ne!(err1, err3);
    }
}
