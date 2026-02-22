//! Unit tests for the SqlBuilder query construction.

use mtgjson_sdk::SqlBuilder;

// ---------------------------------------------------------------------------
// Basic construction
// ---------------------------------------------------------------------------

#[test]
fn new_creates_select_star_from_table() {
    let (sql, params) = SqlBuilder::new("cards").build();
    assert_eq!(sql, "SELECT *\nFROM cards");
    assert!(params.is_empty());
}

#[test]
fn select_replaces_default_star() {
    let (sql, _) = SqlBuilder::new("cards")
        .select(&["name", "setCode"])
        .build();
    assert!(sql.starts_with("SELECT name, setCode\n"));
}

// ---------------------------------------------------------------------------
// WHERE conditions
// ---------------------------------------------------------------------------

#[test]
fn where_eq_adds_equality_with_param() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_eq("setCode", "MH3")
        .build();
    assert!(sql.contains("WHERE setCode = ?"));
    assert_eq!(params, vec!["MH3"]);
}

#[test]
fn where_like_adds_case_insensitive_like() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_like("name", "Lightning%")
        .build();
    assert!(sql.contains("LOWER(name) LIKE LOWER(?)"));
    assert_eq!(params, vec!["Lightning%"]);
}

#[test]
fn where_in_adds_in_clause() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_in("uuid", &["a", "b", "c"])
        .build();
    assert!(sql.contains("uuid IN (?, ?, ?)"));
    assert_eq!(params, vec!["a", "b", "c"]);
}

#[test]
fn where_in_empty_produces_false() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_in("uuid", &[])
        .build();
    assert!(sql.contains("WHERE FALSE"));
    assert!(params.is_empty());
}

#[test]
fn where_gte_adds_comparison() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_gte("manaValue", "3")
        .build();
    assert!(sql.contains("manaValue >= ?"));
    assert_eq!(params, vec!["3"]);
}

#[test]
fn where_lte_adds_comparison() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_lte("manaValue", "5")
        .build();
    assert!(sql.contains("manaValue <= ?"));
    assert_eq!(params, vec!["5"]);
}

#[test]
fn where_regex_adds_regexp_matches() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_regex("text", "^Deal \\d+ damage")
        .build();
    assert!(sql.contains("regexp_matches(text, ?)"));
    assert_eq!(params, vec!["^Deal \\d+ damage"]);
}

#[test]
fn where_fuzzy_adds_jaro_winkler() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_fuzzy("name", "Lightening Bolt", 0.85)
        .build();
    assert!(sql.contains("jaro_winkler_similarity(name, ?) > 0.85"));
    assert_eq!(params, vec!["Lightening Bolt"]);
}

#[test]
fn where_or_creates_or_group() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_or(&[("name = ?", "Bolt"), ("name = ?", "Counter")])
        .build();
    assert!(sql.contains("(name = ? OR name = ?)"));
    assert_eq!(params, vec!["Bolt", "Counter"]);
}

#[test]
fn where_or_empty_is_noop() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_or(&[])
        .build();
    assert!(!sql.contains("WHERE"));
    assert!(params.is_empty());
}

#[test]
fn where_clause_appends_params_in_order() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_eq("setCode", "A25")
        .where_clause("list_contains(colors, ?)", &["R"])
        .build();
    assert!(sql.contains("setCode = ?"));
    assert!(sql.contains("list_contains(colors, ?)"));
    assert_eq!(params, vec!["A25", "R"]);
}

// ---------------------------------------------------------------------------
// DISTINCT
// ---------------------------------------------------------------------------

#[test]
fn distinct_adds_keyword() {
    let (sql, _) = SqlBuilder::new("cards")
        .distinct()
        .build();
    assert!(sql.starts_with("SELECT DISTINCT *"));
}

// ---------------------------------------------------------------------------
// JOIN
// ---------------------------------------------------------------------------

#[test]
fn join_adds_clause() {
    let (sql, _) = SqlBuilder::new("cards c")
        .join("JOIN sets s ON c.setCode = s.code")
        .build();
    assert!(sql.contains("JOIN sets s ON c.setCode = s.code"));
}

// ---------------------------------------------------------------------------
// GROUP BY / HAVING
// ---------------------------------------------------------------------------

#[test]
fn group_by_adds_clause() {
    let (sql, _) = SqlBuilder::new("cards")
        .select(&["setCode", "COUNT(*) AS cnt"])
        .group_by(&["setCode"])
        .build();
    assert!(sql.contains("GROUP BY setCode"));
}

#[test]
fn having_adds_clause_with_params() {
    let (sql, params) = SqlBuilder::new("cards")
        .select(&["setCode", "COUNT(*) AS cnt"])
        .group_by(&["setCode"])
        .having("COUNT(*) > ?", &["5"])
        .build();
    assert!(sql.contains("HAVING COUNT(*) > ?"));
    assert_eq!(params, vec!["5"]);
}

#[test]
fn having_params_ordered_after_where_params() {
    let (_, params) = SqlBuilder::new("cards")
        .select(&["setCode", "COUNT(*) AS cnt"])
        .where_eq("rarity", "uncommon")
        .group_by(&["setCode"])
        .having("COUNT(*) > ?", &["2"])
        .build();
    assert_eq!(params, vec!["uncommon", "2"]);
}

// ---------------------------------------------------------------------------
// ORDER BY
// ---------------------------------------------------------------------------

#[test]
fn order_by_adds_clause() {
    let (sql, _) = SqlBuilder::new("cards")
        .order_by(&["name ASC", "manaValue DESC"])
        .build();
    assert!(sql.contains("ORDER BY name ASC, manaValue DESC"));
}

// ---------------------------------------------------------------------------
// LIMIT / OFFSET
// ---------------------------------------------------------------------------

#[test]
fn limit_adds_clause() {
    let (sql, _) = SqlBuilder::new("cards")
        .limit(10)
        .build();
    assert!(sql.contains("LIMIT 10"));
}

#[test]
fn offset_adds_clause() {
    let (sql, _) = SqlBuilder::new("cards")
        .offset(20)
        .build();
    assert!(sql.contains("OFFSET 20"));
}

#[test]
fn limit_and_offset_together() {
    let (sql, _) = SqlBuilder::new("cards")
        .limit(10)
        .offset(20)
        .build();
    assert!(sql.contains("LIMIT 10"));
    assert!(sql.contains("OFFSET 20"));
}

// ---------------------------------------------------------------------------
// Combined / chained
// ---------------------------------------------------------------------------

#[test]
fn combined_builder_chains_correctly() {
    let (sql, params) = SqlBuilder::new("cards")
        .where_eq("setCode", "MH3")
        .where_like("name", "Lightning%")
        .where_gte("manaValue", "1")
        .order_by(&["name ASC"])
        .limit(10)
        .offset(0)
        .build();

    assert!(sql.contains("setCode = ?"));
    assert!(sql.contains("LOWER(name) LIKE LOWER(?)"));
    assert!(sql.contains("manaValue >= ?"));
    assert!(sql.contains("ORDER BY name ASC"));
    assert!(sql.contains("LIMIT 10"));
    assert!(sql.contains("OFFSET 0"));
    assert_eq!(params.len(), 3);
    assert_eq!(params[0], "MH3");
    assert_eq!(params[1], "Lightning%");
    assert_eq!(params[2], "1");
}

#[test]
fn multiple_where_clauses_joined_with_and() {
    let (sql, _) = SqlBuilder::new("cards")
        .where_eq("setCode", "A25")
        .where_eq("rarity", "uncommon")
        .build();
    assert!(sql.contains("WHERE setCode = ? AND rarity = ?"));
}

#[test]
fn full_query_with_join_and_grouping() {
    let (sql, params) = SqlBuilder::new("cards c")
        .select(&["c.setCode", "COUNT(*) AS cnt"])
        .join("JOIN sets s ON c.setCode = s.code")
        .where_eq("s.type", "masters")
        .group_by(&["c.setCode"])
        .having("COUNT(*) >= ?", &["10"])
        .order_by(&["cnt DESC"])
        .limit(5)
        .build();

    assert!(sql.contains("SELECT c.setCode, COUNT(*) AS cnt"));
    assert!(sql.contains("FROM cards c"));
    assert!(sql.contains("JOIN sets s ON c.setCode = s.code"));
    assert!(sql.contains("WHERE s.type = ?"));
    assert!(sql.contains("GROUP BY c.setCode"));
    assert!(sql.contains("HAVING COUNT(*) >= ?"));
    assert!(sql.contains("ORDER BY cnt DESC"));
    assert!(sql.contains("LIMIT 5"));
    assert_eq!(params, vec!["masters", "10"]);
}
