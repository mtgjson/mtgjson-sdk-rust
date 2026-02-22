use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CardSet — The primary card model (all fields from the full printing chain)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSet {
    // -- CardBase fields --
    pub name: String,
    pub ascii_name: Option<String>,
    pub face_name: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub supertypes: Vec<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub color_identity: Vec<String>,
    pub color_indicator: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub mana_cost: Option<String>,
    pub text: Option<String>,
    pub layout: String,
    pub side: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub identifiers: Option<serde_json::Value>,
    pub is_funny: Option<bool>,
    pub edhrec_saltiness: Option<f64>,
    pub subsets: Option<Vec<String>>,

    // -- CardAtomicBase fields --
    pub converted_mana_cost: Option<f64>,
    pub mana_value: Option<f64>,
    pub face_converted_mana_cost: Option<f64>,
    pub face_mana_value: Option<f64>,
    pub defense: Option<String>,
    pub hand: Option<String>,
    pub life: Option<String>,
    pub edhrec_rank: Option<i64>,
    pub foreign_data: Option<serde_json::Value>,
    pub legalities: Option<serde_json::Value>,
    pub leadership_skills: Option<serde_json::Value>,
    pub rulings: Option<serde_json::Value>,
    pub has_alternative_deck_limit: Option<bool>,
    pub is_reserved: Option<bool>,
    pub is_game_changer: Option<bool>,
    pub printings: Option<Vec<String>>,
    pub purchase_urls: Option<serde_json::Value>,
    pub related_cards: Option<serde_json::Value>,

    // -- CardPrintingBase fields --
    pub uuid: String,
    pub set_code: String,
    pub number: String,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<String>>,
    pub border_color: Option<String>,
    pub frame_version: Option<String>,
    pub frame_effects: Option<Vec<String>>,
    pub watermark: Option<String>,
    pub signature: Option<String>,
    pub security_stamp: Option<String>,
    pub flavor_text: Option<String>,
    pub flavor_name: Option<String>,
    pub face_flavor_name: Option<String>,
    pub original_text: Option<String>,
    pub original_type: Option<String>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type: Option<String>,
    pub face_printed_name: Option<String>,
    #[serde(default)]
    pub availability: Vec<String>,
    pub booster_types: Option<Vec<String>>,
    #[serde(default)]
    pub finishes: Vec<String>,
    pub promo_types: Option<Vec<String>>,
    pub attraction_lights: Option<Vec<i64>>,
    pub is_full_art: Option<bool>,
    pub is_online_only: Option<bool>,
    pub is_oversized: Option<bool>,
    pub is_promo: Option<bool>,
    pub is_reprint: Option<bool>,
    pub is_textless: Option<bool>,
    pub other_face_ids: Option<Vec<String>>,
    pub card_parts: Option<Vec<String>>,
    pub language: Option<String>,
    pub source_products: Option<serde_json::Value>,

    // -- CardPrintingFull fields --
    pub rarity: Option<String>,
    pub duel_deck: Option<String>,
    pub is_rebalanced: Option<bool>,
    pub original_printings: Option<Vec<String>>,
    pub rebalanced_printings: Option<Vec<String>>,
    pub original_release_date: Option<String>,
    pub is_alternative: Option<bool>,
    pub is_story_spotlight: Option<bool>,
    pub is_timeshifted: Option<bool>,
    pub has_content_warning: Option<bool>,
    pub variations: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// CardAtomic — Oracle-only card (no printing fields)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardAtomic {
    // -- CardBase fields --
    pub name: String,
    pub ascii_name: Option<String>,
    pub face_name: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub supertypes: Vec<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub color_identity: Vec<String>,
    pub color_indicator: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub mana_cost: Option<String>,
    pub text: Option<String>,
    pub layout: String,
    pub side: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub identifiers: Option<serde_json::Value>,
    pub is_funny: Option<bool>,
    pub edhrec_saltiness: Option<f64>,
    pub subsets: Option<Vec<String>>,

    // -- CardAtomicBase fields --
    pub converted_mana_cost: Option<f64>,
    pub mana_value: Option<f64>,
    pub face_converted_mana_cost: Option<f64>,
    pub face_mana_value: Option<f64>,
    pub defense: Option<String>,
    pub hand: Option<String>,
    pub life: Option<String>,
    pub edhrec_rank: Option<i64>,
    pub foreign_data: Option<serde_json::Value>,
    pub legalities: Option<serde_json::Value>,
    pub leadership_skills: Option<serde_json::Value>,
    pub rulings: Option<serde_json::Value>,
    pub has_alternative_deck_limit: Option<bool>,
    pub is_reserved: Option<bool>,
    pub is_game_changer: Option<bool>,
    pub printings: Option<Vec<String>>,
    pub purchase_urls: Option<serde_json::Value>,
    pub related_cards: Option<serde_json::Value>,

    // -- Atomic-specific --
    pub first_printing: Option<String>,
}

// ---------------------------------------------------------------------------
// CardToken — Token card
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardToken {
    // -- CardBase fields --
    pub name: String,
    pub ascii_name: Option<String>,
    pub face_name: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub supertypes: Vec<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub color_identity: Vec<String>,
    pub color_indicator: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub mana_cost: Option<String>,
    pub text: Option<String>,
    pub layout: String,
    pub side: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub identifiers: Option<serde_json::Value>,
    pub is_funny: Option<bool>,
    pub edhrec_saltiness: Option<f64>,
    pub subsets: Option<Vec<String>>,

    // -- CardPrintingBase fields --
    pub uuid: String,
    pub set_code: String,
    pub number: String,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<String>>,
    pub border_color: Option<String>,
    pub frame_version: Option<String>,
    pub frame_effects: Option<Vec<String>>,
    pub watermark: Option<String>,
    pub signature: Option<String>,
    pub security_stamp: Option<String>,
    pub flavor_text: Option<String>,
    pub flavor_name: Option<String>,
    pub face_flavor_name: Option<String>,
    pub original_text: Option<String>,
    pub original_type: Option<String>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type: Option<String>,
    pub face_printed_name: Option<String>,
    #[serde(default)]
    pub availability: Vec<String>,
    pub booster_types: Option<Vec<String>>,
    #[serde(default)]
    pub finishes: Vec<String>,
    pub promo_types: Option<Vec<String>>,
    pub attraction_lights: Option<Vec<i64>>,
    pub is_full_art: Option<bool>,
    pub is_online_only: Option<bool>,
    pub is_oversized: Option<bool>,
    pub is_promo: Option<bool>,
    pub is_reprint: Option<bool>,
    pub is_textless: Option<bool>,
    pub other_face_ids: Option<Vec<String>>,
    pub card_parts: Option<Vec<String>>,
    pub language: Option<String>,
    pub source_products: Option<serde_json::Value>,

    // -- Token-specific fields --
    pub orientation: Option<String>,
    pub reverse_related: Option<Vec<String>>,
    pub related_cards: Option<serde_json::Value>,
    pub token_products: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// CardDeck — Card in a deck (all CardSet fields + deck-specific)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDeck {
    // -- CardBase fields --
    pub name: String,
    pub ascii_name: Option<String>,
    pub face_name: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub supertypes: Vec<String>,
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub color_identity: Vec<String>,
    pub color_indicator: Option<Vec<String>>,
    pub produced_mana: Option<Vec<String>>,
    pub mana_cost: Option<String>,
    pub text: Option<String>,
    pub layout: String,
    pub side: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub identifiers: Option<serde_json::Value>,
    pub is_funny: Option<bool>,
    pub edhrec_saltiness: Option<f64>,
    pub subsets: Option<Vec<String>>,

    // -- CardAtomicBase fields --
    pub converted_mana_cost: Option<f64>,
    pub mana_value: Option<f64>,
    pub face_converted_mana_cost: Option<f64>,
    pub face_mana_value: Option<f64>,
    pub defense: Option<String>,
    pub hand: Option<String>,
    pub life: Option<String>,
    pub edhrec_rank: Option<i64>,
    pub foreign_data: Option<serde_json::Value>,
    pub legalities: Option<serde_json::Value>,
    pub leadership_skills: Option<serde_json::Value>,
    pub rulings: Option<serde_json::Value>,
    pub has_alternative_deck_limit: Option<bool>,
    pub is_reserved: Option<bool>,
    pub is_game_changer: Option<bool>,
    pub printings: Option<Vec<String>>,
    pub purchase_urls: Option<serde_json::Value>,
    pub related_cards: Option<serde_json::Value>,

    // -- CardPrintingBase fields --
    pub uuid: String,
    pub set_code: String,
    pub number: String,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<String>>,
    pub border_color: Option<String>,
    pub frame_version: Option<String>,
    pub frame_effects: Option<Vec<String>>,
    pub watermark: Option<String>,
    pub signature: Option<String>,
    pub security_stamp: Option<String>,
    pub flavor_text: Option<String>,
    pub flavor_name: Option<String>,
    pub face_flavor_name: Option<String>,
    pub original_text: Option<String>,
    pub original_type: Option<String>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type: Option<String>,
    pub face_printed_name: Option<String>,
    #[serde(default)]
    pub availability: Vec<String>,
    pub booster_types: Option<Vec<String>>,
    #[serde(default)]
    pub finishes: Vec<String>,
    pub promo_types: Option<Vec<String>>,
    pub attraction_lights: Option<Vec<i64>>,
    pub is_full_art: Option<bool>,
    pub is_online_only: Option<bool>,
    pub is_oversized: Option<bool>,
    pub is_promo: Option<bool>,
    pub is_reprint: Option<bool>,
    pub is_textless: Option<bool>,
    pub other_face_ids: Option<Vec<String>>,
    pub card_parts: Option<Vec<String>>,
    pub language: Option<String>,
    pub source_products: Option<serde_json::Value>,

    // -- CardPrintingFull fields --
    pub rarity: Option<String>,
    pub duel_deck: Option<String>,
    pub is_rebalanced: Option<bool>,
    pub original_printings: Option<Vec<String>>,
    pub rebalanced_printings: Option<Vec<String>>,
    pub original_release_date: Option<String>,
    pub is_alternative: Option<bool>,
    pub is_story_spotlight: Option<bool>,
    pub is_timeshifted: Option<bool>,
    pub has_content_warning: Option<bool>,
    pub variations: Option<Vec<String>>,

    // -- Deck-specific fields --
    pub count: i64,
    pub is_foil: Option<bool>,
    pub is_etched: Option<bool>,
}

// ---------------------------------------------------------------------------
// CardSetDeck — Minimal card reference in a deck
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardSetDeck {
    pub count: i64,
    pub is_foil: Option<bool>,
    pub uuid: String,
}
