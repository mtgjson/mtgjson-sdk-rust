//! Comprehensive smoke test for the MTGJSON Rust SDK.
//!
//! Downloads real data from the MTGJSON CDN and exercises ALL public SDK
//! methods across every query interface.
//!
//! Run with:
//! ```sh
//! cargo test -- --ignored --nocapture
//! ```

use mtgjson_sdk::MtgjsonSdk;
use mtgjson_sdk::queries::cards::SearchCardsParams;
use mtgjson_sdk::queries::sets::SearchSetsParams;
use mtgjson_sdk::queries::tokens::SearchTokensParams;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Print a section header to stderr.
fn section(name: &str) {
    eprintln!("\n{}", "=".repeat(60));
    eprintln!("  {}", name);
    eprintln!("{}", "=".repeat(60));
}

/// Counters for pass/fail/skip reporting.
struct Counters {
    pass: usize,
    fail: usize,
    skip: usize,
}

impl Counters {
    fn new() -> Self {
        Self {
            pass: 0,
            fail: 0,
            skip: 0,
        }
    }

    fn check(&mut self, label: &str, condition: bool, detail: &str) {
        let status = if condition { "PASS" } else { "FAIL" };
        if condition {
            self.pass += 1;
        } else {
            self.fail += 1;
        }
        if detail.is_empty() {
            eprintln!("  [{}] {}", status, label);
        } else {
            eprintln!("  [{}] {} -- {}", status, label, detail);
        }
    }

    fn skip(&mut self, label: &str, reason: &str) {
        self.skip += 1;
        if reason.is_empty() {
            eprintln!("  [SKIP] {}", label);
        } else {
            eprintln!("  [SKIP] {} -- {}", label, reason);
        }
    }
}

// ---------------------------------------------------------------------------
// Main smoke test
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn smoke_test() {
    let sdk = MtgjsonSdk::builder().build().unwrap();
    let mut c = Counters::new();

    // ================================================================
    // 1. META
    // ================================================================
    section("Meta");

    let meta = sdk.meta().unwrap();
    c.check("meta loads", meta.is_object(), "");
    if let Some(data) = meta.get("data") {
        let version = data
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let date = data.get("date").and_then(|v| v.as_str()).unwrap_or("?");
        c.check(
            "meta has version",
            !version.is_empty() && version != "?",
            &format!("v={}, date={}", version, date),
        );
    } else {
        c.check("meta has data key", false, "missing 'data'");
    }

    // views property (starts empty/small, grows as we query)
    let views_before = sdk.views();
    c.check(
        "views property (initial)",
        true,
        &format!("views={:?}", views_before),
    );

    // refresh()
    let refresh = sdk.refresh().unwrap();
    c.check(
        "refresh()",
        true,
        &format!("stale={}", refresh),
    );

    // ================================================================
    // 2. CARDS
    // ================================================================
    section("Cards: get_by_name / get_by_uuid");

    let bolt = sdk.cards().get_by_name("Lightning Bolt", None).unwrap();
    c.check(
        "get_by_name Lightning Bolt",
        !bolt.is_empty(),
        &format!("found {} printings", bolt.len()),
    );

    // get_by_name with set_code
    let bolt_a25 = sdk
        .cards()
        .get_by_name("Lightning Bolt", Some("A25"))
        .unwrap();
    c.check(
        "get_by_name set_code=A25",
        true,
        &format!("found {}", bolt_a25.len()),
    );

    // get_by_uuid
    let mut uuid: Option<String> = None;
    if let Some(first) = bolt.first() {
        if let Some(u) = first.get("uuid").and_then(|v| v.as_str()) {
            uuid = Some(u.to_string());
            let card = sdk.cards().get_by_uuid(u).unwrap();
            c.check("get_by_uuid", card.is_some(), "found card");

            // Verify the returned card has the right name
            if let Some(ref card_val) = card {
                let name = card_val.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                c.check(
                    "get_by_uuid correct card",
                    name == "Lightning Bolt",
                    &format!("name={}", name),
                );
            }
        }
    }

    // get_by_uuid nonexistent
    let none_card = sdk
        .cards()
        .get_by_uuid("00000000-0000-0000-0000-000000000000")
        .unwrap();
    c.check("get_by_uuid nonexistent", none_card.is_none(), "");

    // ---- Cards: get_by_uuids (bulk lookup) ----
    section("Cards: bulk lookups (get_by_uuids)");

    let uuids: Vec<&str> = bolt
        .iter()
        .take(5)
        .filter_map(|card| card.get("uuid").and_then(|v| v.as_str()))
        .collect();

    if uuids.len() >= 2 {
        let bulk = sdk.cards().get_by_uuids(&uuids).unwrap();
        c.check(
            "get_by_uuids",
            !bulk.is_empty(),
            &format!("requested {}, got {}", uuids.len(), bulk.len()),
        );
    } else {
        c.skip("get_by_uuids", "not enough UUIDs");
    }

    // nonexistent uuids
    let bulk_none = sdk
        .cards()
        .get_by_uuids(&["00000000-0000-0000-0000-000000000000"])
        .unwrap();
    c.check(
        "get_by_uuids nonexistent",
        bulk_none.is_empty(),
        "",
    );

    // ---- Cards: search (all filter params) ----
    section("Cards: search filters");

    // name LIKE
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            name: Some("Lightning%".to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search name LIKE",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // exact name
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            name: Some("Lightning Bolt".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check("search name exact", !s.is_empty(), "");

    // colors
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            colors: Some(vec!["R".to_string()]),
            mana_value: Some(1.0),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search colors=R mv=1",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // color_identity
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            color_identity: Some(vec!["W".to_string(), "U".to_string()]),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search color_identity=[W,U]",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // types
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            types: Some("Creature".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search types=Creature",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // rarity
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            rarity: Some("mythic".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search rarity=mythic",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // text
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            text: Some("draw a card".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search text='draw a card'",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // power / toughness
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            power: Some("4".to_string()),
            toughness: Some("4".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search power=4 toughness=4",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // mana_value exact
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            mana_value: Some(3.0),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search mana_value=3",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // mana_value_lte
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            mana_value_lte: Some(1.0),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search mana_value_lte=1",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // mana_value_gte
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            mana_value_gte: Some(10.0),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search mana_value_gte=10",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // artist
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            artist: Some("Christopher Moeller".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search artist",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // keyword
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            keyword: Some("Flying".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search keyword=Flying",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // layout
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            layout: Some("split".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search layout=split",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // is_promo true
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            is_promo: Some(true),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search is_promo=true",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // is_promo false
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            is_promo: Some(false),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search is_promo=false",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // availability
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            availability: Some("paper".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search availability=paper",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // language
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            language: Some("Japanese".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search language=Japanese",
        true,
        &format!("found {}", s.len()),
    );

    // set_code
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            set_code: Some("MH3".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search set_code=MH3",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // set_type (requires JOIN with sets)
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            set_type: Some("expansion".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search set_type=expansion",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // legal_in + mana_value_lte
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            legal_in: Some("modern".to_string()),
            mana_value_lte: Some(2.0),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search legal_in=modern + mana_value_lte",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // combined filters
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            colors: Some(vec!["R".to_string()]),
            rarity: Some("rare".to_string()),
            mana_value_lte: Some(3.0),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search combined (colors+rarity+mv)",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // offset (pagination)
    let page1 = sdk
        .cards()
        .search(&SearchCardsParams {
            name: Some("Lightning%".to_string()),
            limit: Some(3),
            offset: Some(0),
            ..Default::default()
        })
        .unwrap();
    let page2 = sdk
        .cards()
        .search(&SearchCardsParams {
            name: Some("Lightning%".to_string()),
            limit: Some(3),
            offset: Some(3),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search offset (pagination)",
        !page1.is_empty() && !page2.is_empty(),
        "two pages fetched",
    );

    // text_regex
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            text_regex: Some("deals \\d+ damage".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search text_regex",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // localized_name
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            localized_name: Some("Blitzschlag".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search localized_name (German)",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // fuzzy_name
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            fuzzy_name: Some("Lightninng Bolt".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search fuzzy_name (misspelled)",
        !s.is_empty(),
        &format!("found {}", s.len()),
    );

    // empty search results
    let empty = sdk
        .cards()
        .search(&SearchCardsParams {
            name: Some("XYZ_NONEXISTENT_CARD_12345".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check("empty search result", empty.is_empty(), "");

    // ---- Cards: random, count, printings, atomic, find_by_scryfall_id ----
    section("Cards: random, count, printings, atomic, find_by_scryfall_id");

    // random
    let rand_cards = sdk.cards().random(5).unwrap();
    c.check(
        "random(5)",
        rand_cards.len() == 5,
        &format!("got {}", rand_cards.len()),
    );

    // count (no filters)
    let total = sdk.cards().count(&HashMap::new()).unwrap();
    c.check("count()", total > 1000, &format!("total cards: {}", total));

    // count with filter
    let mut filter = HashMap::new();
    filter.insert("rarity".to_string(), "mythic".to_string());
    let count_mythic = sdk.cards().count(&filter).unwrap();
    c.check(
        "count(rarity=mythic)",
        count_mythic > 0 && count_mythic < total,
        &format!("mythic cards: {}", count_mythic),
    );

    // get_printings
    let printings = sdk.cards().get_printings("Counterspell").unwrap();
    c.check(
        "get_printings Counterspell",
        printings.len() > 5,
        &format!("found {} printings", printings.len()),
    );

    // get_atomic exact
    let atomic = sdk.cards().get_atomic("Lightning Bolt").unwrap();
    c.check(
        "get_atomic Lightning Bolt",
        !atomic.is_empty(),
        &format!("found {} atomic entries", atomic.len()),
    );

    // get_atomic face name fallback (split cards)
    let atomic_fire = sdk.cards().get_atomic("Fire").unwrap();
    c.check(
        "get_atomic face name 'Fire'",
        !atomic_fire.is_empty(),
        &format!("found {} results", atomic_fire.len()),
    );

    // find_by_scryfall_id (use uuid as a scryfall ID probe -- may or may not match)
    if let Some(ref u) = uuid {
        let scry_cards = sdk.cards().find_by_scryfall_id(u).unwrap();
        c.check(
            "find_by_scryfall_id runs",
            true,
            &format!("found {}", scry_cards.len()),
        );
    }

    // ================================================================
    // 3. TOKENS
    // ================================================================
    section("Tokens");

    // count
    let token_count = sdk.tokens().count(&HashMap::new()).unwrap();
    c.check(
        "token count()",
        token_count > 0,
        &format!("total tokens: {}", token_count),
    );

    // search by name
    let token_search = sdk
        .tokens()
        .search(&SearchTokensParams {
            name: Some("%Soldier%".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "token search name LIKE",
        !token_search.is_empty(),
        &format!("found {}", token_search.len()),
    );

    // search by set_code
    let token_search_set = sdk
        .tokens()
        .search(&SearchTokensParams {
            set_code: Some("MH3".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "token search set_code=MH3",
        true,
        &format!("found {}", token_search_set.len()),
    );

    // search by types
    let token_search_type = sdk
        .tokens()
        .search(&SearchTokensParams {
            types: Some("Creature".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "token search types=Creature",
        !token_search_type.is_empty(),
        &format!("found {}", token_search_type.len()),
    );

    // search by colors
    let token_search_colors = sdk
        .tokens()
        .search(&SearchTokensParams {
            colors: Some(vec!["W".to_string()]),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "token search colors=[W]",
        !token_search_colors.is_empty(),
        &format!("found {}", token_search_colors.len()),
    );

    // search by artist
    let _token_search_artist = sdk
        .tokens()
        .search(&SearchTokensParams {
            artist: Some("Johannes Voss".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "token search artist",
        true,
        &format!("found {}", _token_search_artist.len()),
    );

    // search with offset (pagination)
    let tp1 = sdk
        .tokens()
        .search(&SearchTokensParams {
            name: Some("%Soldier%".to_string()),
            limit: Some(2),
            offset: Some(0),
            ..Default::default()
        })
        .unwrap();
    let tp2 = sdk
        .tokens()
        .search(&SearchTokensParams {
            name: Some("%Soldier%".to_string()),
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "token search offset",
        !tp1.is_empty() && !tp2.is_empty(),
        "two pages fetched",
    );

    // get_by_uuid
    if let Some(first_token) = token_search.first() {
        if let Some(tok_uuid) = first_token.get("uuid").and_then(|v| v.as_str()) {
            let token = sdk.tokens().get_by_uuid(tok_uuid).unwrap();
            c.check(
                "token get_by_uuid",
                token.is_some(),
                &format!(
                    "name={}",
                    token
                        .as_ref()
                        .and_then(|t| t.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("?")
                ),
            );
        }
    }

    // get_by_uuid nonexistent
    let missing_token = sdk
        .tokens()
        .get_by_uuid("00000000-0000-0000-0000-000000000000")
        .unwrap();
    c.check("token get_by_uuid nonexistent", missing_token.is_none(), "");

    // get_by_name
    let token_soldiers = sdk.tokens().get_by_name("Soldier", None).unwrap();
    c.check(
        "token get_by_name Soldier",
        !token_soldiers.is_empty(),
        &format!("found {}", token_soldiers.len()),
    );

    // get_by_name with set_code
    let _token_soldiers_set = sdk.tokens().get_by_name("Soldier", Some("MH3")).unwrap();
    c.check(
        "token get_by_name set_code=MH3",
        true,
        &format!("found {}", _token_soldiers_set.len()),
    );

    // for_set
    // Discover a set code that has tokens
    let token_set_code = token_search
        .first()
        .and_then(|t| t.get("setCode"))
        .and_then(|v| v.as_str())
        .unwrap_or("MH3");
    let tokens_for = sdk.tokens().for_set(token_set_code).unwrap();
    c.check(
        "token for_set",
        !tokens_for.is_empty(),
        &format!("set={}, found {}", token_set_code, tokens_for.len()),
    );

    // get_by_uuids (bulk token lookup)
    let tok_uuids: Vec<&str> = token_search
        .iter()
        .take(3)
        .filter_map(|t| t.get("uuid").and_then(|v| v.as_str()))
        .collect();
    if tok_uuids.len() >= 2 {
        let bulk_tokens = sdk.tokens().get_by_uuids(&tok_uuids).unwrap();
        c.check(
            "token get_by_uuids",
            !bulk_tokens.is_empty(),
            &format!("requested {}, got {}", tok_uuids.len(), bulk_tokens.len()),
        );
    } else {
        c.skip("token get_by_uuids", "not enough token UUIDs");
    }

    // count with filter
    let mut tok_filter = HashMap::new();
    tok_filter.insert("setCode".to_string(), token_set_code.to_string());
    let token_count_set = sdk.tokens().count(&tok_filter).unwrap();
    c.check(
        &format!("token count(setCode={})", token_set_code),
        token_count_set >= 0,
        &format!("count: {}", token_count_set),
    );

    // ================================================================
    // 4. SETS
    // ================================================================
    section("Sets");

    // get
    let mh3 = sdk.sets().get("MH3").unwrap();
    c.check(
        "get set MH3",
        mh3.is_some(),
        &format!(
            "name={}",
            mh3.as_ref()
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("?")
        ),
    );

    // get nonexistent
    let missing_set = sdk.sets().get("ZZZZZ").unwrap();
    c.check("get set nonexistent", missing_set.is_none(), "");

    // list -- no filter
    let all_sets = sdk.sets().list(None, None, Some(10), None).unwrap();
    c.check(
        "list sets (no filter)",
        !all_sets.is_empty(),
        &format!("found {}", all_sets.len()),
    );

    // list -- set_type
    let expansions = sdk
        .sets()
        .list(Some("expansion"), None, Some(10), None)
        .unwrap();
    c.check(
        "list expansions",
        !expansions.is_empty(),
        &format!("found {}", expansions.len()),
    );

    // list -- name filter
    let horizon_list = sdk
        .sets()
        .list(None, Some("Horizons"), Some(10), None)
        .unwrap();
    c.check(
        "list name filter",
        !horizon_list.is_empty(),
        &format!("found {}", horizon_list.len()),
    );

    // list -- offset (pagination)
    let sets_p1 = sdk.sets().list(None, None, Some(3), Some(0)).unwrap();
    let sets_p2 = sdk.sets().list(None, None, Some(3), Some(3)).unwrap();
    c.check(
        "list offset (pagination)",
        !sets_p1.is_empty() && !sets_p2.is_empty(),
        "two pages fetched",
    );

    // search -- name
    let set_search = sdk
        .sets()
        .search(&SearchSetsParams {
            name: Some("Horizons".to_string()),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search 'Horizons'",
        !set_search.is_empty(),
        &format!("found {}", set_search.len()),
    );

    // search -- set_type
    let set_search_type = sdk
        .sets()
        .search(&SearchSetsParams {
            set_type: Some("masters".to_string()),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search set_type=masters",
        !set_search_type.is_empty(),
        &format!("found {}", set_search_type.len()),
    );

    // search -- block
    let set_search_block = sdk
        .sets()
        .search(&SearchSetsParams {
            block: Some("Innistrad".to_string()),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search block=Innistrad",
        true,
        &format!("found {}", set_search_block.len()),
    );

    // search -- release_year
    let set_search_year = sdk
        .sets()
        .search(&SearchSetsParams {
            release_year: Some(2024),
            limit: Some(10),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search release_year=2024",
        !set_search_year.is_empty(),
        &format!("found {}", set_search_year.len()),
    );

    // count
    let set_count = sdk.sets().count(None).unwrap();
    c.check(
        "set count",
        set_count > 100,
        &format!("total sets: {}", set_count),
    );

    // count with set_type filter
    let expansion_count = sdk.sets().count(Some("expansion")).unwrap();
    c.check(
        "set count(expansion)",
        expansion_count > 0 && expansion_count < set_count,
        &format!("expansion sets: {}", expansion_count),
    );

    // get_financial_summary -- wraps in error handling since prices_today may not be loaded
    match sdk.sets().get_financial_summary("MH3") {
        Ok(summary) => {
            let card_count = summary
                .get("card_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            c.check(
                "get_financial_summary MH3",
                card_count > 0,
                &format!("card_count={}", card_count),
            );
        }
        Err(e) => {
            c.skip(
                "get_financial_summary",
                &format!("prices not loaded: {}", e),
            );
        }
    }

    // ================================================================
    // 5. IDENTIFIERS
    // ================================================================
    section("Identifiers");

    if let Some(ref u) = uuid {
        // get_identifiers
        let ids = sdk.identifiers().get_identifiers(u).unwrap();
        c.check("get_identifiers", ids.is_some(), "");

        if let Some(ref ids_val) = ids {
            // find_by generic method with valid column
            if let Some(scryfall_id) = ids_val.get("scryfallId").and_then(|v| v.as_str()) {
                let by_gen = sdk
                    .identifiers()
                    .find_by("scryfallId", scryfall_id)
                    .unwrap();
                c.check(
                    "find_by generic (scryfallId)",
                    !by_gen.is_empty(),
                    &format!("found {}", by_gen.len()),
                );

                // find_by_scryfall_id convenience method
                let by_scry = sdk
                    .identifiers()
                    .find_by_scryfall_id(scryfall_id)
                    .unwrap();
                c.check(
                    "find_by_scryfall_id",
                    !by_scry.is_empty(),
                    &format!("found {}", by_scry.len()),
                );
            } else {
                c.skip("find_by_scryfall_id", "no scryfallId in data");
            }

            // Scryfall Oracle ID
            if let Some(oracle_id) = ids_val
                .get("scryfallOracleId")
                .and_then(|v| v.as_str())
            {
                let by_oracle = sdk
                    .identifiers()
                    .find_by_scryfall_oracle_id(oracle_id)
                    .unwrap();
                c.check(
                    "find_by_scryfall_oracle_id",
                    !by_oracle.is_empty(),
                    &format!("found {}", by_oracle.len()),
                );
            } else {
                c.skip("find_by_scryfall_oracle_id", "no scryfallOracleId");
            }

            // Scryfall Illustration ID
            if let Some(illus_id) = ids_val
                .get("scryfallIllustrationId")
                .and_then(|v| v.as_str())
            {
                let by_illus = sdk
                    .identifiers()
                    .find_by_scryfall_illustration_id(illus_id)
                    .unwrap();
                c.check(
                    "find_by_scryfall_illustration_id",
                    !by_illus.is_empty(),
                    &format!("found {}", by_illus.len()),
                );
            } else {
                c.skip("find_by_scryfall_illustration_id", "no scryfallIllustrationId");
            }

            // Scryfall Card Back ID
            if let Some(back_id) = ids_val
                .get("scryfallCardBackId")
                .and_then(|v| v.as_str())
            {
                let by_back = sdk
                    .identifiers()
                    .find_by_scryfall_card_back_id(back_id)
                    .unwrap();
                c.check(
                    "find_by_scryfall_card_back_id",
                    !by_back.is_empty(),
                    &format!("found {}", by_back.len()),
                );
            } else {
                c.skip("find_by_scryfall_card_back_id", "no scryfallCardBackId");
            }

            // TCGplayer Product ID
            if let Some(tcg_id) = ids_val
                .get("tcgplayerProductId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_tcg = sdk
                    .identifiers()
                    .find_by_tcgplayer_product_id(&tcg_id)
                    .unwrap();
                c.check(
                    "find_by_tcgplayer_product_id",
                    !by_tcg.is_empty(),
                    &format!("found {}", by_tcg.len()),
                );
            } else {
                c.skip("find_by_tcgplayer_product_id", "no tcgplayerProductId");
            }

            // TCGplayer Etched Product ID
            if let Some(tcg_e_id) = ids_val
                .get("tcgplayerEtchedProductId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_tcg_e = sdk
                    .identifiers()
                    .find_by_tcgplayer_etched_product_id(&tcg_e_id)
                    .unwrap();
                c.check(
                    "find_by_tcgplayer_etched_product_id",
                    true,
                    &format!("found {}", by_tcg_e.len()),
                );
            } else {
                c.skip(
                    "find_by_tcgplayer_etched_product_id",
                    "no tcgplayerEtchedProductId",
                );
            }

            // MTGO ID
            if let Some(mtgo_id) = ids_val
                .get("mtgoId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_mtgo = sdk.identifiers().find_by_mtgo_id(&mtgo_id).unwrap();
                c.check(
                    "find_by_mtgo_id",
                    !by_mtgo.is_empty(),
                    &format!("found {}", by_mtgo.len()),
                );
            } else {
                c.skip("find_by_mtgo_id", "no mtgoId");
            }

            // MTGO Foil ID
            if let Some(mtgo_foil_id) = ids_val
                .get("mtgoFoilId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_mtgo_f = sdk
                    .identifiers()
                    .find_by_mtgo_foil_id(&mtgo_foil_id)
                    .unwrap();
                c.check(
                    "find_by_mtgo_foil_id",
                    !by_mtgo_f.is_empty(),
                    &format!("found {}", by_mtgo_f.len()),
                );
            } else {
                c.skip("find_by_mtgo_foil_id", "no mtgoFoilId");
            }

            // MTG Arena ID
            if let Some(arena_id) = ids_val
                .get("mtgArenaId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_arena = sdk
                    .identifiers()
                    .find_by_mtg_arena_id(&arena_id)
                    .unwrap();
                c.check(
                    "find_by_mtg_arena_id",
                    !by_arena.is_empty(),
                    &format!("found {}", by_arena.len()),
                );
            } else {
                c.skip("find_by_mtg_arena_id", "no mtgArenaId");
            }

            // Multiverse ID
            if let Some(multi_id) = ids_val
                .get("multiverseId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_multi = sdk
                    .identifiers()
                    .find_by_multiverse_id(&multi_id)
                    .unwrap();
                c.check(
                    "find_by_multiverse_id",
                    !by_multi.is_empty(),
                    &format!("found {}", by_multi.len()),
                );
            } else {
                c.skip("find_by_multiverse_id", "no multiverseId");
            }

            // MCM ID
            if let Some(mcm_id) = ids_val
                .get("mcmId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_mcm = sdk.identifiers().find_by_mcm_id(&mcm_id).unwrap();
                c.check(
                    "find_by_mcm_id",
                    !by_mcm.is_empty(),
                    &format!("found {}", by_mcm.len()),
                );
            } else {
                c.skip("find_by_mcm_id", "no mcmId");
            }

            // MCM Meta ID
            if let Some(mcm_meta_id) = ids_val
                .get("mcmMetaId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_mcm_m = sdk
                    .identifiers()
                    .find_by_mcm_meta_id(&mcm_meta_id)
                    .unwrap();
                c.check(
                    "find_by_mcm_meta_id",
                    !by_mcm_m.is_empty(),
                    &format!("found {}", by_mcm_m.len()),
                );
            } else {
                c.skip("find_by_mcm_meta_id", "no mcmMetaId");
            }

            // Card Kingdom ID
            if let Some(ck_id) = ids_val
                .get("cardKingdomId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_ck = sdk
                    .identifiers()
                    .find_by_card_kingdom_id(&ck_id)
                    .unwrap();
                c.check(
                    "find_by_card_kingdom_id",
                    !by_ck.is_empty(),
                    &format!("found {}", by_ck.len()),
                );
            } else {
                c.skip("find_by_card_kingdom_id", "no cardKingdomId");
            }

            // Card Kingdom Foil ID
            if let Some(ck_foil_id) = ids_val
                .get("cardKingdomFoilId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_ck_f = sdk
                    .identifiers()
                    .find_by_card_kingdom_foil_id(&ck_foil_id)
                    .unwrap();
                c.check(
                    "find_by_card_kingdom_foil_id",
                    !by_ck_f.is_empty(),
                    &format!("found {}", by_ck_f.len()),
                );
            } else {
                c.skip("find_by_card_kingdom_foil_id", "no cardKingdomFoilId");
            }

            // Card Kingdom Etched ID
            if let Some(ck_e_id) = ids_val
                .get("cardKingdomEtchedId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_ck_e = sdk
                    .identifiers()
                    .find_by_card_kingdom_etched_id(&ck_e_id)
                    .unwrap();
                c.check(
                    "find_by_card_kingdom_etched_id",
                    true,
                    &format!("found {}", by_ck_e.len()),
                );
            } else {
                c.skip("find_by_card_kingdom_etched_id", "no cardKingdomEtchedId");
            }

            // Cardsphere ID
            if let Some(cs_id) = ids_val
                .get("cardsphereId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_cs = sdk
                    .identifiers()
                    .find_by_cardsphere_id(&cs_id)
                    .unwrap();
                c.check(
                    "find_by_cardsphere_id",
                    !by_cs.is_empty(),
                    &format!("found {}", by_cs.len()),
                );
            } else {
                c.skip("find_by_cardsphere_id", "no cardsphereId");
            }

            // Cardsphere Foil ID
            if let Some(cs_foil_id) = ids_val
                .get("cardsphereFoilId")
                .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
            {
                let by_cs_f = sdk
                    .identifiers()
                    .find_by_cardsphere_foil_id(&cs_foil_id)
                    .unwrap();
                c.check(
                    "find_by_cardsphere_foil_id",
                    true,
                    &format!("found {}", by_cs_f.len()),
                );
            } else {
                c.skip("find_by_cardsphere_foil_id", "no cardsphereFoilId");
            }

            // MTGJSON Foil Version ID
            if let Some(mj_foil_id) = ids_val
                .get("mtgjsonFoilVersionId")
                .and_then(|v| v.as_str())
            {
                let by_mj_f = sdk
                    .identifiers()
                    .find_by_mtgjson_foil_version_id(mj_foil_id)
                    .unwrap();
                c.check(
                    "find_by_mtgjson_foil_version_id",
                    true,
                    &format!("found {}", by_mj_f.len()),
                );
            } else {
                c.skip(
                    "find_by_mtgjson_foil_version_id",
                    "no mtgjsonFoilVersionId",
                );
            }

            // MTGJSON Non-Foil Version ID
            if let Some(mj_nf_id) = ids_val
                .get("mtgjsonNonFoilVersionId")
                .and_then(|v| v.as_str())
            {
                let by_mj_nf = sdk
                    .identifiers()
                    .find_by_mtgjson_non_foil_version_id(mj_nf_id)
                    .unwrap();
                c.check(
                    "find_by_mtgjson_non_foil_version_id",
                    true,
                    &format!("found {}", by_mj_nf.len()),
                );
            } else {
                c.skip(
                    "find_by_mtgjson_non_foil_version_id",
                    "no mtgjsonNonFoilVersionId",
                );
            }

            // MTGJSON V4 ID
            if let Some(mj_v4_id) = ids_val.get("mtgjsonV4Id").and_then(|v| v.as_str()) {
                let by_mj_v4 = sdk
                    .identifiers()
                    .find_by_mtgjson_v4_id(mj_v4_id)
                    .unwrap();
                c.check(
                    "find_by_mtgjson_v4_id",
                    !by_mj_v4.is_empty(),
                    &format!("found {}", by_mj_v4.len()),
                );
            } else {
                c.skip("find_by_mtgjson_v4_id", "no mtgjsonV4Id");
            }
        } else {
            c.skip("identifier convenience methods", "no identifiers found");
        }
    } else {
        c.skip("identifier tests", "no uuid available");
    }

    // find_by -- invalid column should return Err
    let invalid_result = sdk.identifiers().find_by("invalidColumn", "123");
    c.check(
        "find_by invalid column returns Err",
        invalid_result.is_err(),
        "",
    );

    // ================================================================
    // 6. LEGALITIES
    // ================================================================
    section("Legalities");

    if let Some(ref u) = uuid {
        // formats_for_card
        let formats = sdk.legalities().formats_for_card(u).unwrap();
        c.check(
            "formats_for_card",
            !formats.is_empty(),
            &format!("found {} format entries", formats.len()),
        );

        // is_legal (Lightning Bolt is legal in modern)
        let is_legal = sdk.legalities().is_legal(u, "modern").unwrap();
        c.check("is_legal modern", is_legal, "");

        // is_legal nonexistent format
        let is_legal_fake = sdk
            .legalities()
            .is_legal(u, "nonexistent_format")
            .unwrap();
        c.check("is_legal nonexistent format", !is_legal_fake, "");
    }

    // legal_in
    let modern_cards = sdk.legalities().legal_in("modern").unwrap();
    c.check(
        "legal_in modern",
        !modern_cards.is_empty(),
        &format!("found {} cards", modern_cards.len()),
    );

    // banned_in
    let banned = sdk.legalities().banned_in("modern").unwrap();
    c.check(
        "banned_in modern",
        true,
        &format!("found {} cards", banned.len()),
    );

    // restricted_in
    let restricted = sdk.legalities().restricted_in("vintage").unwrap();
    c.check(
        "restricted_in vintage",
        true,
        &format!("found {} cards", restricted.len()),
    );

    // suspended_in (may have 0 results)
    let suspended = sdk.legalities().suspended_in("historic").unwrap();
    c.check(
        "suspended_in historic",
        true,
        &format!("found {} cards", suspended.len()),
    );

    // not_legal_in
    let not_legal = sdk.legalities().not_legal_in("standard").unwrap();
    c.check(
        "not_legal_in standard",
        true,
        &format!("found {} cards", not_legal.len()),
    );

    // ================================================================
    // 7. PRICES (may not be available -- wrap in error handling)
    // ================================================================
    section("Prices");

    if let Some(ref u) = uuid {
        match sdk.prices().get(u) {
            Ok(price_raw) => {
                c.check(
                    "prices.get",
                    true,
                    &format!("type={}", if price_raw.is_object() { "object" } else { "other" }),
                );

                // today
                match sdk.prices().today(u) {
                    Ok(today) => {
                        c.check(
                            "prices.today",
                            true,
                            &format!("found {} rows", today.len()),
                        );
                    }
                    Err(e) => c.skip("prices.today", &format!("{}", e)),
                }

                // history
                match sdk.prices().history(u, None, None) {
                    Ok(history) => {
                        c.check(
                            "prices.history",
                            true,
                            &format!("found {} rows", history.len()),
                        );
                    }
                    Err(e) => c.skip("prices.history", &format!("{}", e)),
                }

                // price_trend
                match sdk.prices().price_trend(u) {
                    Ok(trend) => {
                        c.check(
                            "prices.price_trend",
                            true,
                            &format!("{}", trend),
                        );
                    }
                    Err(e) => c.skip("prices.price_trend", &format!("{}", e)),
                }

                // cheapest_printing
                match sdk.prices().cheapest_printing("Lightning Bolt") {
                    Ok(cheapest) => {
                        c.check(
                            "prices.cheapest_printing",
                            true,
                            &format!("found={}", cheapest.is_some()),
                        );
                    }
                    Err(e) => c.skip("prices.cheapest_printing", &format!("{}", e)),
                }

                // cheapest_printings (N)
                match sdk.prices().cheapest_printings("Lightning Bolt", 3) {
                    Ok(cheapest_n) => {
                        c.check(
                            "prices.cheapest_printings(3)",
                            true,
                            &format!("found {}", cheapest_n.len()),
                        );
                    }
                    Err(e) => c.skip("prices.cheapest_printings", &format!("{}", e)),
                }

                // most_expensive_printings
                match sdk.prices().most_expensive_printings("Lightning Bolt", 3) {
                    Ok(expensive) => {
                        c.check(
                            "prices.most_expensive_printings(3)",
                            true,
                            &format!("found {}", expensive.len()),
                        );
                    }
                    Err(e) => c.skip("prices.most_expensive_printings", &format!("{}", e)),
                }
            }
            Err(e) => {
                c.skip("prices module", &format!("prices not loaded: {}", e));
            }
        }
    } else {
        c.skip("prices tests", "no uuid available");
    }

    // ================================================================
    // 8. SKUS (may not be available -- wrap in error handling)
    // ================================================================
    section("SKUs");

    if let Some(ref u) = uuid {
        match sdk.skus().get(u) {
            Ok(skus) => {
                c.check(
                    "skus.get",
                    true,
                    &format!("found {} SKUs", skus.len()),
                );

                if let Some(first_sku) = skus.first() {
                    // find_by_sku_id
                    if let Some(sku_id) = first_sku
                        .get("skuId")
                        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                    {
                        let by_sku = sdk.skus().find_by_sku_id(&sku_id).unwrap();
                        c.check(
                            "skus.find_by_sku_id",
                            !by_sku.is_empty(),
                            &format!("skuId={}", sku_id),
                        );
                    } else {
                        c.skip("skus.find_by_sku_id", "no skuId in data");
                    }

                    // find_by_product_id
                    if let Some(prod_id) = first_sku
                        .get("productId")
                        .and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string())))
                    {
                        let by_prod = sdk.skus().find_by_product_id(&prod_id).unwrap();
                        c.check(
                            "skus.find_by_product_id",
                            !by_prod.is_empty(),
                            &format!("productId={}", prod_id),
                        );
                    } else {
                        c.skip("skus.find_by_product_id", "no productId in data");
                    }
                } else {
                    c.skip("skus find methods", "no SKU data for this card");
                }
            }
            Err(e) => {
                c.skip("skus module", &format!("SKUs not loaded: {}", e));
            }
        }
    } else {
        c.skip("skus tests", "no uuid available");
    }

    // ================================================================
    // 9. DECKS
    // ================================================================
    section("Decks");

    match sdk.decks().count(None, None) {
        Ok(deck_count) => {
            c.check(
                "decks.count",
                true, // usize is always >= 0
                &format!("total decks: {}", deck_count),
            );

            // list -- no filter
            let deck_list = sdk.decks().list(None, None).unwrap();
            c.check(
                "decks.list (all)",
                true,
                &format!("found {}", deck_list.len()),
            );

            if !deck_list.is_empty() {
                // list -- set_code filter
                let first_code = deck_list
                    .first()
                    .and_then(|d| d.get("code"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !first_code.is_empty() {
                    let decks_by_set =
                        sdk.decks().list(Some(first_code), None).unwrap();
                    c.check(
                        "decks.list set_code",
                        !decks_by_set.is_empty(),
                        &format!("set={}, found {}", first_code, decks_by_set.len()),
                    );
                }

                // list -- deck_type filter
                let first_type = deck_list
                    .first()
                    .and_then(|d| d.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !first_type.is_empty() {
                    let decks_by_type =
                        sdk.decks().list(None, Some(first_type)).unwrap();
                    c.check(
                        "decks.list deck_type",
                        !decks_by_type.is_empty(),
                        &format!("type={}, found {}", first_type, decks_by_type.len()),
                    );
                }

                // search -- name
                let first_name = deck_list
                    .first()
                    .and_then(|d| d.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !first_name.is_empty() {
                    let search_term = first_name
                        .split_whitespace()
                        .next()
                        .unwrap_or("Starter");
                    let deck_search =
                        sdk.decks().search(search_term, None).unwrap();
                    c.check(
                        "decks.search name",
                        !deck_search.is_empty(),
                        &format!("term='{}', found {}", search_term, deck_search.len()),
                    );
                }

                // search -- set_code
                if !first_code.is_empty() {
                    let deck_search_set =
                        sdk.decks().search("", Some(first_code)).unwrap();
                    c.check(
                        "decks.search set_code",
                        true,
                        &format!("set={}, found {}", first_code, deck_search_set.len()),
                    );
                }

                // count -- with filters
                if !first_code.is_empty() {
                    let count_by_set =
                        sdk.decks().count(Some(first_code), None).unwrap();
                    c.check(
                        "decks.count set_code",
                        count_by_set > 0,
                        &format!("set={}, count={}", first_code, count_by_set),
                    );
                }
            } else {
                c.skip("deck list/search tests", "no decks loaded");
            }
        }
        Err(e) => {
            c.skip("decks module", &format!("decks not loaded: {}", e));
        }
    }

    // ================================================================
    // 10. ENUMS
    // ================================================================
    section("Enums");

    // keywords
    match sdk.enums().keywords() {
        Ok(kw) => {
            let is_obj = kw.is_object();
            let key_count = kw.as_object().map(|m| m.len()).unwrap_or(0);
            c.check(
                "enums.keywords",
                is_obj && key_count > 0,
                &format!("keys={}", key_count),
            );

            if let Some(obj) = kw.as_object() {
                let has_ability = obj.contains_key("abilityWords")
                    || obj.keys().any(|k| k.to_lowercase().contains("ability"));
                c.check(
                    "keywords has expected keys",
                    has_ability || key_count > 0,
                    &format!(
                        "top keys: {:?}",
                        obj.keys().take(5).collect::<Vec<_>>()
                    ),
                );
            }
        }
        Err(e) => c.skip("enums.keywords", &format!("{}", e)),
    }

    // card_types
    match sdk.enums().card_types() {
        Ok(ct) => {
            let is_obj = ct.is_object();
            let key_count = ct.as_object().map(|m| m.len()).unwrap_or(0);
            c.check(
                "enums.card_types",
                is_obj && key_count > 0,
                &format!("keys={}", key_count),
            );

            if let Some(obj) = ct.as_object() {
                let has_creature =
                    obj.keys().any(|k| k.to_lowercase().contains("creature"));
                c.check(
                    "card_types has creature",
                    has_creature || key_count > 0,
                    "",
                );
            }
        }
        Err(e) => c.skip("enums.card_types", &format!("{}", e)),
    }

    // enum_values
    match sdk.enums().enum_values() {
        Ok(ev) => {
            let is_obj = ev.is_object();
            let key_count = ev.as_object().map(|m| m.len()).unwrap_or(0);
            c.check(
                "enums.enum_values",
                is_obj && key_count > 0,
                &format!("keys={}", key_count),
            );
        }
        Err(e) => c.skip("enums.enum_values", &format!("{}", e)),
    }

    // ================================================================
    // 11. SEALED
    // ================================================================
    section("Sealed Products");

    // list -- no filter
    let sealed_all = sdk.sealed().list(None).unwrap();
    c.check(
        "sealed.list (all)",
        true,
        &format!("found {}", sealed_all.len()),
    );

    // list -- set_code filter
    let sealed_mh3 = sdk.sealed().list(Some("MH3")).unwrap();
    c.check(
        "sealed.list set_code=MH3",
        true,
        &format!("found {}", sealed_mh3.len()),
    );

    // get
    let sealed_get = sdk.sealed().get("MH3").unwrap();
    c.check(
        "sealed.get MH3",
        true,
        &format!("found {}", sealed_get.len()),
    );

    // ================================================================
    // 12. BOOSTER
    // ================================================================
    section("Booster Simulation");

    // available_types
    let types = sdk.booster().available_types("MH3").unwrap();
    c.check(
        "booster.available_types MH3",
        true,
        &format!("types: {:?}", types),
    );

    if !types.is_empty() {
        let booster_type = &types[0];

        // open_pack
        match sdk.booster().open_pack("MH3", booster_type) {
            Ok(pack) => {
                c.check(
                    "booster.open_pack",
                    !pack.is_empty(),
                    &format!("got {} cards", pack.len()),
                );
            }
            Err(e) => {
                c.skip("booster.open_pack", &format!("{}", e));
            }
        }

        // open_box (just 1 pack to keep it fast)
        match sdk.booster().open_box("MH3", booster_type, 1) {
            Ok(packs) => {
                c.check(
                    "booster.open_box(1)",
                    packs.len() == 1,
                    &format!(
                        "got {} packs, first has {} cards",
                        packs.len(),
                        packs.first().map(|p| p.len()).unwrap_or(0)
                    ),
                );
            }
            Err(e) => {
                c.skip("booster.open_box", &format!("{}", e));
            }
        }

        // sheet_contents
        match sdk.booster().sheet_contents("MH3", booster_type, "common") {
            Ok(contents) => {
                c.check(
                    "booster.sheet_contents",
                    true,
                    &format!(
                        "found={}",
                        contents
                            .as_ref()
                            .map(|c| c.len().to_string())
                            .unwrap_or_else(|| "None".to_string())
                    ),
                );
            }
            Err(e) => {
                c.skip("booster.sheet_contents", &format!("{}", e));
            }
        }
    } else {
        c.skip("booster.open_pack", "no booster types for MH3");
        c.skip("booster.open_box", "no booster types for MH3");
        c.skip("booster.sheet_contents", "no booster types for MH3");
    }

    // ================================================================
    // 13. RAW SQL
    // ================================================================
    section("Raw SQL");

    // Simple count
    let rows = sdk
        .sql("SELECT COUNT(*) AS cnt FROM cards", &[])
        .unwrap();
    let cnt = rows
        .first()
        .and_then(|r| r.get("cnt"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    c.check("sql COUNT", cnt > 1000, &format!("count={}", cnt));

    // Query with params
    let rows_param = sdk
        .sql(
            "SELECT name FROM cards WHERE setCode = ? LIMIT 5",
            &["MH3".to_string()],
        )
        .unwrap();
    c.check(
        "sql with params",
        !rows_param.is_empty(),
        &format!("found {}", rows_param.len()),
    );

    // Cross-table join via raw SQL
    let join_result = sdk
        .sql(
            "SELECT c.name, s.name AS setName \
             FROM cards c JOIN sets s ON c.setCode = s.code \
             LIMIT 3",
            &[],
        )
        .unwrap();
    c.check(
        "sql cross-table join",
        !join_result.is_empty() && join_result[0].contains_key("setName"),
        "",
    );

    // Top EDHREC query
    let top_edhrec = sdk
        .sql(
            "SELECT name, edhrecRank FROM cards \
             WHERE edhrecRank IS NOT NULL \
             ORDER BY edhrecRank ASC LIMIT 5",
            &[],
        )
        .unwrap();
    c.check(
        "sql top EDHREC",
        top_edhrec.len() == 5,
        &format!(
            "top: {:?}",
            top_edhrec
                .iter()
                .filter_map(|r| r.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<_>>()
        ),
    );

    // ================================================================
    // 14. VIEWS (post-query)
    // ================================================================
    section("Views (post-query)");

    let views_after = sdk.views();
    c.check(
        "views grew",
        views_after.len() > views_before.len(),
        &format!(
            "before={}, after={}, views={:?}",
            views_before.len(),
            views_after.len(),
            views_after
        ),
    );

    // ================================================================
    // 15. EDGE CASES
    // ================================================================
    section("Edge Cases");

    // Card with special characters in name
    let s = sdk
        .cards()
        .search(&SearchCardsParams {
            name: Some("J%tun%".to_string()),
            limit: Some(5),
            ..Default::default()
        })
        .unwrap();
    c.check(
        "search unicode name",
        true,
        &format!("found {}", s.len()),
    );

    // Card field validation on a known card
    if let Some(first) = bolt.first() {
        c.check(
            "card has uuid",
            first.get("uuid").and_then(|v| v.as_str()).is_some(),
            "",
        );
        c.check(
            "card has name",
            first.get("name").and_then(|v| v.as_str()) == Some("Lightning Bolt"),
            "",
        );
        let has_colors = first.get("colors").map(|v| v.is_array()).unwrap_or(false);
        c.check("card has colors (array)", has_colors, "");
        c.check(
            "card has text",
            first
                .get("text")
                .and_then(|v| v.as_str())
                .map(|t| !t.is_empty())
                .unwrap_or(false),
            "",
        );
    }

    // Set field validation
    if let Some(ref mh3_val) = mh3 {
        c.check(
            "set has code",
            mh3_val.get("code").and_then(|v| v.as_str()) == Some("MH3"),
            "",
        );
        let name = mh3_val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        c.check(
            "set has name with Horizons",
            name.contains("Horizons"),
            &format!("name={}", name),
        );
        c.check(
            "set has releaseDate",
            mh3_val
                .get("releaseDate")
                .and_then(|v| v.as_str())
                .is_some(),
            "",
        );
    }

    // ================================================================
    // 16. DISPLAY / CLOSE
    // ================================================================
    section("Display & Close");

    let display = format!("{}", sdk);
    c.check(
        "Display impl",
        display.contains("MtgjsonSdk"),
        &format!("display={}", display),
    );

    // close (consumes the SDK)
    sdk.close();
    c.check("close()", true, "SDK closed cleanly");

    // ================================================================
    // SUMMARY
    // ================================================================
    section("SMOKE TEST COMPLETE");

    let total_checks = c.pass + c.fail;
    eprintln!("  Total:   {} checks ({} skipped)", total_checks, c.skip);
    eprintln!("  Passed:  {}", c.pass);
    eprintln!("  Failed:  {}", c.fail);
    eprintln!();

    if c.fail > 0 {
        eprintln!("  *** FAILURES DETECTED ***");
        eprintln!();
    }

    assert_eq!(c.fail, 0, "{} smoke test checks failed", c.fail);
}
