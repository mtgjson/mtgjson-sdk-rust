//! Card queries against the DuckDB-backed parquet data.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// SearchCardsParams
// ---------------------------------------------------------------------------

/// Parameters for the advanced card search.
///
/// All fields are optional. When `None`, the corresponding filter is skipped.
#[derive(Debug, Clone, Default)]
pub struct SearchCardsParams {
    pub name: Option<String>,
    pub fuzzy_name: Option<String>,
    pub localized_name: Option<String>,
    pub set_code: Option<String>,
    pub colors: Option<Vec<String>>,
    pub color_identity: Option<Vec<String>>,
    pub types: Option<String>,
    pub rarity: Option<String>,
    pub legal_in: Option<String>,
    pub mana_value: Option<f64>,
    pub mana_value_lte: Option<f64>,
    pub mana_value_gte: Option<f64>,
    pub text: Option<String>,
    pub text_regex: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub artist: Option<String>,
    pub keyword: Option<String>,
    pub is_promo: Option<bool>,
    pub availability: Option<String>,
    pub language: Option<String>,
    pub layout: Option<String>,
    pub set_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// ---------------------------------------------------------------------------
// CardQuery
// ---------------------------------------------------------------------------

/// Query interface for MTG cards backed by the `cards` parquet view.
pub struct CardQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> CardQuery<'a> {
    /// Create a new `CardQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    // -- Single card lookup ------------------------------------------------

    /// Retrieve a single card by its UUID.
    pub fn get_by_uuid(&self, uuid: &str) -> Result<Option<Value>> {
        self.conn.ensure_views(&["cards"])?;

        let (sql, params) = SqlBuilder::new("cards")
            .where_eq("uuid", uuid)
            .limit(1)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows.into_iter().next().map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
    }

    // -- Batch lookup ------------------------------------------------------

    /// Retrieve multiple cards by their UUIDs (preserves order where possible).
    pub fn get_by_uuids(&self, uuids: &[&str]) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards"])?;

        let (sql, params) = SqlBuilder::new("cards")
            .where_in("uuid", uuids)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    // -- Name lookup -------------------------------------------------------

    /// Get all printings of a card by exact name, optionally filtered by set code.
    pub fn get_by_name(&self, name: &str, set_code: Option<&str>) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards"])?;

        let mut qb = SqlBuilder::new("cards");
        qb.where_eq("name", name);

        if let Some(sc) = set_code {
            qb.where_eq("setCode", sc);
        }

        let (sql, params) = qb.build();
        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Alias for [`get_by_name`](Self::get_by_name) -- returns all printings of the card.
    pub fn get_printings(&self, name: &str) -> Result<Vec<Value>> {
        self.get_by_name(name, None)
    }

    // -- Atomic (oracle-level) lookup --------------------------------------

    /// Get a de-duplicated oracle-level card by name.
    ///
    /// De-duplicates by `(name, faceName)`. If no results are found for an exact name
    /// match, falls back to searching by `faceName`.
    pub fn get_atomic(&self, name: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards"])?;

        // First try: match by name, deduplicate by name + faceName
        let (sql, params) = SqlBuilder::new("cards")
            .select(&["DISTINCT ON (name, faceName) *"])
            .where_eq("name", name)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        if !rows.is_empty() {
            return Ok(rows_to_values(rows));
        }

        // Fallback: search by faceName
        let (sql2, params2) = SqlBuilder::new("cards")
            .select(&["DISTINCT ON (name, faceName) *"])
            .where_eq("faceName", name)
            .build();

        let rows2 = self.conn.execute(&sql2, &params2)?;
        Ok(rows_to_values(rows2))
    }

    // -- Cross-table lookups -----------------------------------------------

    /// Find cards by their Scryfall ID (joins `card_identifiers`).
    pub fn find_by_scryfall_id(&self, scryfall_id: &str) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards", "card_identifiers"])?;

        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_identifiers ci ON c.uuid = ci.uuid")
            .where_eq("ci.scryfallId", scryfall_id)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    // -- Random sampling ---------------------------------------------------

    /// Return `count` randomly-sampled cards.
    pub fn random(&self, count: usize) -> Result<Vec<Value>> {
        self.conn.ensure_views(&["cards"])?;

        let sql = format!("SELECT * FROM cards USING SAMPLE {}", count);
        let rows = self.conn.execute(&sql, &[])?;
        Ok(rows_to_values(rows))
    }

    // -- Count -------------------------------------------------------------

    /// Count cards, optionally filtered by the supplied column/value pairs.
    pub fn count(&self, filters: &HashMap<String, String>) -> Result<i64> {
        self.conn.ensure_views(&["cards"])?;

        let mut qb = SqlBuilder::new("cards");
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

    // -- Advanced search ---------------------------------------------------

    /// Search for cards using a rich set of optional filters.
    ///
    /// Translates each field of [`SearchCardsParams`] into appropriate SQL conditions
    /// (LIKE, exact match, fuzzy match, JOIN, list_contains, regexp, etc.).
    pub fn search(&self, params: &SearchCardsParams) -> Result<Vec<Value>> {
        // Determine which views we need
        let mut views: Vec<&str> = vec!["cards"];
        if params.legal_in.is_some() {
            views.push("card_legalities");
        }
        if params.localized_name.is_some() {
            views.push("card_foreign_data");
        }
        if params.set_type.is_some() {
            views.push("sets");
        }
        self.conn.ensure_views(&views)?;

        let mut qb = SqlBuilder::new("cards");

        // -- name: if contains '%' use LIKE, otherwise exact match ----------
        if let Some(ref name) = params.name {
            if name.contains('%') {
                qb.where_like("cards.name", name);
            } else {
                qb.where_eq("cards.name", name);
            }
        }

        // -- fuzzy_name: jaro_winkler_similarity >= 0.8 ---------------------
        if let Some(ref fuzzy) = params.fuzzy_name {
            qb.where_fuzzy("cards.name", fuzzy, 0.8);
            qb.order_by(&[&format!(
                "jaro_winkler_similarity(cards.name, '{}') DESC",
                fuzzy.replace('\'', "''")
            )]);
        }

        // -- localized_name: JOIN card_foreign_data -------------------------
        if let Some(ref loc_name) = params.localized_name {
            qb.join("JOIN card_foreign_data cfd ON cards.uuid = cfd.uuid");
            qb.where_like("cfd.name", &format!("%{}%", loc_name));
        }

        // -- set_code -------------------------------------------------------
        if let Some(ref sc) = params.set_code {
            qb.where_eq("cards.setCode", sc);
        }

        // -- colors: list_contains for each color ---------------------------
        if let Some(ref colors) = params.colors {
            for color in colors {
                qb.where_clause(
                    "list_contains(cards.colors, ?)",
                    &[color.as_str()],
                );
            }
        }

        // -- color_identity: list_contains for each color -------------------
        if let Some(ref ci) = params.color_identity {
            for color in ci {
                qb.where_clause(
                    "list_contains(cards.colorIdentity, ?)",
                    &[color.as_str()],
                );
            }
        }

        // -- types: LIKE %types% -------------------------------------------
        if let Some(ref types) = params.types {
            qb.where_like("cards.type", &format!("%{}%", types));
        }

        // -- rarity ---------------------------------------------------------
        if let Some(ref rarity) = params.rarity {
            qb.where_eq("cards.rarity", rarity);
        }

        // -- legal_in: JOIN card_legalities ---------------------------------
        if let Some(ref format_name) = params.legal_in {
            qb.join("JOIN card_legalities cl ON cards.uuid = cl.uuid");
            qb.where_eq("cl.format", format_name);
            qb.where_eq("cl.status", "Legal");
        }

        // -- mana_value (exact) --------------------------------------------
        if let Some(mv) = params.mana_value {
            qb.where_eq("cards.manaValue", &mv.to_string());
        }

        // -- mana_value_lte -------------------------------------------------
        if let Some(mv) = params.mana_value_lte {
            qb.where_lte("cards.manaValue", &mv.to_string());
        }

        // -- mana_value_gte -------------------------------------------------
        if let Some(mv) = params.mana_value_gte {
            qb.where_gte("cards.manaValue", &mv.to_string());
        }

        // -- text: LIKE %text% ---------------------------------------------
        if let Some(ref text) = params.text {
            qb.where_like("cards.text", &format!("%{}%", text));
        }

        // -- text_regex: regexp_matches -------------------------------------
        if let Some(ref regex) = params.text_regex {
            qb.where_regex("cards.text", regex);
        }

        // -- power ----------------------------------------------------------
        if let Some(ref power) = params.power {
            qb.where_eq("cards.power", power);
        }

        // -- toughness ------------------------------------------------------
        if let Some(ref toughness) = params.toughness {
            qb.where_eq("cards.toughness", toughness);
        }

        // -- artist ---------------------------------------------------------
        if let Some(ref artist) = params.artist {
            qb.where_like("cards.artist", &format!("%{}%", artist));
        }

        // -- keyword: list_contains(keywords, keyword) ----------------------
        if let Some(ref kw) = params.keyword {
            qb.where_clause(
                "list_contains(cards.keywords, ?)",
                &[kw.as_str()],
            );
        }

        // -- is_promo -------------------------------------------------------
        if let Some(promo) = params.is_promo {
            qb.where_eq("cards.isPromo", if promo { "true" } else { "false" });
        }

        // -- availability: list_contains ------------------------------------
        if let Some(ref avail) = params.availability {
            qb.where_clause(
                "list_contains(cards.availability, ?)",
                &[avail.as_str()],
            );
        }

        // -- language -------------------------------------------------------
        if let Some(ref lang) = params.language {
            qb.where_eq("cards.language", lang);
        }

        // -- layout ---------------------------------------------------------
        if let Some(ref layout) = params.layout {
            qb.where_eq("cards.layout", layout);
        }

        // -- set_type: JOIN sets --------------------------------------------
        if let Some(ref st) = params.set_type {
            qb.join("JOIN sets s ON cards.setCode = s.code");
            qb.where_eq("s.type", st);
        }

        // -- pagination -----------------------------------------------------
        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        qb.limit(limit);
        qb.offset(offset);

        let (sql, sql_params) = qb.build();
        let rows = self.conn.execute(&sql, &sql_params)?;
        Ok(rows_to_values(rows))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a vector of row HashMaps into `serde_json::Value` objects.
fn rows_to_values(rows: Vec<HashMap<String, Value>>) -> Vec<Value> {
    rows.into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .collect()
}
