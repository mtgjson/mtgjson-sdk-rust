use serde::{Deserialize, Serialize};

use super::sub::SealedProductContents;

// ---------------------------------------------------------------------------
// SealedProduct — Full sealed product data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SealedProduct {
    pub uuid: String,
    pub name: String,
    pub category: Option<String>,
    pub set_code: Option<String>,
    pub subtype: Option<String>,
    pub language: Option<String>,
    pub release_date: Option<String>,
    pub card_count: Option<i64>,
    pub product_size: Option<i64>,
    pub contents: Option<SealedProductContents>,
    pub identifiers: Option<serde_json::Value>,
    pub purchase_urls: Option<serde_json::Value>,
}
