use serde::{Deserialize, Serialize};

use crate::parse::TileCounts;
use crate::tile::{Tile, KOKUSHI_TILES};

/// Type of kan (quad) meld
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KanType {
    /// Closed kan (ankan) - all 4 tiles drawn from wall
    /// Concealed for scoring purposes
    Closed,
    /// Open kan (daiminkan) - called from another player's discard
    /// Open for scoring purposes
    Open,
    /// Added kan (shouminkan/kakan) - added 4th tile to an existing pon
    /// Open for scoring purposes (since the original pon was open)
    Added,
}

impl KanType {
    /// Whether this kan type is considered open for scoring
    pub fn is_open(&self) -> bool {
        match self {
            KanType::Closed => false,
            KanType::Open | KanType::Added => true,
        }
    }
}

/// A single meld (group of 3 or 4 tiles)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Meld {
    /// Sequence (e.g., 123m) - stores the lowest tile
    /// Second field indicates if the meld is open (called via chi)
    Shuntsu(Tile, bool),
    /// Triplet (e.g., 111m) - stores the tile
    /// Second field indicates if the meld is open (called via pon)
    Koutsu(Tile, bool),
    /// Kan/Quad (e.g., 1111m) - stores the tile and kan type
    Kan(Tile, KanType),
}

impl Meld {
    /// Create a closed sequence
    pub fn shuntsu(start_tile: Tile) -> Self {
        Meld::Shuntsu(start_tile, false)
    }

    /// Create an open sequence (called via chi)
    pub fn shuntsu_open(start_tile: Tile) -> Self {
        Meld::Shuntsu(start_tile, true)
    }

    /// Create a closed triplet
    pub fn koutsu(tile: Tile) -> Self {
        Meld::Koutsu(tile, false)
    }

    /// Create an open triplet (called via pon)
    pub fn koutsu_open(tile: Tile) -> Self {
        Meld::Koutsu(tile, true)
    }

    /// Create a kan (quad)
    pub fn kan(tile: Tile, kan_type: KanType) -> Self {
        Meld::Kan(tile, kan_type)
    }

    /// Check if this meld is open (called from another player)
    pub fn is_open(&self) -> bool {
        match self {
            Meld::Shuntsu(_, open) => *open,
            Meld::Koutsu(_, open) => *open,
            Meld::Kan(_, kan_type) => kan_type.is_open(),
        }
    }

    /// Get the tile associated with this meld
    pub fn tile(&self) -> Tile {
        match self {
            Meld::Shuntsu(t, _) => *t,
            Meld::Koutsu(t, _) => *t,
            Meld::Kan(t, _) => *t,
        }
    }

    /// Check if this is a triplet or kan (for yaku detection)
    pub fn is_triplet_or_kan(&self) -> bool {
        matches!(self, Meld::Koutsu(_, _) | Meld::Kan(_, _))
    }

    /// Check if this is a sequence
    pub fn is_sequence(&self) -> bool {
        matches!(self, Meld::Shuntsu(_, _))
    }

    /// Check if this meld is concealed (for san ankou, suuankou, etc.)
    /// A meld is concealed if it's not open AND (for triplets/kans) wasn't completed by ron
    pub fn is_concealed(&self) -> bool {
        !self.is_open()
    }
}

/// A complete hand decomposition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandStructure {
    /// Standard hand: 4 melds + 1 pair
    Standard { melds: Vec<Meld>, pair: Tile },
    /// Seven pairs
    Chiitoitsu { pairs: Vec<Tile> },
    /// Thirteen orphans (kokushi musou)
    Kokushi {
        /// The tile that appears twice (the pair)
        pair: Tile,
    },
}

/// Find all valid decompositions of a hand
pub fn decompose_hand(counts: &TileCounts) -> Vec<HandStructure> {
    let mut results = Vec::new();

    // Check for kokushi musou (thirteen orphans)
    if let Some(pair) = check_kokushi(counts) {
        results.push(HandStructure::Kokushi { pair });
    }

    // Check for chiitoitsu
    if is_chiitoitsu(counts) {
        let mut pairs: Vec<Tile> = counts.keys().copied().collect();
        pairs.sort(); // Consistent ordering
        results.push(HandStructure::Chiitoitsu { pairs });
    }

    // Check for standard hands (4 melds + pair)
    for (&pair_tile, &count) in counts {
        if count >= 2 {
            // Try this tile as the pair
            let mut remaining = counts.clone();
            *remaining.get_mut(&pair_tile).unwrap() -= 2;
            if remaining[&pair_tile] == 0 {
                remaining.remove(&pair_tile);
            }

            // Find all ways to form 4 melds from remaining tiles
            let meld_combinations = find_all_meld_combinations(remaining, 4);

            for mut melds in meld_combinations {
                melds.sort_by_key(|m| m.tile());
                results.push(HandStructure::Standard {
                    melds,
                    pair: pair_tile,
                });
            }
        }
    }

    // Remove duplicates (same structure found via different paths)
    results.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    results.dedup();

    results
}

/// Find all valid decompositions of a hand with pre-declared called melds
///
/// The called_melds are already fixed (kans, pons, chis), and we need to
/// find valid decompositions for the remaining tiles in hand.
pub fn decompose_hand_with_melds(
    hand_tiles: &TileCounts,
    called_melds: &[Meld],
) -> Vec<HandStructure> {
    let mut results = Vec::new();

    // Count how many melds we need to form from hand tiles
    let melds_needed = 4 - called_melds.len() as u32;

    // For standard hands with called melds
    for (&pair_tile, &count) in hand_tiles {
        if count >= 2 {
            // Try this tile as the pair
            let mut remaining = hand_tiles.clone();
            *remaining.get_mut(&pair_tile).unwrap() -= 2;
            if remaining[&pair_tile] == 0 {
                remaining.remove(&pair_tile);
            }

            // Find all ways to form the remaining melds
            let meld_combinations = find_all_meld_combinations(remaining, melds_needed);

            for hand_melds in meld_combinations {
                // Combine called melds with hand melds
                let mut all_melds: Vec<Meld> = called_melds.to_vec();
                all_melds.extend(hand_melds);
                all_melds.sort_by_key(|m| m.tile());

                results.push(HandStructure::Standard {
                    melds: all_melds,
                    pair: pair_tile,
                });
            }
        }
    }

    // Note: Chiitoitsu and Kokushi cannot have called melds
    // (Chiitoitsu requires 7 pairs, Kokushi requires specific 13 tiles)
    // So we don't check for those when called_melds is non-empty

    // Remove duplicates
    results.sort_by(|a, b| format!("{:?}", a).cmp(&format!("{:?}", b)));
    results.dedup();

    results
}

/// Find all ways to form exactly `needed` melds from the given tiles
fn find_all_meld_combinations(mut counts: TileCounts, needed: u32) -> Vec<Vec<Meld>> {
    // Remove zero-count entries
    counts.retain(|_, &mut c| c > 0);

    // Base case: no more melds needed
    if needed == 0 {
        if counts.is_empty() {
            return vec![vec![]]; // One valid solution: empty meld list
        } else {
            return vec![]; // Leftover tiles = no valid solutions
        }
    }

    // No tiles left but still need melds
    if counts.is_empty() {
        return vec![];
    }

    let mut results = Vec::new();

    // Get the smallest tile (for consistent processing order)
    let tile = *counts.keys().min().unwrap();
    let count = counts[&tile];

    // Option 1: Form a triplet (koutsu) with this tile
    if count >= 3 {
        let mut after_triplet = counts.clone();
        *after_triplet.get_mut(&tile).unwrap() -= 3;

        for mut sub_result in find_all_meld_combinations(after_triplet, needed - 1) {
            sub_result.insert(0, Meld::koutsu(tile));
            results.push(sub_result);
        }
    }

    // Option 2: Form a sequence (shuntsu) starting with this tile
    if let Tile::Suited { suit, value } = tile {
        if value <= 7 {
            let next1 = Tile::suited(suit, value + 1);
            let next2 = Tile::suited(suit, value + 2);

            let has_seq = counts.get(&next1).copied().unwrap_or(0) >= 1
                && counts.get(&next2).copied().unwrap_or(0) >= 1;

            if has_seq {
                let mut after_seq = counts.clone();
                *after_seq.get_mut(&tile).unwrap() -= 1;
                *after_seq.get_mut(&next1).unwrap() -= 1;
                *after_seq.get_mut(&next2).unwrap() -= 1;

                for mut sub_result in find_all_meld_combinations(after_seq, needed - 1) {
                    sub_result.insert(0, Meld::shuntsu(tile));
                    results.push(sub_result);
                }
            }
        }
    }

    results
}

pub fn is_chiitoitsu(counts: &TileCounts) -> bool {
    counts.len() == 7 && counts.values().all(|&c| c == 2)
}

/// Check if hand is kokushi musou (thirteen orphans).
/// Returns the pair tile if valid, None otherwise.
fn check_kokushi(counts: &TileCounts) -> Option<Tile> {
    let total: u8 = counts.values().sum();
    if total != 14 {
        return None;
    }

    // Must have at least one of each kokushi tile
    for &tile in &KOKUSHI_TILES {
        if counts.get(&tile).copied().unwrap_or(0) < 1 {
            return None;
        }
    }

    // Must have no non-terminal/honor tiles
    for tile in counts.keys() {
        if !tile.is_terminal_or_honor() {
            return None;
        }
    }

    // Find the pair (the tile that appears twice)
    let mut pair_tile = None;
    for &tile in &KOKUSHI_TILES {
        let count = counts.get(&tile).copied().unwrap_or(0);
        if count == 2 {
            if pair_tile.is_some() {
                return None; // More than one pair
            }
            pair_tile = Some(tile);
        } else if count > 2 {
            return None; // More than 2 of any tile
        }
    }

    pair_tile
}

/// Check for kokushi 13-sided wait (all tiles unique before winning tile)
pub fn is_kokushi_13_wait(counts: &TileCounts) -> bool {
    let total: u8 = counts.values().sum();
    if total != 13 {
        return false;
    }

    for &tile in &KOKUSHI_TILES {
        if counts.get(&tile).copied().unwrap_or(0) != 1 {
            return false;
        }
    }

    counts.len() == 13
}

pub fn is_standard_hand(counts: &TileCounts) -> bool {
    for (&tile, &count) in counts {
        if count >= 2 {
            let mut remaining = counts.clone();
            *remaining.get_mut(&tile).unwrap() -= 2;

            if remaining[&tile] == 0 {
                remaining.remove(&tile);
            }

            if can_form_melds(remaining, 4) {
                return true;
            }
        }
    }
    false
}

fn can_form_melds(mut counts: TileCounts, needed: u32) -> bool {
    counts.retain(|_, &mut c| c > 0);

    if needed == 0 {
        return counts.is_empty();
    }

    if counts.is_empty() {
        return false;
    }

    let tile = *counts.keys().min().unwrap();
    let count = counts[&tile];

    if count >= 3 {
        let mut after_triplet = counts.clone();
        *after_triplet.get_mut(&tile).unwrap() -= 3;
        if can_form_melds(after_triplet, needed - 1) {
            return true;
        }
    }

    if let Tile::Suited { suit, value } = tile {
        if value <= 7 {
            let next1 = Tile::suited(suit, value + 1);
            let next2 = Tile::suited(suit, value + 2);

            let has_seq = counts.get(&next1).copied().unwrap_or(0) >= 1
                && counts.get(&next2).copied().unwrap_or(0) >= 1;

            if has_seq {
                let mut after_seq = counts.clone();
                *after_seq.get_mut(&tile).unwrap() -= 1;
                *after_seq.get_mut(&next1).unwrap() -= 1;
                *after_seq.get_mut(&next2).unwrap() -= 1;
                if can_form_melds(after_seq, needed - 1) {
                    return true;
                }
            }
        }
    }

    false
}

pub fn is_winning_hand(counts: &TileCounts) -> bool {
    check_kokushi(counts).is_some() || is_chiitoitsu(counts) || is_standard_hand(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_hand, to_counts};
    use crate::tile::{Honor, Suit};

    #[test]
    fn test_chiitoitsu() {
        let tiles = parse_hand("1122m3344p5566s77z").unwrap();
        let counts = to_counts(&tiles);
        assert!(is_chiitoitsu(&counts));
        assert!(is_winning_hand(&counts));
    }

    #[test]
    fn test_not_chiitoitsu_four_of_kind() {
        let tiles = parse_hand("1111m22m33p44p55s66s").unwrap();
        let counts = to_counts(&tiles);
        assert!(!is_chiitoitsu(&counts));
    }

    #[test]
    fn test_standard_hand() {
        let tiles = parse_hand("123m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        assert!(is_standard_hand(&counts));
        assert!(is_winning_hand(&counts));
    }

    #[test]
    fn test_all_triplets() {
        let tiles = parse_hand("111m222p333s44455z").unwrap();
        let counts = to_counts(&tiles);
        assert!(is_standard_hand(&counts));
    }

    #[test]
    fn test_invalid_hand() {
        let tiles = parse_hand("1234m5678p9s123z").unwrap();
        let counts = to_counts(&tiles);
        assert!(!is_winning_hand(&counts));
    }

    #[test]
    fn test_pinfu_shape() {
        let tiles = parse_hand("123456m789p234s55z").unwrap();
        let counts = to_counts(&tiles);
        assert!(is_standard_hand(&counts));
    }

    // ===== Decomposition Tests =====

    #[test]
    fn test_decompose_simple_hand() {
        // 123m 456p 789s 111z 22z - only one way to decompose
        let tiles = parse_hand("123m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let results = decompose_hand(&counts);

        assert_eq!(results.len(), 1);
        match &results[0] {
            HandStructure::Standard { melds, pair } => {
                assert_eq!(melds.len(), 4);
                assert_eq!(*pair, Tile::honor(Honor::South));
            }
            _ => panic!("Expected standard hand"),
        }
    }

    #[test]
    fn test_decompose_chiitoitsu() {
        let tiles = parse_hand("1122m3344p5566s77z").unwrap();
        let counts = to_counts(&tiles);
        let results = decompose_hand(&counts);

        assert_eq!(results.len(), 1);
        match &results[0] {
            HandStructure::Chiitoitsu { pairs } => {
                assert_eq!(pairs.len(), 7);
            }
            _ => panic!("Expected chiitoitsu"),
        }
    }

    #[test]
    fn test_decompose_multiple_structures() {
        // 111222333m 111z 55z
        // Can be: (111 + 222 + 333) or (123 + 123 + 123) for the man tiles
        let tiles = parse_hand("111222333m11155z").unwrap();
        let counts = to_counts(&tiles);
        let results = decompose_hand(&counts);

        // Should find at least 2 different decompositions
        assert!(
            results.len() >= 2,
            "Expected multiple decompositions, got {}",
            results.len()
        );

        // Verify we have both all-triplet and all-sequence versions
        let has_all_triplets = results.iter().any(|r| match r {
            HandStructure::Standard { melds, .. } => {
                melds.iter().filter(|m| m.is_triplet_or_kan()).count() == 4
            }
            _ => false,
        });

        let has_sequences = results.iter().any(|r| match r {
            HandStructure::Standard { melds, .. } => melds.iter().any(|m| m.is_sequence()),
            _ => false,
        });

        assert!(has_all_triplets, "Should find all-triplet decomposition");
        assert!(has_sequences, "Should find sequence decomposition");
    }

    #[test]
    fn test_decompose_iipeikou_shape() {
        // 112233m 456p 789s 55z - has two identical sequences
        let tiles = parse_hand("112233m456p789s55z").unwrap();
        let counts = to_counts(&tiles);
        let results = decompose_hand(&counts);

        assert!(!results.is_empty());

        // Should have a decomposition with two 123m shuntsu
        let has_iipeikou = results.iter().any(|r| match r {
            HandStructure::Standard { melds, .. } => {
                let seq_count = melds
                    .iter()
                    .filter(|m| *m == &Meld::shuntsu(Tile::suited(Suit::Man, 1)))
                    .count();
                seq_count == 2
            }
            _ => false,
        });

        assert!(
            has_iipeikou,
            "Should find iipeikou (two identical sequences)"
        );
    }

    #[test]
    fn test_decompose_invalid_hand() {
        let tiles = parse_hand("1234m5678p9s12355z").unwrap();
        let counts = to_counts(&tiles);
        let results = decompose_hand(&counts);

        assert!(
            results.is_empty(),
            "Invalid hand should have no decompositions"
        );
    }

    // ===== Kan and Meld State Tests =====

    #[test]
    fn test_meld_constructors() {
        // Test closed melds
        let closed_shuntsu = Meld::shuntsu(Tile::suited(Suit::Man, 1));
        assert!(!closed_shuntsu.is_open());
        assert!(closed_shuntsu.is_sequence());
        assert!(!closed_shuntsu.is_triplet_or_kan());

        let closed_koutsu = Meld::koutsu(Tile::suited(Suit::Pin, 5));
        assert!(!closed_koutsu.is_open());
        assert!(closed_koutsu.is_triplet_or_kan());
        assert!(!closed_koutsu.is_sequence());

        // Test open melds
        let open_shuntsu = Meld::shuntsu_open(Tile::suited(Suit::Sou, 2));
        assert!(open_shuntsu.is_open());

        let open_koutsu = Meld::koutsu_open(Tile::honor(Honor::East));
        assert!(open_koutsu.is_open());
    }

    #[test]
    fn test_kan_types() {
        // Closed kan (ankan)
        let closed_kan = Meld::kan(Tile::suited(Suit::Man, 1), KanType::Closed);
        assert!(!closed_kan.is_open());
        assert!(closed_kan.is_triplet_or_kan());
        assert!(closed_kan.is_concealed());

        // Open kan (daiminkan)
        let open_kan = Meld::kan(Tile::suited(Suit::Pin, 9), KanType::Open);
        assert!(open_kan.is_open());
        assert!(open_kan.is_triplet_or_kan());
        assert!(!open_kan.is_concealed());

        // Added kan (shouminkan)
        let added_kan = Meld::kan(Tile::honor(Honor::White), KanType::Added);
        assert!(added_kan.is_open());
        assert!(added_kan.is_triplet_or_kan());
    }

    #[test]
    fn test_meld_tile() {
        let shuntsu = Meld::shuntsu(Tile::suited(Suit::Man, 3));
        assert_eq!(shuntsu.tile(), Tile::suited(Suit::Man, 3));

        let koutsu = Meld::koutsu(Tile::honor(Honor::Red));
        assert_eq!(koutsu.tile(), Tile::honor(Honor::Red));

        let kan = Meld::kan(Tile::suited(Suit::Sou, 7), KanType::Closed);
        assert_eq!(kan.tile(), Tile::suited(Suit::Sou, 7));
    }

    #[test]
    fn test_kan_type_is_open() {
        assert!(!KanType::Closed.is_open());
        assert!(KanType::Open.is_open());
        assert!(KanType::Added.is_open());
    }
}
