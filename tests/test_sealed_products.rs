//! Tests for sealed_products table (Connection layer — JSON column parsing).

mod common;

#[test]
fn contents_parsed_as_object() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT contents FROM sealed_products WHERE uuid = ?",
            &["sealed-uuid-001"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let contents = &rows[0]["contents"];
    assert!(contents.is_object(), "expected object, got {:?}", contents);
    assert!(contents.get("pack").is_some());
}

#[test]
fn identifiers_parsed_as_object() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT identifiers FROM sealed_products WHERE uuid = ?",
            &["sealed-uuid-001"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let ids = &rows[0]["identifiers"];
    assert!(ids.is_object(), "expected object, got {:?}", ids);
    assert_eq!(ids["tcgplayerProductId"], "162583");
}

#[test]
fn purchase_urls_parsed_as_object() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT purchaseUrls FROM sealed_products WHERE uuid = ?",
            &["sealed-uuid-001"],
        )
        .unwrap();
    assert_eq!(rows.len(), 1);
    let urls = &rows[0]["purchaseUrls"];
    assert!(urls.is_object(), "expected object, got {:?}", urls);
    assert!(urls.get("tcgplayer").is_some());
}

#[test]
fn filter_by_set_code() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT * FROM sealed_products WHERE setCode = ?",
            &["A25"],
        )
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn filter_by_category() {
    let (conn, _tmp) = common::setup_sample_db();
    let rows = conn
        .execute(
            "SELECT * FROM sealed_products WHERE category = ?",
            &["booster_box"],
        )
        .unwrap();
    assert_eq!(rows.len(), 2);
}
