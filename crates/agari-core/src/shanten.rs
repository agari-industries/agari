//! Shanten calculator for Riichi Mahjong
//!
//! Shanten is the minimum number of tile exchanges needed to reach tenpai (ready hand).
//! - Shanten = -1: Complete (winning) hand
//! - Shanten = 0: Tenpai (one tile away from winning)
//! - Shanten = 1: Iishanten (two tiles away)
//! - etc.

use serde::{Deserialize, Serialize};

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

/// Calculate ukeire (tile acceptance) for a hand
///
/// Returns a list of tiles that would improve the hand (reduce shanten)
/// along with the count of how many of each are available.
pub fn calculate_ukeire(counts: &TileCounts) -> UkeireResult {
    calculate_ukeire_with_melds(counts, 0)
}

/// Calculate ukeire (tile acceptance) for a hand with called melds
///
/// `called_melds` is the number of complete melds already called (pon, chi, kan).
/// These melds are not included in `counts` - only the remaining hand tiles are.
pub fn calculate_ukeire_with_melds(counts: &TileCounts, called_melds: u8) -> UkeireResult {
    let current = calculate_shanten_with_melds(counts, called_melds);
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

        let new_shanten = calculate_shanten_with_melds(&test_counts, called_melds);

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
}
