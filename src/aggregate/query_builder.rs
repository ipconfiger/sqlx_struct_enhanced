//! Fluent query builder for SQL aggregation operations.
//!
//! This module provides a type-safe builder pattern for constructing
//! aggregation queries with SUM, AVG, COUNT, MIN, MAX, GROUP BY, HAVING,
//! ORDER BY, LIMIT/OFFSET, and JOIN support.

use sqlx::Database;
use std::marker::PhantomData;

use crate::{get_or_insert_sql, prepare_where};

/// Type of SQL join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER JOIN"),
            JoinType::Left => write!(f, "LEFT JOIN"),
            JoinType::Right => write!(f, "RIGHT JOIN"),
            JoinType::Full => write!(f, "FULL JOIN"),
        }
    }
}

/// Represents a JOIN operation in the query.
#[derive(Debug, Clone, PartialEq)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub condition: String,
}

/// Represents an aggregate function to apply to a column with optional alias.
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Sum(String, Option<String>),      // (column, alias)
    Avg(String, Option<String>),      // (column, alias)
    Count(Option<String>, Option<String>), // (column, alias) - None means COUNT(*)
    Min(String, Option<String>),      // (column, alias)
    Max(String, Option<String>),      // (column, alias)
}

/// Fluent query builder for aggregation queries.
///
/// # Example
///
/// ```ignore
/// use sqlx_struct_enhanced::EnhancedCrud;
///
/// #[derive(EnhancedCrud)]
/// struct Order {
///     id: String,
///     category: String,
///     amount: i32,
/// }
///
/// // Simple aggregation
/// let total: i64 = Order::agg_query()
///     .sum("amount")
///     .build()
///     .fetch_one(&pool)
///     .await?;
///
/// // GROUP BY with HAVING, ORDER BY, LIMIT
/// let results: Vec<(String, i64)> = Order::agg_query()
///     .where_("status = {}", &["active"])
///     .group_by("category")
///     .sum_as("amount", "total")
///     .having("total > {}", &[&1000i64])
///     .order_by("total", "DESC")
///     .limit(10)
///     .build()
///     .fetch_all(&pool)
///     .await?;
///
/// // With JOIN
/// let results: Vec<(String, i64)> = Order::agg_query()
///     .join("customer", "order.customer_id = customer.id")
///     .group_by("customer.region")
///     .sum("order.amount")
///     .build()
///     .fetch_all(&pool)
///     .await?;
/// ```
pub struct AggQueryBuilder<'a, DB: Database> {
    table_name: String,
    joins: Vec<Join>,
    aggregates: Vec<AggregateFunction>,
    group_by_columns: Vec<String>,
    where_clause: Option<String>,
    where_params: Vec<String>,
    having_clause: Option<String>,
    having_params: Vec<String>,
    order_by_clause: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phantom: PhantomData<&'a DB>,
}

impl<'a, DB: Database> AggQueryBuilder<'a, DB> {
    /// Creates a new aggregation query builder for the given table.
    pub fn new(table_name: String) -> Self {
        Self {
            table_name,
            joins: Vec::new(),
            aggregates: Vec::new(),
            group_by_columns: Vec::new(),
            where_clause: None,
            where_params: Vec::new(),
            having_clause: None,
            having_params: Vec::new(),
            order_by_clause: None,
            limit: None,
            offset: None,
            _phantom: PhantomData,
        }
    }

    /// Adds an INNER JOIN with the specified table and condition.
    ///
    /// # Arguments
    ///
    /// * `table` - The table name to join
    /// * `condition` - The join condition (e.g., "order.customer_id = customer.id")
    ///
    /// # Example
    ///
    /// ```ignore
    /// .join("customer", "order.customer_id = customer.id")
    /// ```
    pub fn join(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// Adds a LEFT JOIN with the specified table and condition.
    ///
    /// # Arguments
    ///
    /// * `table` - The table name to join
    /// * `condition` - The join condition
    ///
    /// # Example
    ///
    /// ```ignore
    /// .join_left("product", "order.product_id = product.id")
    /// ```
    pub fn join_left(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Left,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// Adds a RIGHT JOIN with the specified table and condition.
    ///
    /// # Arguments
    ///
    /// * `table` - The table name to join
    /// * `condition` - The join condition
    pub fn join_right(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Right,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// Adds a FULL JOIN with the specified table and condition.
    ///
    /// # Arguments
    ///
    /// * `table` - The table name to join
    /// * `condition` - The join condition
    pub fn join_full(mut self, table: &str, condition: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Full,
            table: table.to_string(),
            condition: condition.to_string(),
        });
        self
    }

    /// Adds a SUM aggregation for the specified column.
    pub fn sum(mut self, column: &str) -> Self {
        self.aggregates.push(AggregateFunction::Sum(column.to_string(), None));
        self
    }

    /// Adds a SUM aggregation with a custom alias.
    pub fn sum_as(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(AggregateFunction::Sum(column.to_string(), Some(alias.to_string())));
        self
    }

    /// Adds an AVG aggregation for the specified column.
    pub fn avg(mut self, column: &str) -> Self {
        self.aggregates.push(AggregateFunction::Avg(column.to_string(), None));
        self
    }

    /// Adds an AVG aggregation with a custom alias.
    pub fn avg_as(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(AggregateFunction::Avg(column.to_string(), Some(alias.to_string())));
        self
    }

    /// Adds a COUNT(*) aggregation.
    pub fn count(mut self) -> Self {
        self.aggregates.push(AggregateFunction::Count(None, None));
        self
    }

    /// Adds a COUNT(*) aggregation with a custom alias.
    pub fn count_as(mut self, alias: &str) -> Self {
        self.aggregates.push(AggregateFunction::Count(None, Some(alias.to_string())));
        self
    }

    /// Adds a COUNT(column) aggregation.
    pub fn count_column(mut self, column: &str) -> Self {
        self.aggregates.push(AggregateFunction::Count(Some(column.to_string()), None));
        self
    }

    /// Adds a COUNT(column) aggregation with a custom alias.
    pub fn count_column_as(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(AggregateFunction::Count(Some(column.to_string()), Some(alias.to_string())));
        self
    }

    /// Adds a MIN aggregation for the specified column.
    pub fn min(mut self, column: &str) -> Self {
        self.aggregates.push(AggregateFunction::Min(column.to_string(), None));
        self
    }

    /// Adds a MIN aggregation with a custom alias.
    pub fn min_as(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(AggregateFunction::Min(column.to_string(), Some(alias.to_string())));
        self
    }

    /// Adds a MAX aggregation for the specified column.
    pub fn max(mut self, column: &str) -> Self {
        self.aggregates.push(AggregateFunction::Max(column.to_string(), None));
        self
    }

    /// Adds a MAX aggregation with a custom alias.
    pub fn max_as(mut self, column: &str, alias: &str) -> Self {
        self.aggregates.push(AggregateFunction::Max(column.to_string(), Some(alias.to_string())));
        self
    }

    /// Adds a GROUP BY clause for the specified column.
    pub fn group_by(mut self, column: &str) -> Self {
        self.group_by_columns.push(column.to_string());
        self
    }

    /// Adds a WHERE clause with the given statement and parameters.
    ///
    /// The statement should use "{}" as parameter placeholders.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .where_("status = {} AND amount > {}", &["active", "100"])
    /// ```
    pub fn where_(mut self, clause: &str, params: &[&str]) -> Self {
        self.where_clause = Some(clause.to_string());
        self.where_params = params.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Adds a HAVING clause with the given statement and parameters.
    ///
    /// The statement should use "{}" as parameter placeholders.
    /// Typically used with aggregate functions and aliases.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .having("SUM(amount) > {}", &[&1000i64])
    /// .having("total > {}", &[&1000i64])  // When using sum_as("amount", "total")
    /// ```
    pub fn having(mut self, clause: &str, params: &[&dyn std::fmt::Display]) -> Self {
        self.having_clause = Some(clause.to_string());
        self.having_params = params.iter().map(|p| p.to_string()).collect();
        self
    }

    /// Adds an ORDER BY clause for the specified column and direction.
    ///
    /// # Arguments
    ///
    /// * `column` - The column name to order by (can be an alias)
    /// * `direction` - Either "ASC" or "DESC" (case-insensitive)
    ///
    /// # Example
    ///
    /// ```ignore
    /// .order_by("amount", "DESC")
    /// .order_by("total", "ASC")
    /// ```
    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        let dir = if direction.to_uppercase() == "DESC" {
            "DESC"
        } else {
            "ASC"
        };
        self.order_by_clause = Some(format!("{} {}", column, dir));
        self
    }

    /// Adds a LIMIT clause to restrict the number of results.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .limit(10)
    /// ```
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Adds an OFFSET clause to skip a number of results.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .offset(20)
    /// ```
    pub fn offset(mut self, n: usize) -> Self {
        self.offset = Some(n);
        self
    }

    /// Builds and returns the SQL query as a string.
    fn build_sql(&self) -> String {
        // Build SELECT clause
        let mut select_parts = Vec::new();

        // Add GROUP BY columns first
        for col in &self.group_by_columns {
            select_parts.push(col.clone());
        }

        // Add aggregate functions
        for agg in &self.aggregates {
            match agg {
                AggregateFunction::Sum(col, alias) => {
                    let expr = format!("SUM({})", col);
                    select_parts.push(if let Some(a) = alias {
                        format!("{} AS {}", expr, a)
                    } else {
                        expr
                    });
                }
                AggregateFunction::Avg(col, alias) => {
                    let expr = format!("AVG({})", col);
                    select_parts.push(if let Some(a) = alias {
                        format!("{} AS {}", expr, a)
                    } else {
                        expr
                    });
                }
                AggregateFunction::Count(None, alias) => {
                    let expr = "COUNT(*)".to_string();
                    select_parts.push(if let Some(a) = alias {
                        format!("{} AS {}", expr, a)
                    } else {
                        expr
                    });
                }
                AggregateFunction::Count(Some(col), alias) => {
                    let expr = format!("COUNT({})", col);
                    select_parts.push(if let Some(a) = alias {
                        format!("{} AS {}", expr, a)
                    } else {
                        expr
                    });
                }
                AggregateFunction::Min(col, alias) => {
                    let expr = format!("MIN({})", col);
                    select_parts.push(if let Some(a) = alias {
                        format!("{} AS {}", expr, a)
                    } else {
                        expr
                    });
                }
                AggregateFunction::Max(col, alias) => {
                    let expr = format!("MAX({})", col);
                    select_parts.push(if let Some(a) = alias {
                        format!("{} AS {}", expr, a)
                    } else {
                        expr
                    });
                }
            }
        }

        let select_clause = select_parts.join(", ");

        // Build FROM and JOIN clauses
        let mut from_clause = format!("FROM {}", self.table_name);
        for join in &self.joins {
            from_clause.push_str(&format!(" {} {} ON {}", join.join_type, join.table, join.condition));
        }

        // Build WHERE clause
        let where_clause = if let Some(ref clause) = self.where_clause {
            let prepared = prepare_where(clause, 1);
            format!("WHERE {}", prepared)
        } else {
            String::new()
        };

        // Build GROUP BY clause
        let group_by_clause = if !self.group_by_columns.is_empty() {
            format!("GROUP BY {}", self.group_by_columns.join(", "))
        } else {
            String::new()
        };

        // Build HAVING clause
        let mut param_offset = 1 + self.where_params.len();
        let having_clause = if let Some(ref clause) = self.having_clause {
            let prepared = prepare_where(clause, param_offset as i32);
            format!("HAVING {}", prepared)
        } else {
            String::new()
        };

        // Build ORDER BY clause
        let order_by_clause = if let Some(ref clause) = self.order_by_clause {
            format!("ORDER BY {}", clause)
        } else {
            String::new()
        };

        // Build LIMIT clause
        let limit_clause = if let Some(_n) = self.limit {
            param_offset += self.having_params.len();
            format!("LIMIT ${}", param_offset)
        } else {
            String::new()
        };

        // Build OFFSET clause
        let offset_clause = if let Some(_n) = self.offset {
            if self.limit.is_some() {
                param_offset += 1;
            } else {
                param_offset += self.having_params.len();
            }
            format!("OFFSET ${}", param_offset)
        } else {
            String::new()
        };

        // Combine all parts
        let mut sql = format!("SELECT {} {}", select_clause, from_clause);
        if !where_clause.is_empty() {
            sql.push_str(" ");
            sql.push_str(&where_clause);
        }
        if !group_by_clause.is_empty() {
            sql.push_str(" ");
            sql.push_str(&group_by_clause);
        }
        if !having_clause.is_empty() {
            sql.push_str(" ");
            sql.push_str(&having_clause);
        }
        if !order_by_clause.is_empty() {
            sql.push_str(" ");
            sql.push_str(&order_by_clause);
        }
        if !limit_clause.is_empty() {
            sql.push_str(" ");
            sql.push_str(&limit_clause);
        }
        if !offset_clause.is_empty() {
            sql.push_str(" ");
            sql.push_str(&offset_clause);
        }

        sql
    }

    /// Builds the query and returns a cached SQL string.
    pub fn build(&self) -> &'static str {
        let cache_key = format!(
            "{}-agg-joins-{:?}-{:?}-groupby-{:?}-where-{:?}-having-{:?}-orderby-{:?}-limit-{:?}-offset-{:?}",
            self.table_name,
            self.joins,
            self.aggregates,
            self.group_by_columns,
            self.where_clause,
            self.having_clause,
            self.order_by_clause,
            self.limit,
            self.offset
        );

        get_or_insert_sql(cache_key, || self.build_sql())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_join() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join("customers", "orders.customer_id = customers.id")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("SELECT SUM(orders.amount) FROM orders INNER JOIN customers ON orders.customer_id = customers.id"));
    }

    #[test]
    fn test_join_with_group_by() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join("customers", "orders.customer_id = customers.id")
            .group_by("customers.region")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("SELECT customers.region, SUM(orders.amount) FROM orders INNER JOIN customers ON orders.customer_id = customers.id GROUP BY customers.region"));
    }

    #[test]
    fn test_left_join() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join_left("products", "orders.product_id = products.id")
            .group_by("products.category")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("LEFT JOIN products ON orders.product_id = products.id"));
    }

    #[test]
    fn test_multiple_joins() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join("customers", "orders.customer_id = customers.id")
            .join("products", "orders.product_id = products.id")
            .group_by("customers.region")
            .group_by("products.category")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("INNER JOIN customers"));
        assert!(sql.contains("INNER JOIN products"));
        assert!(sql.contains("GROUP BY customers.region, products.category"));
    }

    #[test]
    fn test_join_with_where() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join("customers", "orders.customer_id = customers.id")
            .where_("customers.status = {}", &["active"])
            .group_by("customers.region")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("WHERE customers.status = $1"));
    }

    #[test]
    fn test_join_with_having() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join("customers", "orders.customer_id = customers.id")
            .group_by("customers.region")
            .sum_as("orders.amount", "total")
            .having("total > {}", &[&1000i64])
            .order_by("total", "DESC")
            .limit(10);

        let sql = builder.build();
        assert!(sql.contains("HAVING total > $1"));
        assert!(sql.contains("ORDER BY total DESC"));
        assert!(sql.contains("LIMIT $2"));
    }

    #[test]
    fn test_join_with_all_features() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join("customers", "orders.customer_id = customers.id")
            .join_left("products", "orders.product_id = products.id")
            .where_("customers.status = {} AND orders.amount > {}", &["active", "100"])
            .group_by("customers.region")
            .group_by("products.category")
            .sum_as("orders.amount", "total")
            .avg_as("orders.amount", "average")
            .having("total > {}", &[&500i64])
            .order_by("total", "DESC")
            .limit(10)
            .offset(20);

        let sql = builder.build();
        assert!(sql.contains("INNER JOIN customers"));
        assert!(sql.contains("LEFT JOIN products"));
        assert!(sql.contains("WHERE customers.status = $1 AND orders.amount > $2"));
        assert!(sql.contains("GROUP BY customers.region, products.category"));
        assert!(sql.contains("HAVING total > $3"));
        assert!(sql.contains("ORDER BY total DESC"));
        assert!(sql.contains("LIMIT $4"));
        assert!(sql.contains("OFFSET $5"));
    }

    #[test]
    fn test_right_join() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join_right("customers", "orders.customer_id = customers.id")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("RIGHT JOIN customers"));
    }

    #[test]
    fn test_full_join() {
        let builder = AggQueryBuilder::<sqlx::Postgres>::new("orders".to_string())
            .join_full("customers", "orders.customer_id = customers.id")
            .sum("orders.amount");

        let sql = builder.build();
        assert!(sql.contains("FULL JOIN customers"));
    }

    #[test]
    fn test_join_types() {
        assert_eq!(format!("{}", JoinType::Inner), "INNER JOIN");
        assert_eq!(format!("{}", JoinType::Left), "LEFT JOIN");
        assert_eq!(format!("{}", JoinType::Right), "RIGHT JOIN");
        assert_eq!(format!("{}", JoinType::Full), "FULL JOIN");
    }
}
