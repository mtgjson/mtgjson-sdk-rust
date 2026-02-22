//! Card query integration tests against in-memory sample data.

mod common;

use mtgjson_sdk::queries::cards::{CardQuery, SearchCardsParams};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// get_by_uuid
// ---------------------------------------------------------------------------

#[test]
fn get_by_uuid_finds_existing_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let result = cq.get_by_uuid("card-uuid-001").unwrap();
    assert!(result.is_some());
    let card = result.unwrap();
    assert_eq!(card["name"], "Lightning Bolt");
    assert_eq!(card["setCode"], "A25");
}

#[test]
fn get_by_uuid_returns_none_for_unknown() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let result = cq.get_by_uuid("nonexistent-uuid").unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// get_by_name
// ---------------------------------------------------------------------------

#[test]
fn get_by_name_returns_matching_cards() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.get_by_name("Lightning Bolt", None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["uuid"], "card-uuid-001");
}

#[test]
fn get_by_name_with_set_code_filters() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.get_by_name("Lightning Bolt", Some("A25")).unwrap();
    assert_eq!(results.len(), 1);

    let results = cq.get_by_name("Lightning Bolt", Some("MH2")).unwrap();
    assert!(results.is_empty());
}

#[test]
fn get_by_name_returns_empty_for_unknown() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.get_by_name("Nonexistent Card", None).unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// get_by_uuids
// ---------------------------------------------------------------------------

#[test]
fn get_by_uuids_returns_multiple_cards() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.get_by_uuids(&["card-uuid-001", "card-uuid-002"]).unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<&str> = results.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Lightning Bolt"));
    assert!(names.contains(&"Counterspell"));
}

#[test]
fn get_by_uuids_returns_empty_for_unknown() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.get_by_uuids(&["no-such-uuid"]).unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------

#[test]
fn search_with_name_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            name: Some("Counterspell".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["uuid"], "card-uuid-002");
}

#[test]
fn search_with_name_wildcard() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            name: Some("%Bolt%".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn search_with_rarity_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            rarity: Some("uncommon".to_string()),
            ..Default::default()
        })
        .unwrap();
    // All 3 sample cards are uncommon
    assert_eq!(results.len(), 3);
}

#[test]
fn search_with_set_code_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            set_code: Some("A25".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn search_with_layout_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            layout: Some("split".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Fire // Ice");
}

#[test]
fn search_with_mana_value_exact() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            mana_value: Some(1.0),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn search_with_mana_value_range() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            mana_value_gte: Some(1.0),
            mana_value_lte: Some(2.0),
            ..Default::default()
        })
        .unwrap();
    // All 3 cards have mana value 1.0 or 2.0
    assert_eq!(results.len(), 3);
}

#[test]
fn search_with_text_substring() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            text: Some("damage".to_string()),
            ..Default::default()
        })
        .unwrap();
    // Lightning Bolt + Fire // Ice both mention damage
    assert_eq!(results.len(), 2);
}

#[test]
fn search_with_limit() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            limit: Some(1),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn search_with_artist_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq
        .search(&SearchCardsParams {
            artist: Some("Stella".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Counterspell");
}

// ---------------------------------------------------------------------------
// count
// ---------------------------------------------------------------------------

#[test]
fn count_returns_total() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let cnt = cq.count(&HashMap::new()).unwrap();
    assert_eq!(cnt, 3);
}

#[test]
fn count_with_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let mut filters = HashMap::new();
    filters.insert("setCode".to_string(), "A25".to_string());
    let cnt = cq.count(&filters).unwrap();
    assert_eq!(cnt, 2);
}

// ---------------------------------------------------------------------------
// random
// ---------------------------------------------------------------------------

#[test]
fn random_returns_requested_count() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.random(2).unwrap();
    assert_eq!(results.len(), 2);
}

// ---------------------------------------------------------------------------
// get_printings
// ---------------------------------------------------------------------------

#[test]
fn get_printings_returns_all_printings() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    // Lightning Bolt has one printing in our sample data
    let results = cq.get_printings("Lightning Bolt").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["setCode"], "A25");
}

// ---------------------------------------------------------------------------
// get_atomic
// ---------------------------------------------------------------------------

#[test]
fn get_atomic_deduplicates() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    // Should return deduplicated by (name, faceName)
    let results = cq.get_atomic("Lightning Bolt").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn get_atomic_falls_back_to_face_name() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    // "Fire" is a faceName, not a full name
    let results = cq.get_atomic("Fire").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["faceName"], "Fire");
}

// ---------------------------------------------------------------------------
// find_by_scryfall_id
// ---------------------------------------------------------------------------

#[test]
fn find_by_scryfall_id_returns_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.find_by_scryfall_id("scryfall-001").unwrap();
    assert_eq!(results.len(), 1);
    // The result should include card fields from the join
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn find_by_scryfall_id_returns_empty_for_unknown() {
    let (conn, _tmp) = common::setup_sample_db();
    let cq = CardQuery::new(&conn);

    let results = cq.find_by_scryfall_id("nonexistent-scryfall").unwrap();
    assert!(results.is_empty());
}
