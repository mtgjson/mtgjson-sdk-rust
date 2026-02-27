# mtgjson-sdk

A DuckDB-backed Rust query client for [MTGJSON](https://mtgjson.com) card data. Auto-downloads Parquet data from the MTGJSON CDN and exposes the full Magic: The Gathering dataset through a typed Rust API with builder-pattern queries.

## Install

TODO: CRATES.IO STUFF

## Quick Start

```rust
use mtgjson_sdk::MtgjsonSdk;
use std::collections::HashMap;

fn main() -> mtgjson_sdk::Result<()> {
    let sdk = MtgjsonSdk::builder().build()?;

    // Search for cards
    let bolts = sdk.cards().get_by_name("Lightning Bolt", None)?;
    println!("Found {} printings of Lightning Bolt", bolts.len());

    // Get a specific set
    let mh3 = sdk.sets().get("MH3")?;
    if let Some(set) = mh3 {
        println!("{} -- {} cards", set["name"], set["totalSetSize"]);
    }

    // Check format legality
    let uuid = bolts[0]["uuid"].as_str().unwrap();
    let is_legal = sdk.legalities().is_legal(uuid, "modern")?;
    println!("Modern legal: {}", is_legal);

    // Find the cheapest printing
    let cheapest = sdk.prices().cheapest_printing("Lightning Bolt")?;
    if let Some(c) = cheapest {
        println!("Cheapest: ${} ({})", c["price"], c["setCode"]);
    }

    // Raw SQL for anything else
    let rows = sdk.sql("SELECT name, manaValue FROM cards WHERE manaValue = ? LIMIT 5", &["0".into()])?;

    sdk.close();
    Ok(())
}
```

## Use Cases

### Price Tracking

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Find the cheapest printing of any card
let cheapest = sdk.prices().cheapest_printing("Ragavan, Nimble Pilferer")?;

// Price trend over time
if let Some(ref card) = cheapest {
    let uuid = card["uuid"].as_str().unwrap();
    let trend = sdk.prices().price_trend(uuid)?;
    println!("Range: ${} - ${}", trend["min_price"], trend["max_price"]);
    println!("Average: ${} over {} data points", trend["avg_price"], trend["data_points"]);

    // Full price history with date range
    let history = sdk.prices().history(uuid, Some("2024-01-01"), Some("2024-12-31"))?;

    // Most expensive printings across the entire dataset
    let priciest = sdk.prices().most_expensive_printings("Ragavan, Nimble Pilferer", 10)?;
}

sdk.close();
```

### Deck Building Helper

```rust
use mtgjson_sdk::queries::cards::SearchCardsParams;

let sdk = MtgjsonSdk::builder().build()?;

// Find modern-legal red creatures with CMC <= 2
let aggro_creatures = sdk.cards().search(&SearchCardsParams {
    colors: Some(vec!["R".into()]),
    types: Some("Creature".into()),
    mana_value_lte: Some(2.0),
    legal_in: Some("modern".into()),
    limit: Some(50),
    ..Default::default()
})?;

// Check what's banned
let banned = sdk.legalities().banned_in("modern")?;
println!("{} cards banned in Modern", banned.len());

// Search by keyword ability
let flyers = sdk.cards().search(&SearchCardsParams {
    keyword: Some("Flying".into()),
    colors: Some(vec!["W".into(), "U".into()]),
    legal_in: Some("standard".into()),
    ..Default::default()
})?;

// Fuzzy search -- handles typos
let results = sdk.cards().search(&SearchCardsParams {
    fuzzy_name: Some("Ligtning Bolt".into()),  // still finds it!
    ..Default::default()
})?;

// Find cards by foreign-language name
let blitz = sdk.cards().search(&SearchCardsParams {
    localized_name: Some("Blitzschlag".into()),  // German for Lightning Bolt
    ..Default::default()
})?;

sdk.close();
```

### Collection Management

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Cross-reference by Scryfall ID
let cards = sdk.identifiers().find_by_scryfall_id("f7a21fe4-...")?;

// Look up by TCGPlayer product ID
let cards = sdk.identifiers().find_by_tcgplayer_product_id("12345")?;

// Get all identifiers for a card (Scryfall, TCGPlayer, MTGO, Arena, etc.)
let all_ids = sdk.identifiers().get_identifiers("card-uuid-here")?;

sdk.close();
```

### Booster Pack Simulation

```rust
let sdk = MtgjsonSdk::builder().build()?;

// See what booster types are available
let types = sdk.booster().available_types("MH3")?;  // ["draft", "collector", ...]

// Open a single draft pack
let pack = sdk.booster().open_pack("MH3", "draft")?;
for card in &pack {
    println!("  {} ({})", card["name"], card["rarity"]);
}

// Open an entire box
let booster_box = sdk.booster().open_box("MH3", "draft", 36)?;
let total_cards: usize = booster_box.iter().map(|p| p.len()).sum();
println!("Opened {} packs, {} total cards", booster_box.len(), total_cards);

sdk.close();
```

## API Reference

### Cards

```rust
sdk.cards().get_by_uuid("uuid")                        // -> Result<Option<Value>>
sdk.cards().get_by_uuids(&["uuid1", "uuid2"])          // -> Result<Vec<Value>>
sdk.cards().get_by_name("Lightning Bolt", None)        // -> Result<Vec<Value>>
sdk.cards().get_by_name("Lightning Bolt", Some("A25")) // -> Result<Vec<Value>>
sdk.cards().search(&SearchCardsParams {
    name: Some("Lightning%".into()),         // name pattern (% = wildcard)
    fuzzy_name: Some("Ligtning Bolt".into()),// typo-tolerant (Jaro-Winkler)
    localized_name: Some("Blitzschlag".into()), // foreign-language name search
    colors: Some(vec!["R".into()]),          // cards containing these colors
    color_identity: Some(vec!["R".into(), "U".into()]),
    legal_in: Some("modern".into()),         // format legality
    rarity: Some("rare".into()),             // rarity filter
    mana_value: Some(1.0),                   // exact mana value
    mana_value_lte: Some(3.0),              // mana value range
    mana_value_gte: Some(1.0),
    text: Some("damage".into()),             // rules text search
    text_regex: Some(r"deals? \d+ damage".into()), // regex rules text search
    types: Some("Creature".into()),          // type line search
    artist: Some("Christopher Moeller".into()),
    keyword: Some("Flying".into()),          // keyword ability
    is_promo: Some(false),                   // promo status
    availability: Some("paper".into()),      // paper, mtgo
    language: Some("English".into()),        // language filter
    layout: Some("normal".into()),           // card layout
    set_code: Some("MH3".into()),            // filter by set
    set_type: Some("expansion".into()),      // set type (joins sets table)
    power: Some("3".into()),                 // P/T filter
    toughness: Some("3".into()),
    limit: Some(100),                        // pagination
    offset: Some(0),
    ..Default::default()
})                                                     // -> Result<Vec<Value>>
sdk.cards().get_printings("Lightning Bolt")            // all printings across sets
sdk.cards().get_atomic("Lightning Bolt")               // oracle data (no printing info)
sdk.cards().get_atomic("Fire")                         // works with face names (split/MDFC)
sdk.cards().find_by_scryfall_id("...")                 // cross-reference
sdk.cards().random(5)                                  // random cards
sdk.cards().count(&HashMap::new())                     // total count
sdk.cards().count(&HashMap::from([                     // filtered count
    ("setCode".into(), "MH3".into()),
    ("rarity".into(), "rare".into()),
]))
```

### Tokens

```rust
sdk.tokens().get_by_uuid("uuid")                      // -> Result<Option<Value>>
sdk.tokens().get_by_name("Soldier", None)              // -> Result<Vec<Value>>
sdk.tokens().search(&SearchTokensParams {
    name: Some("%Token".into()),
    set_code: Some("MH3".into()),
    colors: Some(vec!["W".into()]),
    ..Default::default()
})
sdk.tokens().for_set("MH3")                           // all tokens for a set
sdk.tokens().count(&HashMap::new())
```

### Sets

```rust
sdk.sets().get("MH3")                                 // -> Result<Option<Value>>
sdk.sets().list(Some("expansion"), None, None, None)   // -> Result<Vec<Value>>
sdk.sets().search(&SearchSetsParams {
    name: Some("Horizons".into()),
    release_year: Some(2024),
    ..Default::default()
})
sdk.sets().get_financial_summary("MH3")                // -> Result<HashMap<String, Value>>
sdk.sets().count(None)                                 // total count
sdk.sets().count(Some("expansion"))                    // filtered by type
```

### Identifiers

```rust
sdk.identifiers().find_by_scryfall_id("...")
sdk.identifiers().find_by_tcgplayer_product_id("...")
sdk.identifiers().find_by_mtgo_id("...")
sdk.identifiers().find_by_mtgo_foil_id("...")
sdk.identifiers().find_by_mtg_arena_id("...")
sdk.identifiers().find_by_multiverse_id("...")
sdk.identifiers().find_by_mcm_id("...")
sdk.identifiers().find_by_card_kingdom_id("...")
sdk.identifiers().find_by_card_kingdom_foil_id("...")
sdk.identifiers().find_by_card_kingdom_etched_id("...")
sdk.identifiers().find_by_cardsphere_id("...")
sdk.identifiers().find_by_cardsphere_foil_id("...")
sdk.identifiers().find_by_scryfall_oracle_id("...")
sdk.identifiers().find_by_scryfall_illustration_id("...")
sdk.identifiers().find_by("scryfallId", "...")         // generic lookup
sdk.identifiers().get_identifiers("uuid")              // all IDs for a card
```

### Legalities

```rust
sdk.legalities().formats_for_card("uuid")              // -> Result<Vec<Value>>
sdk.legalities().legal_in("modern")                    // all modern-legal cards
sdk.legalities().is_legal("uuid", "modern")            // -> Result<bool>
sdk.legalities().banned_in("modern")                   // banned cards
sdk.legalities().restricted_in("vintage")              // restricted cards
sdk.legalities().suspended_in("historic")              // suspended cards
sdk.legalities().not_legal_in("standard")              // not-legal cards
```

### Prices

```rust
sdk.prices().get("uuid")                               // full nested price data
sdk.prices().today("uuid")                             // latest prices (all providers)
sdk.prices().history("uuid", Some("2024-01-01"), Some("2024-12-31"))
sdk.prices().price_trend("uuid")                       // min/max/avg statistics
sdk.prices().cheapest_printing("Lightning Bolt")       // cheapest printing by name
sdk.prices().cheapest_printings("Lightning Bolt", 10)  // N cheapest printings
sdk.prices().most_expensive_printings("Lightning Bolt", 10)
```

### Decks

```rust
sdk.decks().list(Some("MH3"), None)                    // list by set
sdk.decks().search("Eldrazi", None)                    // search by name
sdk.decks().count(None, None)                          // total count
```

### Sealed Products

```rust
sdk.sealed().list(Some("MH3"))                         // sealed products for a set
sdk.sealed().get("MH3")                                // alias for list with set code
```

### SKUs

```rust
sdk.skus().get("uuid")                                 // TCGPlayer SKUs for a card
sdk.skus().find_by_sku_id("123456")
sdk.skus().find_by_product_id("789")
```

### Booster Simulation

```rust
sdk.booster().available_types("MH3")                   // -> Result<Vec<String>>
sdk.booster().open_pack("MH3", "draft")                // -> Result<Vec<Value>>
sdk.booster().open_box("MH3", "draft", 36)             // -> Result<Vec<Vec<Value>>>
sdk.booster().sheet_contents("MH3", "draft", "common") // card weights
```

### Enums

```rust
sdk.enums().keywords()                                 // -> Result<Value>
sdk.enums().card_types()                               // -> Result<Value>
sdk.enums().enum_values()                              // all enum values
```

### Metadata & Utilities

```rust
sdk.meta()                                             // -> Result<Value>
sdk.views()                                            // -> Vec<String>
sdk.refresh()                                          // check for new data -> Result<bool>
sdk.sql("SELECT ...", &["param".into()])               // raw parameterized SQL
sdk.connection()                                       // &Connection for advanced usage
sdk.close()                                            // release resources (consumes self)
```

## Advanced Usage

### Builder Pattern

```rust
use mtgjson_sdk::MtgjsonSdk;
use std::path::PathBuf;
use std::time::Duration;

let sdk = MtgjsonSdk::builder()
    .cache_dir(PathBuf::from("/data/mtgjson-cache"))
    .offline(false)
    .timeout(Duration::from_secs(300))
    .build()?;
```

### Error Handling

All SDK methods return `Result<T, MtgjsonError>`. Use Rust's `?` operator for ergonomic error propagation:

```rust
use mtgjson_sdk::{MtgjsonSdk, MtgjsonError, Result};

fn find_card_price(name: &str) -> Result<()> {
    let sdk = MtgjsonSdk::builder().build()?;

    match sdk.prices().cheapest_printing(name)? {
        Some(card) => println!("${}", card["price"]),
        None => println!("No price data for {}", name),
    }

    Ok(())
}

// MtgjsonError variants:
// - MtgjsonError::DuckDb(_)          -- DuckDB query errors
// - MtgjsonError::Http(_)            -- network/download errors
// - MtgjsonError::Io(_)              -- file system errors
// - MtgjsonError::Json(_)            -- JSON parsing errors
// - MtgjsonError::NotFound(_)        -- entity not found
// - MtgjsonError::InvalidArgument(_) -- invalid input
```

### SQL Builder

The `SqlBuilder` provides safe, parameterized query construction:

```rust
use mtgjson_sdk::SqlBuilder;

let (sql, params) = SqlBuilder::new("cards")
    .select(&["name", "setCode", "manaValue"])
    .where_eq("rarity", "mythic")
    .where_gte("manaValue", "5")
    .where_like("name", "%Dragon%")
    .where_in("setCode", &["MH3", "LTR", "WOE"])
    .order_by(&["manaValue DESC", "name ASC"])
    .limit(25)
    .build();

// sql:    "SELECT name, setCode, manaValue\nFROM cards\nWHERE rarity = ? AND ..."
// params: ["mythic", "5", "%Dragon%", "MH3", "LTR", "WOE"]
```

Additional builder methods: `distinct()`, `join()`, `where_regex()`, `where_fuzzy()`, `where_or()`, `group_by()`, `having()`, `offset()`.

### Raw DuckDB Access

For advanced queries, access the underlying DuckDB connection directly:

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Ensure views are loaded
let _ = sdk.cards().count(&HashMap::new())?;

// Access raw DuckDB connection
let raw = sdk.connection().raw();
raw.execute_batch("CREATE TABLE my_analysis AS SELECT setCode, COUNT(*) as cnt FROM cards GROUP BY setCode")?;

// Query your custom table through the SDK
let rows = sdk.sql("SELECT * FROM my_analysis ORDER BY cnt DESC LIMIT 5", &[])?;
```

### Raw SQL

All user input goes through DuckDB parameter binding (`?` placeholders) to prevent SQL injection:

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Ensure views are registered before querying
let _ = sdk.cards().count(&HashMap::new())?;

// Parameterized queries
let rows = sdk.sql(
    "SELECT name, setCode, rarity FROM cards WHERE manaValue <= ? AND rarity = ?",
    &["2".into(), "mythic".into()],
)?;

// Complex analytics
let rows = sdk.sql(
    "SELECT setCode, COUNT(*) as card_count, AVG(manaValue) as avg_cmc \
     FROM cards GROUP BY setCode ORDER BY card_count DESC LIMIT 10",
    &[],
)?;
```

### Async Usage

Enable the `async` feature to use `AsyncMtgjsonSdk`, an async wrapper that dispatches all blocking SDK operations to a thread pool via `tokio::task::spawn_blocking`:

```toml
[dependencies]
mtgjson-sdk = { version = "0.1", features = ["async"] }
```

```rust
use mtgjson_sdk::AsyncMtgjsonSdk;

#[tokio::main]
async fn main() -> mtgjson_sdk::Result<()> {
    let sdk = AsyncMtgjsonSdk::builder().build().await?;

    // Use .run() to execute any sync SDK method asynchronously
    let bolts = sdk.run(|s| {
        s.cards().get_by_name("Lightning Bolt", None)
    }).await?;

    let sets = sdk.run(|s| {
        s.sets().list(Some("expansion"), None, None, None)
    }).await?;

    // Convenience methods for common operations
    let meta = sdk.meta().await?;
    let rows = sdk.sql("SELECT COUNT(*) FROM cards", &[]).await?;

    Ok(())
}
```

### Auto-Refresh for Long-Running Services

The `refresh()` method checks the CDN for new MTGJSON releases. If a newer version is available, it clears internal state so the next query re-downloads fresh data:

```rust
let sdk = MtgjsonSdk::builder().build()?;

// In a scheduled task or health check:
if sdk.refresh()? {
    println!("New MTGJSON data detected -- cache refreshed");
}
```

## Architecture

```
MTGJSON CDN (Parquet + JSON files)
        |
        | auto-download on first access
        v
Local Cache (platform-specific directory)
        |
        | lazy view registration
        v
DuckDB In-Memory Database
        |
        | parameterized SQL queries
        v
Typed Rust API (serde_json::Value / HashMap / custom structs)
```

**How it works:**

1. **Auto-download**: On first use, the SDK downloads ~15 Parquet files and ~7 JSON files from the MTGJSON CDN to a platform-specific cache directory (`~/.cache/mtgjson-sdk` on Linux, `~/Library/Caches/mtgjson-sdk` on macOS, `AppData/Local/mtgjson-sdk` on Windows).

2. **Lazy loading**: DuckDB views are registered on-demand -- accessing `sdk.cards()` triggers the cards view, `sdk.prices()` triggers price data loading, etc. Only the data you use gets loaded into memory.

3. **Schema adaptation**: The SDK auto-detects array columns in parquet files using a hybrid heuristic (static baseline + dynamic plural detection + blocklist), so it adapts to upstream MTGJSON schema changes without code updates.

4. **Legality UNPIVOT**: Format legality columns are dynamically detected from the parquet schema and UNPIVOTed to `(uuid, format, status)` rows -- automatically scales to new formats.

5. **Price flattening**: Deeply nested JSON price data is streamed to NDJSON and bulk-loaded into DuckDB, minimizing memory overhead.

## Examples

### Deck REST API

A complete REST API built with [Axum](https://github.com/tokio-rs/axum) that serves MTGJSON deck data. Demonstrates the `AsyncMtgjsonSdk` wrapper, CDN integration for individual deck files, and in-memory caching.

**Location:** [`examples/deck-api/`](examples/deck-api/)

```bash
cd examples/deck-api

# On Windows:
set DUCKDB_DOWNLOAD_LIB=1
cargo run

# On Linux/macOS:
cargo run
```

The server starts on `http://localhost:3000` with the following endpoints:

| Endpoint | Description |
|---|---|
| `GET /api/meta` | MTGJSON dataset version and date |
| `GET /api/sets?set_type=expansion` | List sets, optionally filtered by type |
| `GET /api/sets/:code` | Get details for a single set |
| `GET /api/decks?set_code=40K&deck_type=Commander+Deck` | List decks, optionally filtered by set and/or type |
| `GET /api/decks/search?name=Necron` | Search decks by name substring |
| `GET /api/decks/:file_name` | Get full deck contents (mainBoard, sideBoard, commander, etc.) |

**Quick test:**

```bash
# List all Warhammer 40K commander decks
curl http://localhost:3000/api/decks?set_code=40K

# Get the full card list for a deck
curl http://localhost:3000/api/decks/NecronDynasties_40K
```

## Development

### Prerequisites

- Rust 1.70+ (stable)
- On Windows: Visual Studio 2022 Build Tools (MSVC + Windows SDK)

### Setup

```bash
git clone https://github.com/the-muppet2/mtgjson-sdk-rust.git
cd mtgjson-sdk-rust
```

### Building

```bash
# On Windows (uses prebuilt DuckDB binary):
set DUCKDB_DOWNLOAD_LIB=1
cargo build

# On Linux/macOS:
cargo build
```

### Running Tests

```bash
# Unit tests (120+ tests, no network required)
cargo test

# Smoke test (downloads real data from CDN)
cargo test -- --ignored --nocapture
```

### Linting

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## License

MIT
