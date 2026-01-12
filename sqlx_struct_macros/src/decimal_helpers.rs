//! DECIMAL helper methods code generation for EnhancedCrud derive macro.
//!
//! This module provides the code generation logic for automatically creating
//! helper methods on struct fields annotated with `#[crud(decimal(...))]`.

use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Visibility, Type};

/// DECIMAL field metadata extracted from #[crud(decimal(...))] attributes.
#[derive(Clone)]
pub struct DecimalField {
    /// Field name (e.g., "total_amount")
    pub name: Ident,
    /// Total number of digits (integer + fractional)
    pub precision: u8,
    /// Number of digits after decimal point
    pub scale: u8,
    /// Field visibility (pub, private, etc.)
    pub vis: Visibility,
    /// Whether the field is Option<String> (true) or String (false)
    pub is_optional: bool,
    /// SQL cast type extracted from cast_as parameter (e.g., "TEXT", "VARCHAR")
    /// Reserved for future use
    pub _cast_as: Option<String>,
}

impl DecimalField {
    /// Generate method name by appending suffix to field name.
    ///
    /// Example: field "amount" + suffix "as_f64" -> "amount_as_f64"
    fn method_name(&self, suffix: &str) -> Ident {
        Ident::new(
            &format!("{}_{}", self.name, suffix),
            self.name.span()
        )
    }

    /// Generate all helper methods for a single DECIMAL field.
    ///
    /// This generates ~25 methods covering:
    /// - Type conversion (String → f64)
    /// - Chainable arithmetic operations
    /// - Validation and formatting
    /// - Precision control
    pub fn generate_helper_methods(&self) -> TokenStream2 {
        if self.is_optional {
            self.generate_optional_methods()
        } else {
            self.generate_required_methods()
        }
    }

    /// Generate methods for Option<String> fields.
    fn generate_optional_methods(&self) -> TokenStream2 {
        let field_name = &self.name;
        let precision = self.precision;
        let scale = self.scale;
        let vis = &self.vis;

        // Method names
        let as_f64 = self.method_name("as_f64");
        let as_f64_or = self.method_name("as_f64_or");
        let as_f64_unwrap = self.method_name("as_f64_unwrap");
        let add = self.method_name("add");
        let sub = self.method_name("sub");
        let mul = self.method_name("mul");
        let div = self.method_name("div");
        let add_f64 = self.method_name("add_f64");
        let sub_f64 = self.method_name("sub_f64");
        let mul_f64 = self.method_name("mul_f64");
        let div_f64 = self.method_name("div_f64");
        let round = self.method_name("round");
        let abs = self.method_name("abs");
        let neg = self.method_name("neg");
        let validate = self.method_name("validate");
        let is_positive = self.method_name("is_positive");
        let is_negative = self.method_name("is_negative");
        let is_zero = self.method_name("is_zero");
        let format_fn = self.method_name("format"); // 'format' is a reserved word
        let format_currency = self.method_name("format_currency");
        let format_percent = self.method_name("format_percent");
        let truncate = self.method_name("truncate");
        let precision_method = self.method_name("precision");
        let scale_method = self.method_name("scale");
        let clamp = self.method_name("clamp");
        let max_value = self.method_name("max_value");
        let min_value = self.method_name("min_value");

        quote! {
            // ===================================================================
            // SECTION 1: Type Conversion Methods (for Option<String>)
            // ===================================================================

            /// Convert DECIMAL field to f64.
            ///
            /// Returns `None` if field is `None`.
            /// Returns `Err(DecimalError)` if string is not a valid decimal.
            #vis fn #as_f64(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<Option<f64>> {
                match &self.#field_name {
                    None => Ok(None),
                    Some(s) => {
                        s.parse::<f64>()
                            .map(Some)
                            .map_err(|_| ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(s.clone()))
                    }
                }
            }

            /// Convert DECIMAL field to f64, with default value if None.
            #vis fn #as_f64_or(&self, default: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<f64> {
                Ok(self.#as_f64()?.unwrap_or(default))
            }

            /// Convert DECIMAL field to f64, unwrap (panic if None or invalid).
            ///
            /// # Panics
            ///
            /// Panics if field is None or contains invalid decimal string.
            #vis fn #as_f64_unwrap(&self) -> f64 {
                self.#as_f64()
                    .unwrap()
                    .unwrap()
            }

            // ===================================================================
            // SECTION 2: Chainable Arithmetic Operations
            // ===================================================================

            /// Add value to DECIMAL field (mutation).
            ///
            /// Returns `&mut Self` for chaining.
            /// Returns `Err` if field is None or invalid.
            #vis fn #add(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                self.#add_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Subtract value from DECIMAL field (mutation).
            #vis fn #sub(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                self.#sub_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Multiply DECIMAL field by value (mutation).
            #vis fn #mul(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                self.#mul_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Divide DECIMAL field by value (mutation).
            #vis fn #div(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                if value == "0" || value == "0.0" {
                    return Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::DivisionByZero);
                }
                self.#div_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Add f64 value to DECIMAL field (mutation).
            #vis fn #add_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        let result = current_val + value;
                        self.#field_name = Some(format!("{}", result));
                        Ok(self)
                    }
                }
            }

            /// Subtract f64 value from DECIMAL field (mutation).
            #vis fn #sub_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        let result = current_val - value;
                        self.#field_name = Some(format!("{}", result));
                        Ok(self)
                    }
                }
            }

            /// Multiply DECIMAL field by f64 value (mutation).
            #vis fn #mul_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        let result = current_val * value;
                        self.#field_name = Some(format!("{}", result));
                        Ok(self)
                    }
                }
            }

            /// Divide DECIMAL field by f64 value (mutation).
            #vis fn #div_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                if value == 0.0 {
                    return Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::DivisionByZero);
                }
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        let result = current_val / value;
                        self.#field_name = Some(format!("{}", result));
                        Ok(self)
                    }
                }
            }

            /// Round DECIMAL field to specified decimal places (mutation).
            #vis fn #round(&mut self, decimal_places: u32) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        let multiplier = 10_f64.powi(decimal_places as i32);
                        let result = (current_val * multiplier).round() / multiplier;
                        self.#field_name = Some(format!("{}", result));
                        Ok(self)
                    }
                }
            }

            /// Set DECIMAL field to absolute value (mutation).
            #vis fn #abs(&mut self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        self.#field_name = Some(format!("{}", current_val.abs()));
                        Ok(self)
                    }
                }
            }

            /// Negate DECIMAL field (mutation).
            #vis fn #neg(&mut self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        self.#field_name = Some(format!("{}", -current_val));
                        Ok(self)
                    }
                }
            }

            // ===================================================================
            // SECTION 3: Validation and Formatting
            // ===================================================================

            /// Validate DECIMAL field against precision/scale constraints.
            #vis fn #validate(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<bool> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(s) => {
                        // Parse as f64
                        let value = s.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(s.clone())
                        })?;

                        // Check precision/scale constraints
                        let max_int_digits = #precision - #scale;
                        let abs_value = value.abs();

                        // Count integer digits
                        let int_part = abs_value.floor() as i64;
                        let int_digits = if int_part == 0 { 1 } else { (int_part as f64).log10().floor() as i32 + 1 };

                        if int_digits as u8 > max_int_digits {
                            return Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::Overflow {
                                value: s.clone(),
                                precision: #precision,
                                scale: #scale,
                            });
                        }

                        Ok(true)
                    }
                }
            }

            /// Check if DECIMAL field is positive (> 0).
            ///
            /// Returns `None` if field is None.
            #vis fn #is_positive(&self) -> Option<bool> {
                self.#field_name.as_ref().and_then(|s| {
                    s.parse::<f64>().ok().map(|v| v > 0.0)
                })
            }

            /// Check if DECIMAL field is negative (< 0).
            ///
            /// Returns `None` if field is None.
            #vis fn #is_negative(&self) -> Option<bool> {
                self.#field_name.as_ref().and_then(|s| {
                    s.parse::<f64>().ok().map(|v| v < 0.0)
                })
            }

            /// Check if DECIMAL field is zero (= 0).
            ///
            /// Returns `None` if field is None.
            #vis fn #is_zero(&self) -> Option<bool> {
                self.#field_name.as_ref().and_then(|s| {
                    s.parse::<f64>().ok().map(|v| v == 0.0)
                })
            }

            /// Format DECIMAL field with thousands separator.
            #vis fn #format_fn(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<Option<String>> {
                match &self.#field_name {
                    None => Ok(None),
                    Some(s) => {
                        let value = s.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(s.clone())
                        })?;
                        let formatted = ::sqlx_struct_enhanced::decimal_helpers::format_with_thousands_separator(value, 2);
                        Ok(Some(formatted))
                    }
                }
            }

            /// Format DECIMAL field with thousands separator and currency symbol.
            #vis fn #format_currency(&self, symbol: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<Option<String>> {
                match &self.#field_name {
                    None => Ok(None),
                    Some(s) => {
                        let value = s.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(s.clone())
                        })?;
                        let formatted = ::sqlx_struct_enhanced::decimal_helpers::format_with_thousands_separator(value, 2);
                        Ok(Some(format!("{}{}", symbol, formatted)))
                    }
                }
            }

            /// Format DECIMAL field as percentage (multiply by 100 and add %).
            #vis fn #format_percent(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<Option<String>> {
                match &self.#field_name {
                    None => Ok(None),
                    Some(s) => {
                        let value = s.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(s.clone())
                        })?;
                        let formatted = format!("{:.2}%", value * 100.0);
                        Ok(Some(formatted))
                    }
                }
            }

            /// Truncate DECIMAL field to specified decimal places (no rounding).
            #vis fn #truncate(&mut self, decimal_places: u32) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                match &self.#field_name {
                    None => Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::NullValue),
                    Some(current) => {
                        let current_val = current.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(current.clone())
                        })?;
                        let multiplier = 10_f64.powi(decimal_places as i32);
                        let result = (current_val * multiplier).trunc() / multiplier;
                        self.#field_name = Some(format!("{}", result));
                        Ok(self)
                    }
                }
            }

            // ===================================================================
            // SECTION 4: Precision Control
            // ===================================================================

            /// Get precision (total digits) for this field from #[crud(decimal(precision = N))].
            #vis fn #precision_method(&self) -> u8 {
                #precision
            }

            /// Get scale (decimal places) for this field from #[crud(decimal(scale = N))].
            #vis fn #scale_method(&self) -> u8 {
                #scale
            }

            /// Clamp DECIMAL field to fit within precision/scale constraints.
            #vis fn #clamp(&mut self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                // Clamp to max value
                let max = self.#max_value()?;
                let min = self.#min_value()?;

                match &self.#field_name {
                    None => Ok(self),
                    Some(s) => {
                        let value = s.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(s.clone())
                        })?;

                        let max_val = max.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(max)
                        })?;

                        let min_val = min.parse::<f64>().map_err(|_| {
                            ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(min)
                        })?;

                        let clamped = value.max(min_val).min(max_val);
                        self.#field_name = Some(format!("{}", clamped));
                        Ok(self)
                    }
                }
            }

            /// Get maximum value for this field based on precision.
            #vis fn #max_value(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let max_int_digits = #precision - #scale;
                let max_int = if max_int_digits > 0 {
                    "9".repeat(max_int_digits as usize)
                } else {
                    "0".to_string()
                };

                if #scale > 0 {
                    Ok(format!("{}.{}", max_int, "9".repeat(#scale as usize)))
                } else {
                    Ok(max_int)
                }
            }

            /// Get minimum value for this field based on precision.
            #vis fn #min_value(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let max = self.#max_value()?;
                Ok(format!("-{}", max))
            }
        }
    }

    /// Generate methods for String (non-optional) fields.
    fn generate_required_methods(&self) -> TokenStream2 {
        let field_name = &self.name;
        let precision = self.precision;
        let scale = self.scale;
        let vis = &self.vis;

        // Method names
        let as_f64 = self.method_name("as_f64");
        let as_f64_or = self.method_name("as_f64_or");
        let as_f64_unwrap = self.method_name("as_f64_unwrap");
        let add = self.method_name("add");
        let sub = self.method_name("sub");
        let mul = self.method_name("mul");
        let div = self.method_name("div");
        let add_f64 = self.method_name("add_f64");
        let sub_f64 = self.method_name("sub_f64");
        let mul_f64 = self.method_name("mul_f64");
        let div_f64 = self.method_name("div_f64");
        let round = self.method_name("round");
        let abs = self.method_name("abs");
        let neg = self.method_name("neg");
        let validate = self.method_name("validate");
        let is_positive = self.method_name("is_positive");
        let is_negative = self.method_name("is_negative");
        let is_zero = self.method_name("is_zero");
        let format_fn = self.method_name("format");
        let format_currency = self.method_name("format_currency");
        let format_percent = self.method_name("format_percent");
        let truncate = self.method_name("truncate");
        let precision_method = self.method_name("precision");
        let scale_method = self.method_name("scale");
        let clamp = self.method_name("clamp");
        let max_value = self.method_name("max_value");
        let min_value = self.method_name("min_value");

        quote! {
            // ===================================================================
            // SECTION 1: Type Conversion Methods (for String)
            // ===================================================================

            /// Convert DECIMAL field to f64.
            ///
            /// Returns `Err(DecimalError)` if string is not a valid decimal.
            #vis fn #as_f64(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<f64> {
                self.#field_name.parse::<f64>()
                    .map_err(|_| ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone()))
            }

            /// Convert DECIMAL field to f64, with default value (same as as_f64 for String).
            #vis fn #as_f64_or(&self, default: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<f64> {
                self.#as_f64().or(Ok(default))
            }

            /// Convert DECIMAL field to f64, unwrap (panic if invalid).
            ///
            /// # Panics
            ///
            /// Panics if field contains invalid decimal string.
            #vis fn #as_f64_unwrap(&self) -> f64 {
                self.#as_f64().unwrap()
            }

            // ===================================================================
            // SECTION 2: Chainable Arithmetic Operations
            // ===================================================================

            /// Add value to DECIMAL field (mutation).
            ///
            /// Returns `&mut Self` for chaining.
            #vis fn #add(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                self.#add_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Subtract value from DECIMAL field (mutation).
            #vis fn #sub(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                self.#sub_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Multiply DECIMAL field by value (mutation).
            #vis fn #mul(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                self.#mul_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Divide DECIMAL field by value (mutation).
            #vis fn #div(&mut self, value: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                if value == "0" || value == "0.0" {
                    return Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::DivisionByZero);
                }
                self.#div_f64(value.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(value.to_string())
                })?)
            }

            /// Add f64 value to DECIMAL field (mutation).
            #vis fn #add_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let result = current_val + value;
                self.#field_name = format!("{}", result);
                Ok(self)
            }

            /// Subtract f64 value from DECIMAL field (mutation).
            #vis fn #sub_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let result = current_val - value;
                self.#field_name = format!("{}", result);
                Ok(self)
            }

            /// Multiply DECIMAL field by f64 value (mutation).
            #vis fn #mul_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let result = current_val * value;
                self.#field_name = format!("{}", result);
                Ok(self)
            }

            /// Divide DECIMAL field by f64 value (mutation).
            #vis fn #div_f64(&mut self, value: f64) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                if value == 0.0 {
                    return Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::DivisionByZero);
                }
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let result = current_val / value;
                self.#field_name = format!("{}", result);
                Ok(self)
            }

            /// Round DECIMAL field to specified decimal places (mutation).
            #vis fn #round(&mut self, decimal_places: u32) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let multiplier = 10_f64.powi(decimal_places as i32);
                let result = (current_val * multiplier).round() / multiplier;
                self.#field_name = format!("{}", result);
                Ok(self)
            }

            /// Set DECIMAL field to absolute value (mutation).
            #vis fn #abs(&mut self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                self.#field_name = format!("{}", current_val.abs());
                Ok(self)
            }

            /// Negate DECIMAL field (mutation).
            #vis fn #neg(&mut self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                self.#field_name = format!("{}", -current_val);
                Ok(self)
            }

            // ===================================================================
            // SECTION 3: Validation and Formatting
            // ===================================================================

            /// Validate DECIMAL field against precision/scale constraints.
            #vis fn #validate(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<bool> {
                // Parse as f64
                let value = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;

                // Check precision/scale constraints
                let max_int_digits = #precision - #scale;
                let abs_value = value.abs();

                // Count integer digits
                let int_part = abs_value.floor() as i64;
                let int_digits = if int_part == 0 { 1 } else { (int_part as f64).log10().floor() as i32 + 1 };

                if int_digits as u8 > max_int_digits {
                    return Err(::sqlx_struct_enhanced::decimal_helpers::DecimalError::Overflow {
                        value: self.#field_name.clone(),
                        precision: #precision,
                        scale: #scale,
                    });
                }

                Ok(true)
            }

            /// Check if DECIMAL field is positive (> 0).
            #vis fn #is_positive(&self) -> bool {
                self.#field_name.parse::<f64>().ok().map(|v| v > 0.0).unwrap_or(false)
            }

            /// Check if DECIMAL field is negative (< 0).
            #vis fn #is_negative(&self) -> bool {
                self.#field_name.parse::<f64>().ok().map(|v| v < 0.0).unwrap_or(false)
            }

            /// Check if DECIMAL field is zero (= 0).
            #vis fn #is_zero(&self) -> bool {
                self.#field_name.parse::<f64>().ok().map(|v| v == 0.0).unwrap_or(false)
            }

            /// Format DECIMAL field with thousands separator.
            #vis fn #format_fn(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let value = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                Ok(::sqlx_struct_enhanced::decimal_helpers::format_with_thousands_separator(value, 2))
            }

            /// Format DECIMAL field with currency symbol.
            #vis fn #format_currency(&self, symbol: &str) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let value = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let formatted = ::sqlx_struct_enhanced::decimal_helpers::format_with_thousands_separator(value, 2);
                Ok(format!("{}{}", symbol, formatted))
            }

            /// Format DECIMAL field as percentage.
            #vis fn #format_percent(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let value = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                Ok(format!("{:.2}%", value * 100.0))
            }

            /// Truncate DECIMAL field to specified decimal places (no rounding).
            #vis fn #truncate(&mut self, decimal_places: u32) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                let current_val = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;
                let multiplier = 10_f64.powi(decimal_places as i32);
                let result = (current_val * multiplier).trunc() / multiplier;
                self.#field_name = format!("{}", result);
                Ok(self)
            }

            // ===================================================================
            // SECTION 4: Precision Control
            // ===================================================================

            /// Get precision (total digits) for this field from #[crud(decimal(precision = N))].
            #vis fn #precision_method(&self) -> u8 {
                #precision
            }

            /// Get scale (decimal places) for this field from #[crud(decimal(scale = N))].
            #vis fn #scale_method(&self) -> u8 {
                #scale
            }

            /// Clamp DECIMAL field to fit within precision/scale constraints.
            #vis fn #clamp(&mut self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<&mut Self> {
                // Clamp to max value
                let max = self.#max_value()?;
                let min = self.#min_value()?;

                let value = self.#field_name.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(self.#field_name.clone())
                })?;

                let max_val = max.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(max)
                })?;

                let min_val = min.parse::<f64>().map_err(|_| {
                    ::sqlx_struct_enhanced::decimal_helpers::DecimalError::InvalidFormat(min)
                })?;

                let clamped = value.max(min_val).min(max_val);
                self.#field_name = format!("{}", clamped);
                Ok(self)
            }

            /// Get maximum value for this field based on precision.
            #vis fn #max_value(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let max_int_digits = #precision - #scale;
                let max_int = if max_int_digits > 0 {
                    "9".repeat(max_int_digits as usize)
                } else {
                    "0".to_string()
                };

                if #scale > 0 {
                    Ok(format!("{}.{}", max_int, "9".repeat(#scale as usize)))
                } else {
                    Ok(max_int)
                }
            }

            /// Get minimum value for this field based on precision.
            #vis fn #min_value(&self) -> ::sqlx_struct_enhanced::decimal_helpers::DecimalResult<String> {
                let max = self.#max_value()?;
                Ok(format!("-{}", max))
            }
        }
    }
}

/// Extract DECIMAL fields from struct with #[crud(decimal(precision = X, scale = Y))].
///
/// This function parses the struct attributes and identifies all fields
/// annotated with the `decimal` attribute, collecting their metadata.
///
/// # Arguments
///
/// * `input` - The DeriveInput from the derive macro
///
/// # Returns
///
/// Vector of `DecimalField` structs containing metadata for each DECIMAL field
pub fn extract_decimal_fields(input: &DeriveInput) -> Vec<DecimalField> {
    let mut decimal_fields = Vec::new();

    if let syn::Data::Struct(data_struct) = &input.data {
        for field in &data_struct.fields {
            let field_name = field.ident.as_ref().expect("Field must have name");
            let vis = field.vis.clone();

            // Parse #[crud(decimal(...))] attributes
            for attr in &field.attrs {
                // Check if this is a crud attribute containing "decimal"
                let attr_str = attr.tokens.to_string();

                if attr_str.contains("decimal") {
                    // Parse: decimal(precision = N, scale = M)
                    let mut precision = None;
                    let mut scale = None;

                    // Extract precision value
                    if let Some(precision_pos) = attr_str.find("precision") {
                        let remaining = &attr_str[precision_pos..];
                        if let Some(eq_pos) = remaining.find('=') {
                            let after_eq = &remaining[eq_pos + 1..];
                            // Skip whitespace and extract number
                            let value_str: String = after_eq
                                .chars()
                                .skip_while(|c| c.is_whitespace())
                                .take_while(|c| c.is_digit(10))
                                .collect();
                            if let Ok(p) = value_str.parse::<u8>() {
                                precision = Some(p);
                            }
                        }
                    }

                    // Extract scale value
                    if let Some(scale_pos) = attr_str.find("scale") {
                        let remaining = &attr_str[scale_pos..];
                        if let Some(eq_pos) = remaining.find('=') {
                            let after_eq = &remaining[eq_pos + 1..];
                            // Skip whitespace and extract number
                            let value_str: String = after_eq
                                .chars()
                                .skip_while(|c| c.is_whitespace())
                                .take_while(|c| c.is_digit(10))
                                .collect();
                            if let Ok(s) = value_str.parse::<u8>() {
                                scale = Some(s);
                            }
                        }
                    }

                    // Extract cast_as value (NEW)
                    let mut cast_as_from_decimal = None;
                    if let Some(cast_pos) = attr_str.find("cast_as") {
                        let remaining = &attr_str[cast_pos..];
                        if let Some(eq_pos) = remaining.find('=') {
                            let after_eq = &remaining[eq_pos + 1..];
                            // Skip whitespace and extract quoted string value
                            let value_str: String = after_eq
                                .chars()
                                .skip_while(|c| c.is_whitespace() || *c == '=')
                                .skip_while(|c| c.is_whitespace())
                                .take_while(|c| *c != ',' && *c != ')')
                                .collect();
                            let cleaned = value_str.trim().trim_matches('"').trim_matches('\'');
                            if !cleaned.is_empty() {
                                cast_as_from_decimal = Some(cleaned.to_string());
                            }
                        }
                    }

                    // Apply default "TEXT" if no cast_as specified (NEW)
                    let final_cast_as = cast_as_from_decimal.or_else(|| Some("TEXT".to_string()));

                    // Only add if both precision and scale are found
                    if let (Some(p), Some(s)) = (precision, scale) {
                        // Check if field is Option<String> or String
                        let is_optional = is_option_string(&field.ty);

                        decimal_fields.push(DecimalField {
                            name: field_name.clone(),
                            precision: p,
                            scale: s,
                            vis: vis.clone(),
                            is_optional,
                            _cast_as: final_cast_as,
                        });
                    }
                }
            }
        }
    }

    decimal_fields
}

/// Check if a type is `Option<String>` (true) or just `String` (false).
fn is_option_string(ty: &Type) -> bool {
    // Check if the type is Option<String>
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                // Check if the inner type is String
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            if let syn::Type::Path(inner_path) = inner_ty {
                                if let Some(inner_segment) = inner_path.path.segments.last() {
                                    return inner_segment.ident == "String";
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Generate complete DecimalHelpers implementation for a struct.
///
/// This function generates an impl block containing all helper methods
/// for all DECIMAL fields in the struct.
///
/// # Arguments
///
/// * `struct_name` - Name of the struct (e.g., "Order")
/// * `decimal_fields` - Slice of DECIMAL field metadata
///
/// # Returns
///
/// TokenStream containing the complete impl block
pub fn generate_decimal_helpers_impl(
    struct_name: &Ident,
    decimal_fields: &[DecimalField]
) -> TokenStream2 {
    let helper_methods: Vec<TokenStream2> = decimal_fields
        .iter()
        .map(|field| field.generate_helper_methods())
        .collect();

    quote! {
        impl #struct_name {
            #(#helper_methods)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_extract_decimal_fields() {
        let input_str = r#"
            struct Order {
                id: String,
                #[crud(decimal(precision = 10, scale = 2))]
                #[crud(cast_as = "TEXT")]
                total_amount: Option<String>,
                name: String,
                #[crud(decimal(precision = 5, scale = 2))]
                discount: Option<String>,
            }
        "#;

        let input: DeriveInput = parse_str(input_str).unwrap();
        let fields = extract_decimal_fields(&input);

        assert_eq!(fields.len(), 2);

        assert_eq!(fields[0].name.to_string(), "total_amount");
        assert_eq!(fields[0].precision, 10);
        assert_eq!(fields[0].scale, 2);

        assert_eq!(fields[1].name.to_string(), "discount");
        assert_eq!(fields[1].precision, 5);
        assert_eq!(fields[1].scale, 2);
    }
}
