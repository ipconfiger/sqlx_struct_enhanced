# DECIMAL Support Feature - Quick Start Guide

## TL;DR

Add `#[sqlx(cast_as = "TEXT")]` attribute to automatically cast NUMERIC → String in EnhancedCrud queries.

## The Problem

```rust
// This FAILS at runtime:
#[derive(EnhancedCrud)]
pub struct User {
    pub id: Uuid,
    pub commission_rate: Option<String>,  // NUMERIC(5,2) in DB
}

let user = User::by_pk().bind(&id).fetch_one(&pool).await?;
// Error: NUMERIC type incompatible with String
```

## The Solution

```rust
// This WORKS:
#[derive(EnhancedCrud)]
pub struct User {
    pub id: Uuid,
    #[sqlx(cast_as = "TEXT")]
    pub commission_rate: Option<String>,
}

let user = User::by_pk().bind(&id).fetch_one(&pool).await?;
// ✅ Generates: SELECT id, commission_rate::TEXT as commission_rate FROM users...
```

## Impact

- Affects **33% of models** in typical business apps (financial, geospatial, scientific)
- Current workaround: Manual SQL queries (defeats EnhancedCrud's purpose)
- No backward compatibility issues (opt-in feature)

## Implementation

**Files to modify**:
1. `sqlx_struct_macros/src/struct_schema_parser.rs` - Parse `#[sqlx(cast_as)]` attribute
2. `sqlx_struct_enhanced/src/lib.rs` - Generate explicit column list with casting
3. `sqlx_struct_macros/src/lib.rs` - Pass column metadata to Scheme

**Estimated time**: 3-5 days

See full details in [FEATURE_REQUEST_DECIMAL_SUPPORT.md](./FEATURE_REQUEST_DECIMAL_SUPPORT.md)

## Quick Reference for Claude Code

If you're using Claude Code to implement this feature, start with:

1. **Read the full spec**: `FEATURE_REQUEST_DECIMAL_SUPPORT.md`
2. **Focus on Option A**: Field-level cast attribute (recommended approach)
3. **Key sections**:
   - "Detailed Implementation Plan (Option A)"
   - "Step 1-4" for specific code changes
   - "Test Cases" for validation

4. **Implementation priority**:
   - Step 1: Extend `StructColumn` struct
   - Step 2: Parse attribute in macro
   - Step 3: Generate column list with casting
   - Step 4: Update macro to pass metadata

## Related Files

- Full spec: `FEATURE_REQUEST_DECIMAL_SUPPORT.md`
- Real-world usage: `/Users/alex/Projects/workspace/sdb_project/backend/src/models/user.rs`
- Test case: `/Users/alex/Projects/workspace/sdb_project/backend/tests/test_minimal_enhanced_crud.rs`
