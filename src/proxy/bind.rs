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
    // Existing types (unchanged for backward compatibility)
    String(String),
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    /// DECIMAL type converted to String for NUMERIC columns
    Decimal(String),

    // Additional numeric types
    I8(i8),
    I16(i16),
    F32(f32),

    // Date/time types (all converted to String for consistency)
    /// NaiveDate converted to ISO 8601 date string (YYYY-MM-DD)
    NaiveDate(String),
    /// NaiveTime converted to ISO 8601 time string (HH:MM:SS.nnnnnnnnn)
    NaiveTime(String),
    /// NaiveDateTime converted to ISO 8601 datetime string (YYYY-MM-DD HH:MM:SS.nnnnnnnnn)
    NaiveDateTime(String),
    /// DateTime<Utc> converted to ISO 8601 datetime with timezone (YYYY-MM-DD HH:MM:SS.nnnnnnnnn+00:00)
    DateTimeUtc(String),

    // JSON type (converted to String)
    /// serde_json::Value converted to JSON string
    Json(String),

    // Binary type
    /// Binary data stored as Vec<u8>
    Binary(Vec<u8>),

    // UUID type (converted to String)
    /// uuid::Uuid converted to UUID string format (123e4567-e89b-12d3-a456-426614174000)
    Uuid(String),

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
            BindValue::I8(i) => format!("i8({})", i),
            BindValue::I16(i) => format!("i16({})", i),
            BindValue::F32(f) => format!("f32({})", f),
            BindValue::NaiveDate(s) => format!("NaiveDate(\"{}\") [converted]", s),
            BindValue::NaiveTime(s) => format!("NaiveTime(\"{}\") [converted]", s),
            BindValue::NaiveDateTime(s) => format!("NaiveDateTime(\"{}\") [converted]", s),
            BindValue::DateTimeUtc(s) => format!("DateTimeUtc(\"{}\") [converted]", s),
            BindValue::Json(s) => format!("Json(\"{}\") [converted]", s),
            BindValue::Binary(bytes) => format!("Binary({} bytes)", bytes.len()),
            BindValue::Uuid(s) => format!("Uuid(\"{}\") [converted]", s),
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
// Additional Numeric Types (always available, no feature gate)
// ============================================================================

impl<DB: Database> BindProxy<DB> for i8 {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::I8(self)
    }
}

impl<DB: Database> BindProxy<DB> for i16 {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::I16(self)
    }
}

impl<DB: Database> BindProxy<DB> for u8 {
    fn into_bind_value(self) -> BindValue<DB> {
        // Convert to String because SQLx doesn't support unsigned integers for all databases
        BindValue::String(self.to_string())
    }
}

impl<DB: Database> BindProxy<DB> for u16 {
    fn into_bind_value(self) -> BindValue<DB> {
        // Convert to String because SQLx doesn't support unsigned integers for all databases
        BindValue::String(self.to_string())
    }
}

impl<DB: Database> BindProxy<DB> for u32 {
    fn into_bind_value(self) -> BindValue<DB> {
        // Convert to String because SQLx doesn't support unsigned integers for all databases
        BindValue::String(self.to_string())
    }
}

impl<DB: Database> BindProxy<DB> for u64 {
    fn into_bind_value(self) -> BindValue<DB> {
        // Convert to String because SQLx doesn't support unsigned integers for all databases
        BindValue::String(self.to_string())
    }
}

impl<DB: Database> BindProxy<DB> for f32 {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::F32(self)
    }
}

// ============================================================================
// Binary Types (always available)
// ============================================================================

impl<DB: Database> BindProxy<DB> for Vec<u8> {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::Binary(self)
    }
}

impl<'a, DB: Database> BindProxy<DB> for &'a [u8] {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::Binary(self.to_vec())
    }
}

// ============================================================================
// Chrono Date/Time Types (feature: "chrono")
// ============================================================================

#[cfg(feature = "chrono")]
impl<DB: Database> BindProxy<DB> for chrono::NaiveDate {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::NaiveDate(self.format("%Y-%m-%d").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<'a, DB: Database> BindProxy<DB> for &'a chrono::NaiveDate {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::NaiveDate(self.format("%Y-%m-%d").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<DB: Database> BindProxy<DB> for chrono::NaiveTime {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::NaiveTime(self.format("%H:%M:%S%.9f").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<'a, DB: Database> BindProxy<DB> for &'a chrono::NaiveTime {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::NaiveTime(self.format("%H:%M:%S%.9f").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<DB: Database> BindProxy<DB> for chrono::NaiveDateTime {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::NaiveDateTime(self.format("%Y-%m-%d %H:%M:%S%.9f").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<'a, DB: Database> BindProxy<DB> for &'a chrono::NaiveDateTime {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::NaiveDateTime(self.format("%Y-%m-%d %H:%M:%S%.9f").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<DB: Database> BindProxy<DB> for chrono::DateTime<chrono::Utc> {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::DateTimeUtc(self.format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string())
    }
}

#[cfg(feature = "chrono")]
impl<'a, DB: Database> BindProxy<DB> for &'a chrono::DateTime<chrono::Utc> {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::DateTimeUtc(self.format("%Y-%m-%d %H:%M:%S%.9f%:z").to_string())
    }
}

// ============================================================================
// UUID Type (feature: "uuid")
// ============================================================================

#[cfg(feature = "uuid")]
impl<DB: Database> BindProxy<DB> for uuid::Uuid {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::Uuid(self.to_string())
    }
}

#[cfg(feature = "uuid")]
impl<'a, DB: Database> BindProxy<DB> for &'a uuid::Uuid {
    fn into_bind_value(self) -> BindValue<DB> {
        BindValue::Uuid(self.to_string())
    }
}

// ============================================================================
// JSON Type (feature: "json")
// ============================================================================

#[cfg(feature = "json")]
impl<DB: Database> BindProxy<DB> for serde_json::Value {
    fn into_bind_value(self) -> BindValue<DB> {
        match serde_json::to_string(&self) {
            Ok(json_str) => BindValue::Json(json_str),
            Err(_) => BindValue::Json("{}".to_string()), // Fallback to empty object
        }
    }
}

#[cfg(feature = "json")]
impl<'a, DB: Database> BindProxy<DB> for &'a serde_json::Value {
    fn into_bind_value(self) -> BindValue<DB> {
        match serde_json::to_string(self) {
            Ok(json_str) => BindValue::Json(json_str),
            Err(_) => BindValue::Json("{}".to_string()),
        }
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

    // ============================================================================
    // Tests for additional numeric types
    // ============================================================================

    #[test]
    fn test_bind_proxy_i8() {
        let i: i8 = 127;
        let value = i.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::I8(v) => assert_eq!(v, 127),
            _ => panic!("Expected I8 variant"),
        }
    }

    #[test]
    fn test_bind_proxy_i16() {
        let i: i16 = 32767;
        let value = i.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::I16(v) => assert_eq!(v, 32767),
            _ => panic!("Expected I16 variant"),
        }
    }

    #[test]
    fn test_bind_proxy_u8() {
        let u: u8 = 255;
        let value = u.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::String(s) => assert_eq!(s, "255"),
            _ => panic!("Expected String variant (u8 converts to String)"),
        }
    }

    #[test]
    fn test_bind_proxy_u16() {
        let u: u16 = 65535;
        let value = u.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::String(s) => assert_eq!(s, "65535"),
            _ => panic!("Expected String variant (u16 converts to String)"),
        }
    }

    #[test]
    fn test_bind_proxy_u32() {
        let u: u32 = 4294967295;
        let value = u.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::String(s) => assert_eq!(s, "4294967295"),
            _ => panic!("Expected String variant (u32 converts to String)"),
        }
    }

    #[test]
    fn test_bind_proxy_u64() {
        let u: u64 = 18446744073709551615;
        let value = u.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::String(s) => assert_eq!(s, "18446744073709551615"),
            _ => panic!("Expected String variant (u64 converts to String)"),
        }
    }

    #[test]
    fn test_bind_proxy_f32() {
        let f: f32 = 3.14159;
        let value = f.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::F32(v) => assert!((v - 3.14159).abs() < 0.0001),
            _ => panic!("Expected F32 variant"),
        }
    }

    // ============================================================================
    // Tests for binary types
    // ============================================================================

    #[test]
    fn test_bind_proxy_vec_u8() {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let value = data.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Binary(bytes) => assert_eq!(bytes, vec![1, 2, 3, 4, 5]),
            _ => panic!("Expected Binary variant"),
        }
    }

    #[test]
    fn test_bind_proxy_u8_slice() {
        let data: &[u8] = &[10, 20, 30];
        let value = data.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Binary(bytes) => assert_eq!(bytes, vec![10, 20, 30]),
            _ => panic!("Expected Binary variant"),
        }
    }

    // ============================================================================
    // Tests for chrono date/time types
    // ============================================================================

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_naive_date() {
        use chrono::NaiveDate;
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let value = date.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::NaiveDate(s) => assert_eq!(s, "2024-01-15"),
            _ => panic!("Expected NaiveDate variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_naive_date_ref() {
        use chrono::NaiveDate;
        let date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let value = (&date).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::NaiveDate(s) => assert_eq!(s, "2024-12-31"),
            _ => panic!("Expected NaiveDate variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_naive_time() {
        use chrono::NaiveTime;
        let time = NaiveTime::from_hms_micro_opt(14, 30, 45, 123456).unwrap();
        let value = time.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::NaiveTime(s) => {
                // Format string produces 9 decimal places (padded with zeros if needed)
                assert_eq!(s, "14:30:45.123456000");
            }
            _ => panic!("Expected NaiveTime variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_naive_time_ref() {
        use chrono::NaiveTime;
        let time = NaiveTime::from_hms_nano_opt(23, 59, 59, 999999999).unwrap();
        let value = (&time).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::NaiveTime(s) => {
                assert_eq!(s, "23:59:59.999999999");
            }
            _ => panic!("Expected NaiveTime variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_naive_date_time() {
        use chrono::NaiveDateTime;
        let dt = NaiveDateTime::from_timestamp_opt(1704067200, 0).unwrap(); // 2024-01-01 00:00:00
        let value = dt.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::NaiveDateTime(s) => {
                assert!(s.starts_with("2024-01-01"));
            }
            _ => panic!("Expected NaiveDateTime variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_naive_date_time_ref() {
        use chrono::NaiveDateTime;
        let dt = NaiveDateTime::from_timestamp_opt(1704067200, 0).unwrap();
        let value = (&dt).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::NaiveDateTime(s) => {
                assert!(s.starts_with("2024-01-01"));
            }
            _ => panic!("Expected NaiveDateTime variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_date_time_utc() {
        use chrono::{DateTime, Utc, TimeZone};
        let dt = Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 45).unwrap();
        let value = dt.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::DateTimeUtc(s) => {
                assert!(s.contains("2024-06-15"));
                assert!(s.contains("12:30:45"));
            }
            _ => panic!("Expected DateTimeUtc variant"),
        }
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_proxy_date_time_utc_ref() {
        use chrono::{DateTime, Utc, TimeZone};
        let dt = Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 45).unwrap();
        let value = (&dt).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::DateTimeUtc(s) => {
                assert!(s.contains("2024-06-15"));
                assert!(s.contains("12:30:45"));
            }
            _ => panic!("Expected DateTimeUtc variant"),
        }
    }

    // ============================================================================
    // Tests for UUID type
    // ============================================================================

    #[test]
    #[cfg(feature = "uuid")]
    fn test_bind_proxy_uuid() {
        use uuid::Uuid;
        let u = Uuid::new_v4();
        let u_str = u.to_string();
        let value = u.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Uuid(s) => assert_eq!(s, u_str),
            _ => panic!("Expected Uuid variant"),
        }
    }

    #[test]
    #[cfg(feature = "uuid")]
    fn test_bind_proxy_uuid_ref() {
        use uuid::Uuid;
        let u = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let value = (&u).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Uuid(s) => {
                assert_eq!(s, "123e4567-e89b-12d3-a456-426614174000");
            }
            _ => panic!("Expected Uuid variant"),
        }
    }

    // ============================================================================
    // Tests for JSON type
    // ============================================================================

    #[test]
    #[cfg(feature = "json")]
    fn test_bind_proxy_json_value() {
        use serde_json::json;
        let json_val = json!({"name": "test", "value": 42});
        let value = json_val.into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Json(s) => {
                assert!(s.contains("test"));
                assert!(s.contains("42"));
            }
            _ => panic!("Expected Json variant"),
        }
    }

    #[test]
    #[cfg(feature = "json")]
    fn test_bind_proxy_json_value_ref() {
        use serde_json::json;
        let json_val = json!({"array": [1, 2, 3]});
        let value = (&json_val).into_bind_value();
        match value {
            BindValue::<sqlx::Postgres>::Json(s) => {
                // JSON is serialized without spaces by default
                assert!(s.contains("array"));
                assert!(s.contains("[1,2,3]"));
            }
            _ => panic!("Expected Json variant"),
        }
    }

    // ============================================================================
    // Tests for debug() method
    // ============================================================================

    #[test]
    fn test_bind_value_debug_i8() {
        let value = BindValue::<sqlx::Postgres>::I8(-128);
        assert_eq!(value.debug(), "i8(-128)");
    }

    #[test]
    fn test_bind_value_debug_f32() {
        let value = BindValue::<sqlx::Postgres>::F32(3.14);
        assert_eq!(value.debug(), "f32(3.14)");
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_bind_value_debug_naive_date() {
        let value = BindValue::<sqlx::Postgres>::NaiveDate("2024-01-15".to_string());
        assert!(value.debug().contains("[converted]"));
        assert!(value.debug().contains("2024-01-15"));
    }
}
