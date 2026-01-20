# Agari

Agari is a comprehensive Riichi Mahjong scoring engine written in idiomatic, modern Rust. It transforms raw hand notations into detailed scoring results, handling the complex interplay between hand decomposition, wait patterns, situational yaku, and minipoint (fu) calculation.

---

## ## Core Architecture & Module Breakdown

The system is designed as a pipeline, moving from string parsing to recursive decomposition, and finally to mathematical scoring.

| Module | Primary Responsibility | Key Data Structures |
| --- | --- | --- |
| **`tile.rs`** | Low-level tile definitions and individual tile parsing. | `Tile`, `Suit`, `Honor` |
| **`parse.rs`** | Hand-string parsing and validation (14-tile rule, max 4 copies). | `ParsedHand`, `TileCounts` |
| **`hand.rs`** | Recursive backtracking to find all valid hand interpretations. | `Meld`, `HandStructure` |
| **`wait.rs`** | Identifying the "winning shape" to determine fu and Pinfu eligibility. | `WaitType` |
| **`yaku.rs`** | Pattern matching for scoring conditions (Tanyao, Honitsu, etc.). | `Yaku`, `YakuResult` |
| **`scoring.rs`** | The final calculator for Fu, Han, and point payouts. | `ScoringResult`, `Payment` |
| **`context.rs`** | Tracking game metadata (winds, dora indicators, win type). | `GameContext` |
| **`display.rs`** | Pretty-printing tiles using Unicode Mahjong glyphs (🀄). | N/A |

---

## ## The Scoring Pipeline

The engine follows a linear transformation of data to ensure that hands with multiple interpretations (e.g., a hand that could be viewed as either all-triplets or a series of sequences) are scored optimally for the player.

1. **Parsing & Counting:** The input string (e.g., `123m456p0s...`) is parsed into a `TileCounts` map. It explicitly handles **Akadora** (red fives) using the `0` notation, which is stored in the `GameContext`.
2. **Decomposition:** The `decompose_hand` function uses recursive backtracking to identify every possible way to form 4 melds and 1 pair (or 7 pairs for Chiitoitsu).
3. **Wait Detection:** For every valid structure, the engine checks how the `winning_tile` fits. This determines if the wait was "difficult" (2 fu for Kanchan/Penchan/Tanki) or "ideal" (0 fu for Ryanmen).
4. **Yaku & Dora:** The engine iterates through the yaku list. It handles han-reduction for open hands (e.g., Honitsu drops from 3 han to 2) and calculates the total Han by adding regular Dora, Ura Dora, and Akadora.
5. **Fu Calculation:** Minipoints are summed based on triplets (simple vs. terminal/honor), the wait type, and the pair type, then rounded up to the nearest 10 (with the 25-fu Chiitoitsu exception).

---

## ## Scoring Logic & Mathematics

The final score is derived using the standard Riichi Mahjong base point formula. For hands below the "Mangan" limit (usually 5 Han, or 4 Han with high Fu), the basic points are calculated as:

Once basic points are established, the `Payment` struct applies the necessary multipliers based on whether the winner is the **Dealer (Oya)** or a **Non-dealer (Ko)**:

* **Ron (Dealer):**  (paid by discarder).
* **Ron (Non-dealer):**  (paid by discarder).
* **Tsumo (Dealer):**  from each of the 3 players.
* **Tsumo (Non-dealer):**  from the Dealer,  from the other Non-dealers.

All final payments are rounded up to the nearest 100 points.

---

## ## Key Technical Implementation Details

### ### Recursive Decomposition (`hand.rs`)

The decomposition logic is robust. It sorts tiles to ensure consistent processing and uses a "pick-a-triplet-or-sequence" branching strategy. This is essential for hands like `111222333m`, which the code correctly identifies as either three triplets () or three identical sequences ().

### ### Pinfu Validation (`wait.rs`)

The `is_pinfu` function is a strict implementation of the four traditional requirements:

1. **Closed Hand:** Verified via `context.is_open`.
2. **No Triplets:** All melds must be `Meld::Shuntsu`.
3. **Valueless Pair:** Pair cannot be dragons or the player's own/round wind.
4. **Ryanmen Wait:** The winning tile must complete a two-sided sequence (e.g., 2-3 waiting on 1-4).

### ### Elegant Display (`display.rs`)

The code includes a sophisticated Unicode mapper. Instead of just printing "1m", it can output the actual Mahjong tile characters (🀇, 🀐, 🀙), making the CLI output significantly more readable for players.
