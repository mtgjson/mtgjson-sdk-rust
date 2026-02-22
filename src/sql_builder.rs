//! SQL builder with parameterized query construction.
//!
//! All user-supplied values go through DuckDB's parameter binding (`?` placeholders),
//! never through string interpolation. Builder methods return `&mut Self` for chaining.
//!
//! # Example
//!
//! ```rust
//! use mtgjson_sdk::SqlBuilder;
//! let (sql, params) = SqlBuilder::new("cards")
//!     .where_eq("setCode", "MH3")
//!     .where_like("name", "Lightning%")
//!     .order_by(&["name ASC"])
//!     .limit(10)
//!     .build();
//! ```

/// Builds parameterized SQL queries safely.
///
/// All user-supplied values go through DuckDB's parameter binding (`?` placeholders),
/// never through string interpolation. Methods return `&mut Self` for chaining.
pub struct SqlBuilder {
    select_cols: Vec<String>,
    is_distinct: bool,
    from_table: String,
    joins: Vec<String>,
    where_clauses: Vec<String>,
    params: Vec<String>,
    group_by_cols: Vec<String>,
    having_clauses: Vec<String>,
    order_by_cols: Vec<String>,
    limit_val: Option<usize>,
    offset_val: Option<usize>,
}

impl SqlBuilder {
    /// Create a builder targeting the given table or view.
    pub fn new(table: &str) -> Self {
        Self {
            select_cols: vec!["*".to_string()],
            is_distinct: false,
            from_table: table.to_string(),
            joins: Vec::new(),
            where_clauses: Vec::new(),
            params: Vec::new(),
            group_by_cols: Vec::new(),
            having_clauses: Vec::new(),
            order_by_cols: Vec::new(),
            limit_val: None,
            offset_val: None,
        }
    }

    /// Set the columns to select (replaces the default `*`).
    pub fn select(&mut self, cols: &[&str]) -> &mut Self {
        self.select_cols = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Add DISTINCT to the SELECT clause.
    pub fn distinct(&mut self) -> &mut Self {
        self.is_distinct = true;
        self
    }

    /// Add a JOIN clause.
    ///
    /// The clause should be a full JOIN expression, e.g.
    /// `"JOIN sets s ON cards.setCode = s.code"`.
    pub fn join(&mut self, clause: &str) -> &mut Self {
        self.joins.push(clause.to_string());
        self
    }

    /// Add a WHERE condition with `?` placeholders for each param.
    ///
    /// The caller provides a condition using `?` for each parameter value.
    /// Parameters are appended in order.
    pub fn where_clause(&mut self, condition: &str, params: &[&str]) -> &mut Self {
        self.where_clauses.push(condition.to_string());
        self.params.extend(params.iter().map(|p| p.to_string()));
        self
    }

    /// Add a case-insensitive LIKE condition.
    ///
    /// Generates: `LOWER({column}) LIKE LOWER(?)`
    pub fn where_like(&mut self, column: &str, value: &str) -> &mut Self {
        self.where_clauses
            .push(format!("LOWER({}) LIKE LOWER(?)", column));
        self.params.push(value.to_string());
        self
    }

    /// Add an IN condition with parameterized values.
    ///
    /// Empty values list produces `FALSE`.
    pub fn where_in(&mut self, column: &str, values: &[&str]) -> &mut Self {
        if values.is_empty() {
            self.where_clauses.push("FALSE".to_string());
            return self;
        }
        let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
        self.where_clauses
            .push(format!("{} IN ({})", column, placeholders.join(", ")));
        self.params.extend(values.iter().map(|v| v.to_string()));
        self
    }

    /// Add an equality condition: `{column} = ?`.
    pub fn where_eq(&mut self, column: &str, value: &str) -> &mut Self {
        self.where_clauses
            .push(format!("{} = ?", column));
        self.params.push(value.to_string());
        self
    }

    /// Add a greater-than-or-equal condition: `{column} >= ?`.
    pub fn where_gte(&mut self, column: &str, value: &str) -> &mut Self {
        self.where_clauses
            .push(format!("{} >= ?", column));
        self.params.push(value.to_string());
        self
    }

    /// Add a less-than-or-equal condition: `{column} <= ?`.
    pub fn where_lte(&mut self, column: &str, value: &str) -> &mut Self {
        self.where_clauses
            .push(format!("{} <= ?", column));
        self.params.push(value.to_string());
        self
    }

    /// Add a regex match condition using DuckDB's `regexp_matches`.
    ///
    /// Generates: `regexp_matches({column}, ?)`
    pub fn where_regex(&mut self, column: &str, pattern: &str) -> &mut Self {
        self.where_clauses
            .push(format!("regexp_matches({}, ?)", column));
        self.params.push(pattern.to_string());
        self
    }

    /// Add a fuzzy string match condition using Jaro-Winkler similarity.
    ///
    /// Generates: `jaro_winkler_similarity({column}, ?) > {threshold}`
    ///
    /// The threshold must be between 0.0 and 1.0 (inclusive).
    pub fn where_fuzzy(&mut self, column: &str, value: &str, threshold: f64) -> &mut Self {
        self.where_clauses.push(format!(
            "jaro_winkler_similarity({}, ?) > {}",
            column, threshold
        ));
        self.params.push(value.to_string());
        self
    }

    /// Add OR-combined conditions.
    ///
    /// Each condition is a `(sql_fragment, param_value)` tuple where the fragment
    /// uses `?` as a placeholder.
    ///
    /// # Example
    ///
    /// ```rust
    /// use mtgjson_sdk::SqlBuilder;
    /// let mut builder = SqlBuilder::new("cards");
    /// builder.where_or(&[("name = ?", "Bolt"), ("name = ?", "Counter")]);
    /// // -> WHERE (name = ? OR name = ?)
    /// ```
    pub fn where_or(&mut self, conditions: &[(&str, &str)]) -> &mut Self {
        if conditions.is_empty() {
            return self;
        }
        let mut or_parts = Vec::with_capacity(conditions.len());
        for (cond, param) in conditions {
            or_parts.push(cond.to_string());
            self.params.push(param.to_string());
        }
        self.where_clauses
            .push(format!("({})", or_parts.join(" OR ")));
        self
    }

    /// Add GROUP BY columns.
    pub fn group_by(&mut self, cols: &[&str]) -> &mut Self {
        self.group_by_cols
            .extend(cols.iter().map(|c| c.to_string()));
        self
    }

    /// Add a HAVING condition with `?` placeholders.
    pub fn having(&mut self, condition: &str, params: &[&str]) -> &mut Self {
        self.having_clauses.push(condition.to_string());
        self.params.extend(params.iter().map(|p| p.to_string()));
        self
    }

    /// Add ORDER BY clauses (e.g. `"name ASC"`, `"price DESC"`).
    pub fn order_by(&mut self, clauses: &[&str]) -> &mut Self {
        self.order_by_cols
            .extend(clauses.iter().map(|c| c.to_string()));
        self
    }

    /// Set the maximum number of rows to return.
    pub fn limit(&mut self, n: usize) -> &mut Self {
        self.limit_val = Some(n);
        self
    }

    /// Set the number of rows to skip before returning results.
    pub fn offset(&mut self, n: usize) -> &mut Self {
        self.offset_val = Some(n);
        self
    }

    /// Build the final SQL string and parameter list.
    ///
    /// Returns a tuple of `(sql_string, params_list)` ready for execution.
    pub fn build(&self) -> (String, Vec<String>) {
        let distinct = if self.is_distinct { "DISTINCT " } else { "" };
        let cols = self.select_cols.join(", ");
        let mut parts = vec![
            format!("SELECT {}{}", distinct, cols),
            format!("FROM {}", self.from_table),
        ];

        for j in &self.joins {
            parts.push(j.clone());
        }

        if !self.where_clauses.is_empty() {
            parts.push(format!("WHERE {}", self.where_clauses.join(" AND ")));
        }

        if !self.group_by_cols.is_empty() {
            parts.push(format!("GROUP BY {}", self.group_by_cols.join(", ")));
        }

        if !self.having_clauses.is_empty() {
            parts.push(format!("HAVING {}", self.having_clauses.join(" AND ")));
        }

        if !self.order_by_cols.is_empty() {
            parts.push(format!("ORDER BY {}", self.order_by_cols.join(", ")));
        }

        if let Some(n) = self.limit_val {
            parts.push(format!("LIMIT {}", n));
        }

        if let Some(n) = self.offset_val {
            parts.push(format!("OFFSET {}", n));
        }

        (parts.join("\n"), self.params.clone())
    }
}
