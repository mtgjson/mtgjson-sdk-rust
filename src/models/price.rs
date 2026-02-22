use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// PriceRow — Single price data point (query result)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRow {
    pub uuid: String,
    pub source: String,
    pub provider: String,
    pub currency: String,
    pub category: String,
    pub finish: String,
    pub date: String,
    pub price: f64,
}

// ---------------------------------------------------------------------------
// PriceTrend — Aggregated price trend data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PriceTrend {
    pub min_price: f64,
    pub max_price: f64,
    pub avg_price: f64,
    pub first_date: String,
    pub last_date: String,
    pub data_points: i64,
}

// ---------------------------------------------------------------------------
// FinancialSummary — Aggregated financial summary for a collection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FinancialSummary {
    pub card_count: i64,
    pub total_value: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub date: String,
}
