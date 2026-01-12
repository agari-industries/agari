use crate::tile::Tile;
use crate::parse::TileCounts;

/// A single meld (group of 3 tiles)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Meld {
    /// Sequence (e.g., 123m) - stores the lowest tile
    Shuntsu(Tile),
    /// Triplet (e.g., 111m) - stores the tile
    Koutsu(Tile),
}

/// A complete hand decomposition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandStructure {
    /// Standard hand: 4 melds + 1 pair
    Standard {
        melds: Vec<Meld>,
        pair: Tile,
    },
    /// Seven pairs
    Chiitoitsu {
        pairs: Vec<Tile>,
    },
}

/// Find all valid decompositions of a hand
pub fn decompose_hand(counts: &TileCounts) -> Vec<HandStructure> {
    let mut results = Vec::new();
    
    // Check for chiitoitsu
    if is_chiitoitsu(counts) {
        let mut pairs: Vec<Tile> = counts.keys().copied().collect();
        pairs.sort();  // Consistent ordering
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
                melds.sort_by_key(|m| match m {
                    Meld::Shuntsu(t) | Meld::Koutsu(t) => *t,
                });
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

/// Find all ways to form exactly `needed` melds from the given tiles
fn find_all_meld_combinations(mut counts: TileCounts, needed: u32) -> Vec<Vec<Meld>> {
    // Remove zero-count entries
    counts.retain(|_, &mut c| c > 0);
    
    // Base case: no more melds needed
    if needed == 0 {
        if counts.is_empty() {
            return vec![vec![]];  // One valid solution: empty meld list
        } else {
            return vec![];  // Leftover tiles = no valid solutions
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
            sub_result.insert(0, Meld::Koutsu(tile));
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
                    sub_result.insert(0, Meld::Shuntsu(tile));
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
    is_chiitoitsu(counts) || is_standard_hand(counts)
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
        assert!(results.len() >= 2, "Expected multiple decompositions, got {}", results.len());
        
        // Verify we have both all-triplet and all-sequence versions
        let has_all_triplets = results.iter().any(|r| {
            match r {
                HandStructure::Standard { melds, .. } => {
                    melds.iter().filter(|m| matches!(m, Meld::Koutsu(_))).count() == 4
                }
                _ => false,
            }
        });
        
        let has_sequences = results.iter().any(|r| {
            match r {
                HandStructure::Standard { melds, .. } => {
                    melds.iter().any(|m| matches!(m, Meld::Shuntsu(_)))
                }
                _ => false,
            }
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
        let has_iipeikou = results.iter().any(|r| {
            match r {
                HandStructure::Standard { melds, .. } => {
                    let seq_count = melds.iter()
                        .filter(|m| *m == &Meld::Shuntsu(Tile::suited(Suit::Man, 1)))
                        .count();
                    seq_count == 2
                }
                _ => false,
            }
        });
        
        assert!(has_iipeikou, "Should find iipeikou (two identical sequences)");
    }

    #[test]
    fn test_decompose_invalid_hand() {
        let tiles = parse_hand("1234m5678p9s12355z").unwrap();
        let counts = to_counts(&tiles);
        let results = decompose_hand(&counts);
        
        assert!(results.is_empty(), "Invalid hand should have no decompositions");
    }
}
