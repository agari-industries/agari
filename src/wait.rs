//! Wait type detection for Riichi Mahjong hands.
//!
//! The "wait" describes what shape the hand was in before the winning tile
//! completed it. This affects fu calculation and Pinfu eligibility.

use crate::context::GameContext;
use crate::hand::{HandStructure, Meld};
use crate::tile::{Honor, Tile};

/// The type of wait that led to the winning hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaitType {
    /// Two-sided sequence wait (e.g., 23 waiting on 1 or 4)
    /// 0 fu
    Ryanmen,

    /// Middle/closed wait (e.g., 13 waiting on 2)
    /// 2 fu
    Kanchan,

    /// Edge wait (e.g., 12 waiting on 3, or 89 waiting on 7)
    /// 2 fu
    Penchan,

    /// Dual triplet wait (e.g., 11+22 as pairs, waiting on either)
    /// 0 fu (but the resulting triplet contributes fu)
    Shanpon,

    /// Single tile / pair wait (e.g., holding one 5m, waiting for another)
    /// 2 fu
    Tanki,

    /// 13-sided kokushi wait (waiting on any of the 13 orphans)
    /// Special yakuman wait
    Kokushi13,
}

impl WaitType {
    /// Fu value contributed by this wait type
    pub fn fu(&self) -> u8 {
        match self {
            WaitType::Ryanmen => 0,
            WaitType::Shanpon => 0,
            WaitType::Kanchan => 2,
            WaitType::Penchan => 2,
            WaitType::Tanki => 2,
            WaitType::Kokushi13 => 0, // Yakuman, fu doesn't matter
        }
    }

    /// Is this a "good" wait (multiple outs)?
    pub fn is_good_wait(&self) -> bool {
        matches!(
            self,
            WaitType::Ryanmen | WaitType::Shanpon | WaitType::Kokushi13
        )
    }
}

/// Detect all possible wait types for a given hand structure and winning tile.
///
/// Returns multiple wait types when the winning tile could have completed
/// the hand in different ways (e.g., a tile that's both in a sequence and
/// could complete a pair).
///
/// # Arguments
/// * `structure` - The completed hand structure
/// * `winning_tile` - The tile that completed the hand
///
/// # Returns
/// A vector of all possible wait types. Empty if the winning tile doesn't
/// appear in the structure (shouldn't happen with valid input).
pub fn detect_wait_types(structure: &HandStructure, winning_tile: Tile) -> Vec<WaitType> {
    match structure {
        HandStructure::Chiitoitsu { pairs } => {
            // Chiitoitsu is always tanki wait
            // (you're always waiting on one tile to complete a pair)
            if pairs.contains(&winning_tile) {
                vec![WaitType::Tanki]
            } else {
                vec![]
            }
        }

        HandStructure::Kokushi { pair } => {
            // Kokushi: tanki on pair tile, or could be 13-wait
            if *pair == winning_tile {
                vec![WaitType::Tanki]
            } else {
                // If not waiting on pair, it's 13-sided wait
                vec![WaitType::Kokushi13]
            }
        }

        HandStructure::Standard { melds, pair } => {
            let mut wait_types = Vec::new();

            // Check if winning tile completed the pair (tanki wait)
            if *pair == winning_tile {
                wait_types.push(WaitType::Tanki);
            }

            // Check each meld
            // Note: Only closed melds can be affected by the winning tile.
            // Open melds (chi/pon) were already complete when called.
            for meld in melds {
                match meld {
                    Meld::Koutsu(t, is_open) if *t == winning_tile && !is_open => {
                        // Closed triplet contains winning tile → was shanpon wait
                        // (had a pair, waiting for third to make triplet)
                        wait_types.push(WaitType::Shanpon);
                    }

                    Meld::Shuntsu(start_tile, is_open) if !is_open => {
                        // Only check closed sequences for wait type
                        if let Some(wt) = check_shuntsu_wait(*start_tile, winning_tile) {
                            wait_types.push(wt);
                        }
                    }

                    // Kans and open melds don't affect wait type detection
                    _ => {}
                }
            }

            wait_types
        }
    }
}

/// Check if a winning tile completes a sequence, and if so, what wait type.
///
/// # Arguments
/// * `start_tile` - The lowest tile in the sequence (e.g., 2m for 234m)
/// * `winning_tile` - The tile that completed the hand
///
/// # Returns
/// Some(WaitType) if the winning tile is part of this sequence, None otherwise.
fn check_shuntsu_wait(start_tile: Tile, winning_tile: Tile) -> Option<WaitType> {
    // Sequences only exist for suited tiles
    let (suit, start_val) = match start_tile {
        Tile::Suited { suit, value } => (suit, value),
        Tile::Honor(_) => return None, // Can't happen, but handle gracefully
    };

    let (w_suit, w_val) = match winning_tile {
        Tile::Suited { suit, value } => (suit, value),
        Tile::Honor(_) => return None, // Honor can't be in a sequence
    };

    // Must be same suit
    if suit != w_suit {
        return None;
    }

    // Check if winning tile is in this sequence (start, start+1, start+2)
    if w_val < start_val || w_val > start_val + 2 {
        return None;
    }

    // Determine wait type based on position in sequence
    Some(wait_type_for_shuntsu_position(start_val, w_val))
}

/// Determine wait type for a sequence based on which tile completed it.
///
/// For a sequence starting at value V (tiles: V, V+1, V+2):
/// - If won with V (low): came from V+1,V+2 shape
/// - If won with V+1 (middle): came from V,V+2 shape (kanchan)
/// - If won with V+2 (high): came from V,V+1 shape
fn wait_type_for_shuntsu_position(start_val: u8, winning_val: u8) -> WaitType {
    if winning_val == start_val {
        // Won with low tile of sequence
        // Pre-win shape was (start+1, start+2), e.g., "89" waiting
        if start_val + 2 == 9 {
            // Had 89, only 7 completes it (can't wait on 10) → penchan
            WaitType::Penchan
        } else {
            // e.g., had 45, waiting on 3 or 6 → ryanmen
            WaitType::Ryanmen
        }
    } else if winning_val == start_val + 1 {
        // Won with middle tile → kanchan (always)
        // e.g., had 35, only 4 completes it
        WaitType::Kanchan
    } else {
        // Won with high tile (start + 2)
        // Pre-win shape was (start, start+1), e.g., "12" waiting
        if start_val == 1 {
            // Had 12, only 3 completes it (can't wait on 0) → penchan
            WaitType::Penchan
        } else {
            // e.g., had 34, waiting on 2 or 5 → ryanmen
            WaitType::Ryanmen
        }
    }
}

/// Check if a hand structure qualifies for Pinfu.
///
/// Pinfu requirements:
/// 1. Standard hand (not chiitoitsu)
/// 2. All four melds are sequences (no triplets)
/// 3. Pair is NOT yakuhai (not dragons, not round/seat wind)
/// 4. Wait is ryanmen (two-sided)
/// 5. Hand must be closed (checked via context)
///
/// # Arguments
/// * `structure` - The hand structure to check
/// * `winning_tile` - The tile that completed the hand
/// * `context` - Game context (for wind information and open/closed status)
///
/// # Returns
/// true if all Pinfu conditions are met
pub fn is_pinfu(structure: &HandStructure, winning_tile: Tile, context: &GameContext) -> bool {
    // Must be closed hand
    if context.is_open {
        return false;
    }

    match structure {
        HandStructure::Chiitoitsu { .. } => false,

        HandStructure::Kokushi { .. } => false, // Kokushi can never be pinfu

        HandStructure::Standard { melds, pair } => {
            // 1. All melds must be sequences (no triplets or kans)
            let all_sequences = melds.iter().all(|m| m.is_sequence());
            if !all_sequences {
                return false;
            }

            // 2. Pair must not be yakuhai
            if is_yakuhai_pair(*pair, context) {
                return false;
            }

            // 3. Must have ryanmen wait
            let wait_types = detect_wait_types(structure, winning_tile);
            wait_types.contains(&WaitType::Ryanmen)
        }
    }
}

/// Check if a pair tile would be yakuhai (scoring).
///
/// Dragons are always yakuhai. Winds are yakuhai if they match
/// the round wind or seat wind.
fn is_yakuhai_pair(pair: Tile, context: &GameContext) -> bool {
    match pair {
        Tile::Honor(honor) => {
            match honor {
                // Dragons are always yakuhai
                Honor::White | Honor::Green | Honor::Red => true,
                // Winds are yakuhai if they're value winds
                wind => wind == context.round_wind || wind == context.seat_wind,
            }
        }
        // Suited tiles are never yakuhai
        Tile::Suited { .. } => false,
    }
}

/// Find the best (lowest fu) wait type for Pinfu eligibility.
///
/// Used when checking for Pinfu - we want to see if a ryanmen wait is possible.
pub fn best_wait_type(structure: &HandStructure, winning_tile: Tile) -> Option<WaitType> {
    let wait_types = detect_wait_types(structure, winning_tile);

    // Prefer waits with 0 fu, and among 0-fu waits, prefer Ryanmen (for Pinfu eligibility)
    // Priority order: Ryanmen (0) > Shanpon (1) > Kanchan (2) > Penchan (3) > Tanki (4) > Kokushi13 (5)
    wait_types.into_iter().min_by_key(|wt| {
        let priority = match wt {
            WaitType::Ryanmen => 0,
            WaitType::Shanpon => 1,
            WaitType::Kanchan => 2,
            WaitType::Penchan => 3,
            WaitType::Tanki => 4,
            WaitType::Kokushi13 => 5,
        };
        (wt.fu(), priority)
    })
}

/// Find the highest fu wait type for scoring purposes.
///
/// In Mahjong, when a hand can be interpreted multiple ways, we choose the
/// interpretation that gives the player the highest score. For wait types,
/// this means preferring waits with more fu (kanchan/penchan/tanki over ryanmen).
pub fn best_wait_type_for_scoring(
    structure: &HandStructure,
    winning_tile: Tile,
) -> Option<WaitType> {
    let wait_types = detect_wait_types(structure, winning_tile);

    // Prefer waits with higher fu for maximum scoring
    // Priority order (highest fu first): Tanki (2) > Kanchan (2) > Penchan (2) > Shanpon (0) > Ryanmen (0)
    wait_types.into_iter().max_by_key(|wt| {
        let priority = match wt {
            WaitType::Tanki => 5,     // 2 fu, prefer tanki if equal fu
            WaitType::Kanchan => 4,   // 2 fu
            WaitType::Penchan => 3,   // 2 fu
            WaitType::Shanpon => 1,   // 0 fu
            WaitType::Ryanmen => 0,   // 0 fu
            WaitType::Kokushi13 => 2, // 0 fu, yakuman anyway
        };
        (wt.fu(), priority)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::WinType;
    use crate::hand::decompose_hand;
    use crate::parse::{parse_hand, to_counts};
    use crate::tile::Suit;

    // ===== Basic Wait Type Tests =====

    #[test]
    fn test_ryanmen_middle_sequence() {
        // 234m - winning with 2 (from 34 wait) is ryanmen
        let wt = wait_type_for_shuntsu_position(2, 2);
        assert_eq!(wt, WaitType::Ryanmen);

        // 234m - winning with 4 (from 23 wait) is ryanmen
        let wt = wait_type_for_shuntsu_position(2, 4);
        assert_eq!(wt, WaitType::Ryanmen);
    }

    #[test]
    fn test_kanchan() {
        // 234m - winning with 3 (from 24 wait) is kanchan
        let wt = wait_type_for_shuntsu_position(2, 3);
        assert_eq!(wt, WaitType::Kanchan);

        // 567m - winning with 6 (from 57 wait) is kanchan
        let wt = wait_type_for_shuntsu_position(5, 6);
        assert_eq!(wt, WaitType::Kanchan);
    }

    #[test]
    fn test_penchan_low() {
        // 123m - winning with 3 (from 12 wait) is penchan
        let wt = wait_type_for_shuntsu_position(1, 3);
        assert_eq!(wt, WaitType::Penchan);
    }

    #[test]
    fn test_penchan_high() {
        // 789m - winning with 7 (from 89 wait) is penchan
        let wt = wait_type_for_shuntsu_position(7, 7);
        assert_eq!(wt, WaitType::Penchan);
    }

    #[test]
    fn test_ryanmen_at_edges_not_penchan() {
        // 123m - winning with 1 (from 23 wait) is ryanmen (23 waits on 1 or 4)
        let wt = wait_type_for_shuntsu_position(1, 1);
        assert_eq!(wt, WaitType::Ryanmen);

        // 789m - winning with 9 (from 78 wait) is ryanmen (78 waits on 6 or 9)
        let wt = wait_type_for_shuntsu_position(7, 9);
        assert_eq!(wt, WaitType::Ryanmen);
    }

    // ===== Full Hand Wait Detection =====

    #[test]
    fn test_detect_tanki_wait() {
        // 123m 456p 789s 111z 77z - tanki wait on 7z
        let tiles = parse_hand("123m456p789s11177z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let winning_tile = Tile::honor(Honor::Red);
        let wait_types = detect_wait_types(&structures[0], winning_tile);

        assert!(wait_types.contains(&WaitType::Tanki));
    }

    #[test]
    fn test_detect_shanpon_wait() {
        // 123m 456p 789s 111z 22z - if we won on 1z, it's shanpon
        let tiles = parse_hand("123m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let winning_tile = Tile::honor(Honor::East);
        let wait_types = detect_wait_types(&structures[0], winning_tile);

        assert!(wait_types.contains(&WaitType::Shanpon));
    }

    #[test]
    fn test_detect_ryanmen_wait() {
        // 234m 456p 789s 111z 22z - won on 4m (from 23m wait)
        let tiles = parse_hand("234m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let winning_tile = Tile::suited(Suit::Man, 4);
        let wait_types = detect_wait_types(&structures[0], winning_tile);

        assert!(wait_types.contains(&WaitType::Ryanmen));
    }

    #[test]
    fn test_detect_kanchan_wait() {
        // 123m 456p 789s 111z 22z - won on 2m... wait this doesn't work
        // Let's use: 135m... no that's not a valid hand
        // Better: 234m where we won on 3m (from 24m wait)
        let tiles = parse_hand("234m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let winning_tile = Tile::suited(Suit::Man, 3);
        let wait_types = detect_wait_types(&structures[0], winning_tile);

        assert!(wait_types.contains(&WaitType::Kanchan));
    }

    #[test]
    fn test_multiple_wait_types() {
        // A hand where the winning tile could be interpreted multiple ways
        // 111m 123m 456p 789s 22z - won on 1m
        // Could be: shanpon (completing 111) or ryanmen (completing 123 from 23)
        let tiles = parse_hand("111123m456p789s22z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let winning_tile = Tile::suited(Suit::Man, 1);

        // Check that at least one structure has multiple wait interpretations
        let has_multiple_waits = structures.iter().any(|structure| {
            let wait_types = detect_wait_types(structure, winning_tile);
            wait_types.len() > 1
        });

        // At minimum, there should be structures with different wait types
        assert!(!structures.is_empty());
        // The hand 111123m should allow multiple wait interpretations for 1m
        assert!(
            has_multiple_waits,
            "Should find structure with multiple wait types"
        );
    }

    #[test]
    fn test_chiitoitsu_always_tanki() {
        let tiles = parse_hand("1122m3344p5566s77z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        // Find the chiitoitsu structure
        let chiitoi = structures
            .iter()
            .find(|s| matches!(s, HandStructure::Chiitoitsu { .. }))
            .expect("Should have chiitoitsu structure");

        let winning_tile = Tile::honor(Honor::Red); // 7z
        let wait_types = detect_wait_types(chiitoi, winning_tile);

        assert_eq!(wait_types, vec![WaitType::Tanki]);
    }

    // ===== Pinfu Tests =====

    #[test]
    fn test_pinfu_basic() {
        // All sequences, non-yakuhai pair, ryanmen wait
        // 123m 456m 789p 234s 55s - won on 3s (ryanmen from 24s... no wait)
        // Better: 123m 456m 789p 234s 55p - won on 4s (from 23s wait - ryanmen)
        let tiles = parse_hand("123456m789p234s55p").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let winning_tile = Tile::suited(Suit::Sou, 4); // Ryanmen from 23s

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(has_pinfu, "Should qualify for pinfu");
    }

    #[test]
    fn test_pinfu_fails_with_triplet() {
        // Has a triplet, can't be pinfu
        let tiles = parse_hand("123m456p789s11155z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let winning_tile = Tile::honor(Honor::White);

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(!has_pinfu, "Triplet hand can't be pinfu");
    }

    #[test]
    fn test_pinfu_fails_with_yakuhai_pair() {
        // Dragon pair = yakuhai pair, not pinfu
        let tiles = parse_hand("123m456m789p234s55z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let winning_tile = Tile::suited(Suit::Sou, 4);

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(!has_pinfu, "Yakuhai pair (dragon) means no pinfu");
    }

    #[test]
    fn test_pinfu_fails_with_value_wind_pair() {
        // Seat wind pair = yakuhai pair, not pinfu
        let tiles = parse_hand("123m456m789p234s22z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        // South is seat wind, so 2z (south) pair is yakuhai
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let winning_tile = Tile::suited(Suit::Sou, 4);

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(!has_pinfu, "Value wind pair means no pinfu");
    }

    #[test]
    fn test_pinfu_ok_with_non_value_wind_pair() {
        // West wind pair when seat is South, round is East = not yakuhai
        let tiles = parse_hand("123m456m789p234s33z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let winning_tile = Tile::suited(Suit::Sou, 4); // Ryanmen

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(has_pinfu, "Non-value wind pair allows pinfu");
    }

    #[test]
    fn test_pinfu_fails_with_kanchan_wait() {
        // All sequences, good pair, but kanchan wait
        let tiles = parse_hand("123m456m789p234s55p").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let winning_tile = Tile::suited(Suit::Sou, 3); // Kanchan from 24s

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(!has_pinfu, "Kanchan wait means no pinfu");
    }

    #[test]
    fn test_pinfu_fails_when_open() {
        let tiles = parse_hand("123m456m789p234s55p").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South).open(); // Open hand!
        let winning_tile = Tile::suited(Suit::Sou, 4);

        let has_pinfu = structures
            .iter()
            .any(|s| is_pinfu(s, winning_tile, &context));

        assert!(!has_pinfu, "Open hand can't be pinfu");
    }

    // ===== Best Wait Type Tests =====

    #[test]
    fn test_best_wait_prefers_ryanmen() {
        // Hand where winning tile has multiple interpretations
        let tiles = parse_hand("111123m456p789s22z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let winning_tile = Tile::suited(Suit::Man, 1);

        // For structures where multiple waits exist, best should be ryanmen (0 fu)
        for structure in &structures {
            if let Some(best) = best_wait_type(structure, winning_tile) {
                // If ryanmen is available, it should be chosen
                let all_waits = detect_wait_types(structure, winning_tile);
                if all_waits.contains(&WaitType::Ryanmen) {
                    assert_eq!(best, WaitType::Ryanmen);
                }
            }
        }
    }
}
