//! Sealed product queries against the DuckDB-backed parquet data.
//!
//! Sealed product data lives in the `sets` table's `sealedProduct` column. This module
//! gracefully returns empty results if the column doesn't exist in the schema.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// SealedQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG sealed products derived from set data.
pub struct SealedQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> SealedQuery<'a> {
    /// Create a new `SealedQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Check whether the `sealedProduct` column exists on the `sets` table.
    fn has_sealed_column(&self) -> bool {
        // Try a lightweight probe query; if it fails, the column doesn't exist.
        let sql = "SELECT sealedProduct FROM sets LIMIT 0";
        self.conn.execute(sql, &[]).is_ok()
    }

    /// List all sealed products, optionally filtered by set code.
    ///
    /// Returns an empty vector if the `sealedProduct` column is not present.
    pub fn list(&self, set_code: Option<&str>) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["sets"])?;

        if !self.has_sealed_column() {
            return Ok(Vec::new());
        }

        let mut qb = SqlBuilder::new("sets");
        qb.select(&["code", "name", "sealedProduct"]);

        if let Some(sc) = set_code {
            let upper = sc.to_uppercase();
            qb.where_eq("code", &upper);
        }

        // Only include sets that actually have sealed product data
        qb.where_clause("sealedProduct IS NOT NULL", &[]);

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;

        // Flatten: each row may contain a list of sealed products under the
        // `sealedProduct` key. We extract and tag each product with the set code.
        let mut results: Vec<Value> = Vec::new();
        for row in rows {
            let code = row
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let set_name = row
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(Value::Array(products)) = row.get("sealedProduct") {
                for product in products {
                    let mut p = product.clone();
                    if let Value::Object(ref mut map) = p {
                        map.insert("setCode".to_string(), Value::String(code.clone()));
                        map.insert("setName".to_string(), Value::String(set_name.clone()));
                    }
                    results.push(p);
                }
            }
        }

        Ok(results)
    }

    /// Get sealed products for a specific set code.
    ///
    /// Returns an empty vector if the `sealedProduct` column is not present or the
    /// set has no sealed products.
    pub fn get(&self, set_code: &str) -> Result<Vec<Value>> {
        self.list(Some(set_code))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn rows_to_values(rows: Vec<HashMap<String, Value>>) -> Vec<Value> {
    rows.into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .collect()
}
