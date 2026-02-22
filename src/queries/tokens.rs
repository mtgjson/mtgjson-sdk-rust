//! Token queries against the DuckDB-backed parquet data.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// SearchTokensParams
// ---------------------------------------------------------------------------

/// Parameters for the advanced token search.
#[derive(Debug, Clone, Default)]
pub struct SearchTokensParams {
    pub name: Option<String>,
    pub set_code: Option<String>,
    pub colors: Option<Vec<String>>,
    pub types: Option<String>,
    pub artist: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// ---------------------------------------------------------------------------
// TokenQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG tokens backed by the `tokens` parquet view.
pub struct TokenQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> TokenQuery<'a> {
    /// Create a new `TokenQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Retrieve a single token by its UUID.
    pub fn get_by_uuid(&self, uuid: &str) -> Result<Option<Value>> {
        self.conn.ensure_views(&["tokens"])?;

        let (sql, params) = SqlBuilder::new("tokens")
            .where_eq("uuid", uuid)
            .limit(1)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows.into_iter().next().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
    }

    /// Retrieve multiple tokens by their UUIDs.
    pub fn get_by_uuids(&self, uuids: &[&str]) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tokens"])?;

        let (sql, params) = SqlBuilder::new("tokens")
            .where_in("uuid", uuids)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all tokens with the given name, optionally filtered by set code.
    pub fn get_by_name(&self, name: &str, set_code: Option<&str>) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tokens"])?;

        let mut qb = SqlBuilder::new("tokens");
        qb.where_eq("name", name);

        if let Some(sc) = set_code {
            qb.where_eq("setCode", sc);
        }

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Search tokens using a combination of filters.
    pub fn search(&self, params: &SearchTokensParams) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tokens"])?;

        let mut qb = SqlBuilder::new("tokens");

        if let Some(ref name) = params.name {
            if name.contains('%') {
                qb.where_like("tokens.name", name);
            } else {
                qb.where_eq("tokens.name", name);
            }
        }

        if let Some(ref sc) = params.set_code {
            qb.where_eq("tokens.setCode", sc);
        }

        if let Some(ref colors) = params.colors {
            for color in colors {
                qb.where_clause(
                    "list_contains(tokens.colors, ?)",
                    &[color.as_str()],
                );
            }
        }

        if let Some(ref types) = params.types {
            qb.where_like("tokens.type", &format!("%{}%", types));
        }

        if let Some(ref artist) = params.artist {
            qb.where_like("tokens.artist", &format!("%{}%", artist));
        }

        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        qb.limit(limit);
        qb.offset(offset);

        let (sql, sql_params) = qb.build();
        let rows = self.conn.execute(&sql, &sql_params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all tokens for a specific set code.
    pub fn for_set(&self, set_code: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tokens"])?;

        let (sql, params) = SqlBuilder::new("tokens")
            .where_eq("setCode", set_code)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Count tokens, optionally filtered by the supplied column/value pairs.
    pub fn count(&self, filters: &HashMap<String, String>) -> Result<i64> {
        self.conn.ensure_views(&["tokens"])?;

        let mut qb = SqlBuilder::new("tokens");
        qb.select(&["COUNT(*) AS cnt"]);

        for (col, val) in filters {
            qb.where_eq(col, val);
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
