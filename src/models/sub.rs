use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Meta {
    pub date: String,
    pub version: String,
}

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Identifiers {
    pub abu_id: Option<String>,
    pub cardtrader_id: Option<String>,
    pub csi_id: Option<String>,
    pub miniaturemarket_id: Option<String>,
    pub mvp_id: Option<String>,
    pub scg_id: Option<String>,
    pub tnt_id: Option<String>,
    pub card_kingdom_etched_id: Option<String>,
    pub card_kingdom_foil_id: Option<String>,
    pub card_kingdom_id: Option<String>,
    pub cardsphere_id: Option<String>,
    pub cardsphere_foil_id: Option<String>,
    pub deckbox_id: Option<String>,
    pub mcm_id: Option<String>,
    pub mcm_meta_id: Option<String>,
    pub mtg_arena_id: Option<String>,
    pub mtgjson_foil_version_id: Option<String>,
    pub mtgjson_non_foil_version_id: Option<String>,
    #[serde(rename = "mtgjsonV4Id")]
    pub mtgjson_v4_id: Option<String>,
    pub mtgo_foil_id: Option<String>,
    pub mtgo_id: Option<String>,
    pub multiverse_id: Option<String>,
    pub scryfall_id: Option<String>,
    pub scryfall_card_back_id: Option<String>,
    pub scryfall_illustration_id: Option<String>,
    pub scryfall_oracle_id: Option<String>,
    pub tcgplayer_etched_product_id: Option<String>,
    pub tcgplayer_product_id: Option<String>,
}

// ---------------------------------------------------------------------------
// LeadershipSkills
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeadershipSkills {
    pub brawl: bool,
    pub commander: bool,
    pub oathbreaker: bool,
}

// ---------------------------------------------------------------------------
// Legalities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Legalities {
    pub alchemy: Option<String>,
    pub brawl: Option<String>,
    pub commander: Option<String>,
    pub duel: Option<String>,
    pub explorer: Option<String>,
    pub future: Option<String>,
    pub gladiator: Option<String>,
    pub historic: Option<String>,
    pub historicbrawl: Option<String>,
    pub legacy: Option<String>,
    pub modern: Option<String>,
    pub oathbreaker: Option<String>,
    pub oldschool: Option<String>,
    pub pauper: Option<String>,
    pub paupercommander: Option<String>,
    pub penny: Option<String>,
    pub pioneer: Option<String>,
    pub predh: Option<String>,
    pub premodern: Option<String>,
    pub standard: Option<String>,
    pub standardbrawl: Option<String>,
    pub timeless: Option<String>,
    pub vintage: Option<String>,
}

// ---------------------------------------------------------------------------
// PurchaseUrls
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseUrls {
    pub card_kingdom: Option<String>,
    pub card_kingdom_etched: Option<String>,
    pub card_kingdom_foil: Option<String>,
    pub cardmarket: Option<String>,
    pub tcgplayer: Option<String>,
    pub tcgplayer_etched: Option<String>,
}

// ---------------------------------------------------------------------------
// RelatedCards
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RelatedCards {
    #[serde(default)]
    pub reverse_related: Vec<String>,
    #[serde(default)]
    pub spellbook: Vec<String>,
    #[serde(default)]
    pub tokens: Vec<String>,
}

// ---------------------------------------------------------------------------
// Rulings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rulings {
    pub date: String,
    pub text: String,
}

// ---------------------------------------------------------------------------
// SourceProducts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceProducts {
    #[serde(default)]
    pub etched: Vec<String>,
    #[serde(default)]
    pub foil: Vec<String>,
    #[serde(default)]
    pub nonfoil: Vec<String>,
}

// ---------------------------------------------------------------------------
// ForeignDataIdentifiers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForeignDataIdentifiers {
    pub multiverse_id: Option<String>,
    pub scryfall_id: Option<String>,
}

// ---------------------------------------------------------------------------
// ForeignData
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForeignData {
    pub language: String,
    pub face_name: Option<String>,
    pub flavor_text: Option<String>,
    pub identifiers: Option<ForeignDataIdentifiers>,
    pub multiverse_id: Option<i64>,
    pub name: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub uuid: Option<String>,
}

// ---------------------------------------------------------------------------
// Translations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Translations {
    #[serde(rename = "Ancient Greek")]
    pub ancient_greek: Option<String>,
    #[serde(rename = "Arabic")]
    pub arabic: Option<String>,
    #[serde(rename = "Chinese Simplified")]
    pub chinese_simplified: Option<String>,
    #[serde(rename = "Chinese Traditional")]
    pub chinese_traditional: Option<String>,
    #[serde(rename = "French")]
    pub french: Option<String>,
    #[serde(rename = "German")]
    pub german: Option<String>,
    #[serde(rename = "Hebrew")]
    pub hebrew: Option<String>,
    #[serde(rename = "Italian")]
    pub italian: Option<String>,
    #[serde(rename = "Japanese")]
    pub japanese: Option<String>,
    #[serde(rename = "Korean")]
    pub korean: Option<String>,
    #[serde(rename = "Latin")]
    pub latin: Option<String>,
    #[serde(rename = "Phyrexian")]
    pub phyrexian: Option<String>,
    #[serde(rename = "Portuguese (Brazil)")]
    pub portuguese_brazil: Option<String>,
    #[serde(rename = "Russian")]
    pub russian: Option<String>,
    #[serde(rename = "Sanskrit")]
    pub sanskrit: Option<String>,
    #[serde(rename = "Spanish")]
    pub spanish: Option<String>,
}

// ---------------------------------------------------------------------------
// TcgplayerSkus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcgplayerSkus {
    pub condition: String,
    pub finish: Option<String>,
    pub language: String,
    pub printing: String,
    pub product_id: i64,
    pub sku_id: i64,
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Keywords {
    #[serde(default)]
    pub ability_words: Vec<String>,
    #[serde(default)]
    pub keyword_abilities: Vec<String>,
    #[serde(default)]
    pub keyword_actions: Vec<String>,
}

// ---------------------------------------------------------------------------
// CardType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CardType {
    #[serde(default)]
    pub sub_types: Vec<String>,
    #[serde(default)]
    pub super_types: Vec<String>,
}

// ---------------------------------------------------------------------------
// BoosterSheet
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoosterSheet {
    pub allow_duplicates: Option<bool>,
    pub balance_colors: Option<bool>,
    #[serde(default)]
    pub cards: HashMap<String, i64>,
    pub foil: bool,
    pub fixed: Option<bool>,
    pub total_weight: i64,
}

// ---------------------------------------------------------------------------
// BoosterPack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoosterPack {
    #[serde(default)]
    pub contents: HashMap<String, i64>,
    pub weight: i64,
}

// ---------------------------------------------------------------------------
// BoosterConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoosterConfig {
    #[serde(default)]
    pub boosters: Vec<BoosterPack>,
    pub boosters_total_weight: i64,
    pub name: Option<String>,
    #[serde(default)]
    pub sheets: HashMap<String, BoosterSheet>,
    #[serde(default)]
    pub source_set_codes: Vec<String>,
}

// ---------------------------------------------------------------------------
// PricePoints
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PricePoints {
    #[serde(default)]
    pub etched: HashMap<String, f64>,
    #[serde(default)]
    pub foil: HashMap<String, f64>,
    #[serde(default)]
    pub normal: HashMap<String, f64>,
}

// ---------------------------------------------------------------------------
// PriceList
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceList {
    pub buylist: Option<PricePoints>,
    pub currency: String,
    pub retail: Option<PricePoints>,
}

// ---------------------------------------------------------------------------
// PriceFormats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceFormats {
    pub mtgo: Option<HashMap<String, PriceList>>,
    pub paper: Option<HashMap<String, PriceList>>,
}

// ---------------------------------------------------------------------------
// SealedProductCard
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedProductCard {
    pub finishes: Option<Vec<String>>,
    pub foil: Option<bool>,
    pub name: String,
    pub number: String,
    pub set: String,
    pub uuid: String,
}

// ---------------------------------------------------------------------------
// SealedProductDeck
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedProductDeck {
    pub name: String,
    pub set: String,
}

// ---------------------------------------------------------------------------
// SealedProductOther
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedProductOther {
    pub name: String,
}

// ---------------------------------------------------------------------------
// SealedProductPack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedProductPack {
    pub code: String,
    pub set: String,
}

// ---------------------------------------------------------------------------
// SealedProductSealed
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedProductSealed {
    pub count: i64,
    pub name: String,
    pub set: String,
    pub uuid: Option<String>,
}

// ---------------------------------------------------------------------------
// SealedProductContents
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SealedProductContents {
    pub card: Option<Vec<SealedProductCard>>,
    pub deck: Option<Vec<SealedProductDeck>>,
    pub other: Option<Vec<SealedProductOther>>,
    pub pack: Option<Vec<SealedProductPack>>,
    pub sealed: Option<Vec<SealedProductSealed>>,
    pub variable: Option<Vec<serde_json::Value>>,
}
