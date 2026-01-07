//! Parser for extracting schema information from Rust struct definitions
//!
//! This module provides functionality to parse Rust structs and extract
//! database schema information including columns, types, and migration attributes.

use proc_macro2::{TokenStream, Ident};
use syn::{DeriveInput, Data, DataStruct, Fields, Type, PathSegment, PathArguments};
use quote::{quote, ToTokens};
use std::collections::HashMap;

/// Parsed schema information from a Rust struct
#[derive(Debug, Clone)]
pub struct StructSchema {
    /// Struct name (PascalCase)
    pub struct_name: String,
    /// Table name (snake_case derived from struct name)
    pub table_name: String,
    /// Original table name if renamed
    pub rename_from: Option<String>,
    /// Column definitions
    pub columns: Vec<StructColumn>,
    /// Primary key field name (first field)
    pub primary_key: String,
}

/// Column information extracted from a struct field
#[derive(Debug, Clone)]
pub struct StructColumn {
    /// Field name
    pub name: String,
    /// Field type
    pub rust_type: String,
    /// Corresponding SQL type
    pub sql_type: String,
    /// Whether the field is optional (Option<T>)
    pub nullable: bool,
    /// Original name if renamed
    pub rename_from: Option<String>,
    /// Data migration specification
    pub data_migration: Option<DataMigrationSpec>,
    /// Type casting directive for SQL queries (e.g., Some("TEXT") for NUMERIC→TEXT)
    pub cast_as: Option<String>,
    /// Decimal precision specification (optional, for NUMERIC/DECIMAL types)
    pub decimal_precision: Option<(u32, u32)>, // (precision, scale)
}

/// Data migration specification from attributes
#[derive(Debug, Clone)]
pub struct DataMigrationSpec {
    pub migration_type: DataMigrationType,
    pub expression: Option<String>,
    pub callback_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataMigrationType {
    Default { value: String },
    Compute { expression: String },
    Callback { function_name: String },
}

/// Parser for struct schema information
pub struct StructSchemaParser;

impl StructSchemaParser {
    /// Parse a struct derive input to extract schema information
    pub fn parse(input: &DeriveInput) -> Result<StructSchema, String> {
        // Get struct name
        let struct_name = &input.ident;
        let struct_name_str = struct_name.to_string();

        // Convert to snake_case for table name
        let table_name = to_snake_case(&struct_name_str);

        // Parse struct-level attributes
        let (table_name, rename_from) = Self::parse_struct_attributes(input, &table_name)?;

        // Extract columns from struct fields
        let columns = Self::parse_fields(&input.data, &input.attrs)?;

        // Get primary key (first field)
        let primary_key = columns.first()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "id".to_string());

        Ok(StructSchema {
            struct_name: struct_name_str,
            table_name,
            rename_from,
            columns,
            primary_key,
        })
    }

    /// Parse struct-level attributes for migration options
    fn parse_struct_attributes(
        input: &DeriveInput,
        default_table_name: &str,
    ) -> Result<(String, Option<String>), String> {
        let mut table_name = default_table_name.to_string();
        let mut rename_from = None;

        // Parse #[migration(...)] attributes
        for attr in &input.attrs {
            // Check if this is a migration attribute
            let path_str = quote::quote!(#attr).to_string();
            if path_str.contains("migration") {
                // Parse the attribute tokens
                let tokens = attr.tokens.to_string();

                // Parse rename_from = "old_table_name"
                if let Some(rename_pos) = tokens.find("rename_from") {
                    let remaining = &tokens[rename_pos..];
                    if let Some(eq_pos) = remaining.find('=') {
                        let value_str = &remaining[eq_pos + 1..];
                        // Find the next comma or end
                        let end_pos = value_str.find(',').unwrap_or(value_str.len());
                        let value = value_str[..end_pos].trim().trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            rename_from = Some(value.to_string());
                        }
                    }
                }
            }
        }

        Ok((table_name, rename_from))
    }

    /// Parse struct fields to extract column information
    fn parse_fields(data: &Data, attrs: &[syn::Attribute]) -> Result<Vec<StructColumn>, String> {
        let struct_data = match data {
            Data::Struct(s) => s,
            _ => return Err("Can only derive migration on structs".to_string()),
        };

        match &struct_data.fields {
            Fields::Named(fields) => {
                let mut columns = Vec::new();

                for field in &fields.named {
                    let column = Self::parse_field(field)?;
                    columns.push(column);
                }

                Ok(columns)
            }
            Fields::Unnamed(_) => Err("Unnamed fields are not supported for migrations".to_string()),
            Fields::Unit => Err("Unit structs are not supported for migrations".to_string()),
        }
    }

    /// Parse a single struct field
    fn parse_field(field: &syn::Field) -> Result<StructColumn, String> {
        // Get field name
        let field_name = field.ident.as_ref()
            .ok_or_else(|| "Field must be named".to_string())?
            .to_string();

        // Parse field type
        let (rust_type, nullable) = Self::parse_field_type(&field.ty)?;

        // Parse field attributes
        let (rename_from, data_migration, cast_as, decimal_precision) = Self::parse_field_attributes(field)?;

        // Map Rust type to SQL type (with optional decimal precision)
        let sql_type = Self::map_rust_type_to_sql_with_precision(&rust_type, decimal_precision);

        Ok(StructColumn {
            name: field_name,
            rust_type,
            sql_type,
            nullable,
            rename_from,
            data_migration,
            cast_as,
            decimal_precision,
        })
    }

    /// Parse field type and determine if it's nullable
    fn parse_field_type(ty: &Type) -> Result<(String, bool), String> {
        let type_str = ty.into_token_stream().to_string();

        // Check if it's Option<T>
        if let Type::Path(path) = ty {
            if let Some(segment) = path.path.segments.last() {
                if segment.ident == "Option" {
                    // Extract the inner type T
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(arg) = args.args.first() {
                            let inner_type = arg.into_token_stream().to_string();
                            return Ok((inner_type, true));
                        }
                    }
                }
            }
        }

        Ok((type_str, false))
    }

    /// Parse field-level attributes
    fn parse_field_attributes(
        field: &syn::Field
    ) -> Result<(Option<String>, Option<DataMigrationSpec>, Option<String>, Option<(u32, u32)>), String> {
        let mut rename_from = None;
        let mut data_migration = None;
        let mut cast_as = None;
        let mut decimal_precision = None;

        for attr in &field.attrs {
            let path_str = quote::quote!(#attr).to_string();

            // Parse #[crud(...)] attributes
            if path_str.contains("crud") {
                let tokens = attr.tokens.to_string();

                // Parse cast_as = "TYPE"
                if let Some(cast_pos) = tokens.find("cast_as") {
                    let remaining = &tokens[cast_pos..];
                    if let Some(eq_pos) = remaining.find('=') {
                        let value_str = &remaining[eq_pos + 1..];
                        let end_pos = value_str.find(',').unwrap_or(value_str.len());
                        let value = value_str[..end_pos]
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'');
                        if !value.is_empty() {
                            cast_as = Some(value.to_string());
                        }
                    }
                }

                // Parse decimal(precision = X, scale = Y)
                if let Some(decimal_pos) = tokens.find("decimal") {
                    let remaining = &tokens[decimal_pos..];
                    // Extract content inside parentheses: decimal(precision = 10, scale = 2)
                    if let Some(open_paren) = remaining.find('(') {
                        if let Some(close_paren) = remaining.find(')') {
                            let params_str = &remaining[open_paren + 1..close_paren];
                            let mut precision = None;
                            let mut scale = None;

                            // Parse precision = X
                            if let Some(prec_pos) = params_str.find("precision") {
                                let prec_remaining = &params_str[prec_pos..];
                                if let Some(eq_pos) = prec_remaining.find('=') {
                                    let value_str = &prec_remaining[eq_pos + 1..];
                                    let end_pos = value_str.find(',').unwrap_or(value_str.len());
                                    let value = value_str[..end_pos].trim();
                                    if let Ok(p) = value.parse::<u32>() {
                                        precision = Some(p);
                                    }
                                }
                            }

                            // Parse scale = Y
                            if let Some(scale_pos) = params_str.find("scale") {
                                let scale_remaining = &params_str[scale_pos..];
                                if let Some(eq_pos) = scale_remaining.find('=') {
                                    let value_str = &scale_remaining[eq_pos + 1..];
                                    let end_pos = value_str.find(',').unwrap_or(value_str.len());
                                    let value = value_str[..end_pos].trim();
                                    if let Ok(s) = value.parse::<u32>() {
                                        scale = Some(s);
                                    }
                                }
                            }

                            if let (Some(p), Some(s)) = (precision, scale) {
                                decimal_precision = Some((p, s));
                            }
                        }
                    }
                }
            }

            // Parse #[migration(...)] attributes
            if path_str.contains("migration") {
                let tokens = attr.tokens.to_string();

                // Parse rename_from = "old_name"
                if let Some(rename_pos) = tokens.find("rename_from") {
                    let remaining = &tokens[rename_pos..];
                    if let Some(eq_pos) = remaining.find('=') {
                        let value_str = &remaining[eq_pos + 1..];
                        let end_pos = value_str.find(',').unwrap_or(value_str.len());
                        let value = value_str[..end_pos].trim().trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            rename_from = Some(value.to_string());
                        }
                    }
                }

                // Parse default = "value"
                if let Some(default_pos) = tokens.find("default") {
                    let remaining = &tokens[default_pos..];
                    if let Some(eq_pos) = remaining.find('=') {
                        let value_str = &remaining[eq_pos + 1..];
                        let end_pos = value_str.find(',').unwrap_or(value_str.len());
                        let value = value_str[..end_pos].trim().trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            data_migration = Some(DataMigrationSpec {
                                migration_type: DataMigrationType::Default { value: value.to_string() },
                                expression: None,
                                callback_name: None,
                            });
                        }
                    }
                }

                // Parse compute = "expression"
                if let Some(compute_pos) = tokens.find("compute") {
                    let remaining = &tokens[compute_pos..];
                    if let Some(eq_pos) = remaining.find('=') {
                        let value_str = &remaining[eq_pos + 1..];
                        let end_pos = value_str.find(',').unwrap_or(value_str.len());
                        let value = value_str[..end_pos].trim().trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            let expr = value.to_string();
                            data_migration = Some(DataMigrationSpec {
                                migration_type: DataMigrationType::Compute { expression: expr.clone() },
                                expression: Some(expr),
                                callback_name: None,
                            });
                        }
                    }
                }

                // Parse data_migration = "function_name"
                if let Some(migrate_pos) = tokens.find("data_migration") {
                    let remaining = &tokens[migrate_pos..];
                    if let Some(eq_pos) = remaining.find('=') {
                        let value_str = &remaining[eq_pos + 1..];
                        let end_pos = value_str.find(',').unwrap_or(value_str.len());
                        let value = value_str[..end_pos].trim().trim_matches('"').trim_matches('\'');
                        if !value.is_empty() {
                            let func_name = value.to_string();
                            data_migration = Some(DataMigrationSpec {
                                migration_type: DataMigrationType::Callback { function_name: func_name.clone() },
                                expression: None,
                                callback_name: Some(func_name),
                            });
                        }
                    }
                }
            }
        }

        Ok((rename_from, data_migration, cast_as, decimal_precision))
    }

    /// Map Rust type to SQL type with optional decimal precision
    fn map_rust_type_to_sql_with_precision(rust_type: &str, decimal_precision: Option<(u32, u32)>) -> String {
        // Remove generic parameters and whitespace
        let clean_type = rust_type
            .split('<')
            .next()
            .unwrap_or(rust_type)
            .trim()
            .to_string();

        // If decimal precision is specified and type is Decimal-related, use it
        if decimal_precision.is_some() {
            let is_decimal_type = matches!(
                clean_type.as_str(),
                "rust_decimal::Decimal" | "Decimal" |
                "bigdecimal::BigDecimal" | "BigDecimal" |
                "num_bigint::BigInt" | "BigInt"
            );

            if is_decimal_type {
                if let Some((p, s)) = decimal_precision {
                    return format!("NUMERIC({}, {})", p, s);
                }
            }
        }

        // Otherwise use default mapping
        Self::map_rust_type_to_sql(rust_type)
    }

    /// Map Rust type to SQL type
    fn map_rust_type_to_sql(rust_type: &str) -> String {
        // Remove generic parameters and whitespace
        let clean_type = rust_type
            .split('<')
            .next()
            .unwrap_or(rust_type)
            .trim()
            .to_string();

        match clean_type.as_str() {
            "String" => "VARCHAR(500)".to_string(),
            "str" => "TEXT".to_string(),
            "i8" | "i16" => "SMALLINT".to_string(),
            "i32" => "INTEGER".to_string(),
            "i64" => "BIGINT".to_string(),
            "u8" | "u16" => "SMALLINT".to_string(),
            "u32" => "INTEGER".to_string(),
            "u64" => "BIGINT".to_string(),
            "f32" => "REAL".to_string(),
            "f64" => "DOUBLE PRECISION".to_string(),
            "bool" => "BOOLEAN".to_string(),
            "Vec" | "[]" => "JSONB".to_string(),
            "chrono::DateTime" | "DateTime" => "TIMESTAMPTZ".to_string(),
            "chrono::NaiveDate" | "NaiveDate" => "DATE".to_string(),
            "chrono::NaiveTime" | "NaiveTime" => "TIME".to_string(),
            "uuid::Uuid" | "Uuid" => "UUID".to_string(),
            "serde_json::Value" | "Value" | "JSON" => "JSONB".to_string(),
            "bytes::Bytes" | "Bytes" | "Vec" | "u8" => "BYTEA".to_string(),
            "rust_decimal::Decimal" | "Decimal" => "NUMERIC(18,6)".to_string(),
            "bigdecimal::BigDecimal" | "BigDecimal" => "NUMERIC(30,10)".to_string(),
            "num_bigint::BigInt" | "BigInt" => "NUMERIC".to_string(),
            _ => "VARCHAR(500)".to_string(), // Default to VARCHAR for unknown types
        }
    }

    /// Generate code to construct TableDef at compile time
    pub fn generate_table_def_code(schema: &StructSchema) -> TokenStream {
        let table_name = &schema.table_name;
        let rename_from = &schema.rename_from;
        let primary_key = &schema.primary_key;

        // Generate column definitions
        let column_defs: Vec<TokenStream> = schema.columns.iter()
            .map(|col| Self::generate_column_def_code(col))
            .collect();

        quote! {
            ::sqlx_struct_enhanced::migration::TableDef {
                name: #table_name.to_string(),
                rename_from: #rename_from.map(|s| s.to_string()),
                columns: vec![#(#column_defs),*],
                indexes: vec![],
                primary_key: #primary_key.to_string(),
            }
        }
    }

    /// Generate code for a single ColumnDef
    fn generate_column_def_code(column: &StructColumn) -> TokenStream {
        let name = &column.name;
        let sql_type = &column.sql_type;
        let nullable = column.nullable;
        let rename_from = &column.rename_from;

        // Handle data migration
        let data_migration_code = if let Some(spec) = &column.data_migration {
            match &spec.migration_type {
                DataMigrationType::Default { value } => {
                    quote! {
                        Some(::sqlx_struct_enhanced::migration::DataMigration {
                            migration_type: ::sqlx_struct_enhanced::migration::DataMigrationType::Default {
                                value: #value.to_string()
                            },
                            expression: None,
                            callback_name: None,
                        })
                    }
                }
                DataMigrationType::Compute { expression } => {
                    quote! {
                        Some(::sqlx_struct_enhanced::migration::DataMigration {
                            migration_type: ::sqlx_struct_enhanced::migration::DataMigrationType::Compute {
                                expression: #expression.to_string()
                            },
                            expression: Some(#expression.to_string()),
                            callback_name: None,
                        })
                    }
                }
                DataMigrationType::Callback { function_name } => {
                    quote! {
                        Some(::sqlx_struct_enhanced::migration::DataMigration {
                            migration_type: ::sqlx_struct_enhanced::migration::DataMigrationType::Callback {
                                function_name: #function_name.to_string()
                            },
                            expression: None,
                            callback_name: Some(#function_name.to_string()),
                        })
                    }
                }
            }
        } else {
            quote! { None }
        };

        quote! {
            ::sqlx_struct_enhanced::migration::ColumnDef {
                name: #name.to_string(),
                sql_type: #sql_type.to_string(),
                nullable: #nullable,
                default: None,
                rename_from: #rename_from.map(|s| s.to_string()),
                data_migration: #data_migration_code,
            }
        }
    }
}

/// Convert PascalCase or camelCase to snake_case
fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_char_was_uppercase = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            // Add underscore before uppercase letter if:
            // 1. Not first character AND
            // 2. Previous char was lowercase OR next char is lowercase
            let next_is_lower = chars.peek().map(|c| c.is_lowercase()).unwrap_or(false);
            if !result.is_empty() && (!prev_char_was_uppercase || next_is_lower) {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
            prev_char_was_uppercase = true;
        } else {
            result.push(ch);
            prev_char_was_uppercase = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("getUser"), "get_user");
        assert_eq!(to_snake_case("APIResponse"), "api_response");
        assert_eq!(to_snake_case("user_profile"), "user_profile");
    }

    #[test]
    fn test_map_rust_type_to_sql() {
        assert_eq!(StructSchemaParser::map_rust_type_to_sql("String"), "VARCHAR(500)");
        assert_eq!(StructSchemaParser::map_rust_type_to_sql("i32"), "INTEGER");
        assert_eq!(StructSchemaParser::map_rust_type_to_sql("i64"), "BIGINT");
        assert_eq!(StructSchemaParser::map_rust_type_to_sql("bool"), "BOOLEAN");
        assert_eq!(StructSchemaParser::map_rust_type_to_sql("f64"), "DOUBLE PRECISION");
    }

    // Phase 1 tests for cast_as attribute
    #[test]
    fn test_struct_column_has_cast_as_field() {
        // Verify StructColumn with cast_as
        let column = StructColumn {
            name: "test_field".to_string(),
            rust_type: "String".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            rename_from: None,
            data_migration: None,
            cast_as: Some("TEXT".to_string()),
            decimal_precision: None,
        };

        assert_eq!(column.name, "test_field");
        assert_eq!(column.cast_as, Some("TEXT".to_string()));
    }

    #[test]
    fn test_struct_column_without_cast_as() {
        // Verify StructColumn without cast_as (backward compatibility)
        let column = StructColumn {
            name: "normal_field".to_string(),
            rust_type: "String".to_string(),
            sql_type: "TEXT".to_string(),
            nullable: false,
            rename_from: None,
            data_migration: None,
            cast_as: None,
            decimal_precision: None,
        };

        assert_eq!(column.name, "normal_field");
        assert_eq!(column.cast_as, None);
    }

    // Decimal precision tests
    #[test]
    fn test_struct_column_with_decimal_precision() {
        // Verify StructColumn with decimal precision
        let column = StructColumn {
            name: "price".to_string(),
            rust_type: "String".to_string(),
            sql_type: "NUMERIC(10,2)".to_string(),
            nullable: true,
            rename_from: None,
            data_migration: None,
            cast_as: Some("TEXT".to_string()),
            decimal_precision: Some((10, 2)),
        };

        assert_eq!(column.name, "price");
        assert_eq!(column.sql_type, "NUMERIC(10,2)");
        assert_eq!(column.decimal_precision, Some((10, 2)));
    }

    #[test]
    fn test_map_rust_type_to_sql_with_decimal_precision() {
        // Test that Decimal types map correctly
        assert_eq!(
            StructSchemaParser::map_rust_type_to_sql("rust_decimal::Decimal"),
            "NUMERIC(18,6)"
        );
        assert_eq!(
            StructSchemaParser::map_rust_type_to_sql("Decimal"),
            "NUMERIC(18,6)"
        );
        assert_eq!(
            StructSchemaParser::map_rust_type_to_sql("bigdecimal::BigDecimal"),
            "NUMERIC(30,10)"
        );
        assert_eq!(
            StructSchemaParser::map_rust_type_to_sql("BigDecimal"),
            "NUMERIC(30,10)"
        );
        assert_eq!(
            StructSchemaParser::map_rust_type_to_sql("num_bigint::BigInt"),
            "NUMERIC"
        );
    }

    #[test]
    fn test_map_rust_type_to_sql_with_custom_precision() {
        // Test custom precision override
        let result = StructSchemaParser::map_rust_type_to_sql_with_precision(
            "Decimal",
            Some((10, 2))
        );
        assert_eq!(result, "NUMERIC(10, 2)");  // Note: has spaces

        let result = StructSchemaParser::map_rust_type_to_sql_with_precision(
            "BigDecimal",
            Some((20, 4))
        );
        assert_eq!(result, "NUMERIC(20, 4)");

        // Test that non-decimal types ignore precision
        let result = StructSchemaParser::map_rust_type_to_sql_with_precision(
            "String",
            Some((10, 2))
        );
        assert_eq!(result, "VARCHAR(500)"); // Ignores precision for non-decimal types
    }

    #[test]
    fn test_map_rust_type_to_sql_without_custom_precision() {
        // Test default precision when None is provided
        let result = StructSchemaParser::map_rust_type_to_sql_with_precision(
            "Decimal",
            None
        );
        assert_eq!(result, "NUMERIC(18,6)"); // Uses default

        let result = StructSchemaParser::map_rust_type_to_sql_with_precision(
            "BigDecimal",
            None
        );
        assert_eq!(result, "NUMERIC(30,10)"); // Uses default
    }
}
