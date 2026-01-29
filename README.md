# Agari

Agari is a comprehensive Riichi Mahjong scoring engine written in idiomatic, modern Rust. It transforms raw hand notations into detailed scoring results, handling the complex interplay between hand decomposition, wait patterns, situational yaku, and minipoint (fu) calculation.

---

## Core Architecture & Module Breakdown

The system is designed as a pipeline, moving from string parsing to recursive decomposition, and finally to mathematical scoring.

| Module | Primary Responsibility | Key Data Structures |
| --- | --- | --- |
| **`tile.rs`** | Low-level tile definitions and individual tile parsing. | `Tile`, `Suit`, `Honor` |
| **`parse.rs`** | Hand-string parsing, meld notation, and validation. | `ParsedHand`, `CalledMeld`, `TileCounts` |
| **`hand.rs`** | Recursive backtracking to find all valid hand interpretations. | `Meld`, `KanType`, `HandStructure` |
| **`wait.rs`** | Identifying the "winning shape" to determine fu and Pinfu eligibility. | `WaitType` |
| **`yaku.rs`** | Pattern matching for scoring conditions (Tanyao, Honitsu, etc.). | `Yaku`, `YakuResult` |
| **`scoring.rs`** | The final calculator for Fu, Han, and point payouts. | `ScoringResult`, `Payment` |
| **`context.rs`** | Tracking game metadata (winds, dora indicators, win type). | `GameContext` |
| **`shanten.rs`** | Shanten calculator and ukeire (tile acceptance) analysis. | `ShantenResult`, `UkeireResult` |
| **`display.rs`** | Pretty-printing tiles using Unicode Mahjong glyphs (🀄). | N/A |

---

## Hand Notation

### Basic Tiles

Standard notation uses numbers followed by a suit letter:
- `m` = Man (Characters)
- `p` = Pin (Dots)
- `s` = Sou (Bamboo)
- `z` = Honors (1=East, 2=South, 3=West, 4=North, 5=White, 6=Green, 7=Red)

Examples:
- `123m` = 1-2-3 of Man
- `55z` = Pair of White Dragons
- `1234567z` = All seven honor types

### Red Fives (Akadora)

Use `0` instead of `5` to indicate a red five:
- `0m` = Red 5-man
- `0p` = Red 5-pin
- `0s` = Red 5-sou

### Ergonomic Honor Tile Notation

You can use intuitive letter-based notation for honor tiles instead of the numeric `z` notation. This works both in the CLI and when using the library's `parse_hand` and `parse_hand_with_aka` functions.

**Winds:**
- `e` = East (1z)
- `s` = South (2z)
- `w` = West (3z)
- `n` = North (4z)

**Dragons:**
- `wh` = White Dragon (5z)
- `g` = Green Dragon (6z)
- `r` = Red Dragon (7z)

This notation works everywhere—hand strings, called melds, winning tile, dora, and ura dora:

```bash
# Before (numeric z notation)
agari "123m456p789s11144z" -w 1z -d 35z

# After (letter notation)
agari "123m456p789seeenn" -w e -d wwh
```

The parser handles ambiguous sequences like `wwhwwwh` correctly (West, White, West, West, White).

### Called Melds (Kans, Pons, Chis)

Bracket notation for declaring melds:

| Notation | Type | Example |
| --- | --- | --- |
| `[1111m]` | Closed kan (ankan) | Four concealed 1-man |
| `(1111m)` | Open kan (daiminkan) | Called kan of 1-man |
| `(111m)` | Open triplet (pon) | Called pon of 1-man |
| `(123m)` | Open sequence (chi) | Called chi of 1-2-3 man |

**Tile Count Rules:**
- Standard hand: 14 tiles
- With 1 kan: 15 tiles
- With 2 kans: 16 tiles
- With 3 kans: 17 tiles
- With 4 kans: 18 tiles

Examples:
```bash
# Hand with closed kan (15 tiles total)
agari "[1111m]222333m555p11z" -t

# Hand with closed + open kan (16 tiles total)
agari "[1111m](2222p)345678s11z" -t

# Hand with open pon
agari "(111m)456m789p123s11z" -w 1z
```

---

## The Scoring Pipeline

The engine follows a linear transformation of data to ensure that hands with multiple interpretations (e.g., a hand that could be viewed as either all-triplets or a series of sequences) are scored optimally for the player.

1. **Parsing & Counting:** The input string (e.g., `123m456p0s...`) is parsed into a `TileCounts` map. It explicitly handles **Akadora** (red fives) using the `0` notation, which is stored in the `GameContext`.
2. **Decomposition:** The `decompose_hand` function uses recursive backtracking to identify every possible way to form 4 melds and 1 pair (or 7 pairs for Chiitoitsu). When called melds are present, `decompose_hand_with_melds` is used to incorporate pre-declared kans, pons, and chis.
3. **Wait Detection:** For every valid structure, the engine checks how the `winning_tile` fits. This determines if the wait was "difficult" (2 fu for Kanchan/Penchan/Tanki) or "ideal" (0 fu for Ryanmen).
4. **Yaku & Dora:** The engine iterates through the yaku list. It handles han-reduction for open hands (e.g., Honitsu drops from 3 han to 2) and calculates the total Han by adding regular Dora, Ura Dora, and Akadora.
5. **Fu Calculation:** Minipoints are summed based on triplets (simple vs. terminal/honor), kans (closed vs. open, simple vs. terminal/honor), the wait type, and the pair type, then rounded up to the nearest 10 (with the 25-fu Chiitoitsu exception).

---

## Shanten & Ukeire Analysis

The shanten calculator determines how many tile exchanges are needed to reach tenpai:

- **-1**: Complete (winning) hand
- **0**: Tenpai (one tile away from winning)
- **1**: Iishanten (one tile exchange from tenpai)
- **2+**: Multiple exchanges needed

The calculator evaluates three hand types and returns the best (lowest) shanten:
1. **Standard**: 4 melds + 1 pair
2. **Chiitoitsu**: 7 pairs
3. **Kokushi**: 13 orphans

**Ukeire** (tile acceptance) shows which tiles would improve the hand, along with how many of each are still available.

```bash
# Calculate shanten
agari 123m456p789s1112z --shanten

# Calculate shanten with ukeire
agari 123m456p789s112z --ukeire
```

---

## Scoring Logic & Mathematics

The final score is derived using the standard Riichi Mahjong base point formula. For hands below the "Mangan" limit (usually 5 Han, or 4 Han with high Fu), the basic points are calculated as:

```
basic_points = fu × 2^(2 + han)
```

Once basic points are established, the `Payment` struct applies the necessary multipliers based on whether the winner is the **Dealer (Oya)** or a **Non-dealer (Ko)**:

* **Ron (Dealer):** basic × 6 (paid by discarder).
* **Ron (Non-dealer):** basic × 4 (paid by discarder).
* **Tsumo (Dealer):** basic × 2 from each of the 3 players.
* **Tsumo (Non-dealer):** basic × 2 from the Dealer, basic × 1 from the other Non-dealers.

All final payments are rounded up to the nearest 100 points.

---

## Key Technical Implementation Details

### Recursive Decomposition (`hand.rs`)

The decomposition logic is robust. It sorts tiles to ensure consistent processing and uses a "pick-a-triplet-or-sequence" branching strategy. This is essential for hands like `111222333m`, which the code correctly identifies as either three triplets or three identical sequences.

### Kan Support (`hand.rs`, `parse.rs`)

Kans are fully supported with the `KanType` enum distinguishing between:
- **Closed (Ankan)**: Concealed kan, earns more fu
- **Open (Daiminkan)**: Called kan from discard
- **Added (Shouminkan)**: Added to existing pon (for Chankan detection)

Each meld tracks its open/closed status independently, enabling accurate fu calculation:
- Closed kan of simples: 16 fu
- Open kan of simples: 8 fu
- Closed kan of terminals/honors: 32 fu
- Open kan of terminals/honors: 16 fu

### Pinfu Validation (`wait.rs`)

The `is_pinfu` function is a strict implementation of the four traditional requirements:

1. **Closed Hand:** Verified via `context.is_open`.
2. **No Triplets:** All melds must be `Meld::Shuntsu`.
3. **Valueless Pair:** Pair cannot be dragons or the player's own/round wind.
4. **Ryanmen Wait:** The winning tile must complete a two-sided sequence (e.g., 2-3 waiting on 1-4).

### Shanten Algorithm (`shanten.rs`)

The shanten calculator uses a 34-element array representation for fast computation. For standard hands, it:
1. Converts tile counts to the array format
2. Tries extracting each possible pair
3. Counts complete melds and incomplete melds (taatsu) for each suit
4. Applies the formula: `shanten = 8 - 2×melds - taatsu - (1 if pair)`

Special handling exists for Chiitoitsu (counting pairs and unique tiles) and Kokushi (counting terminal/honor coverage).

### Elegant Display (`display.rs`)

The code includes a sophisticated Unicode mapper. Instead of just printing "1m", it can output the actual Mahjong tile characters (🀇, 🀐, 🀙), making the CLI output significantly more readable for players.

---

## CLI Usage

```text
# Basic hand scoring
agari 123m456p789s11122z

# Tsumo win with riichi
agari 123m456p789s11122z -t -r

# Specify winning tile and dora
agari 123m456p789s11122z -w 2m -d 1m

# Hand with kan
agari "[1111m]222333m555p11z" -t

# Shanten analysis
agari 123m456p789s1112z --shanten

# Ukeire (tile acceptance) analysis  
agari 123m456p789s112z --ukeire

# All options
agari <HAND> [OPTIONS]

OPTIONS:
    -w, --win <TILE>      Winning tile (e.g., 2m, 5z)
    -t, --tsumo           Win by self-draw (default: ron)
    -o, --open            Hand is open (has called tiles)
    -r, --riichi          Riichi declared
    --double-riichi       Double riichi (first turn)
    --ippatsu             Ippatsu (win within one turn of riichi)
    --round <WIND>        Round wind: e/s/w/n (default: e)
    --seat <WIND>         Seat wind: e/s/w/n (default: e)
    -d, --dora <TILES>    Dora indicators (e.g., 58m or 5m,8m)
    --ura <TILES>         Ura dora indicators (e.g., 29p or 2p,9p)
    --last-tile           Win on last tile (Haitei/Houtei)
    --rinshan             Win on kan replacement tile
    --chankan             Ron on another player's added kan
    --tenhou              Dealer's first draw win
    --chiihou             Non-dealer's first draw win
    --shanten             Calculate shanten instead of score
    --ukeire              Show ukeire with shanten
    --ascii               Use ASCII output instead of Unicode
    --all                 Show all possible interpretations
    --json                Output results as JSON
    -h, --help            Show help message
```

---

## Using as a Library

Agari can be used as a Rust library for building Mahjong applications, bots, or analysis tools.

### Add to Cargo.toml

```toml
[dependencies]
agari = { git = "https://github.com/ryblogs/agari" }
```

### Basic Scoring Example

```rust
use agari::context::{GameContext, WinType};
use agari::hand::decompose_hand;
use agari::parse::{parse_hand, to_counts};
use agari::scoring::calculate_score;
use agari::tile::Honor;
use agari::yaku::detect_yaku_with_context;

fn main() {
    // Parse the hand
    let tiles = parse_hand("123m456p789s11122z").unwrap();
    let counts = to_counts(&tiles);

    // Find all valid hand structures
    let structures = decompose_hand(&counts);

    // Set up game context
    let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
        .riichi()
        .with_winning_tile(tiles[0])
        .with_dora(vec![tiles[1]]);

    // Score each interpretation and find the best
    let best = structures
        .iter()
        .map(|s| {
            let yaku = detect_yaku_with_context(s, &counts, &context);
            let score = calculate_score(s, &yaku, &context);
            (s, yaku, score)
        })
        .max_by(|a, b| {
            a.2.payment.total.cmp(&b.2.payment.total)
                .then_with(|| a.2.han.cmp(&b.2.han))
        })
        .unwrap();

    println!("Han: {}, Fu: {}", best.2.han, best.2.fu.total);
    println!("Payment: {} points", best.2.payment.total);
}
```

### Scoring with Called Melds

```rust
use agari::context::{GameContext, WinType};
use agari::hand::decompose_hand_with_melds;
use agari::parse::{parse_hand_with_aka, to_counts};
use agari::scoring::calculate_score;
use agari::tile::Honor;
use agari::yaku::detect_yaku_with_context;

fn main() {
    // Parse hand with called melds: (123p) is an open chi
    let parsed = parse_hand_with_aka("456m789s11z(123p)(777z)").unwrap();
    let counts = to_counts(&parsed.tiles);

    // Extract called melds
    let called_melds: Vec<_> = parsed.called_melds
        .iter()
        .map(|cm| cm.meld.clone())
        .collect();

    // Decompose with pre-declared melds
    let structures = decompose_hand_with_melds(&counts, &called_melds);

    let context = GameContext::new(WinType::Ron, Honor::East, Honor::South)
        .open()  // Mark hand as open
        .with_winning_tile(parsed.tiles[0]);

    // ... score as above
}
```

### Shanten Calculation

```rust
use agari::parse::{parse_hand, to_counts};
use agari::shanten::{calculate_shanten, calculate_ukeire};

fn main() {
    let tiles = parse_hand("123m456p789s1112z").unwrap();
    let counts = to_counts(&tiles);

    // Calculate shanten (-1 = complete, 0 = tenpai, 1+ = tiles away)
    let result = calculate_shanten(&counts);
    println!("Shanten: {}", result.shanten);
    println!("Best type: {:?}", result.best_type);

    // Calculate ukeire (tile acceptance)
    let ukeire = calculate_ukeire(&counts);
    for (tile, count) in &ukeire.tiles {
        println!("{:?}: {} available", tile, count);
    }
}
```

### Key Types

| Type | Description |
|------|-------------|
| `Tile` | A single tile (suited or honor) |
| `TileCounts` | HashMap of tile → count for a hand |
| `HandStructure` | Decomposed hand (Standard, Chiitoitsu, or Kokushi) |
| `Meld` | A group of tiles (Shuntsu, Koutsu, or Kan) |
| `GameContext` | Win type, winds, dora, riichi status, etc. |
| `YakuResult` | Detected yaku with han breakdown |
| `ScoringResult` | Final score with fu, han, payment |

### GameContext Builder

```rust
let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
    .riichi()                              // Declare riichi
    .ippatsu()                             // Ippatsu enabled
    .with_winning_tile(tile)               // Set agari tile
    .with_dora(vec![indicator1, indicator2]) // Dora indicators
    .with_ura_dora(vec![ura1])             // Ura dora (with riichi)
    .with_aka(1)                           // Red five count
    .open()                                // Hand is open
    .last_tile()                           // Haitei/Houtei
    .rinshan()                             // Rinshan kaihou
    .chankan()                             // Chankan
    .tenhou()                              // Tenhou (dealer first draw)
    .chiihou();                            // Chiihou (non-dealer first draw)
```

---

## Building & Testing

```bash
# Build release binary
cargo build --release

# Run tests
cargo test

# Run the CLI
./target/release/agari 123m456p789s11122z -t
```
