//! Shanten calculator for Riichi Mahjong
//!
//! Shanten is the minimum number of tile exchanges needed to reach tenpai (ready hand).
//! - Shanten = -1: Complete (winning) hand
//! - Shanten = 0: Tenpai (one tile away from winning)
//! - Shanten = 1: Iishanten (two tiles away)
//! - etc.

use serde::{Deserialize, Serialize};

use crate::parse::TileCounts;
use crate::tile::{Honor, Suit, Tile, KOKUSHI_TILES};
use std::cmp::min;

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
    let standard = calculate_standard_shanten(counts);
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
    // Convert to array representation for faster calculation
    // Index 0-8: man 1-9, 9-17: pin 1-9, 18-26: sou 1-9, 27-33: honors
    let tiles = counts_to_array(counts);

    let mut best_shanten = 8i8; // Maximum possible shanten

    // Try with and without a pair extracted
    // Without pair
    let (melds, taatsu) = count_melds_and_taatsu(&tiles);
    let shanten = calculate_shanten_value(melds, taatsu, false);
    best_shanten = min(best_shanten, shanten);

    // Try extracting each possible pair
    for i in 0..34 {
        if tiles[i] >= 2 {
            let mut tiles_copy = tiles;
            tiles_copy[i] -= 2;
            let (melds, taatsu) = count_melds_and_taatsu(&tiles_copy);
            let shanten = calculate_shanten_value(melds, taatsu, true);
            best_shanten = min(best_shanten, shanten);
        }
    }

    best_shanten
}

/// Convert TileCounts to a 34-element array
fn counts_to_array(counts: &TileCounts) -> [u8; 34] {
    let mut arr = [0u8; 34];

    for (&tile, &count) in counts {
        let idx = tile_to_index(tile);
        arr[idx] = count;
    }

    arr
}

/// Convert a tile to its array index (0-33)
fn tile_to_index(tile: Tile) -> usize {
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

/// Convert array index back to tile
fn index_to_tile(idx: usize) -> Tile {
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
    for i in 27..34 {
        if tiles[i] >= 3 {
            melds += 1;
            tiles[i] -= 3;
        }
        if tiles[i] >= 2 {
            taatsu += 1;
            tiles[i] -= 2;
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
    for i in start..(start + 9) {
        if remaining[i] >= 2 {
            taatsu += 1;
            remaining[i] -= 2;
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
    for i in start..(start + 9) {
        tiles[i] = remaining[i];
    }

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
    for i in start..(start + 9) {
        while remaining[i] >= 3 {
            melds += 1;
            remaining[i] -= 3;
        }
    }

    (melds, remaining)
}

/// Extract melds preferring triplets first
fn extract_melds_triplets_first(tiles: &[u8; 34], start: usize) -> (u8, [u8; 34]) {
    let mut remaining = *tiles;
    let mut melds = 0u8;

    // Extract triplets first
    for i in start..(start + 9) {
        while remaining[i] >= 3 {
            melds += 1;
            remaining[i] -= 3;
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

/// Calculate shanten value from meld and taatsu counts
fn calculate_shanten_value(melds: u8, taatsu: u8, has_pair: bool) -> i8 {
    // If we have 4+ melds and a pair, we have a complete hand
    if melds >= 4 && has_pair {
        return -1;
    }

    // Maximum useful taatsu is (4 - melds) because we need exactly 4 melds
    // Use saturating_sub to avoid overflow when melds > 4
    let max_useful_taatsu = 4u8.saturating_sub(melds);
    let useful_taatsu = min(taatsu, max_useful_taatsu);

    // Base shanten: need 4 melds, each meld needs 3 tiles
    // Start with 8 (worst case: no progress)
    // Subtract 2 for each complete meld (saves 2 tile changes)
    // Subtract 1 for each taatsu (saves 1 tile change)
    // Subtract 1 if we have a pair (saves 1 tile change for the pair)

    let mut shanten = 8i8 - (2 * melds.min(4) as i8) - (useful_taatsu as i8);

    if has_pair {
        shanten -= 1;
    }

    // However, if melds + useful_taatsu > 4, we have too many blocks
    // We can only use 4 blocks total (excluding the pair)
    let total_blocks = melds.min(4) + useful_taatsu;
    if total_blocks > 4 {
        // Each excess block means we counted a taatsu that won't help
        shanten += (total_blocks - 4) as i8;
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
    let shanten = 6 - pairs + (7 - unique_tiles).max(0);

    shanten
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
    let shanten = 13 - unique_terminals - if has_pair { 1 } else { 0 };

    shanten
}

/// Calculate ukeire (tile acceptance) for a hand
///
/// Returns a list of tiles that would improve the hand (reduce shanten)
/// along with the count of how many of each are available.
pub fn calculate_ukeire(counts: &TileCounts) -> UkeireResult {
    let current = calculate_shanten(counts);
    let mut accepting_tiles = Vec::new();
    let mut total_count = 0u8;

    // Try adding each possible tile and see if it improves shanten
    for idx in 0..34 {
        let tile = index_to_tile(idx);

        // Skip if already have 4 of this tile
        let current_count = counts.get(&tile).copied().unwrap_or(0);
        if current_count >= 4 {
            continue;
        }

        // Add the tile temporarily
        let mut test_counts = counts.clone();
        *test_counts.entry(tile).or_insert(0) += 1;

        let new_shanten = calculate_shanten(&test_counts);

        if new_shanten.shanten < current.shanten {
            let available = 4 - current_count;
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
    /// How many are available (4 - already in hand)
    pub available: u8,
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
            s >= 3 && s <= 7,
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
}
