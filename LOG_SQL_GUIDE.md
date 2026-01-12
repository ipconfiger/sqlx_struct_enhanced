# log_sql Feature 使用指南

## 问题：看不到 SQL 输出

如果你启用了 `log_sql` feature 但看不到 SQL 输出，请按以下步骤检查：

## 解决方案

### 1. 在你的项目 Cargo.toml 中正确配置

**重要：必须同时声明 features 和路径**

```toml
[dependencies]
sqlx_struct_enhanced = { path = "../sqlx_struct_enhanced", features = ["postgres", "log_sql"] }
```

### 2. 完全重新编译

```bash
# 清理所有编译缓存
cargo clean

# 重新编译
cargo build --features log_sql

# 运行
cargo run --features log_sql
```

### 3. 如果使用 workspace

如果你的项目和 `sqlx_struct_enhanced` 在同一个 workspace 中：

**项目结构：**
```
workspace/
├── Cargo.toml          # workspace 根目录
├── sqlx_struct_enhanced/  # 库的位置
└── your_project/       # 你的项目
    └── Cargo.toml
```

**workspace/Cargo.toml:**
```toml
[workspace]
members = ["sqlx_struct_enhanced", "your_project"]

[workspace.dependencies]
sqlx_struct_enhanced = { path = "sqlx_struct_enhanced" }
```

**your_project/Cargo.toml:**
```toml
[dependencies]
sqlx_struct_enhanced = { workspace = true, features = ["postgres", "log_sql"] }
```

### 4. 验证是否生效

创建一个测试文件：

```rust
// test_log.rs
use sqlx::FromRow;
use sqlx::Row;
use sqlx_struct_enhanced::EnhancedCrud;

#[derive(Debug, Clone, FromRow, EnhancedCrud)]
struct TestUser {
    pub id: String,
    pub name: String,
}

fn main() {
    let mut user = TestUser {
        id: "test-id".to_string(),
        name: "Test".to_string(),
    };

    println!("Before insert_bind()");
    let _ = user.insert_bind();
    println!("After insert_bind() - check above for [SQLxEnhanced] log");
}
```

运行：
```bash
cargo run --test test_log --features log_sql 2>&1 | grep SQLxEnhanced
```

**应该看到：**
```
[SQLxEnhanced] INSERT SQL: INSERT INTO "test_user" ("id","name") VALUES ($1,$2)
```

## 常见问题

### Q: cargo expand 报错 "error: unused feature: log_sql"

**A:** 这通常是因为 cargo-expand 没有正确传递 feature。可以忽略这个警告，这不影响实际运行。

### Q: 重新编译后仍然看不到输出

**A:** 检查以下几点：

1. 确认你的代码确实调用了这些方法：
   - `insert_bind()`, `update_bind()`, `delete_bind()`
   - `YourStruct::by_pk()`, `YourStruct::where_query()`
   - `YourStruct::bulk_insert()`, `YourStruct::bulk_update()`

2. 确认你的宏 derive 是正确的：
   ```rust
   #[derive(EnhancedCrud)]
   struct YourStruct { ... }
   ```

3. 检查是否有 stderr 重定向（日志输出到 stderr）：
   ```bash
   # 默认应该能看到，如果被重定向到文件：
   cargo run --features log_sql 2> log.txt
   cat log.txt | grep SQLxEnhanced
   ```

### Q: 在我的库中看不到输出

**A:** 如果你的库 A 依赖 `sqlx_struct_enhanced`，而应用 B 依赖库 A：

**lib/Cargo.toml:**
```toml
[dependencies]
sqlx_struct_enhanced = { path = "../sqlx_struct_enhanced", features = ["postgres"] }
# 不要在这里声明 log_sql
```

**app/Cargo.toml:**
```toml
[dependencies]
your-lib = { path = "../lib" }
sqlx_struct_enhanced = { path = "../sqlx_struct_enhanced", features = ["postgres", "log_sql"] }
# 必须在最终应用中声明 log_sql
```

## 技术细节

`log_sql` feature 通过条件编译在宏生成的代码中插入 `eprintln!` 语句：

```rust
#[cfg(feature = "log_sql")]
eprintln!("[SQLxEnhanced] INSERT SQL: {}", sql);
```

因此：
1. 必须在编译时启用 feature（`--features log_sql`）
2. 必须完全重新编译才能生效（`cargo clean`）
3. 日志输出到 stderr，不会影响 stdout
4. 每次调用方法都会打印，方便调试

## 完整示例

参见 `examples/verify_log_sql.rs` 和 `examples/test_decimal_cast.rs`。
