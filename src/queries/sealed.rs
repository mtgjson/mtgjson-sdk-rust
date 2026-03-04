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

    /// List all sealed products, optionally filtered by set code and/or category.
    ///
    /// Returns an empty vector if the `sealedProduct` column is not present.
    pub fn list(
        &self,
        set_code: Option<&str>,
        category: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Value>> {
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
        let limit = limit.unwrap_or(100);

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
                        // Apply category filter if specified
                        if let Some(cat) = category {
                            let product_cat = map
                                .get("category")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if product_cat != cat {
                                continue;
                            }
                        }

                        map.insert("setCode".to_string(), Value::String(code.clone()));
                        map.insert("setName".to_string(), Value::String(set_name.clone()));
                    }
                    results.push(p);
                    if results.len() >= limit {
                        return Ok(results);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get a single sealed product by its UUID.
    ///
    /// Returns `None` if the product is not found or the `sealedProduct` column
    /// is not present.
    pub fn get(&self, uuid: &str) -> Result<Option<Value>> {
        self.conn.ensure_views(&["sets"])?;

        if !self.has_sealed_column() {
            return Ok(None);
        }

        // Search across all sets for a sealed product with the given UUID
        let rows = self.conn.execute(
            "SELECT code, name, sealedProduct FROM sets WHERE sealedProduct IS NOT NULL",
            &[],
        )?;

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
                    let product_uuid = product
                        .get("uuid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if product_uuid == uuid {
                        let mut p = product.clone();
                        if let Value::Object(ref mut map) = p {
                            map.insert("setCode".to_string(), Value::String(code.clone()));
                            map.insert(
                                "setName".to_string(),
                                Value::String(set_name.clone()),
                            );
                        }
                        return Ok(Some(p));
                    }
                }
            }
        }

        Ok(None)
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
