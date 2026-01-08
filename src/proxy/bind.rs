// BindProxy Trait and BindValue Enum
//
// This module defines the type conversion interface and the enum that holds
// converted values ready for binding to database queries.

use sqlx::Database;
use std::marker::PhantomData;

/// Values that can be bound to database queries with automatic type conversion.
///
/// This enum wraps different types and converts them to database-compatible values.
#[derive(Debug, Clone)]
pub enum BindValue<DB: Database> {
    String(String),
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    /// DECIMAL type converted to String for NUMERIC columns
    Decimal(String),
    /// PhantomData to make the DB type parameter used
    _Marker(PhantomData<DB>),
}

impl<DB: Database> BindValue<DB> {
    /// Get a debug representation
    pub fn debug(&self) -> String {
        match self {
            BindValue::String(s) => format!("String(\"{}\")", s),
            BindValue::I32(i) => format!("i32({})", i),
            BindValue::I64(i) => format!("i64({})", i),
            BindValue::F64(f) => format!("f64({})", f),
            BindValue::Bool(b) => format!("bool({})", b),
            BindValue::Decimal(s) => format!("Decimal(\"{}\") [converted]", s),
            BindValue::_Marker(_) => format!("_Marker"),
        }
    }
}

/// Trait for types that can be converted to bind values with automatic type conversion.
///
/// Implement this trait for custom types to enable automatic conversion when using
/// `bind_proxy` on enhanced queries.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::proxy::BindProxy;
///
/// impl BindProxy<sqlx::Postgres> for rust_decimal::Decimal {
///     fn into_bind_value(self) -> BindValue<sqlx::Postgres> {
///         BindValue::Decimal(self.to_string())
///     }
/// }
/// ```
pub trait BindProxy<DB: Database> {
    fn into_bind_value(self) -> BindValue<DB>;
}

// ============================================================================
// Implement BindProxy for basic types (generic across all databases)
// ============================================================================

impl<DB: Database> BindProxy<DB> for String {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::String(self)
    }
}

impl<DB: Database> BindProxy<DB> for i32 {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::I32(self)
    }
}

impl<DB: Database> BindProxy<DB> for i64 {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::I64(self)
    }
}

impl<DB: Database> BindProxy<DB> for f64 {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::F64(self)
    }
}

impl<DB: Database> BindProxy<DB> for bool {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::Bool(self)
    }
}

// Reference implementations
impl<'a, DB: Database> BindProxy<DB> for &'a str {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::String(self.to_string())
    }
}

// ============================================================================
// Optional rust_decimal support (works for all databases)
// ============================================================================

#[cfg(feature = "decimal")]
impl<DB: Database> BindProxy<DB> for rust_decimal::Decimal {
    fn into_bind_value(self) -> BindValue<DB> {
        // Convert DECIMAL to String for NUMERIC columns
        BindValue::Decimal(self.to_string())
    }
}

#[cfg(feature = "decimal")]
impl<'a, DB: Database> BindProxy<DB> for &'a rust_decimal::Decimal {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::Decimal(self.to_string())
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_value_string() {
        let value = BindValue::<sqlx::Postgres>::String("test".to_string());
        assert_eq!(value.debug(), "String(\"test\")");
    }

    #[test]
    fn test_bind_value_decimal() {
        let value = BindValue::<sqlx::Postgres>::Decimal("123.456".to_string());
        assert!(value.debug().contains("[converted]"));
    }

    #[test]
    fn test_bind_proxy_string() {
        let s = "hello".to_string();
        let value = s.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_bind_proxy_i32() {
        let i = 42;
        let value = i.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::I32(v) => assert_eq!(v, 42),
            _ => panic!("Expected I32 variant"),
        }
    }

    #[test]
    #[cfg(feature = "decimal")]
    fn test_bind_proxy_decimal() {
        use rust_decimal::Decimal;
        let d = Decimal::from_str_exact("99.99").unwrap();
        let value = d.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Decimal(s) => assert_eq!(s, "99.99"),
            _ => panic!("Expected Decimal variant"),
        }
    }

    #[test]
    #[cfg(feature = "decimal")]
    fn test_bind_proxy_decimal_ref() {
        use rust_decimal::Decimal;
        let d = Decimal::from_str_exact("123.456").unwrap();
        let value = (&d).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Decimal(s) => assert_eq!(s, "123.456"),
            _ => panic!("Expected Decimal variant"),
        }
    }

    #[test]
    fn test_bind_proxy_bool() {
        let b = true;
        let value = b.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Bool(v) => assert_eq!(v, true),
            _ => panic!("Expected Bool variant"),
        }
    }
}
