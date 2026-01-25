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
    -d, --dora <TILES>    Dora indicators (comma-separated)
    --ura <TILES>         Ura dora indicators (with riichi only)
    --last-tile           Win on last tile (Haitei/Houtei)
    --rinshan             Win on kan replacement tile
    --chankan             Ron on another player's added kan
    --tenhou              Dealer's first draw win
    --chiihou             Non-dealer's first draw win
    --shanten             Calculate shanten instead of score
    --ukeire              Show ukeire with shanten
    --ascii               Use ASCII output instead of Unicode
    --all                 Show all possible interpretations
    -h, --help            Show help message
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
