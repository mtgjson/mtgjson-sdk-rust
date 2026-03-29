//! Tests for set_decks table (Connection layer — JSON column parsing).

mod common;

#[test]
fn main_board_parsed_as_array() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT mainBoard FROM set_decks WHERE code = ?",
            &["A25_DECK1"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let board = rows[0]["mainBoard"].as_array().expect("expected array");
    assert_eq!(board.len(), 2);
    assert_eq!(board[0]["uuid"], "card-uuid-001");
    assert_eq!(board[0]["count"], 4);
}

#[test]
fn side_board_parsed_as_array() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT sideBoard FROM set_decks WHERE code = ?",
            &["A25_DECK1"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let board = rows[0]["sideBoard"].as_array().expect("expected array");
    assert_eq!(board.len(), 1);
}

#[test]
fn tokens_parsed_as_array() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT tokens FROM set_decks WHERE code = ?",
            &["A25_DECK1"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let tokens = rows[0]["tokens"].as_array().expect("expected array");
    assert_eq!(tokens[0]["uuid"], "token-uuid-001");
}

#[test]
fn sealed_product_uuids_parsed_as_array() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT sealedProductUuids FROM set_decks WHERE code = ?",
            &["A25_DECK1"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let uuids = rows[0]["sealedProductUuids"]
        .as_array()
        .expect("expected array");
    assert_eq!(uuids[0], "sealed-uuid-001");
}

#[test]
fn source_set_codes_parsed_as_array() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT sourceSetCodes FROM set_decks WHERE code = ?",
            &["A25_DECK1"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let codes = rows[0]["sourceSetCodes"]
        .as_array()
        .expect("expected array");
    assert_eq!(codes[0], "A25");
}

#[test]
fn commander_parsed_as_array() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT commander FROM set_decks WHERE code = ?",
            &["A25_DECK1"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let cmdr = rows[0]["commander"].as_array().expect("expected array");
    assert!(cmdr.is_empty());
}

#[test]
fn filter_by_set_code() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT * FROM set_decks WHERE setCode = ?",
            &["MH2"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["name"], "Modern Horizons 2 Theme Deck");
}
