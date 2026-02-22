//! Identifier queries that join `card_identifiers` with `cards`.
//!
//! Provides a generic `find_by` method plus 16 named convenience methods for every
//! known identifier column (Scryfall, TCGplayer, MTGO, etc.).

use std::collections::HashMap;

use serde_json::Value;

use crate::error::{MtgjsonError, Result};
use crate::sql_builder::SqlBuilder;

// ---------------------------------------------------------------------------
// Known identifier columns
// ---------------------------------------------------------------------------

/// All identifier column names that exist in the `card_identifiers` parquet table.
pub const KNOWN_ID_COLUMNS: &[&str] = &[
    "cardKingdomEtchedId",
    "cardKingdomFoilId",
    "cardKingdomId",
    "cardsphereId",
    "cardsphereFoilId",
    "mcmId",
    "mcmMetaId",
    "mtgArenaId",
    "mtgjsonFoilVersionId",
    "mtgjsonNonFoilVersionId",
    "mtgjsonV4Id",
    "mtgoFoilId",
    "mtgoId",
    "multiverseId",
    "scryfallId",
    "scryfallCardBackId",
    "scryfallIllustrationId",
    "scryfallOracleId",
    "tcgplayerEtchedProductId",
    "tcgplayerProductId",
];

// ---------------------------------------------------------------------------
// IdentifierQuery
// ---------------------------------------------------------------------------

/// Query interface for looking up cards by external identifiers.
pub struct IdentifierQuery<'a> {
    conn: &'a crate::connection::Connection,
}

impl<'a> IdentifierQuery<'a> {
    /// Create a new `IdentifierQuery` bound to the given connection.
    pub fn new(conn: &'a crate::connection::Connection) -> Self {
        Self { conn }
    }

    /// Generic find: look up cards whose `column` in `card_identifiers` matches `value`.
    ///
    /// Returns full card rows (joined from the `cards` view).
    ///
    /// Returns `Err(InvalidArgument)` if `column` is not in [`KNOWN_ID_COLUMNS`].
    pub fn find_by(&self, column: &str, value: &str) -> Result<Vec<Value>> {
        if !KNOWN_ID_COLUMNS.contains(&column) {
            return Err(MtgjsonError::InvalidArgument(format!(
                "Unknown identifier column: '{}'. Valid columns: {:?}",
                column, KNOWN_ID_COLUMNS
            )));
        }

        self.conn.ensure_views(&["cards", "card_identifiers"])?;

        let condition = format!("ci.{} = ?", column);
        let (sql, params) = SqlBuilder::new("cards c")
            .join("JOIN card_identifiers ci ON c.uuid = ci.uuid")
            .where_clause(&condition, &[value])
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows_to_values(rows))
    }

    /// Get all known identifiers for a card UUID.
    ///
    /// Returns the full `card_identifiers` row as a JSON object.
    pub fn get_identifiers(&self, uuid: &str) -> Result<Option<Value>> {
        self.conn.ensure_views(&["card_identifiers"])?;

        let (sql, params) = SqlBuilder::new("card_identifiers")
            .where_eq("uuid", uuid)
            .limit(1)
            .build();

        let rows = self.conn.execute(&sql, &params)?;
        Ok(rows
            .into_iter()
            .next()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null)))
    }

    // -- Convenience methods (one per known column) -------------------------

    /// Find cards by Card Kingdom etched product ID.
    pub fn find_by_card_kingdom_etched_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("cardKingdomEtchedId", value)
    }

    /// Find cards by Card Kingdom foil product ID.
    pub fn find_by_card_kingdom_foil_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("cardKingdomFoilId", value)
    }

    /// Find cards by Card Kingdom product ID.
    pub fn find_by_card_kingdom_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("cardKingdomId", value)
    }

    /// Find cards by Cardsphere ID.
    pub fn find_by_cardsphere_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("cardsphereId", value)
    }

    /// Find cards by Cardsphere foil ID.
    pub fn find_by_cardsphere_foil_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("cardsphereFoilId", value)
    }

    /// Find cards by MCM (Cardmarket) ID.
    pub fn find_by_mcm_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mcmId", value)
    }

    /// Find cards by MCM meta ID.
    pub fn find_by_mcm_meta_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mcmMetaId", value)
    }

    /// Find cards by MTG Arena ID.
    pub fn find_by_mtg_arena_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mtgArenaId", value)
    }

    /// Find cards by MTGJSON foil version ID.
    pub fn find_by_mtgjson_foil_version_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mtgjsonFoilVersionId", value)
    }

    /// Find cards by MTGJSON non-foil version ID.
    pub fn find_by_mtgjson_non_foil_version_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mtgjsonNonFoilVersionId", value)
    }

    /// Find cards by MTGJSON v4 ID.
    pub fn find_by_mtgjson_v4_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mtgjsonV4Id", value)
    }

    /// Find cards by MTGO foil ID.
    pub fn find_by_mtgo_foil_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mtgoFoilId", value)
    }

    /// Find cards by MTGO ID.
    pub fn find_by_mtgo_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("mtgoId", value)
    }

    /// Find cards by Multiverse ID.
    pub fn find_by_multiverse_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("multiverseId", value)
    }

    /// Find cards by Scryfall ID.
    pub fn find_by_scryfall_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("scryfallId", value)
    }

    /// Find cards by Scryfall card back ID.
    pub fn find_by_scryfall_card_back_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("scryfallCardBackId", value)
    }

    /// Find cards by Scryfall illustration ID.
    pub fn find_by_scryfall_illustration_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("scryfallIllustrationId", value)
    }

    /// Find cards by Scryfall Oracle ID.
    pub fn find_by_scryfall_oracle_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("scryfallOracleId", value)
    }

    /// Find cards by TCGplayer etched product ID.
    pub fn find_by_tcgplayer_etched_product_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("tcgplayerEtchedProductId", value)
    }

    /// Find cards by TCGplayer product ID.
    pub fn find_by_tcgplayer_product_id(&self, value: &str) -> Result<Vec<Value>> {
        self.find_by("tcgplayerProductId", value)
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
