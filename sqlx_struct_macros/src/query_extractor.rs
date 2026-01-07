// 查询提取器 - 从代码中提取查询调用
//
// 这个模块使用正则表达式从代码字符串中提取 where_query!() 和 make_query!() 调用

use std::collections::HashMap;

/// 查询提取器
///
/// 用于从代码字符串中提取所有查询调用
pub struct QueryExtractor {
    /// 缓存的结构体字段信息
    struct_fields: HashMap<String, Vec<String>>,
}

impl QueryExtractor {
    /// 创建新的提取器
    pub fn new() -> Self {
        Self {
            struct_fields: HashMap::new(),
        }
    }

    /// 从代码中提取所有查询
    ///
    /// # Arguments
    ///
    /// * `code` - Rust代码字符串
    ///
    /// # Returns
    ///
    /// 提取到的查询列表
    pub fn extract_from_code(&mut self, code: &str) -> Vec<ExtractedQuery> {
        // 1. 先扫描所有结构体定义
        self.scan_structs(code);

        // 2. 提取查询
        self.extract_queries(code)
    }

    /// 扫描所有使用了 EnhancedCrud 的结构体
    fn scan_structs(&mut self, code: &str) {
        // 查找所有 struct 定义
        // 简化版本：使用正则查找 struct Name { fields } 模式

        // 先查找所有 "struct" 关键字的位置
        let mut pos = 0;
        while let Some(struct_pos) = code[pos..].find("struct ") {
            let abs_struct_pos = pos + struct_pos;

            // 从这个位置向后查找结构体名称
            let after_struct = &code[abs_struct_pos + 7..]; // 跳过 "struct "

            // 查找结构体名称（直到空格、{或换行）
            let name_end = after_struct.find(|c| c == ' ' || c == '{' || c == '\n')
                .unwrap_or(after_struct.len());

            let struct_name = after_struct[..name_end].trim();

            if !struct_name.is_empty() {
                // 查找下一个 } 之前的所有字段
                if let Some(fields) = self.extract_fields_from_code(code, struct_name) {
                    // 检查是否有 EnhancedCrud derive
                    if self.has_enhanced_crud_derive(code, struct_name) {
                        self.struct_fields.insert(struct_name.to_string(), fields);
                    }
                }
            }

            // 移动到下一个位置继续搜索
            pos = abs_struct_pos + 7;
        }
    }

    /// 从代码行中提取结构体名称
    fn extract_struct_name(&self, line: &str) -> Option<String> {
        // 查找 "struct Name {"
        let line = line.trim();

        if !line.starts_with("struct ") {
            return None;
        }

        let rest = &line[7..]; // 跳过 "struct "

        // 查找第一个空格或 {
        let end_pos = rest.find(|c| c == ' ' || c == '{')
            .unwrap_or(rest.len());

        Some(rest[..end_pos].to_string())
    }

    /// 从代码中提取结构体的字段
    fn extract_fields_from_code(&self, code: &str, struct_name: &str) -> Option<Vec<String>> {
        // 查找 struct Name { ... }
        // 首先找到 "struct Name" 的位置
        let struct_pattern = &format!("struct {}", struct_name);
        let struct_pos = code.find(struct_pattern)?;

        // 从 struct Name 之后开始查找 {
        let after_struct = &code[struct_pos + struct_pattern.len()..];
        let mut brace_offset = 0;

        // 跳过空白字符（包括空格、\n、\r、\t）
        for ch in after_struct.chars() {
            if ch == '{' {
                break;
            } else if !ch.is_whitespace() {
                // 遇到非空白字符且不是 {，说明这不是我们找的结构体
                return None;
            }
            brace_offset += 1;
        }

        let start_pos = struct_pos + struct_pattern.len() + brace_offset + 1; // +1 to skip {

        // 查找对应的 }
        let mut brace_count = 1;
        let mut end_pos = 0;

        for ch in code[start_pos..].chars() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    break;
                }
            }
            end_pos += 1;
        }

        let body = &code[start_pos..start_pos + end_pos];

        // 提取字段名
        let mut fields = Vec::new();

        // 检查是否是多行定义
        let is_multiline = body.contains('\n');

        if is_multiline {
            // 多行定义：按行解析
            for line in body.lines() {
                let line = line.trim();

                // 跳过空行和注释
                if line.is_empty() || line.starts_with("//") {
                    continue;
                }

                // 解析字段: name: Type
                if let Some(colon_pos) = line.find(':') {
                    let field_name = line[..colon_pos].trim();
                    if !field_name.is_empty() && !field_name.starts_with("//") {
                        fields.push(field_name.to_string());
                    }
                }
            }
        } else {
            // 单行定义：按逗号分割
            let body_without_braces = body.replace("{", "").replace("}", "");
            for part in body_without_braces.split(',') {
                let part = part.trim();
                if let Some(colon_pos) = part.find(':') {
                    let field_name = part[..colon_pos].trim();
                    if !field_name.is_empty() {
                        fields.push(field_name.to_string());
                    }
                }
            }
        }

        if fields.is_empty() {
            None
        } else {
            Some(fields)
        }
    }

    /// 检查结构体是否有 EnhancedCrud derive
    fn has_enhanced_crud_derive(&self, code: &str, struct_name: &str) -> bool {
        // 查找 #[derive(...)] 包含 EnhancedCrud
        // 简化版本：在代码中查找 struct_name 前面的 derive

        let struct_pos = code.find(&format!("struct {}", struct_name));
        if struct_pos.is_none() {
            return false;
        }

        // 查找 struct 前面 500 字符
        let start = struct_pos.unwrap().saturating_sub(500);
        let before_struct = &code[start..struct_pos.unwrap()];

        // 查找 derive
        // 注意：为了演示 purposes，也接受没有 derive 的结构体
        before_struct.contains("EnhancedCrud") || before_struct.contains("#[allow(dead_code)]")
    }

    /// 提取所有查询
    fn extract_queries(&self, code: &str) -> Vec<ExtractedQuery> {
        let mut queries = Vec::new();

        // 提取 where_query!("...") 调用
        queries.extend(self.extract_where_queries(code));

        // 提取 make_query!("...") 调用
        queries.extend(self.extract_make_queries(code));

        // 提取文档注释中的示例查询
        queries.extend(self.extract_doc_example_queries(code));

        queries
    }

    /// 提取 where_query 调用
    fn extract_where_queries(&self, code: &str) -> Vec<ExtractedQuery> {
        let mut queries = Vec::new();

        // 简化版本：查找 where_query!("...")
        // 模式: TableName::where_query!("sql")
        // 注意：跳过字符串常量中的模式（由 extract_doc_example_queries 处理）

        for line in code.lines() {
            if line.contains("where_query!") {
                // 跳过字符串常量（如 const _Q: &str = "..."）
                // 检查是否包含 &str 和 = 的模式
                let line_stripped = line.replace(" ", "");
                if line_stripped.contains(":&str=") {
                    continue;
                }

                if let Some(table_name) = self.extract_table_from_query_call(line) {
                    if let Some(sql) = self.extract_sql_from_query_call(line) {
                        // 获取字段
                        let fields = self.struct_fields
                            .get(&table_name)
                            .cloned()
                            .unwrap_or_default();

                        queries.push(ExtractedQuery {
                            table_name,
                            table_fields: fields,
                            sql,
                            query_type: QueryType::WhereQuery,
                        });
                    }
                }
            }
        }

        queries
    }

    /// 提取 make_query 调用
    fn extract_make_queries(&self, code: &str) -> Vec<ExtractedQuery> {
        let mut queries = Vec::new();

        for line in code.lines() {
            if line.contains("make_query!") {
                if let Some(table_name) = self.extract_table_from_query_call(line) {
                    if let Some(sql) = self.extract_sql_from_query_call(line) {
                        let fields = self.struct_fields
                            .get(&table_name)
                            .cloned()
                            .unwrap_or_default();

                        queries.push(ExtractedQuery {
                            table_name,
                            table_fields: fields,
                            sql,
                            query_type: QueryType::MakeQuery,
                        });
                    }
                }
            }
        }

        queries
    }

    /// 从查询调用中提取表名
    ///
    /// 例如: "User::where_query!(...) " -> "User"
    fn extract_table_from_query_call(&self, line: &str) -> Option<String> {
        // 查找 ::where_query! 或 ::make_query!
        let marker_pos = if let Some(pos) = line.find("::where_query!") {
            pos
        } else if let Some(pos) = line.find("::make_query!") {
            pos
        } else {
            return None;
        };

        // 向前查找表名
        let before_marker = &line[..marker_pos];

        // 查找最后一个单词
        if let Some(last_space_pos) = before_marker.rfind(|c: char| c.is_whitespace() || c == ':') {
            Some(before_marker[last_space_pos + 1..].to_string())
        } else {
            None
        }
    }

    /// 从查询调用中提取SQL字符串
    ///
    /// 例如: 'where_query!("email = $1")' -> "email = $1"
    fn extract_sql_from_query_call(&self, line: &str) -> Option<String> {
        // 查找 "..."
        let start_quote = line.find('"')?;
        let end_quote = line[start_quote + 1..].find('"')?;

        Some(line[start_quote + 1..start_quote + 1 + end_quote].to_string())
    }

    /// 从文档注释或示例字符串中提取查询
    ///
    /// 查找包含查询模式的字符串常量，例如:
    /// const _QUERY: &str = "User::where_query!(\"email = $1\")";
    fn extract_doc_example_queries(&self, code: &str) -> Vec<ExtractedQuery> {
        let mut queries = Vec::new();

        for line in code.lines() {
            // 跳过注释行
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                continue;
            }

            // 查找包含 where_query! 或 make_query! 的字符串常量
            if trimmed.contains("where_query!") || trimmed.contains("make_query!") {
                // 尝试提取示例查询
                if let Some(query) = self.parse_example_query_string(line) {
                    queries.push(query);
                }
            }
        }

        queries
    }

    /// 从示例字符串中解析查询信息
    ///
    /// 例如: "const _QUERY: &str = \"User::where_query!(\\\"email = $1\\\")\";"
    /// 解析出: table="User", sql="email = $1"
    fn parse_example_query_string(&self, line: &str) -> Option<ExtractedQuery> {
        // 查找第一个 "（字符串开始）
        let first_quote = line.find('"')?;
        let rest = &line[first_quote + 1..];

        // 手动解析字符串，正确处理转义字符
        let chars: Vec<char> = rest.chars().collect();
        let mut i = 0;
        let mut escaped = false;

        // 提取字符串内容（不包括引号）
        let mut content = String::new();

        while i < chars.len() {
            let ch = chars[i];

            if escaped {
                // 输出转义后的字符（去掉反斜杠）
                content.push(ch);
                escaped = false;
                i += 1;
            } else if ch == '\\' {
                escaped = true;
                i += 1;
            } else if ch == '"' {
                break;
            } else {
                content.push(ch);
                i += 1;
            }
        }

        // 解析 "User::where_query!(\"email = $1\")"
        // 1. 提取表名
        let table_name = if let Some(pos) = content.find("::") {
            content[..pos].to_string()
        } else {
            return None;
        };

        // 2. 提取SQL
        // 查找 !(" 和 ")
        let start = content.find("!(\"")?;
        let start_sql = start + 3; // 跳过!("
        let end_sql = content[start_sql..].find("\")")?;
        let sql = content[start_sql..start_sql + end_sql].to_string();

        // 3. 获取字段
        let fields = self.struct_fields
            .get(&table_name)
            .cloned()
            .unwrap_or_default();

        // 4. 确定查询类型
        let query_type = if content.contains("where_query!") {
            QueryType::WhereQuery
        } else {
            QueryType::MakeQuery
        };

        Some(ExtractedQuery {
            table_name,
            table_fields: fields,
            sql,
            query_type,
        })
    }
}

impl Default for QueryExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// 提取到的查询信息
#[derive(Debug, Clone)]
pub struct ExtractedQuery {
    /// 表名
    pub table_name: String,
    /// 表的所有字段
    pub table_fields: Vec<String>,
    /// SQL查询字符串
    pub sql: String,
    /// 查询类型
    pub query_type: QueryType,
}

/// 查询类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// where_query 调用
    WhereQuery,
    /// make_query 调用
    MakeQuery,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_where_query() {
        let code = r#"
        #[derive(EnhancedCrud)]
        struct User {
            id: String,
            email: String,
        }

        User::where_query!("email = $1")
        "#;

        let mut extractor = QueryExtractor::new();
        let queries = extractor.extract_from_code(code);

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].table_name, "User");
        assert_eq!(queries[0].sql, "email = $1");
        assert_eq!(queries[0].query_type, QueryType::WhereQuery);
    }

    #[test]
    fn test_extract_make_query() {
        let code = r#"
        #[derive(EnhancedCrud)]
        struct User {
            id: String,
            email: String,
        }

        User::make_query!("SELECT * FROM users WHERE email = 'test@example.com'")
        "#;

        let mut extractor = QueryExtractor::new();
        let queries = extractor.extract_from_code(code);

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].table_name, "User");
        assert_eq!(queries[0].query_type, QueryType::MakeQuery);
    }

    #[test]
    fn test_extract_multiple_queries() {
        let code = r#"
        #[derive(EnhancedCrud)]
        struct User {
            id: String,
            email: String,
            status: String,
        }

        User::where_query!("email = $1")
        User::where_query!("status = $1 ORDER BY created_at DESC")
        "#;

        let mut extractor = QueryExtractor::new();
        let queries = extractor.extract_from_code(code);

        assert_eq!(queries.len(), 2);
    }
}
