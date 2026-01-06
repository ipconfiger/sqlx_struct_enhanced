/// Integration tests for Phase 3 features
///
/// This file demonstrates the new Phase 3 features:
/// - Custom table names
/// - Batch operations (when implemented)
/// - Transactions (when implemented)
#[cfg(feature = "postgres")]
use sqlx::postgres::Postgres;

#[cfg(feature = "postgres")]
use sqlx::query::{Query, QueryAs};

#[cfg(feature = "postgres")]
use sqlx::database::HasArguments;

use sqlx::FromRow;
use sqlx_struct_enhanced::{EnhancedCrud, Scheme};

// Example 1: Custom table name
#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "custom_users_table"]
struct CustomUser {
    id: String,
    username: String,
    email: String,
}

// Example 2: Default table name (would be "my_models")
#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct MyModel {
    id: String,
    name: String,
}

// Example 3: Table name with prefix
#[derive(Debug, Clone, FromRow, EnhancedCrud)]
#[table_name = "app_settings"]
struct Settings {
    key: String,
    value: String,
}

#[cfg(test)]
mod phase3_tests {
    use super::*;

    #[test]
    fn test_custom_table_name_macro() {
        // This test verifies that the macro compiles with custom table names
        // The actual SQL generation is tested in unit tests

        // Verify the structs can be created
        let user = CustomUser {
            id: "1".to_string(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        assert_eq!(user.id, "1");

        let model = MyModel {
            id: "1".to_string(),
            name: "test".to_string(),
        };
        assert_eq!(model.id, "1");

        let settings = Settings {
            key: "theme".to_string(),
            value: "dark".to_string(),
        };
        assert_eq!(settings.key, "theme");
    }

    #[test]
    fn test_custom_vs_default_table_names() {
        // CustomUser should use "custom_users_table"
        // MyModel should use "my_models" (snake_case)
        // Settings should use "app_settings"

        // These would be verified by the actual database queries
        // For now, we just verify compilation
    }
}

// Example usage documentation:
//
// #[derive(EnhancedCrud)]
// #[table_name = "my_custom_table"]
// struct MyStruct {
//     id: String,
//     field1: String,
// }
//
// Without the attribute, it would use "my_struct" (snake_case)
