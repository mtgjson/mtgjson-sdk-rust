//! TCGplayer SKU queries backed by the `TcgplayerSkus.parquet` data loaded into DuckDB.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// SkuQuery
// ---------------------------------------------------------------------------

/// Query interface for TCGplayer SKU data backed by a DuckDB view.
pub struct SkuQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> SkuQuery<'a> {
    /// Create a new `SkuQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Get all SKUs for a card by its UUID.
    pub fn get(&self, uuid: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tcgplayer_skus"])?;

        let (sql, params) = SqlBuilder::new("tcgplayer_skus")
            .where_eq("uuid", uuid)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Find the card/SKU entry for a specific TCGplayer SKU ID.
    pub fn find_by_sku_id(&self, sku_id: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tcgplayer_skus"])?;

        let (sql, params) = SqlBuilder::new("tcgplayer_skus")
            .where_eq("skuId", sku_id)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Find all SKUs for a given TCGplayer product ID.
    pub fn find_by_product_id(&self, product_id: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["tcgplayer_skus"])?;

        let (sql, params) = SqlBuilder::new("tcgplayer_skus")
            .where_eq("productId", product_id)
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
