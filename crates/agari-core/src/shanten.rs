//! Shanten calculator for Riichi Mahjong
//!
//! Shanten is the minimum number of tile exchanges needed to reach tenpai (ready hand).
//! - Shanten = -1: Complete (winning) hand
//! - Shanten = 0: Tenpai (one tile away from winning)
//! - Shanten = 1: Iishanten (two tiles away)
//! - etc.

use serde::{Deserialize, Serialize};

use crate::hand::KanType;
use crate::parse::TileCounts;
use crate::tile::{Honor, KOKUSHI_TILES, Suit, Tile};
use std::cmp::{max, min};

/// Result of shanten calculation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShantenResult {
    /// The shanten value (-1 = complete, 0 = tenpai, 1+ = tiles needed)
    pub shanten: i8,
    /// The type of hand structure that gives the best shanten
    pub best_type: ShantenType,
}

/// Type of hand structure for shanten calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShantenType {
    /// Standard 4 melds + 1 pair
    Standard,
    /// Seven pairs (chiitoitsu)
    Chiitoitsu,
    /// Thirteen orphans (kokushi)
    Kokushi,
}

/// Calculate the shanten for a hand
///
/// Returns the minimum shanten across all possible hand types
/// (standard, chiitoitsu, kokushi)
pub fn calculate_shanten(counts: &TileCounts) -> ShantenResult {
    calculate_shanten_with_melds(counts, 0)
}

/// Calculate shanten for a hand with called melds
///
/// `called_melds` is the number of complete melds already called (pon, chi, kan).
/// These melds are not included in `counts` - only the remaining hand tiles are.
///
/// For example, with 3 called pons and 4 tiles in hand (waiting for a pair),
/// pass `called_melds = 3` and counts containing only the 4 hand tiles.
pub fn calculate_shanten_with_melds(counts: &TileCounts, called_melds: u8) -> ShantenResult {
    let standard = calculate_standard_shanten_with_melds(counts, called_melds);

    // Chiitoitsu and Kokushi are not possible with called melds
    if called_melds > 0 {
        return ShantenResult {
            shanten: standard,
            best_type: ShantenType::Standard,
        };
    }

    let chiitoi = calculate_chiitoitsu_shanten(counts);
    let kokushi = calculate_kokushi_shanten(counts);

    // Return the best (lowest) shanten
    if standard <= chiitoi && standard <= kokushi {
        ShantenResult {
            shanten: standard,
            best_type: ShantenType::Standard,
        }
    } else if chiitoi <= kokushi {
        ShantenResult {
            shanten: chiitoi,
            best_type: ShantenType::Chiitoitsu,
        }
    } else {
        ShantenResult {
            shanten: kokushi,
            best_type: ShantenType::Kokushi,
        }
    }
}

/// Calculate shanten for standard hand (4 melds + 1 pair)
///
/// Uses a recursive approach that counts:
/// - Complete melds (3 tiles forming a sequence or triplet)
/// - Taatsu/incomplete melds (2 tiles that can form a meld with 1 more)
/// - Pairs
///
/// Formula: shanten = 8 - 2*melds - max(taatsu + pairs, melds + 1)
/// With adjustment for having a pair
pub fn calculate_standard_shanten(counts: &TileCounts) -> i8 {
    calculate_standard_shanten_with_melds(counts, 0)
}

/// Calculate shanten for standard hand with called melds
///
/// `called_melds` is the number of complete melds already called.
fn calculate_standard_shanten_with_melds(counts: &TileCounts, called_melds: u8) -> i8 {
    // Convert to array representation for faster calculation
    // Index 0-8: man 1-9, 9-17: pin 1-9, 18-26: sou 1-9, 27-33: honors
    let tiles = counts_to_array(counts);

    // Count total tiles in hand
    let total_hand_tiles: u8 = tiles.iter().sum();

    // Calculate minimum tiles needed for tenpai with this many called melds
    // Tenpai requires: (4 - called_melds - 1) complete melds + 1 taatsu + 1 pair
    // OR: (4 - called_melds) complete melds + 1 floating tile (tanki wait)
    // Minimum is: max(1, 13 - 3 * called_melds) for called_melds < 4
    // For 4 called melds: 1 tile minimum (tanki wait)
    let min_tenpai_tiles: u8 = if called_melds >= 4 {
        1
    } else {
        13u8.saturating_sub(3 * called_melds)
    };

    // If we don't have enough tiles for tenpai, calculate how many we're short
    // and add that to the formula-based shanten
    let tile_deficit = min_tenpai_tiles.saturating_sub(total_hand_tiles);

    let mut best_shanten = 8i8; // Maximum possible shanten

    // Try with and without a pair extracted
    // Without pair
    let (melds, taatsu) = count_melds_and_taatsu(&tiles);
    let shanten =
        calculate_shanten_value_with_called(melds, taatsu, false, called_melds, tile_deficit);
    best_shanten = min(best_shanten, shanten);

    // Try extracting each possible pair
    for i in 0..34 {
        if tiles[i] >= 2 {
            let mut tiles_copy = tiles;
            tiles_copy[i] -= 2;
            let (melds, taatsu) = count_melds_and_taatsu(&tiles_copy);
            let shanten = calculate_shanten_value_with_called(
                melds,
                taatsu,
                true,
                called_melds,
                tile_deficit,
            );
            best_shanten = min(best_shanten, shanten);
        }
    }

    best_shanten
}

/// Convert a 34-element array back to TileCounts.
///
/// Index mapping: 0–8 = 1m–9m, 9–17 = 1p–9p, 18–26 = 1s–9s, 27–33 = honors (East…Red).
/// Zero counts are omitted from the resulting map.
pub fn array_to_tilecounts(arr: &[u8; 34]) -> TileCounts {
    let mut counts = TileCounts::new();
    for (idx, &count) in arr.iter().enumerate() {
        if count > 0 {
            counts.insert(index_to_tile(idx), count);
        }
    }
    counts
}

/// Convert TileCounts to a 34-element array.
///
/// Index mapping: 0–8 = 1m–9m, 9–17 = 1p–9p, 18–26 = 1s–9s, 27–33 = honors (East…Red).
pub fn counts_to_array(counts: &TileCounts) -> [u8; 34] {
    let mut arr = [0u8; 34];

    for (&tile, &count) in counts {
        let idx = tile_to_index(tile);
        arr[idx] = count;
    }

    arr
}

/// Convert a tile to its array index (0-33).
///
/// Man 1–9 → 0–8, Pin 1–9 → 9–17, Sou 1–9 → 18–26, honors → 27–33.
pub fn tile_to_index(tile: Tile) -> usize {
    match tile {
        Tile::Suited { suit, value } => {
            let base = match suit {
                Suit::Man => 0,
                Suit::Pin => 9,
                Suit::Sou => 18,
            };
            base + (value as usize - 1)
        }
        Tile::Honor(honor) => {
            27 + match honor {
                Honor::East => 0,
                Honor::South => 1,
                Honor::West => 2,
                Honor::North => 3,
                Honor::White => 4,
                Honor::Green => 5,
                Honor::Red => 6,
            }
        }
    }
}

/// Convert array index (0–33) back to a [`Tile`].
pub fn index_to_tile(idx: usize) -> Tile {
    if idx < 27 {
        let suit = match idx / 9 {
            0 => Suit::Man,
            1 => Suit::Pin,
            _ => Suit::Sou,
        };
        let value = (idx % 9) as u8 + 1;
        Tile::suited(suit, value)
    } else {
        let honor = match idx - 27 {
            0 => Honor::East,
            1 => Honor::South,
            2 => Honor::West,
            3 => Honor::North,
            4 => Honor::White,
            5 => Honor::Green,
            _ => Honor::Red,
        };
        Tile::honor(honor)
    }
}

/// Count complete melds and incomplete melds (taatsu) in the tiles
fn count_melds_and_taatsu(tiles: &[u8; 34]) -> (u8, u8) {
    let mut tiles = *tiles;
    let mut melds = 0u8;
    let mut taatsu = 0u8;

    // Process each suit separately (indices 0-8, 9-17, 18-26)
    for suit_start in [0, 9, 18] {
        let (suit_melds, suit_taatsu) = count_suit_melds(&mut tiles, suit_start);
        melds += suit_melds;
        taatsu += suit_taatsu;
    }

    // Process honors (27-33) - can only form triplets, not sequences
    for tile_count in tiles.iter_mut().skip(27) {
        if *tile_count >= 3 {
            melds += 1;
            *tile_count -= 3
        }
        if *tile_count >= 2 {
            taatsu += 1;
            *tile_count -= 2;
        }
    }

    (melds, taatsu)
}

/// Count melds and taatsu for a single suit
fn count_suit_melds(tiles: &mut [u8; 34], start: usize) -> (u8, u8) {
    let mut melds = 0u8;
    let mut taatsu = 0u8;

    // First pass: extract complete melds greedily
    // We try multiple orderings and take the best result
    let (m1, remaining1) = extract_melds_sequences_first(tiles, start);
    let (m2, remaining2) = extract_melds_triplets_first(tiles, start);

    // Choose the approach that gives more melds
    let (best_melds, mut remaining) = if m1 >= m2 {
        (m1, remaining1)
    } else {
        (m2, remaining2)
    };
    melds += best_melds;

    // Second pass: count taatsu (incomplete melds) from remaining tiles
    // Pairs
    for count in remaining.iter_mut().skip(start).take(9) {
        if *count >= 2 {
            taatsu += 1;
            *count -= 2;
        }
    }

    // Ryanmen/Penchan (adjacent tiles like 12, 23, 89)
    for i in start..(start + 8) {
        if remaining[i] >= 1 && remaining[i + 1] >= 1 {
            taatsu += 1;
            remaining[i] -= 1;
            remaining[i + 1] -= 1;
        }
    }

    // Kanchan (gap like 13, 24)
    for i in start..(start + 7) {
        if remaining[i] >= 1 && remaining[i + 2] >= 1 {
            taatsu += 1;
            remaining[i] -= 1;
            remaining[i + 2] -= 1;
        }
    }

    // Update the original tiles array
    tiles[start..(start + 9)].copy_from_slice(&remaining[start..(start + 9)]);

    (melds, taatsu)
}

/// Extract melds preferring sequences first
fn extract_melds_sequences_first(tiles: &[u8; 34], start: usize) -> (u8, [u8; 34]) {
    let mut remaining = *tiles;
    let mut melds = 0u8;

    // Extract sequences first
    for i in start..(start + 7) {
        while remaining[i] >= 1 && remaining[i + 1] >= 1 && remaining[i + 2] >= 1 {
            melds += 1;
            remaining[i] -= 1;
            remaining[i + 1] -= 1;
            remaining[i + 2] -= 1;
        }
    }

    // Then triplets
    for count in remaining.iter_mut().skip(start).take(9) {
        while *count >= 3 {
            melds += 1;
            *count -= 3;
        }
    }

    (melds, remaining)
}

/// Extract melds preferring triplets first
fn extract_melds_triplets_first(tiles: &[u8; 34], start: usize) -> (u8, [u8; 34]) {
    let mut remaining = *tiles;
    let mut melds = 0u8;

    // Extract triplets first
    for count in remaining.iter_mut().skip(start).take(9) {
        while *count >= 3 {
            melds += 1;
            *count -= 3;
        }
    }

    // Then sequences
    for i in start..(start + 7) {
        while remaining[i] >= 1 && remaining[i + 1] >= 1 && remaining[i + 2] >= 1 {
            melds += 1;
            remaining[i] -= 1;
            remaining[i + 1] -= 1;
            remaining[i + 2] -= 1;
        }
    }

    (melds, remaining)
}

/// Calculate shanten value from meld and taatsu counts, accounting for called melds
///
/// `tile_deficit` is how many tiles short we are of the minimum needed for tenpai.
/// This ensures we don't report tenpai when there aren't enough tiles to form a valid wait.
fn calculate_shanten_value_with_called(
    melds: u8,
    taatsu: u8,
    has_pair: bool,
    called_melds: u8,
    tile_deficit: u8,
) -> i8 {
    // Total melds = melds found in hand + called melds
    let total_melds = melds + called_melds;

    // If we have 4+ melds and a pair, we have a complete hand
    // But only if we have enough tiles (no deficit)
    if total_melds >= 4 && has_pair && tile_deficit == 0 {
        return -1;
    }

    // Maximum useful taatsu is (4 - total_melds) because we need exactly 4 melds
    // Use saturating_sub to avoid overflow when total_melds > 4
    let max_useful_taatsu = 4u8.saturating_sub(total_melds);
    let useful_taatsu = min(taatsu, max_useful_taatsu);

    // Base shanten: need 4 melds, each meld needs 3 tiles
    // Start with 8 (worst case: no progress)
    // Subtract 2 for each complete meld (saves 2 tile changes)
    // Subtract 1 for each taatsu (saves 1 tile change)
    // Subtract 1 if we have a pair (saves 1 tile change for the pair)

    let mut shanten = 8i8 - (2 * total_melds.min(4) as i8) - (useful_taatsu as i8);

    if has_pair {
        shanten -= 1;
    }

    // However, if total_melds + useful_taatsu > 4, we have too many blocks
    // We can only use 4 blocks total (excluding the pair)
    let total_blocks = total_melds.min(4) + useful_taatsu;
    if total_blocks > 4 {
        // Each excess block means we counted a taatsu that won't help
        shanten += (total_blocks - 4) as i8;
    }

    // If we don't have enough tiles to form a valid tenpai, we can't be tenpai
    // Add the tile deficit to shanten (each missing tile is one more step away)
    if shanten >= 0 {
        shanten = max(shanten, tile_deficit as i8);
    }

    shanten
}

/// Calculate shanten for chiitoitsu (seven pairs)
///
/// Formula: 6 - pairs + max(0, 7 - unique_tiles)
/// We need 7 different pairs. Each pair we have reduces shanten by 1.
/// If we have fewer than 7 unique tiles, we need to draw new tiles too.
pub fn calculate_chiitoitsu_shanten(counts: &TileCounts) -> i8 {
    let mut pairs = 0i8;
    let mut unique_tiles = 0i8;

    for &count in counts.values() {
        if count >= 1 {
            unique_tiles += 1;
        }
        if count >= 2 {
            pairs += 1;
        }
    }

    // We need 7 pairs from 7 different tiles
    // Each pair reduces shanten by 1 from base of 6
    // But we also need 7 unique tiles

    6 - pairs + (7 - unique_tiles).max(0)
}

/// Calculate shanten for kokushi (thirteen orphans)
///
/// We need all 13 terminal/honor tiles, plus one duplicate.
/// Formula: 13 - unique_terminals - has_pair
pub fn calculate_kokushi_shanten(counts: &TileCounts) -> i8 {
    let mut unique_terminals = 0i8;
    let mut has_pair = false;

    for &tile in &KOKUSHI_TILES {
        let count = counts.get(&tile).copied().unwrap_or(0);
        if count >= 1 {
            unique_terminals += 1;
        }
        if count >= 2 {
            has_pair = true;
        }
    }

    // We need 13 unique terminals + 1 pair
    // Base shanten is 13 (need all 13 + 1 for pair = 14 tiles, but we have 13)

    13 - unique_terminals - if has_pair { 1 } else { 0 }
}

/// Calculate theoretical ukeire (tile acceptance) for a hand.
///
/// Returns a list of tiles that would improve the hand (reduce shanten)
/// along with the count of how many of each are available. Assumes a full
/// 136-tile deck — the only tiles subtracted are those in the hand itself.
/// For a practical calculation that accounts for visible tiles on the table,
/// see [`calculate_ukeire_with_visible`].
pub fn calculate_ukeire(counts: &TileCounts) -> UkeireResult {
    calculate_ukeire_inner(counts, 0, None)
}

/// Calculate theoretical ukeire (tile acceptance) for a hand with called melds.
///
/// `called_melds` is the number of complete melds already called (pon, chi, kan).
/// These melds are not included in `counts` — only the remaining hand tiles are.
/// Assumes a full 136-tile deck — the only tiles subtracted are those in the hand.
/// For a practical calculation that accounts for visible tiles on the table,
/// see [`calculate_ukeire_with_melds_and_visible`].
pub fn calculate_ukeire_with_melds(counts: &TileCounts, called_melds: u8) -> UkeireResult {
    calculate_ukeire_inner(counts, called_melds, None)
}

/// Calculate practical ukeire (tile acceptance) accounting for visible tiles.
///
/// Like [`calculate_ukeire`], but subtracts `visible_counts` from the available
/// tile pool. `visible_counts` should include all tiles the player can see:
/// all discard ponds, all open melds on the table, dora indicators, etc.
/// Tiles in the player's own hand should NOT be included in `visible_counts`
/// (they are already accounted for).
pub fn calculate_ukeire_with_visible(
    counts: &TileCounts,
    visible_counts: &TileCounts,
) -> UkeireResult {
    calculate_ukeire_inner(counts, 0, Some(visible_counts))
}

/// Calculate practical ukeire (tile acceptance) with called melds and visible tiles.
///
/// Combines meld-aware shanten calculation with practical tile availability.
/// `called_melds` is the number of complete melds already called (pon, chi, kan).
/// `visible_counts` should include all tiles the player can see on the table
/// (discard ponds, open melds, dora indicators, etc.) — these are subtracted
/// from the theoretical maximum of 4 per tile type.
pub fn calculate_ukeire_with_melds_and_visible(
    counts: &TileCounts,
    called_melds: u8,
    visible_counts: &TileCounts,
) -> UkeireResult {
    calculate_ukeire_inner(counts, called_melds, Some(visible_counts))
}

/// Shared ukeire implementation.
///
/// When `visible_counts` is `None`, available copies = 4 - hand_count (theoretical).
/// When `visible_counts` is `Some`, available copies = 4 - hand_count - visible_count (practical).
fn calculate_ukeire_inner(
    counts: &TileCounts,
    called_melds: u8,
    visible_counts: Option<&TileCounts>,
) -> UkeireResult {
    let current = calculate_shanten_with_melds(counts, called_melds);
    let mut accepting_tiles = Vec::new();
    let mut total_count = 0u8;

    // Try adding each possible tile and see if it improves shanten
    for idx in 0..34 {
        let tile = index_to_tile(idx);

        let hand_count = counts.get(&tile).copied().unwrap_or(0);
        let visible_count = visible_counts
            .and_then(|vc| vc.get(&tile).copied())
            .unwrap_or(0);

        // Skip if already have 4 of this tile in hand (can't test adding a 5th)
        if hand_count >= 4 {
            continue;
        }

        // Add the tile temporarily
        let mut test_counts = counts.clone();
        *test_counts.entry(tile).or_insert(0) += 1;

        let new_shanten = calculate_shanten_with_melds(&test_counts, called_melds);

        if new_shanten.shanten < current.shanten {
            let available = 4u8.saturating_sub(hand_count + visible_count);
            accepting_tiles.push(UkeireTile { tile, available });
            total_count += available;
        }
    }

    UkeireResult {
        shanten: current.shanten,
        tiles: accepting_tiles,
        total_count,
    }
}

/// Result of ukeire calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UkeireResult {
    /// Current shanten value
    pub shanten: i8,
    /// Tiles that would improve the hand
    pub tiles: Vec<UkeireTile>,
    /// Total count of all accepting tiles
    pub total_count: u8,
}

/// A single tile that improves the hand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UkeireTile {
    /// The tile
    pub tile: Tile,
    /// How many copies are available to draw.
    /// Theoretical: 4 - hand_count. Practical: 4 - hand_count - visible_count.
    pub available: u8,
}

// ============================================================================
// Chi / Kan utility functions
// ============================================================================

/// Return all valid chi (sequence call) combinations for a discarded tile.
///
/// Given the tiles in a player's hand and a `discarded_tile`, this function
/// returns every pair of tiles already in the hand that would form a three-tile
/// sequence together with the discard.  Only suited tiles (man, pin, sou) can
/// form sequences — honor tiles always produce an empty result.
///
/// Each returned `(Tile, Tile)` is the pair of hand tiles used (not the
/// discard itself), listed in index order.
pub fn valid_chi_combinations(hand: &TileCounts, discarded_tile: Tile) -> Vec<(Tile, Tile)> {
    let arr = counts_to_array(hand);
    let d = tile_to_index(discarded_tile);

    // Honor tiles (index > 26) cannot form sequences
    if d > 26 {
        return vec![];
    }

    let val = d % 9; // position within suit (0-8)
    let mut combos = Vec::new();

    // Discarded tile is the HIGH end of sequence: (d-2, d-1, d)
    if val >= 2 && arr[d - 2] > 0 && arr[d - 1] > 0 {
        combos.push((index_to_tile(d - 2), index_to_tile(d - 1)));
    }
    // Discarded tile is the MIDDLE of sequence: (d-1, d, d+1)
    if (1..=7).contains(&val) && arr[d - 1] > 0 && arr[d + 1] > 0 {
        combos.push((index_to_tile(d - 1), index_to_tile(d + 1)));
    }
    // Discarded tile is the LOW end of sequence: (d, d+1, d+2)
    if val <= 6 && arr[d + 1] > 0 && arr[d + 2] > 0 {
        combos.push((index_to_tile(d + 1), index_to_tile(d + 2)));
    }

    combos
}

/// Compute the best possible shanten after calling chi with a specific combo.
///
/// Removes the two `combo` tiles from the hand, then tries every possible
/// discard and returns the minimum resulting shanten (with `num_melds + 1`
/// called melds).
///
/// # Panics
///
/// Panics if either combo tile is not present in the hand.
pub fn shanten_after_chi(hand: &TileCounts, combo: (Tile, Tile), num_melds: u8) -> i8 {
    let mut arr = counts_to_array(hand);
    let idx0 = tile_to_index(combo.0);
    let idx1 = tile_to_index(combo.1);

    arr[idx0] = arr[idx0]
        .checked_sub(1)
        .expect("combo tile not in hand");
    arr[idx1] = arr[idx1]
        .checked_sub(1)
        .expect("combo tile not in hand");

    let new_melds = num_melds + 1;
    let mut best = i8::MAX;

    for i in 0..34 {
        if arr[i] > 0 {
            arr[i] -= 1;
            let counts = array_to_tilecounts(&arr);
            let s = calculate_shanten_with_melds(&counts, new_melds).shanten;
            if s < best {
                best = s;
            }
            arr[i] += 1;
        }
    }

    best
}

/// Compute shanten after declaring kan.
///
/// - [`KanType::Open`] (daiminkan): removes 3 tiles from hand, increments melds.
/// - [`KanType::Closed`] (ankan): removes 4 tiles from hand, increments melds.
/// - [`KanType::Added`] (kakan): removes 1 tile from hand, melds unchanged.
///
/// # Panics
///
/// Panics if the hand does not contain enough copies of `kan_tile`.
pub fn shanten_after_kan(
    hand: &TileCounts,
    kan_tile: Tile,
    kan_type: KanType,
    num_melds: u8,
) -> i8 {
    let mut arr = counts_to_array(hand);
    let kt = tile_to_index(kan_tile);

    let (remove, melds) = match kan_type {
        KanType::Open => (3u8, num_melds + 1),
        KanType::Closed => (4u8, num_melds + 1),
        KanType::Added => (1u8, num_melds),
    };

    arr[kt] = arr[kt]
        .checked_sub(remove)
        .expect("not enough tiles in hand for kan");

    let counts = array_to_tilecounts(&arr);
    calculate_shanten_with_melds(&counts, melds).shanten
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_hand, to_counts};

    fn shanten(hand: &str) -> i8 {
        let tiles = parse_hand(hand).unwrap();
        let counts = to_counts(&tiles);
        calculate_shanten(&counts).shanten
    }

    fn shanten_type(hand: &str) -> ShantenType {
        let tiles = parse_hand(hand).unwrap();
        let counts = to_counts(&tiles);
        calculate_shanten(&counts).best_type
    }

    // ===== Complete Hand Tests =====

    #[test]
    fn test_complete_standard_hand() {
        // 123m 456p 789s 111z 22z - complete hand
        assert_eq!(shanten("123m456p789s11122z"), -1);
    }

    #[test]
    fn test_complete_chiitoitsu() {
        // Seven pairs - complete
        assert_eq!(shanten("1122m3344p5566s77z"), -1);
    }

    #[test]
    fn test_complete_kokushi() {
        // Thirteen orphans with pair on 1m
        assert_eq!(shanten("19m19p19s12345677z"), -1);
    }

    // ===== Tenpai Tests (shanten = 0) =====

    #[test]
    fn test_tenpai_standard() {
        // 123m 456p 789s 111z 2z - waiting on 2z
        assert_eq!(shanten("123m456p789s1112z"), 0);
    }

    #[test]
    fn test_tenpai_chiitoitsu() {
        // Six pairs + one single - waiting on pair
        assert_eq!(shanten("1122m3344p5566s7z"), 0);
    }

    #[test]
    fn test_tenpai_kokushi() {
        // 12 different terminals + 1 pair, waiting on 13th
        assert_eq!(shanten("19m19p19s1234567z"), 0);
    }

    // ===== Iishanten Tests (shanten = 1) =====

    #[test]
    fn test_iishanten_standard() {
        // Almost complete, needs 2 tiles
        assert_eq!(shanten("123m456p789s112z"), 1);
    }

    #[test]
    fn test_iishanten_chiitoitsu() {
        // Five pairs + two singles
        assert_eq!(shanten("1122m3344p5566s"), 1);
    }

    // ===== Various Shanten Tests =====

    #[test]
    fn test_high_shanten() {
        // Very disconnected hand
        assert!(shanten("1379m1379p1379s1z") >= 4);
    }

    #[test]
    fn test_multi_shanten() {
        // Hand with some structure but very scattered
        // 123m is one meld, but rest is very disconnected
        let s = shanten("123m147p258s12345z");
        // High shanten due to scattered tiles
        assert!(
            (3..=7).contains(&s),
            "Expected shanten between 3 and 7, got {}",
            s
        );

        // A more connected hand should have lower shanten
        let s2 = shanten("123m456p789s11234z");
        assert!(
            s2 <= 3,
            "Expected shanten <= 3 for connected hand, got {}",
            s2
        );
    }

    // ===== Best Type Tests =====

    #[test]
    fn test_best_type_standard() {
        // Standard hand shape
        assert_eq!(shanten_type("123m456p789s1112z"), ShantenType::Standard);
    }

    #[test]
    fn test_best_type_chiitoitsu() {
        // Seven pairs shape
        assert_eq!(shanten_type("1122m3344p5566s7z"), ShantenType::Chiitoitsu);
    }

    #[test]
    fn test_best_type_kokushi() {
        // Kokushi shape
        assert_eq!(shanten_type("19m19p19s1234567z"), ShantenType::Kokushi);
    }

    // ===== Ukeire Tests =====

    #[test]
    fn test_ukeire_tenpai() {
        // Tenpai hand waiting on specific tiles
        let tiles = parse_hand("123m456p789s1112z").unwrap();
        let counts = to_counts(&tiles);
        let ukeire = calculate_ukeire(&counts);

        assert_eq!(ukeire.shanten, 0);
        assert!(!ukeire.tiles.is_empty());
    }

    #[test]
    fn test_ukeire_iishanten() {
        // Iishanten has multiple improving tiles
        let tiles = parse_hand("123m456p789s112z").unwrap();
        let counts = to_counts(&tiles);
        let ukeire = calculate_ukeire(&counts);

        assert_eq!(ukeire.shanten, 1);
        assert!(ukeire.total_count > 0);
    }

    #[test]
    fn test_ukeire_complete_hand() {
        // Complete hand has no improving tiles (already best)
        let tiles = parse_hand("123m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let ukeire = calculate_ukeire(&counts);

        assert_eq!(ukeire.shanten, -1);
        // No tiles improve a complete hand
        assert!(ukeire.tiles.is_empty());
    }

    // ===== Ukeire with Called Melds Tests =====

    #[test]
    fn test_ukeire_with_called_pon_tenpai() {
        // 23678p234567s with called pon of 2z - tenpai
        // Hand tiles: 2,3,6,7,8p + 2,3,4,5,6,7s (11 tiles)
        // Called: (222z) = 1 meld
        // Should be waiting on 1p/4p (shanpon or sequence wait)
        use crate::parse::parse_hand_with_aka;
        let parsed = parse_hand_with_aka("23678p234567s(222z)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds = parsed.called_melds.len() as u8;

        let ukeire = calculate_ukeire_with_melds(&counts, called_melds);

        assert_eq!(ukeire.shanten, 0, "Hand should be tenpai");
        // A tenpai hand with called melds should have very few waits, not 34
        assert!(
            ukeire.tiles.len() <= 5,
            "Tenpai hand should have at most a few waits, got {}",
            ukeire.tiles.len()
        );
        assert!(
            ukeire.total_count <= 20,
            "Total accepting tiles should be reasonable, got {}",
            ukeire.total_count
        );
    }

    #[test]
    fn test_ukeire_with_two_called_melds_iishanten() {
        // 234568m with called chi 789p and called pon of white dragons
        // Hand tiles: 2,3,4,5,6,8m (6 tiles, since 2 called melds = 6 tiles consumed)
        // Should have a reasonable number of improving tiles, not 34
        use crate::parse::parse_hand_with_aka;
        let parsed = parse_hand_with_aka("234568m(789p)(whwhwh)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds = parsed.called_melds.len() as u8;

        let ukeire = calculate_ukeire_with_melds(&counts, called_melds);

        assert_eq!(ukeire.shanten, 1, "Hand should be iishanten");
        assert!(
            ukeire.tiles.len() <= 10,
            "Iishanten hand with 2 called melds should have limited waits, got {}",
            ukeire.tiles.len()
        );
        assert!(
            ukeire.total_count <= 40,
            "Total accepting tiles should be reasonable, got {}",
            ukeire.total_count
        );
    }

    #[test]
    fn test_ukeire_without_melds_matches_original() {
        // Verify calculate_ukeire_with_melds(counts, 0) matches calculate_ukeire(counts)
        let tiles = parse_hand("123m456p789s1112z").unwrap();
        let counts = to_counts(&tiles);

        let ukeire_original = calculate_ukeire(&counts);
        let ukeire_with_melds = calculate_ukeire_with_melds(&counts, 0);

        assert_eq!(ukeire_original.shanten, ukeire_with_melds.shanten);
        assert_eq!(ukeire_original.tiles.len(), ukeire_with_melds.tiles.len());
        assert_eq!(ukeire_original.total_count, ukeire_with_melds.total_count);
    }

    #[test]
    fn test_ukeire_called_melds_vs_no_melds_differ() {
        // The same tile counts should give different ukeire results
        // when called_melds is 0 vs > 0
        use crate::parse::parse_hand_with_aka;
        let parsed = parse_hand_with_aka("23678p234567s(222z)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds = parsed.called_melds.len() as u8;

        let ukeire_correct = calculate_ukeire_with_melds(&counts, called_melds);
        let ukeire_wrong = calculate_ukeire_with_melds(&counts, 0);

        // With 0 called melds, 11 tiles can't form 4 melds + pair,
        // so shanten will be higher and many more tiles "improve" the hand
        assert!(
            ukeire_wrong.tiles.len() > ukeire_correct.tiles.len(),
            "Ignoring called melds should produce more (incorrect) accepting tiles: wrong={}, correct={}",
            ukeire_wrong.tiles.len(),
            ukeire_correct.tiles.len()
        );
    }

    // ===== Ukeire with Visible Tiles Tests =====

    #[test]
    fn test_ukeire_with_visible_reduces_count() {
        // Tenpai hand: 123m456p789s1112z — waiting on 2z
        let tiles = parse_hand("123m456p789s1112z").unwrap();
        let counts = to_counts(&tiles);

        let theoretical = calculate_ukeire(&counts);

        // Suppose 2 copies of 2z are visible on the table
        let mut visible = TileCounts::new();
        visible.insert(Tile::honor(Honor::South), 2);

        let practical = calculate_ukeire_with_visible(&counts, &visible);

        assert_eq!(theoretical.shanten, practical.shanten);

        // Find 2z in both results
        let theo_2z = theoretical
            .tiles
            .iter()
            .find(|t| t.tile == Tile::honor(Honor::South));
        let prac_2z = practical
            .tiles
            .iter()
            .find(|t| t.tile == Tile::honor(Honor::South));

        assert!(theo_2z.is_some(), "2z should be a theoretical wait");
        assert!(prac_2z.is_some(), "2z should still be a practical wait");

        // Theoretical: 4 - 1 (in hand) = 3 available
        assert_eq!(theo_2z.unwrap().available, 3);
        // Practical: 4 - 1 (hand) - 2 (visible) = 1 available
        assert_eq!(prac_2z.unwrap().available, 1);

        assert!(
            practical.total_count < theoretical.total_count,
            "Practical total ({}) should be less than theoretical ({})",
            practical.total_count,
            theoretical.total_count
        );
    }

    #[test]
    fn test_ukeire_with_visible_shows_zero_available_when_all_copies_seen() {
        // Tenpai hand: 123m456p789s1112z — waiting on 2z
        let tiles = parse_hand("123m456p789s1112z").unwrap();
        let counts = to_counts(&tiles);

        // All remaining 3 copies of 2z are visible
        let mut visible = TileCounts::new();
        visible.insert(Tile::honor(Honor::South), 3);

        let practical = calculate_ukeire_with_visible(&counts, &visible);

        // 2z should still appear as a wait but with 0 available
        let prac_2z = practical
            .tiles
            .iter()
            .find(|t| t.tile == Tile::honor(Honor::South));
        assert!(
            prac_2z.is_some(),
            "2z should still appear as a wait even with 0 available"
        );
        assert_eq!(prac_2z.unwrap().available, 0);
    }

    #[test]
    fn test_ukeire_with_visible_no_visible_matches_theoretical() {
        let tiles = parse_hand("123m456p789s1112z").unwrap();
        let counts = to_counts(&tiles);

        let theoretical = calculate_ukeire(&counts);
        let practical = calculate_ukeire_with_visible(&counts, &TileCounts::new());

        assert_eq!(theoretical.shanten, practical.shanten);
        assert_eq!(theoretical.tiles.len(), practical.tiles.len());
        assert_eq!(theoretical.total_count, practical.total_count);
    }

    #[test]
    fn test_ukeire_with_melds_and_visible() {
        // 23678p234567s with called pon of 2z — tenpai
        use crate::parse::parse_hand_with_aka;
        let parsed = parse_hand_with_aka("23678p234567s(222z)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds = parsed.called_melds.len() as u8;

        let theoretical = calculate_ukeire_with_melds(&counts, called_melds);

        // Some waits are visible on the table
        let mut visible = TileCounts::new();
        // Imagine 1p has 2 copies in discard ponds
        visible.insert(Tile::suited(Suit::Pin, 1), 2);

        let practical = calculate_ukeire_with_melds_and_visible(&counts, called_melds, &visible);

        assert_eq!(theoretical.shanten, practical.shanten);
        assert!(
            practical.total_count <= theoretical.total_count,
            "Practical total ({}) should be <= theoretical ({})",
            practical.total_count,
            theoretical.total_count
        );
    }

    // ===== Index Conversion Tests =====

    #[test]
    fn test_tile_index_roundtrip() {
        // Verify all tiles convert correctly
        for idx in 0..34 {
            let tile = index_to_tile(idx);
            let back = tile_to_index(tile);
            assert_eq!(
                idx, back,
                "Tile {:?} at index {} converted back to {}",
                tile, idx, back
            );
        }
    }

    #[test]
    fn test_specific_tile_indices() {
        assert_eq!(tile_to_index(Tile::suited(Suit::Man, 1)), 0);
        assert_eq!(tile_to_index(Tile::suited(Suit::Man, 9)), 8);
        assert_eq!(tile_to_index(Tile::suited(Suit::Pin, 1)), 9);
        assert_eq!(tile_to_index(Tile::suited(Suit::Sou, 1)), 18);
        assert_eq!(tile_to_index(Tile::honor(Honor::East)), 27);
        assert_eq!(tile_to_index(Tile::honor(Honor::Red)), 33);
    }

    // ===== Regression Tests =====

    #[test]
    fn test_sequences_first_then_triplet_extraction() {
        // Regression test: ensure triplets are correctly extracted after sequences
        // Hand: 233344455666m1p (13 tiles)
        // Optimal decomposition: 234m + 345m + 345m + 666m = 4 melds, waiting for 1p pair
        // This should be tenpai (shanten = 0)
        //
        // The sequences-first algorithm should:
        // 1. Extract sequences: 234m, 345m, 345m (3 melds)
        // 2. Extract remaining triplet: 666m (1 meld)
        // Total: 4 melds
        //
        // If triplet extraction incorrectly uses `> 3` instead of `>= 3`,
        // it will fail to extract the 666m triplet, giving only 3 melds
        // and incorrectly reporting shanten = 1 instead of 0.
        assert_eq!(
            shanten("233344455666m1p"),
            0,
            "Hand 233344455666m1p should be tenpai (shanten=0), not iishanten"
        );
    }

    #[test]
    fn test_extract_melds_sequences_first_with_remaining_triplet() {
        // Direct test of the internal meld extraction logic
        // Input: 2(x1), 3(x3), 4(x3), 5(x2), 6(x3) in manzu
        // After extracting sequences 234, 345, 345, we should have 6(x3) left
        // which should be extracted as a triplet
        let mut tiles = [0u8; 34];
        tiles[1] = 1; // 2m
        tiles[2] = 3; // 3m
        tiles[3] = 3; // 4m
        tiles[4] = 2; // 5m
        tiles[5] = 3; // 6m

        let (melds, remaining) = extract_melds_sequences_first(&tiles, 0);

        assert_eq!(
            melds, 4,
            "Should extract 4 melds (3 sequences + 1 triplet), got {}",
            melds
        );
        assert_eq!(
            remaining[5], 0,
            "All 6m tiles should be extracted as triplet, but {} remain",
            remaining[5]
        );
    }

    // ===== Chi Combination Tests =====

    /// Helper: build a TileCounts from a 34-element array.
    fn counts_from_arr(arr: [u8; 34]) -> TileCounts {
        array_to_tilecounts(&arr)
    }

    #[test]
    fn test_chi_no_combos() {
        let mut arr = [0u8; 34];
        arr[5] = 1; // 6m
        let combos = valid_chi_combinations(&counts_from_arr(arr), index_to_tile(0));
        assert!(combos.is_empty());
    }

    #[test]
    fn test_chi_low_end() {
        // 2m+3m in hand, discard 1m → combo (2m, 3m)
        let mut arr = [0u8; 34];
        arr[1] = 1; // 2m
        arr[2] = 1; // 3m
        let combos = valid_chi_combinations(&counts_from_arr(arr), index_to_tile(0));
        assert_eq!(combos, vec![(index_to_tile(1), index_to_tile(2))]);
    }

    #[test]
    fn test_chi_high_end() {
        // 7m+8m in hand, discard 9m → combo (7m, 8m)
        let mut arr = [0u8; 34];
        arr[6] = 1; // 7m
        arr[7] = 1; // 8m
        let combos = valid_chi_combinations(&counts_from_arr(arr), index_to_tile(8));
        assert_eq!(combos, vec![(index_to_tile(6), index_to_tile(7))]);
    }

    #[test]
    fn test_chi_two_combos() {
        // 1m,2m,4m in hand, discard 3m → (1m,2m) and (2m,4m)
        let mut arr = [0u8; 34];
        arr[0] = 1; // 1m
        arr[1] = 1; // 2m
        arr[3] = 1; // 4m
        let combos = valid_chi_combinations(&counts_from_arr(arr), index_to_tile(2));
        assert_eq!(
            combos,
            vec![
                (index_to_tile(0), index_to_tile(1)),
                (index_to_tile(1), index_to_tile(3)),
            ]
        );
    }

    #[test]
    fn test_chi_three_combos() {
        // 3m,4m,6m,7m in hand, discard 5m → (3m,4m), (4m,6m), (6m,7m)
        let mut arr = [0u8; 34];
        arr[2] = 1; // 3m
        arr[3] = 1; // 4m
        arr[5] = 1; // 6m
        arr[6] = 1; // 7m
        let combos = valid_chi_combinations(&counts_from_arr(arr), index_to_tile(4));
        assert_eq!(
            combos,
            vec![
                (index_to_tile(2), index_to_tile(3)),
                (index_to_tile(3), index_to_tile(5)),
                (index_to_tile(5), index_to_tile(6)),
            ]
        );
    }

    #[test]
    fn test_chi_honor_tile() {
        let mut arr = [0u8; 34];
        arr[27] = 3; // East x3
        let combos = valid_chi_combinations(&counts_from_arr(arr), index_to_tile(27));
        assert!(combos.is_empty());
    }

    // ===== Shanten After Chi Tests =====

    #[test]
    fn test_shanten_after_chi_good() {
        // Tenpai: 123m 456m 789m 1p 123s (13 tiles, shanten 0)
        let mut arr = [0u8; 34];
        arr[..9].fill(1); // 1-9m
        arr[9] = 1; // 1p
        arr[18] = 1; // 1s
        arr[19] = 1; // 2s
        arr[20] = 1; // 3s
        let hand = counts_from_arr(arr);

        let before = calculate_shanten(&hand).shanten;
        assert_eq!(before, 0);

        // Chi 1s from opponent using (2s, 3s)
        let after = shanten_after_chi(&hand, (index_to_tile(19), index_to_tile(20)), 0);
        assert!(after <= before, "Expected shanten <= {before} after good chi, got {after}");
    }

    #[test]
    fn test_shanten_after_chi_bad() {
        // Scattered hand: all odd tiles
        let mut arr = [0u8; 34];
        for &i in &[0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24] {
            arr[i] = 1;
        }
        let hand = counts_from_arr(arr);

        let before = calculate_shanten(&hand).shanten;
        // Chi 2m(1) with combo (1m, 3m) = (0, 2)
        let after = shanten_after_chi(&hand, (index_to_tile(0), index_to_tile(2)), 0);
        assert!(
            after >= before - 1,
            "Unexpectedly good shanten after bad chi: {before} -> {after}"
        );
    }

    // ===== Shanten After Kan Tests =====

    #[test]
    fn test_shanten_after_kan_open() {
        // 123m 456m 789m EEE 1p (13 tiles, shanten 0)
        let mut arr = [0u8; 34];
        arr[..9].fill(1); // 1-9m
        arr[27] = 3; // East x3
        arr[9] = 1; // 1p
        let hand = counts_from_arr(arr);

        let before = calculate_shanten(&hand).shanten;
        assert_eq!(before, 0);

        // Open kan on East: remove 3, melds 0→1
        let after = shanten_after_kan(&hand, index_to_tile(27), KanType::Open, 0);
        assert_eq!(after, 0, "Expected 0 after open kan, got {after}");
    }

    #[test]
    fn test_shanten_after_kan_closed() {
        // 123m 456m EEEE 1p (11 tiles, 0 melds)
        let mut arr = [0u8; 34];
        arr[..6].fill(1); // 1-6m
        arr[27] = 4; // East x4
        arr[9] = 1; // 1p
        let hand = counts_from_arr(arr);

        // Closed kan on East: remove 4, melds 0→1
        // After: 123m 456m 1p (7 tiles) with 1 meld
        let after = shanten_after_kan(&hand, index_to_tile(27), KanType::Closed, 0);
        // 7 tiles + 1 meld is quite short of a full hand, so shanten is high
        assert!(after >= 0, "Shanten should be non-negative after closed kan: {after}");
    }

    #[test]
    fn test_shanten_after_kan_added() {
        // 123m 456m 789m E (10 tiles, 1 meld already — pon of East)
        let mut arr = [0u8; 34];
        arr[..9].fill(1); // 1-9m
        arr[27] = 1; // East (the 4th, adding to existing pon)
        let hand = counts_from_arr(arr);

        let before = calculate_shanten_with_melds(&hand, 1).shanten;
        // Added kan: remove 1 East, melds stay at 1
        let after = shanten_after_kan(&hand, index_to_tile(27), KanType::Added, 1);
        // After removing the East, hand is 123m 456m 789m with 1 meld
        // That's 9 tiles + 1 meld, missing a pair → shanten should be reasonable
        assert!((-1..=2).contains(&after),
            "Unexpected shanten after added kan: {before} -> {after}");
    }
}
