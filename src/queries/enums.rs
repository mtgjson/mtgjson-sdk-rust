//! Enum/keyword queries backed by JSON files loaded via the cache manager.
//!
//! These queries operate on `Keywords.json`, `CardTypes.json`, and `EnumValues.json`
//! and do not require DuckDB at all.

use serde_json::Value;

use crate::connection::Connection;
use crate::error::Result;

// ---------------------------------------------------------------------------
// EnumQuery
// ---------------------------------------------------------------------------

/// Query interface for MTGJSON enum/keyword data backed by cached JSON files.
pub struct EnumQuery<'a> {
    conn: &'a Connection,
}

impl<'a> EnumQuery<'a> {
    /// Create a new `EnumQuery` bound to the given connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get all keyword categories.
    ///
    /// Loads `Keywords.json` and returns its `data` payload. The resulting object
    /// has keys like `"abilityWords"`, `"keywordAbilities"`, `"keywordActions"`, each
    /// mapping to an array of strings.
    pub fn keywords(&self) -> Result<Value> {
        let data = self.conn.cache.borrow_mut().load_json("keywords")?;
        Ok(extract_data(data))
    }

    /// Get all card type definitions.
    ///
    /// Loads `CardTypes.json` and returns its `data` payload. The resulting object
    /// has keys for each card type (e.g., `"creature"`, `"instant"`, `"land"`), each
    /// containing `subTypes` and `superTypes` arrays.
    pub fn card_types(&self) -> Result<Value> {
        let data = self.conn.cache.borrow_mut().load_json("card_types")?;
        Ok(extract_data(data))
    }

    /// Get the full enum values reference.
    ///
    /// Loads `EnumValues.json` and returns its `data` payload. Contains all valid
    /// enum values used across the MTGJSON data model.
    pub fn enum_values(&self) -> Result<Value> {
        let data = self.conn.cache.borrow_mut().load_json("enum_values")?;
        Ok(extract_data(data))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the `"data"` field from a JSON wrapper, or return the value as-is
/// if there is no wrapper.
fn extract_data(value: Value) -> Value {
    match value {
        Value::Object(ref map) => {
            if let Some(data) = map.get("data") {
                data.clone()
            } else {
                value
            }
        }
        _ => value,
    }
}
