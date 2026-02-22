//! Token query integration tests against in-memory sample data.

mod common;

use mtgjson_sdk::queries::tokens::{SearchTokensParams, TokenQuery};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// get_by_uuid
// ---------------------------------------------------------------------------

#[test]
fn get_by_uuid_finds_token() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let result = tq.get_by_uuid("token-uuid-001").unwrap();
    assert!(result.is_some());
    let token = result.unwrap();
    assert_eq!(token["name"], "Soldier");
    assert_eq!(token["setCode"], "A25");
}

#[test]
fn get_by_uuid_returns_none_for_unknown() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let result = tq.get_by_uuid("nonexistent-token").unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// get_by_name
// ---------------------------------------------------------------------------

#[test]
fn get_by_name_returns_matching_tokens() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq.get_by_name("Goblin", None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["setCode"], "MH2");
}

#[test]
fn get_by_name_with_set_code() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq.get_by_name("Soldier", Some("A25")).unwrap();
    assert_eq!(results.len(), 1);

    let results = tq.get_by_name("Soldier", Some("MH2")).unwrap();
    assert!(results.is_empty());
}

#[test]
fn get_by_name_returns_empty_for_unknown() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq.get_by_name("Dragon", None).unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// get_by_uuids
// ---------------------------------------------------------------------------

#[test]
fn get_by_uuids_returns_multiple_tokens() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq.get_by_uuids(&["token-uuid-001", "token-uuid-002"]).unwrap();
    assert_eq!(results.len(), 2);
}

// ---------------------------------------------------------------------------
// for_set
// ---------------------------------------------------------------------------

#[test]
fn for_set_returns_tokens_in_set() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq.for_set("A25").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Soldier");
}

#[test]
fn for_set_returns_empty_for_set_without_tokens() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq.for_set("XXX").unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------

#[test]
fn search_with_name_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq
        .search(&SearchTokensParams {
            name: Some("Soldier".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Soldier");
}

#[test]
fn search_with_set_code_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq
        .search(&SearchTokensParams {
            set_code: Some("MH2".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Goblin");
}

#[test]
fn search_with_artist_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq
        .search(&SearchTokensParams {
            artist: Some("Kopinski".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Goblin");
}

#[test]
fn search_with_types_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let results = tq
        .search(&SearchTokensParams {
            types: Some("Goblin".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Goblin");
}

// ---------------------------------------------------------------------------
// count
// ---------------------------------------------------------------------------

#[test]
fn count_returns_total() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let cnt = tq.count(&HashMap::new()).unwrap();
    assert_eq!(cnt, 2);
}

#[test]
fn count_with_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let tq = TokenQuery::new(&conn);

    let mut filters = HashMap::new();
    filters.insert("setCode".to_string(), "A25".to_string());
    let cnt = tq.count(&filters).unwrap();
    assert_eq!(cnt, 1);
}
