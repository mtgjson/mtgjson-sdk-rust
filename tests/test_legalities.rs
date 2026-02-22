//! Legality query integration tests against in-memory sample data.

mod common;

use mtgjson_sdk::queries::legalities::LegalityQuery;

// ---------------------------------------------------------------------------
// formats_for_card
// ---------------------------------------------------------------------------

#[test]
fn formats_for_card_returns_all_formats() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.formats_for_card("card-uuid-001").unwrap();
    // card-uuid-001 has entries for modern, vintage, standard
    assert_eq!(results.len(), 3);

    let formats: Vec<&str> = results.iter().map(|r| r["format"].as_str().unwrap()).collect();
    assert!(formats.contains(&"modern"));
    assert!(formats.contains(&"vintage"));
    assert!(formats.contains(&"standard"));
}

#[test]
fn formats_for_card_returns_empty_for_unknown_uuid() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.formats_for_card("nonexistent-uuid").unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// is_legal
// ---------------------------------------------------------------------------

#[test]
fn is_legal_returns_true_for_legal_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let result = lq.is_legal("card-uuid-001", "modern").unwrap();
    assert!(result);
}

#[test]
fn is_legal_returns_false_for_restricted_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    // card-uuid-001 is "Restricted" in vintage, not "Legal"
    let result = lq.is_legal("card-uuid-001", "vintage").unwrap();
    assert!(!result);
}

#[test]
fn is_legal_returns_false_for_not_legal_card() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let result = lq.is_legal("card-uuid-001", "standard").unwrap();
    assert!(!result);
}

#[test]
fn is_legal_returns_false_for_unknown_format() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let result = lq.is_legal("card-uuid-001", "nonexistent_format").unwrap();
    assert!(!result);
}

#[test]
fn is_legal_returns_false_for_unknown_uuid() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let result = lq.is_legal("nonexistent-uuid", "modern").unwrap();
    assert!(!result);
}

// ---------------------------------------------------------------------------
// legal_in
// ---------------------------------------------------------------------------

#[test]
fn legal_in_returns_cards_with_legal_status() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.legal_in("modern").unwrap();
    // All 3 cards are "Legal" in modern
    assert_eq!(results.len(), 3);
}

#[test]
fn legal_in_vintage_returns_only_legal_not_restricted() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.legal_in("vintage").unwrap();
    // card-uuid-001 is Restricted in vintage, so only uuid-002 and uuid-003 are Legal
    assert_eq!(results.len(), 2);

    let names: Vec<&str> = results.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(!names.contains(&"Lightning Bolt"));
}

#[test]
fn legal_in_returns_empty_for_unknown_format() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.legal_in("nonexistent_format").unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// restricted_in
// ---------------------------------------------------------------------------

#[test]
fn restricted_in_returns_restricted_cards() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.restricted_in("vintage").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Lightning Bolt");
}

#[test]
fn restricted_in_returns_empty_when_none_restricted() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.restricted_in("modern").unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// not_legal_in
// ---------------------------------------------------------------------------

#[test]
fn not_legal_in_returns_not_legal_cards() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    let results = lq.not_legal_in("standard").unwrap();
    // card-uuid-001 and card-uuid-002 have "Not Legal" in standard
    assert_eq!(results.len(), 2);
}

// ---------------------------------------------------------------------------
// banned_in
// ---------------------------------------------------------------------------

#[test]
fn banned_in_returns_empty_when_none_banned() {
    let (conn, _tmp) = common::setup_sample_db();
    let lq = LegalityQuery::new(&conn);

    // No cards are banned in our sample data
    let results = lq.banned_in("modern").unwrap();
    assert!(results.is_empty());
}
