//! Deck queries backed by the `DeckList.json` file loaded via the cache manager.
//!
//! Unlike parquet-backed queries, deck data is stored as a JSON array in memory.
//! The `DeckQuery` loads the deck list from the cache on first access and performs
//! in-memory filtering.

use serde_json::Value;

use crate::connection::Connection;
use crate::error::Result;

// ---------------------------------------------------------------------------
// DeckQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG decks backed by the cached `DeckList.json` data.
pub struct DeckQuery<'a> {
    conn: &'a Connection,
}

impl<'a> DeckQuery<'a> {
    /// Create a new `DeckQuery` bound to the given connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Load the deck list from the cache.
    ///
    /// Returns a `Vec<Value>` representing the array of deck objects.
    fn load_decks(&self) -> Result<Vec<Value>> {
        let data = self.conn.cache.borrow_mut().load_json("deck_list")?;
        match data {
            Value::Object(map) => {
                // DeckList.json has { "data": [...] } structure
                if let Some(Value::Array(arr)) = map.get("data") {
                    Ok(arr.clone())
                } else {
                    // Try the top-level value as an array
                    Ok(Vec::new())
                }
            }
            Value::Array(arr) => Ok(arr),
            _ => Ok(Vec::new()),
        }
    }

    /// List all decks, optionally filtered by set code and/or deck type.
    pub fn list(
        &self,
        set_code: Option<&str>,
        deck_type: Option<&str>,
    ) -> Result<Vec<Value>> {
        let decks = self.load_decks()?;

        let filtered: Vec<Value> = decks
            .into_iter()
            .filter(|d| {
                if let Some(sc) = set_code {
                    let matches = d
                        .get("code")
                        .and_then(|v| v.as_str())
                        .map(|c| c.eq_ignore_ascii_case(sc))
                        .unwrap_or(false);
                    if !matches {
                        return false;
                    }
                }
                if let Some(dt) = deck_type {
                    let matches = d
                        .get("type")
                        .and_then(|v| v.as_str())
                        .map(|t| t.eq_ignore_ascii_case(dt))
                        .unwrap_or(false);
                    if !matches {
                        return false;
                    }
                }
                true
            })
            .collect();

        Ok(filtered)
    }

    /// Search for decks by name substring, optionally filtered by set code.
    pub fn search(&self, name: &str, set_code: Option<&str>) -> Result<Vec<Value>> {
        let decks = self.load_decks()?;
        let name_lower = name.to_lowercase();

        let filtered: Vec<Value> = decks
            .into_iter()
            .filter(|d| {
                let name_match = d
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|n| n.to_lowercase().contains(&name_lower))
                    .unwrap_or(false);
                if !name_match {
                    return false;
                }
                if let Some(sc) = set_code {
                    let sc_match = d
                        .get("code")
                        .and_then(|v| v.as_str())
                        .map(|c| c.eq_ignore_ascii_case(sc))
                        .unwrap_or(false);
                    if !sc_match {
                        return false;
                    }
                }
                true
            })
            .collect();

        Ok(filtered)
    }

    /// Count decks, optionally filtered by set code and/or deck type.
    pub fn count(
        &self,
        set_code: Option<&str>,
        deck_type: Option<&str>,
    ) -> Result<usize> {
        let filtered = self.list(set_code, deck_type)?;
        Ok(filtered.len())
    }
}
