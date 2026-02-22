//! Identifier query integration tests against in-memory sample data.

mod common;

use mtgjson_sdk::queries::identifiers::IdentifierQuery;

// ---------------------------------------------------------------------------
// find_by (generic)
// ---------------------------------------------------------------------------

#[test]
fn find_by_scryfall_id_returns_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by("scryfallId", "scryfall-001").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn find_by_tcgplayer_product_id_returns_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by("tcgplayerProductId", "67890").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Counterspell");
}

#[test]
fn find_by_returns_empty_for_unknown_value() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by("scryfallId", "nonexistent").unwrap();
    assert!(results.is_empty());
}

#[test]
fn find_by_invalid_column_returns_error() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let result = iq.find_by("invalidColumn", "some-value");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Unknown identifier column"));
}

// ---------------------------------------------------------------------------
// get_identifiers
// ---------------------------------------------------------------------------

#[test]
fn get_identifiers_returns_all_ids_for_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let result = iq.get_identifiers("card-uuid-001").unwrap();
    assert!(result.is_some());

    let ids = result.unwrap();
    assert_eq!(ids["uuid"], "card-uuid-001");
    assert_eq!(ids["scryfallId"], "scryfall-001");
    assert_eq!(ids["tcgplayerProductId"], "12345");
}

#[test]
fn get_identifiers_returns_none_for_unknown_uuid() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let result = iq.get_identifiers("nonexistent-uuid").unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// Convenience methods
// ---------------------------------------------------------------------------

#[test]
fn find_by_scryfall_id_convenience() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by_scryfall_id("scryfall-002").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Counterspell");
}

#[test]
fn find_by_tcgplayer_product_id_convenience() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by_tcgplayer_product_id("12345").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn find_by_mtgo_id_convenience() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by_mtgo_id("mtgo-002").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Counterspell");
}

#[test]
fn find_by_mtg_arena_id_convenience() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by_mtg_arena_id("arena-001").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn find_by_multiverse_id_convenience() {
    let (conn, _tmp) = common::setup_sample_db();
    let iq = IdentifierQuery::new(&conn);

    let results = iq.find_by_multiverse_id("100002").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Counterspell");
}
