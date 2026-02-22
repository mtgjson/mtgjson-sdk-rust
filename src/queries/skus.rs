//! TCGplayer SKU queries backed by the `TcgplayerSkus.json.gz` data loaded into DuckDB.
//!
//! Similar to prices, the SKU data is loaded (from `TcgplayerSkus.json.gz` via the cache
//! manager, flattened, and imported into a DuckDB `tcgplayer_skus` table) at SDK
//! initialization time before creating a `SkuQuery`.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// SkuQuery
// ---------------------------------------------------------------------------

/// Query interface for TCGplayer SKU data backed by a DuckDB table.
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
        let (sql, params) = SqlBuilder::new("tcgplayer_skus")
            .where_eq("uuid", uuid)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Find the card/SKU entry for a specific TCGplayer SKU ID.
    pub fn find_by_sku_id(&self, sku_id: &str) -> Result<Vec<Value>> {
        let (sql, params) = SqlBuilder::new("tcgplayer_skus")
            .where_eq("skuId", sku_id)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Find all SKUs for a given TCGplayer product ID.
    pub fn find_by_product_id(&self, product_id: &str) -> Result<Vec<Value>> {
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
