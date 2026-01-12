// Simple test to check if macro expansion works
// This test verifies that DECIMAL helper methods are generated

// NOTE: The unit test for extract_decimal_fields is disabled because
// proc-macro crates cannot export non-macro items. The functionality
// is tested indirectly through integration tests in decimal_bind_test.rs
// and other integration tests that use the EnhancedCrud derive macro.

#[cfg(test)]
mod tests {
    // Test that the extract_decimal_fields function works correctly
    // DISABLED: Cannot access private modules from proc-macro crates
    /*
    #[test]
    fn test_extract_decimal_fields_unit() {
        use sqlx_struct_macros::decimal_helpers::extract_decimal_fields;
        use syn::parse_str;

        let input_str = r#"
            struct TestOrder {
                id: String,
                #[crud(decimal(precision = 10, scale = 2))]
                #[crud(cast_as = "TEXT")]
                amount: Option<String>,
                #[crud(decimal(precision = 5, scale = 2))]
                rate: Option<String>,
            }
        "#;

        let input: syn::DeriveInput = parse_str(input_str).unwrap();
        let fields = extract_decimal_fields(&input);

        // Verify that both DECIMAL fields were extracted
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name.to_string(), "amount");
        assert_eq!(fields[0].precision, 10);
        assert_eq!(fields[0].scale, 2);

        assert_eq!(fields[1].name.to_string(), "rate");
        assert_eq!(fields[1].precision, 5);
        assert_eq!(fields[1].scale, 2);
    }
    */
}

