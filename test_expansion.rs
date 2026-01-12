use sqlx_struct_enhanced::EnhancedCrud;

#[derive(EnhancedCrud)]
struct TestStruct {
    id: String,
    
    #[crud(decimal(precision = 10, scale = 2))]
    #[crud(cast_as = "TEXT")]
    amount: Option<String>,
}
