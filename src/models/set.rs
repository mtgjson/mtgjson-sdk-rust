use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::card::{CardSet, CardToken};

// ---------------------------------------------------------------------------
// SetList — Summary info for a set (used in set list endpoints)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetList {
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub release_date: String,
    pub base_set_size: i64,
    pub total_set_size: i64,
    pub keyrune_code: String,
    pub translations: Option<serde_json::Value>,
    pub block: Option<String>,
    pub parent_code: Option<String>,
    pub mtgo_code: Option<String>,
    pub token_set_code: Option<String>,
    pub mcm_id: Option<i64>,
    pub mcm_id_extras: Option<i64>,
    pub mcm_name: Option<String>,
    pub tcgplayer_group_id: Option<i64>,
    pub cardsphere_set_id: Option<i64>,
    pub is_foil_only: Option<bool>,
    pub is_non_foil_only: Option<bool>,
    pub is_online_only: Option<bool>,
    pub is_paper_only: Option<bool>,
    pub is_foreign_only: Option<bool>,
    pub is_partial_preview: Option<bool>,
    pub languages: Option<Vec<String>>,
    pub decks: Option<Vec<serde_json::Value>>,
    pub sealed_product: Option<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// MtgSet — Full set data including cards and tokens
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MtgSet {
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub release_date: String,
    pub base_set_size: i64,
    pub total_set_size: i64,
    pub keyrune_code: String,
    pub translations: Option<serde_json::Value>,
    pub block: Option<String>,
    pub parent_code: Option<String>,
    pub mtgo_code: Option<String>,
    pub token_set_code: Option<String>,
    pub mcm_id: Option<i64>,
    pub mcm_id_extras: Option<i64>,
    pub mcm_name: Option<String>,
    pub tcgplayer_group_id: Option<i64>,
    pub cardsphere_set_id: Option<i64>,
    pub is_foil_only: Option<bool>,
    pub is_non_foil_only: Option<bool>,
    pub is_online_only: Option<bool>,
    pub is_paper_only: Option<bool>,
    pub is_foreign_only: Option<bool>,
    pub is_partial_preview: Option<bool>,
    pub languages: Option<Vec<String>>,
    pub decks: Option<Vec<serde_json::Value>>,
    pub sealed_product: Option<Vec<serde_json::Value>>,

    // -- Full set specific fields --
    #[serde(default)]
    pub cards: Vec<CardSet>,
    #[serde(default)]
    pub tokens: Vec<CardToken>,
    pub booster: Option<HashMap<String, serde_json::Value>>,
}
