use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// DeckList — Summary info for a deck
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckList {
    pub code: String,
    pub name: String,
    pub file_name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub release_date: Option<String>,
}
