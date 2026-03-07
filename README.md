# mtgjson-sdk

[![Crates.io](https://img.shields.io/crates/v/mtgjson-sdk)](https://crates.io/crates/mtgjson-sdk)
[![docs.rs](https://img.shields.io/docsrs/mtgjson-sdk)](https://docs.rs/mtgjson-sdk)
[![License](https://img.shields.io/crates/l/mtgjson-sdk)](https://crates.io/crates/mtgjson-sdk)

**Official Rust SDK for [MTGJSON](https://mtgjson.com)**, built and maintained by the MTGJSON team.

A high-performance, DuckDB-backed Rust query client for [MTGJSON](https://mtgjson.com).

Unlike traditional SDKs that rely on rate-limited REST APIs, `mtgjson-sdk` implements a local data warehouse architecture. It synchronizes optimized Parquet data from the MTGJSON CDN to your local machine, utilizing DuckDB to execute complex analytics, fuzzy searches, and booster simulations with sub-millisecond latency.

## Key Features

*   **Vectorized Execution**: Powered by DuckDB for high-speed OLAP queries on the full MTG dataset.
*   **Offline-First**: Data is cached locally, allowing for full functionality without an active internet connection.
*   **Fuzzy Search**: Built-in Jaro-Winkler similarity matching to handle typos and approximate name lookups.
*   **Data Science Integration**: Optional Polars DataFrame output via the `polars` cargo feature for zero-copy data transfer.
*   **Fully Async**: Thread-safe async wrapper (`AsyncMtgjsonSdk`) using `tokio::task::spawn_blocking`.
*   **Booster Simulation**: Accurate pack opening logic using official MTGJSON weights and sheet configurations.

## Install

```bash
cargo add mtgjson-sdk
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
mtgjson-sdk = "0.1"
```

Optional features:

```toml
mtgjson-sdk = { version = "0.1", features = ["async"] }   # async support via tokio
mtgjson-sdk = { version = "0.1", features = ["polars"] }   # Polars DataFrame output
```

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

## Architecture

By using DuckDB, the SDK leverages columnar storage and vectorized execution, making it significantly faster than SQLite or standard JSON parsing for MTG's relational dataset.

1.  **Synchronization**: On first use, the SDK lazily downloads Parquet and JSON files from the MTGJSON CDN to a platform-specific cache directory (`~/.cache/mtgjson-sdk` on Linux, `~/Library/Caches/mtgjson-sdk` on macOS, `AppData/Local/mtgjson-sdk` on Windows).
2.  **Virtual Schema**: DuckDB views are registered on-demand. Accessing `sdk.cards()` registers the card view; accessing `sdk.prices()` registers price data. You only pay the memory cost for the data you query.
3.  **Dynamic Adaptation**: The SDK introspects Parquet metadata to automatically handle schema changes, plural-column array conversion, and format legality unpivoting.
4.  **Materialization**: Queries return `serde_json::Value` objects for flexible consumption, or typed structs via `execute_into<T>()` for ergonomic deserialization.

## Use Cases

### Price Analytics

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Find the cheapest printing of a card by name
let cheapest = sdk.prices().cheapest_printing("Ragavan, Nimble Pilferer")?;

// Aggregate statistics (min, max, avg) for a specific card
if let Some(ref card) = cheapest {
    let uuid = card["uuid"].as_str().unwrap();
    let trend = sdk.prices().price_trend(uuid)?;
    println!("Range: ${} - ${}", trend["min_price"], trend["max_price"]);
    println!("Average: ${} over {} data points", trend["avg_price"], trend["data_points"]);

    // Historical price lookup with date filtering
    let history = sdk.prices().history(uuid, Some("2024-01-01"), Some("2024-12-31"))?;

    // Top 10 most expensive printings across the entire dataset
    let priciest = sdk.prices().most_expensive_printings("Ragavan, Nimble Pilferer", 10)?;
}

sdk.close();
```

### Advanced Card Search

The `search()` method supports ~20 composable filters that can be combined freely:

```rust
use mtgjson_sdk::queries::cards::SearchCardsParams;

let sdk = MtgjsonSdk::builder().build()?;

// Complex filters: Modern-legal red creatures with CMC <= 2
let aggro = sdk.cards().search(&SearchCardsParams {
    colors: Some(vec!["R".into()]),
    types: Some("Creature".into()),
    mana_value_lte: Some(2.0),
    legal_in: Some("modern".into()),
    limit: Some(50),
    ..Default::default()
})?;

// Typo-tolerant fuzzy search (Jaro-Winkler similarity)
let results = sdk.cards().search(&SearchCardsParams {
    fuzzy_name: Some("Ligtning Bolt".into()),  // still finds it!
    ..Default::default()
})?;

// Rules text search using regular expressions
let burn = sdk.cards().search(&SearchCardsParams {
    text_regex: Some(r"deals? \d+ damage to any target".into()),
    ..Default::default()
})?;

// Search by keyword ability across formats
let flyers = sdk.cards().search(&SearchCardsParams {
    keyword: Some("Flying".into()),
    colors: Some(vec!["W".into(), "U".into()]),
    legal_in: Some("standard".into()),
    ..Default::default()
})?;

// Find cards by foreign-language name
let blitz = sdk.cards().search(&SearchCardsParams {
    localized_name: Some("Blitzschlag".into()),  // German for Lightning Bolt
    ..Default::default()
})?;

sdk.close();
```

<details>
<summary>All <code>SearchCardsParams</code> fields</summary>

| Field | Type | Description |
|---|---|---|
| `name` | `Option<String>` | Name pattern (`%` = wildcard) |
| `fuzzy_name` | `Option<String>` | Typo-tolerant Jaro-Winkler match |
| `localized_name` | `Option<String>` | Foreign-language name search |
| `colors` | `Option<Vec<String>>` | Cards containing these colors |
| `color_identity` | `Option<Vec<String>>` | Color identity filter |
| `legal_in` | `Option<String>` | Format legality |
| `rarity` | `Option<String>` | Rarity filter |
| `mana_value` | `Option<f64>` | Exact mana value |
| `mana_value_lte` | `Option<f64>` | Mana value upper bound |
| `mana_value_gte` | `Option<f64>` | Mana value lower bound |
| `text` | `Option<String>` | Rules text substring |
| `text_regex` | `Option<String>` | Rules text regex |
| `types` | `Option<String>` | Type line search |
| `artist` | `Option<String>` | Artist name |
| `keyword` | `Option<String>` | Keyword ability |
| `is_promo` | `Option<bool>` | Promo status |
| `availability` | `Option<String>` | `"paper"` or `"mtgo"` |
| `language` | `Option<String>` | Language filter |
| `layout` | `Option<String>` | Card layout |
| `set_code` | `Option<String>` | Set code |
| `set_type` | `Option<String>` | Set type (joins sets table) |
| `power` | `Option<String>` | Power filter |
| `toughness` | `Option<String>` | Toughness filter |
| `limit` / `offset` | `Option<usize>` | Pagination |

</details>

### Collection & Cross-Reference

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Cross-reference by any external ID system
let cards = sdk.identifiers().find_by_scryfall_id("f7a21fe4-...")?;
let cards = sdk.identifiers().find_by_tcgplayer_product_id("12345")?;
let cards = sdk.identifiers().find_by_mtgo_id("67890")?;

// Get all external identifiers for a card
let all_ids = sdk.identifiers().get_identifiers("card-uuid-here")?;
// -> Scryfall, TCGPlayer, MTGO, Arena, Cardmarket, Card Kingdom, Cardsphere, ...

// TCGPlayer SKU variants (foil, etched, etc.)
let skus = sdk.skus().get("card-uuid-here")?;

sdk.close();
```

### Booster Simulation

```rust
let sdk = MtgjsonSdk::builder().build()?;

// See available booster types for a set
let types = sdk.booster().available_types("MH3")?;  // ["draft", "collector", ...]

// Open a single draft pack using official set weights
let pack = sdk.booster().open_pack("MH3", "draft")?;
for card in &pack {
    println!("  {} ({})", card["name"], card["rarity"]);
}

// Simulate opening a full box (36 packs)
let booster_box = sdk.booster().open_box("MH3", "draft", 36)?;
let total_cards: usize = booster_box.iter().map(|p| p.len()).sum();
println!("Opened {} packs, {} total cards", booster_box.len(), total_cards);

sdk.close();
```

## API Reference

### Core Data

```rust
// Cards
sdk.cards().get_by_uuid("uuid")               // single card lookup
sdk.cards().get_by_uuids(&["uuid1", "uuid2"]) // batch lookup
sdk.cards().get_by_name("Lightning Bolt", None)// all printings of a name
sdk.cards().search(&SearchCardsParams{...})    // composable filters (see above)
sdk.cards().get_printings("Lightning Bolt")    // all printings across sets
sdk.cards().get_atomic("Lightning Bolt")       // oracle data (no printing info)
sdk.cards().find_by_scryfall_id("...")         // cross-reference shortcut
sdk.cards().random(5)                          // random cards
sdk.cards().count(&HashMap::new())             // total (or filtered with kwargs)

// Tokens
sdk.tokens().get_by_uuid("uuid")
sdk.tokens().get_by_name("Soldier", None)
sdk.tokens().search(&SearchTokensParams { name: Some("%Token".into()), .. })
sdk.tokens().for_set("MH3")
sdk.tokens().count(&HashMap::new())

// Sets
sdk.sets().get("MH3")
sdk.sets().list(Some("expansion"), None, None, None)
sdk.sets().search(&SearchSetsParams { name: Some("Horizons".into()), .. })
sdk.sets().get_financial_summary("MH3")
sdk.sets().count(None)
```

### Playability

```rust
// Legalities
sdk.legalities().formats_for_card("uuid")      // -> Result<Vec<Value>>
sdk.legalities().legal_in("modern")            // all modern-legal cards
sdk.legalities().is_legal("uuid", "modern")    // -> Result<bool>
sdk.legalities().banned_in("modern")           // also: restricted_in, suspended_in

// Decks & Sealed Products
sdk.decks().list(Some("MH3"), None)
sdk.decks().search("Eldrazi", None)
sdk.decks().count(None, None)
sdk.sealed().list(Some("MH3"))
sdk.sealed().get("MH3")
```

### Market & Identifiers

```rust
// Prices
sdk.prices().get("uuid")                       // full nested price data
sdk.prices().today("uuid")                     // latest prices (all providers)
sdk.prices().history("uuid", Some("2024-01-01"), Some("2024-12-31"))
sdk.prices().price_trend("uuid")               // min/max/avg statistics
sdk.prices().cheapest_printing("Lightning Bolt")
sdk.prices().most_expensive_printings("Lightning Bolt", 10)

// Identifiers (supports all major external ID systems)
sdk.identifiers().find_by_scryfall_id("...")
sdk.identifiers().find_by_tcgplayer_product_id("...")
sdk.identifiers().find_by_mtgo_id("...")
sdk.identifiers().find_by_mtg_arena_id("...")
sdk.identifiers().find_by_multiverse_id("...")
sdk.identifiers().find_by_mcm_id("...")
sdk.identifiers().find_by_card_kingdom_id("...")
sdk.identifiers().find_by("scryfallId", "...")  // generic lookup
sdk.identifiers().get_identifiers("uuid")       // all IDs for a card

// SKUs
sdk.skus().get("uuid")
sdk.skus().find_by_sku_id("123456")
sdk.skus().find_by_product_id("789")
```

### Booster & Enums

```rust
sdk.booster().available_types("MH3")
sdk.booster().open_pack("MH3", "draft")
sdk.booster().open_box("MH3", "draft", 36)
sdk.booster().sheet_contents("MH3", "draft", "common")

sdk.enums().keywords()
sdk.enums().card_types()
sdk.enums().enum_values()
```

### System

```rust
sdk.meta()                                     // version and build date
sdk.views()                                    // registered view names
sdk.refresh()                                  // check CDN for new data -> Result<bool>
sdk.sql(query, &params)                        // raw parameterized SQL
sdk.connection()                               // &Connection for advanced usage
sdk.close()                                    // release resources (consumes self)
```

## Performance and Memory

When querying large datasets (thousands of cards), use `sql()` for bulk analysis to avoid materializing large `Vec<Value>` collections. With the `polars` cargo feature, use `sql_df()` for zero-copy DataFrame handoff from DuckDB.

```rust
// Use raw SQL for bulk analysis
let rows = sdk.sql(
    "SELECT setCode, COUNT(*) as card_count, AVG(manaValue) as avg_cmc \
     FROM cards GROUP BY setCode ORDER BY card_count DESC LIMIT 10",
    &[],
)?;
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

### SqlBuilder

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

    // Convenience methods for common operations
    let meta = sdk.meta().await?;
    let rows = sdk.sql("SELECT COUNT(*) FROM cards", &[]).await?;

    Ok(())
}
```

### Auto-Refresh for Long-Running Services

```rust
// In a scheduled task or health check:
if sdk.refresh()? {
    println!("New MTGJSON data detected -- cache refreshed");
}
```

### Raw SQL

All user input goes through DuckDB parameter binding (`?` placeholders):

```rust
let sdk = MtgjsonSdk::builder().build()?;

// Ensure views are registered before querying
let _ = sdk.cards().count(&HashMap::new())?;

// Parameterized queries
let rows = sdk.sql(
    "SELECT name, setCode, rarity FROM cards WHERE manaValue <= ? AND rarity = ?",
    &["2".into(), "mythic".into()],
)?;
```

## Examples

### Deck REST API (`examples/deck-api`)

A complete REST API built with [Axum](https://github.com/tokio-rs/axum) that serves MTGJSON deck data. Demonstrates the `AsyncMtgjsonSdk` wrapper, CDN integration for individual deck files, and in-memory caching.

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

```bash
git clone https://github.com/the-muppet2/mtgjson-sdk-rust.git
cd mtgjson-sdk-rust

# On Windows (uses prebuilt DuckDB binary):
set DUCKDB_DOWNLOAD_LIB=1
cargo build

# On Linux/macOS:
cargo build

# Tests (120+ tests, no network required)
cargo test

# Smoke test (downloads real data from CDN)
cargo test -- --ignored --nocapture

# Linting
cargo clippy -- -D warnings
cargo fmt --check
```

## License

MIT
