//! Set query integration tests against in-memory sample data.

mod common;

use mtgjson_sdk::queries::sets::{SearchSetsParams, SetQuery};

// ---------------------------------------------------------------------------
// get
// ---------------------------------------------------------------------------

#[test]
fn get_by_code_finds_set() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let result = sq.get("A25").unwrap();
    assert!(result.is_some());
    let set = result.unwrap();
    assert_eq!(set["name"], "Masters 25");
    assert_eq!(set["type"], "masters");
}

#[test]
fn get_by_code_is_case_insensitive() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let result = sq.get("a25").unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap()["name"], "Masters 25");
}

#[test]
fn get_returns_none_for_unknown_code() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let result = sq.get("ZZZZ").unwrap();
    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

#[test]
fn list_returns_all_sets() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq.list(None, None, None, None).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn list_ordered_by_release_date_desc() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq.list(None, None, None, None).unwrap();
    // MH2 (2021-06-18) should come before A25 (2018-03-16)
    assert_eq!(results[0]["code"], "MH2");
    assert_eq!(results[1]["code"], "A25");
}

#[test]
fn list_with_type_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq.list(Some("masters"), None, None, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["code"], "A25");
}

#[test]
fn list_with_name_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq.list(None, Some("Horizons"), None, None).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["code"], "MH2");
}

#[test]
fn list_with_limit() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq.list(None, None, Some(1), None).unwrap();
    assert_eq!(results.len(), 1);
}

// ---------------------------------------------------------------------------
// search
// ---------------------------------------------------------------------------

#[test]
fn search_with_name_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq
        .search(&SearchSetsParams {
            name: Some("Masters".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["name"], "Masters 25");
}

#[test]
fn search_with_type_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq
        .search(&SearchSetsParams {
            set_type: Some("draft_innovation".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["code"], "MH2");
}

#[test]
fn search_with_release_year_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq
        .search(&SearchSetsParams {
            release_year: Some(2021),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["code"], "MH2");
}

#[test]
fn search_with_no_matches() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let results = sq
        .search(&SearchSetsParams {
            name: Some("Nonexistent".to_string()),
            ..Default::default()
        })
        .unwrap();
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// count
// ---------------------------------------------------------------------------

#[test]
fn count_returns_total() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let cnt = sq.count(None).unwrap();
    assert_eq!(cnt, 2);
}

#[test]
fn count_with_type_filter() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let cnt = sq.count(Some("masters")).unwrap();
    assert_eq!(cnt, 1);
}

#[test]
fn count_with_unknown_type_returns_zero() {
    let (conn, _tmp) = common::setup_sample_db();
    let sq = SetQuery::new(&conn);

    let cnt = sq.count(Some("nonexistent_type")).unwrap();
    assert_eq!(cnt, 0);
}
