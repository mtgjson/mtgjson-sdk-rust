//! Set queries against the DuckDB-backed parquet data.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// SearchSetsParams
// ---------------------------------------------------------------------------

/// Parameters for the set search method.
#[derive(Debug, Clone, Default)]
pub struct SearchSetsParams {
    pub name: Option<String>,
    pub set_type: Option<String>,
    pub block: Option<String>,
    pub release_year: Option<i32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// ---------------------------------------------------------------------------
// SetQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG sets backed by the `sets` parquet view.
pub struct SetQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> SetQuery<'a> {
    /// Create a new `SetQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Get a single set by its code (case-insensitive -- uppercased before lookup).
    pub fn get(&self, code: &str) -> Result<Option<Value>> {
        self.conn.ensure_views(&["sets"])?;

        let upper = code.to_uppercase();
        let (sql, params) = SqlBuilder::new("sets")
            .where_eq("code", &upper)
            .limit(1)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows.into_iter().next().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
    }

    /// List all sets ordered by release date (descending).
    ///
    /// Supports optional filtering by `set_type` and `name` (substring match), plus
    /// `limit` / `offset` pagination.
    pub fn list(
        &self,
        set_type: Option<&str>,
        name: Option<&str>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["sets"])?;

        let mut qb = SqlBuilder::new("sets");
        qb.order_by(&["releaseDate DESC"]);

        if let Some(st) = set_type {
            qb.where_eq("type", st);
        }

        if let Some(n) = name {
            qb.where_like("name", &format!("%{}%", n));
        }

        if let Some(l) = limit {
            qb.limit(l);
        }
        if let Some(o) = offset {
            qb.offset(o);
        }

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Search sets using a combination of filters.
    ///
    /// - `name`: substring LIKE match
    /// - `set_type`: exact match on `type`
    /// - `block`: exact match on `block`
    /// - `release_year`: matches the year portion of `releaseDate`
    pub fn search(&self, params: &SearchSetsParams) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["sets"])?;

        let mut qb = SqlBuilder::new("sets");
        qb.order_by(&["releaseDate DESC"]);

        if let Some(ref name) = params.name {
            qb.where_like("name", &format!("%{}%", name));
        }

        if let Some(ref st) = params.set_type {
            qb.where_eq("type", st);
        }

        if let Some(ref block) = params.block {
            qb.where_eq("block", block);
        }

        if let Some(year) = params.release_year {
            qb.where_clause(
                "EXTRACT(YEAR FROM CAST(releaseDate AS DATE)) = ?",
                &[&year.to_string()],
            );
        }

        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        qb.limit(limit);
        qb.offset(offset);

        let (sql, sql_params) = qb.build();
        let rows = self.conn.execute(&sql, &sql_params)?;
        Ok(rows_to_values(rows))
    }

    /// Get a financial summary for the given set code.
    ///
    /// Requires the `prices_today` table to have been loaded.
    /// Returns a map with keys: `card_count`, `total_value`, `avg_value`,
    /// `min_value`, `max_value`, `date`.
    pub fn get_financial_summary(&self, set_code: &str) -> Result<HashMap<String, Value>> {
        self.conn.ensure_views(&["cards"])?;

        let upper = set_code.to_uppercase();

        let sql = r#"
            SELECT
                COUNT(DISTINCT c.uuid) AS card_count,
                COALESCE(SUM(p.price), 0) AS total_value,
                COALESCE(AVG(p.price), 0) AS avg_value,
                COALESCE(MIN(p.price), 0) AS min_value,
                COALESCE(MAX(p.price), 0) AS max_value,
                MAX(p.date) AS date
            FROM cards c
            JOIN prices_today p ON c.uuid = p.uuid
            WHERE c.setCode = ?
        "#;

        let rows = self.conn.execute(sql, &[upper])?;

        if let Some(row) = rows.into_iter().next() {
            Ok(row)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Count all sets, optionally filtered by `set_type`.
    pub fn count(&self, set_type: Option<&str>) -> Result<i64> {
        self.conn.ensure_views(&["sets"])?;

        let mut qb = SqlBuilder::new("sets");
        qb.select(&["COUNT(*) AS cnt"]);

        if let Some(st) = set_type {
            qb.where_eq("type", st);
        }

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;

        let cnt = rows
            .first()
            .and_then(|r| r.get("cnt"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(cnt)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rows_to_values(rows: Vec<HashMap<String, Value>>) -> Vec<Value> {
    rows.into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .collect()
}
