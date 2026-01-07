# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`sqlx_struct_enhanced` is a Rust crate that provides auto-generated CRUD operations for SQLx through a derive macro. It generates type-safe SQL queries for PostgreSQL, MySQL, and SQLite based on struct definitions.

## Architecture

This is a Cargo workspace with two members:
- `sqlx_struct_enhanced` - Main library with traits and SQL generation logic
- `sqlx_struct_macros` - Procedural macro crate that derives `EnhancedCrud`

### Key Components

1. **Derive Macro** (`sqlx_struct_macros/src/lib.rs`):
   - `EnhancedCrud` derive macro generates trait implementations
   - Uses `Schema` to extract table name (snake_case of struct name) and field names
   - Uses `SqlBuilder` to generate code that creates `Scheme` instances and binds parameters
   - Three separate implementations (Postgres, MySQL, SQLite) under feature flags

2. **Trait Definition** (`src/traits.rs`):
   - `EnhancedCrud` trait defines CRUD methods
   - Feature-gated for each database backend (Postgres, MySQL, SQLite)
   - Methods return SQLx `Query` or `QueryAs` types

3. **SQL Generation** (`src/lib.rs`):
   - `Scheme` struct: Contains table metadata and generates SQL strings
   - SQL caching via `Cache` (RwLock<HashMap>) to avoid regenerating
   - Database-specific parameter syntax: `$1, $2` (Postgres) vs `?` (MySQL/SQLite)
   - Field wrapping: `"field"` (Postgres) vs `` `field` `` (MySQL) vs `field` (SQLite)

4. **Compile-Time Index Analysis** (`sqlx_struct_macros/src/`):
   - **`analyze_queries`** attribute macro for automatic index recommendations
   - **`simple_parser.rs`**: Simplified SQL parser for extracting WHERE and ORDER BY columns
   - **`query_extractor.rs`**: Extracts query patterns and struct fields from code
   - **`compile_time_analyzer.rs`**: Main analysis logic that prints recommendations
   - Zero runtime overhead - all analysis happens at compile time

5. **Usage Pattern**:
   ```rust
   #[derive(EnhancedCrud)]
   struct MyTable { id: String, name: String, value: i32 }

   // Generates implementation with:
   // - insert_bind(), update_bind(), delete_bind() - instance methods
   // - by_pk(), make_query(), where_query(), count_query() - static methods
   // - Table name: "my_table" (auto-converted to snake_case)
   // - ID field: First field (id)

   // Index analysis:
   #[sqlx_struct_macros::analyze_queries]
   mod my_queries {
       #[derive(EnhancedCrud)]
       struct User { id: String, email: String }

       impl User {
           fn find_by_email(email: &str) {
               let _ = User::where_query!("email = $1");
               // During compilation, macro will recommend: CREATE INDEX idx_user_email ON User (email)
           }
       }
   }
   ```

## Development Commands

### Build
```bash
cargo build
```

### Build with specific database feature
```bash
cargo build --features postgres  # default
cargo build --features mysql
cargo build --features sqlite
```

### Run tests
```bash
cargo test
```

**Note**: Tests require a running PostgreSQL instance at `postgres://postgres:@127.0.0.1/test-sqlx-tokio`

### Run a specific test
```bash
cargo test test_something_async
```

### Check without building
```bash
cargo check
```

### Run compile-time index analysis example
```bash
cargo build --example compile_time_analysis
# View index recommendations in build output
```

## Important Implementation Details

- **First field is primary key**: The macro assumes the first struct field is the ID/primary key
- **Table naming**: Struct names are auto-converted to snake_case for table names
- **Memory leak**: SQL strings are leaked with `Box::leak()` to get `&'static str` required by SQLx
- **Parameter placeholder replacement**: `{}` in WHERE clauses is replaced with database-specific placeholders
- **Feature flag duplication**: The derive macro has near-identical code for each database backend (lines 6-77 for Postgres, 80-151 for MySQL, 154-225 for SQLite) - changes typically need to be applied to all three

## Known Issues in Code

- `src/traits.rs:35,47` - `count_query()` for MySQL and SQLite return `QueryAs` with `Postgres` type instead of their respective database types (copy-paste error)
- `src/lib.rs` - `Scheme` struct and `Cache` are marked `#[allow(dead_code)]` but are actually used by the macro-generated code

## Usage Documentation

**For detailed usage instructions and API reference, see:**

- **[USAGE.md](USAGE.md)** - Complete usage guide and API reference
  - Quick start guide
  - Complete API reference for all CRUD methods
  - Bulk operations documentation
  - Advanced features (custom table names, transactions)
  - Common patterns and best practices
  - Testing guidelines

- **[COMPILE_TIME_INDEX_ANALYSIS.md](COMPILE_TIME_INDEX_ANALYSIS.md)** - Compile-time index analysis guide 🆕
  - How `#[analyze_queries]` macro works
  - Index recommendation rules
  - Best practices for using index analysis
  - Integration with existing code
  - Limitations and future enhancements
- Troubleshooting guide
- Migration examples from raw SQLx

When working on projects that integrate this crate, refer to USAGE.md for comprehensive usage examples and patterns.
