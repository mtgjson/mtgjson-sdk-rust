use std::collections::HashMap;
use std::path::PathBuf;

pub const CDN_BASE: &str = "https://mtgjson.com/api/v5";
pub const META_URL: &str = "https://mtgjson.com/api/v5/Meta.json";

pub fn parquet_files() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        // Flat normalized tables
        ("cards", "parquet/cards.parquet"),
        ("tokens", "parquet/tokens.parquet"),
        ("sets", "parquet/sets.parquet"),
        ("card_identifiers", "parquet/cardIdentifiers.parquet"),
        ("card_legalities", "parquet/cardLegalities.parquet"),
        ("card_foreign_data", "parquet/cardForeignData.parquet"),
        ("card_rulings", "parquet/cardRulings.parquet"),
        ("card_purchase_urls", "parquet/cardPurchaseUrls.parquet"),
        ("set_translations", "parquet/setTranslations.parquet"),
        ("token_identifiers", "parquet/tokenIdentifiers.parquet"),
        // Booster tables
        (
            "set_booster_content_weights",
            "parquet/setBoosterContentWeights.parquet",
        ),
        (
            "set_booster_contents",
            "parquet/setBoosterContents.parquet",
        ),
        (
            "set_booster_sheet_cards",
            "parquet/setBoosterSheetCards.parquet",
        ),
        ("set_booster_sheets", "parquet/setBoosterSheets.parquet"),
        // Full nested
        ("all_printings", "parquet/AllPrintings.parquet"),
        // Prices and SKUs
        ("all_prices_today", "parquet/AllPricesToday.parquet"),
        ("all_prices", "parquet/AllPrices.parquet"),
        ("tcgplayer_skus", "parquet/TcgplayerSkus.parquet"),
    ])
}

pub fn json_files() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("keywords", "Keywords.json"),
        ("card_types", "CardTypes.json"),
        ("deck_list", "DeckList.json"),
        ("enum_values", "EnumValues.json"),
        ("meta", "Meta.json"),
    ])
}

pub fn default_cache_dir() -> PathBuf {
    if let Some(cache) = dirs::cache_dir() {
        cache.join("mtgjson-sdk")
    } else {
        PathBuf::from(".mtgjson-sdk-cache")
    }
}
