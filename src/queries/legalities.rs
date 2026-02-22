//! Legality queries against the DuckDB-backed `card_legalities` view.
//!
//! The `card_legalities` view is expected to be an UNPIVOT'd version of the legalities
//! columns, producing rows of `(uuid, format, status)`.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// LegalityQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG card legalities across all formats.
pub struct LegalityQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> LegalityQuery<'a> {
    /// Create a new `LegalityQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Get all format/status pairs for a given card UUID.
    ///
    /// Returns a list of objects each containing `format` and `status` keys.
    pub fn formats_for_card(&self, uuid: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["card_legalities"])?;

        let (sql, params) = SqlBuilder::new("card_legalities")
            .where_eq("uuid", uuid)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Check whether a specific card is legal in a specific format.
    ///
    /// Returns `true` if the card's status for that format is `"Legal"`.
    pub fn is_legal(&self, uuid: &str, format: &str) -> Result<bool> {
        self.conn.ensure_views(&["card_legalities"])?;

        let (sql, params) = SqlBuilder::new("card_legalities")
            .select(&["COUNT(*) AS cnt"])
            .where_eq("uuid", uuid)
            .where_eq("format", format)
            .where_eq("status", "Legal")
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        let cnt = rows
            .first()
            .and_then(|r| r.get("cnt"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(cnt > 0)
    }

    /// Get all cards that are legal in the given format.
    ///
    /// Joins `card_legalities` with `cards` to return full card data.
    pub fn legal_in(&self, format: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "card_legalities"])?;

        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_legalities cl ON c.uuid = cl.uuid")
            .where_eq("cl.format", format)
            .where_eq("cl.status", "Legal")
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all cards that are banned in the given format.
    pub fn banned_in(&self, format: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "card_legalities"])?;

        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_legalities cl ON c.uuid = cl.uuid")
            .where_eq("cl.format", format)
            .where_eq("cl.status", "Banned")
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all cards that are restricted in the given format.
    pub fn restricted_in(&self, format: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "card_legalities"])?;

        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_legalities cl ON c.uuid = cl.uuid")
            .where_eq("cl.format", format)
            .where_eq("cl.status", "Restricted")
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all cards that are suspended in the given format.
    pub fn suspended_in(&self, format: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "card_legalities"])?;

        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_legalities cl ON c.uuid = cl.uuid")
            .where_eq("cl.format", format)
            .where_eq("cl.status", "Suspended")
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all cards that are not legal in the given format.
    pub fn not_legal_in(&self, format: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "card_legalities"])?;

        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_legalities cl ON c.uuid = cl.uuid")
            .where_eq("cl.format", format)
            .where_eq("cl.status", "Not Legal")
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
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
