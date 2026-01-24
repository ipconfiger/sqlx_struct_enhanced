// Test JOIN with subquery alias resolution
//
// This example tests the fix for recursive alias extraction in JOIN queries with subqueries
// The issue was that subquery aliases (like m1, uc, ac, c) were not being resolved to actual table names
//
// Run: cargo build --example test_join_subquery_analysis
// Check the compiler output for index recommendations

fn main() {
    println!("Testing JOIN subquery alias resolution");
    println!("Check compiler output above for index recommendations");
}

#[sqlx_struct_macros::analyze_queries]
mod test_queries {
    // Mock struct for testing
    #[allow(dead_code)]
    struct Merchant {
        merchant_id: String,
        city_id: String,
        channel_id: String,
        audit_status: i32,
        ts: i64,
    }

    #[allow(dead_code)]
    struct MerchantChannel {
        merchant_id: String,
        channel_id: String,
    }

    #[allow(dead_code)]
    struct MerchantCouponType {
        merchant_id: String,
        coupon_type_id: String,
    }

    #[allow(dead_code)]
    struct CouponType {
        coupon_type_id: String,
        ts: i64,
    }

    #[allow(dead_code)]
    impl Merchant {
        // Test case 1: JOIN with subquery containing aliases
        // This should generate indexes on actual table names, NOT aliases
        //
        // Expected behavior:
        // - Extract aliases: m->merchant, mc->merchant_channel, m1->merchant_coupon_type, c->coupon_type
        // - Generate indexes on:
        //   - "merchant" for: merchant_id, city_id, audit_status, ts
        //   - "merchant_channel" for: merchant_id, channel_id
        //   - "merchant_coupon_type" for: merchant_id, coupon_type_id
        //   - "coupon_type" for: coupon_type_id, ts
        // NOT on: "m1", "c", "uc", "ac" (these are aliases!)
        fn test_join_with_subquery(city_id: &str, channel_id: &str) {
            // This query string will be analyzed by the macro
            let _ = "Merchant::make_query!(\"SELECT m.* FROM [Self] AS m JOIN merchant_channel AS mc ON mc.merchant_id = m.merchant_id WHERE m.merchant_id in (SELECT m1.merchant_id FROM merchant_coupon_type as m1 JOIN coupon_type as c ON m1.coupon_type_id = c.coupon_type_id GROUP BY m1.merchant_id ORDER BY MAX(c.ts) DESC LIMIT 20) AND m.city_id = $1 AND m.audit_status > 0 AND mc.channel_id = $2 ORDER BY ts DESC\")";
            let _ = city_id;
            let _ = channel_id;
        }

        // Test case 2: Multiple subqueries with different aliases
        fn test_multiple_subqueries(user_id: &str, activity_id: &str) {
            // This query pattern is from tongue project
            // Expected indexes on actual table names:
            // - "user_coupon" for: coupon_id, user_id
            // - "activity_coupon" for: coupon_type_id, activity_id
            // NOT on: "uc", "ac" (these are aliases!)
            let _ = "CouponPair::make_query!(\"SELECT cp.* FROM [Self] as cp JOIN activity_coupon as ac ON cp.coupon_type_id = ac.coupon_type_id JOIN user_coupon as uc ON cp.coupon_id = uc.coupon_id WHERE uc.user_id = $1 AND ac.activity_id = $2\")";
            let _ = user_id;
            let _ = activity_id;
        }
    }
}
