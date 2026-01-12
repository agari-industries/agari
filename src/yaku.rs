//! Yaku (scoring pattern) detection for Riichi Mahjong hands.

use std::collections::HashMap;
use crate::hand::{HandStructure, Meld};
use crate::tile::{Tile, Suit, Honor};
use crate::context::{GameContext, WinType, count_dora};
use crate::parse::TileCounts;

/// Represents a scoring pattern (yaku)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Yaku {
    // === 1 han ===
    Riichi,           // Declared riichi (closed only)
    Ippatsu,          // Win within 1 turn of riichi
    MenzenTsumo,      // Self-draw with closed hand
    Tanyao,           // All simples (no terminals or honors)
    Pinfu,            // All sequences, valueless pair, two-sided wait (TODO: wait detection)
    Iipeikou,         // Two identical sequences
    Yakuhai(Honor),   // Triplet of dragons or value winds
    RinshanKaihou,    // Win on kan replacement tile
    HaiteiRaoyue,     // Tsumo on last drawable tile
    HouteiRaoyui,     // Ron on last discard
    
    // === 2 han ===
    DoubleRiichi,     // Riichi on first turn
    Toitoi,           // All triplets
    SanshokuDoujun,   // Same sequence in all 3 suits
    Ittsu,            // 1-9 straight in one suit
    Chiitoitsu,       // Seven pairs
    Chanta,           // All groups contain terminal or honor
    SanAnkou,         // Three concealed triplets
    
    // === 3 han ===
    Honitsu,          // Half flush (one suit + honors)
    Junchan,          // All groups contain terminal (no honors)
    Ryanpeikou,       // Two pairs of identical sequences
    
    // === 6 han ===
    Chinitsu,         // Full flush (one suit only)
    
    // === Yakuman (limit hands) ===
    Tenhou,           // Dealer wins on initial deal
    Chiihou,          // Non-dealer wins on first draw
}

impl Yaku {
    /// Base han value (for closed hands)
    /// Yakuman return 13 han as a convention (actual scoring treats them specially)
    pub fn han(&self) -> u8 {
        match self {
            // 1 han
            Yaku::Riichi => 1,
            Yaku::Ippatsu => 1,
            Yaku::MenzenTsumo => 1,
            Yaku::Tanyao => 1,
            Yaku::Pinfu => 1,
            Yaku::Iipeikou => 1,
            Yaku::Yakuhai(_) => 1,
            Yaku::RinshanKaihou => 1,
            Yaku::HaiteiRaoyue => 1,
            Yaku::HouteiRaoyui => 1,
            
            // 2 han
            Yaku::DoubleRiichi => 2,
            Yaku::Toitoi => 2,
            Yaku::SanshokuDoujun => 2,
            Yaku::Ittsu => 2,
            Yaku::Chiitoitsu => 2,
            Yaku::Chanta => 2,
            Yaku::SanAnkou => 2,
            
            // 3 han
            Yaku::Honitsu => 3,
            Yaku::Junchan => 3,
            Yaku::Ryanpeikou => 3,
            
            // 6 han
            Yaku::Chinitsu => 6,
            
            // Yakuman (13 han equivalent)
            Yaku::Tenhou => 13,
            Yaku::Chiihou => 13,
        }
    }
    
    /// Han value when hand is open (some yaku lose 1 han)
    pub fn han_open(&self) -> Option<u8> {
        match self {
            // These yaku are invalid when open
            Yaku::Riichi => None,
            Yaku::DoubleRiichi => None,
            Yaku::Ippatsu => None,
            Yaku::MenzenTsumo => None,
            Yaku::Pinfu => None,
            Yaku::Iipeikou => None,
            Yaku::Ryanpeikou => None,
            Yaku::Tenhou => None,   // By definition, must be closed
            Yaku::Chiihou => None,  // By definition, must be closed
            
            // These lose 1 han when open
            Yaku::SanshokuDoujun => Some(1),
            Yaku::Ittsu => Some(1),
            Yaku::Chanta => Some(1),
            Yaku::Honitsu => Some(2),
            Yaku::Junchan => Some(2),
            Yaku::Chinitsu => Some(5),
            
            // These keep the same han when open
            Yaku::Tanyao => Some(1),
            Yaku::Yakuhai(_) => Some(1),
            Yaku::RinshanKaihou => Some(1),
            Yaku::HaiteiRaoyue => Some(1),
            Yaku::HouteiRaoyui => Some(1),
            Yaku::Toitoi => Some(2),
            Yaku::Chiitoitsu => Some(2), // Can't actually be open, but logically same
            Yaku::SanAnkou => Some(2),
        }
    }
    
    /// Check if this yaku is valid when the hand is open
    pub fn valid_when_open(&self) -> bool {
        self.han_open().is_some()
    }
    
    /// Check if this is a yakuman (limit hand)
    pub fn is_yakuman(&self) -> bool {
        matches!(self, Yaku::Tenhou | Yaku::Chiihou)
    }
}

/// Result of yaku detection
#[derive(Debug, Clone)]
pub struct YakuResult {
    pub yaku_list: Vec<Yaku>,
    pub total_han: u8,
    pub dora_count: u8,
    pub is_yakuman: bool,
}

impl YakuResult {
    /// Total han including dora (for non-yakuman hands)
    pub fn total_han_with_dora(&self) -> u8 {
        if self.is_yakuman {
            // Yakuman don't add dora (already at limit)
            self.total_han
        } else {
            self.total_han + self.dora_count
        }
    }
}

/// Detect yaku with full game context
pub fn detect_yaku_with_context(
    structure: &HandStructure,
    counts: &TileCounts,
    context: &GameContext,
) -> YakuResult {
    let mut yaku_list = Vec::new();
    let is_open = context.is_open;
    
    // === Yakuman checks first (these override everything) ===
    
    // Tenhou: Dealer wins on initial deal (must be tsumo, closed, dealer)
    if context.is_tenhou && context.win_type == WinType::Tsumo && !is_open && context.is_dealer() {
        yaku_list.push(Yaku::Tenhou);
    }
    
    // Chiihou: Non-dealer wins on first self-draw (must be tsumo, closed, not dealer)
    if context.is_chiihou && context.win_type == WinType::Tsumo && !is_open && !context.is_dealer() {
        yaku_list.push(Yaku::Chiihou);
    }
    
    // If we have yakuman, skip regular yaku detection
    let has_yakuman = yaku_list.iter().any(|y| y.is_yakuman());
    
    if !has_yakuman {
        // === Context-based yaku (riichi, tsumo, situational, etc.) ===
        
        // Riichi / Double Riichi
        if context.is_riichi && !is_open {
            if context.is_double_riichi {
                yaku_list.push(Yaku::DoubleRiichi);
            } else {
                yaku_list.push(Yaku::Riichi);
            }
            
            // Ippatsu (only with riichi)
            if context.is_ippatsu {
                yaku_list.push(Yaku::Ippatsu);
            }
        }
        
        // Menzen Tsumo (closed hand + self-draw)
        if context.win_type == WinType::Tsumo && !is_open {
            yaku_list.push(Yaku::MenzenTsumo);
        }
        
        // Rinshan Kaihou (win on kan replacement tile - must be tsumo)
        if context.is_rinshan && context.win_type == WinType::Tsumo {
            yaku_list.push(Yaku::RinshanKaihou);
        }
        
        // Haitei Raoyue (tsumo on last drawable tile)
        if context.is_last_tile && context.win_type == WinType::Tsumo {
            yaku_list.push(Yaku::HaiteiRaoyue);
        }
        
        // Houtei Raoyui (ron on last discard)
        if context.is_last_tile && context.win_type == WinType::Ron {
            yaku_list.push(Yaku::HouteiRaoyui);
        }
        
        // === Structure-based yaku ===
        
        match structure {
            HandStructure::Chiitoitsu { pairs } => {
                yaku_list.push(Yaku::Chiitoitsu);
                
                if pairs.iter().all(|t| t.is_simple()) {
                    yaku_list.push(Yaku::Tanyao);
                }
                
                if let Some(flush) = check_flush_tiles(pairs) {
                    yaku_list.push(flush);
                }
            }
            
            HandStructure::Standard { melds, pair } => {
                let all_tiles = collect_all_tiles(melds, *pair);
                
                // === 1 han yaku ===
                
                // Tanyao
                if all_tiles.iter().all(|t| t.is_simple()) {
                    yaku_list.push(Yaku::Tanyao);
                }
                
                // Iipeikou / Ryanpeikou (closed only)
                if !is_open {
                    if let Some(peikou) = check_peikou(melds) {
                        yaku_list.push(peikou);
                    }
                }
                
                // Yakuhai (dragons and value winds)
                for yaku in check_yakuhai(melds, context) {
                    yaku_list.push(yaku);
                }
                
                // === 2 han yaku ===
                
                // Toitoi
                if melds.iter().all(|m| matches!(m, Meld::Koutsu(_))) {
                    yaku_list.push(Yaku::Toitoi);
                }
                
                // Sanshoku doujun
                if check_sanshoku(melds) {
                    yaku_list.push(Yaku::SanshokuDoujun);
                }
                
                // Ittsu
                if check_ittsu(melds) {
                    yaku_list.push(Yaku::Ittsu);
                }
                
                // Chanta (but not junchan)
                if check_chanta(melds, *pair) && !check_junchan(melds, *pair) {
                    yaku_list.push(Yaku::Chanta);
                }
                
                // San Ankou (three concealed triplets) - simplified: all triplets when closed + tsumo
                if !is_open && context.win_type == WinType::Tsumo {
                    let triplet_count = melds.iter()
                        .filter(|m| matches!(m, Meld::Koutsu(_)))
                        .count();
                    if triplet_count >= 3 {
                        yaku_list.push(Yaku::SanAnkou);
                    }
                }
                
                // === 3 han yaku ===
                
                // Junchan
                if check_junchan(melds, *pair) {
                    yaku_list.push(Yaku::Junchan);
                }
                
                // Honitsu / Chinitsu
                if let Some(flush) = check_flush_tiles(&all_tiles) {
                    yaku_list.push(flush);
                }
            }
        }
    }
    
    // Check for yakuman in final list
    let is_yakuman = yaku_list.iter().any(|y| y.is_yakuman());
    
    // Filter out invalid yaku for open hands and calculate han
    let total_han: u8 = if is_open {
        yaku_list.retain(|y| y.valid_when_open());
        yaku_list.iter()
            .filter_map(|y| y.han_open())
            .sum()
    } else {
        yaku_list.iter().map(|y| y.han()).sum()
    };
    
    // Count dora
    let dora_count = count_dora(counts, context);
    
    YakuResult { yaku_list, total_han, dora_count, is_yakuman }
}

/// Detect yaku without game context (backwards compatibility)
pub fn detect_yaku(structure: &HandStructure) -> YakuResult {
    // Create a minimal context
    let dummy_context = GameContext::new(WinType::Ron, Honor::East, Honor::East);
    let empty_counts = TileCounts::new();
    detect_yaku_with_context(structure, &empty_counts, &dummy_context)
}

// ============ Helper Functions ============

/// Collect all tiles from melds and pair
fn collect_all_tiles(melds: &[Meld], pair: Tile) -> Vec<Tile> {
    let mut tiles = vec![pair, pair];
    
    for meld in melds {
        match meld {
            Meld::Koutsu(t) => {
                tiles.push(*t);
                tiles.push(*t);
                tiles.push(*t);
            }
            Meld::Shuntsu(t) => {
                tiles.push(*t);
                if let Tile::Suited { suit, value } = t {
                    tiles.push(Tile::suited(*suit, value + 1));
                    tiles.push(Tile::suited(*suit, value + 2));
                }
            }
        }
    }
    
    tiles
}

/// Check for iipeikou (2 identical sequences) or ryanpeikou (2 pairs of identical sequences)
fn check_peikou(melds: &[Meld]) -> Option<Yaku> {
    let sequences: Vec<_> = melds.iter()
        .filter_map(|m| match m {
            Meld::Shuntsu(t) => Some(*t),
            _ => None,
        })
        .collect();
    
    if sequences.len() < 2 {
        return None;
    }
    
    // Count occurrences of each sequence
    let mut seq_counts: HashMap<Tile, u8> = HashMap::new();
    for t in &sequences {
        *seq_counts.entry(*t).or_insert(0) += 1;
    }
    
    let pairs_of_sequences = seq_counts.values().filter(|&&c| c >= 2).count();
    
    if pairs_of_sequences >= 2 {
        Some(Yaku::Ryanpeikou)
    } else if pairs_of_sequences == 1 {
        Some(Yaku::Iipeikou)
    } else {
        None
    }
}

/// Check for yakuhai (dragons always, winds if they're value winds)
fn check_yakuhai(melds: &[Meld], context: &GameContext) -> Vec<Yaku> {
    let mut result = Vec::new();
    
    for meld in melds {
        if let Meld::Koutsu(Tile::Honor(honor)) = meld {
            match honor {
                // Dragons are always yakuhai
                Honor::White | Honor::Green | Honor::Red => {
                    result.push(Yaku::Yakuhai(*honor));
                }
                // Winds are yakuhai if they match round or seat wind
                // Double wind (both round and seat) = 2 yakuhai
                Honor::East | Honor::South | Honor::West | Honor::North => {
                    if *honor == context.round_wind {
                        result.push(Yaku::Yakuhai(*honor));
                    }
                    if *honor == context.seat_wind {
                        result.push(Yaku::Yakuhai(*honor));
                    }
                }
            }
        }
    }
    
    result
}

/// Check for sanshoku doujun (same sequence in all 3 suits)
fn check_sanshoku(melds: &[Meld]) -> bool {
    let sequences: Vec<(Suit, u8)> = melds.iter()
        .filter_map(|m| match m {
            Meld::Shuntsu(Tile::Suited { suit, value }) => Some((*suit, *value)),
            _ => None,
        })
        .collect();
    
    // Group by starting value
    let mut by_value: HashMap<u8, Vec<Suit>> = HashMap::new();
    for (suit, value) in sequences {
        by_value.entry(value).or_default().push(suit);
    }
    
    // Check if any value has all 3 suits
    for suits in by_value.values() {
        let has_man = suits.contains(&Suit::Man);
        let has_pin = suits.contains(&Suit::Pin);
        let has_sou = suits.contains(&Suit::Sou);
        if has_man && has_pin && has_sou {
            return true;
        }
    }
    
    false
}

/// Check for ittsu (1-9 straight in one suit)
fn check_ittsu(melds: &[Meld]) -> bool {
    let sequences: Vec<(Suit, u8)> = melds.iter()
        .filter_map(|m| match m {
            Meld::Shuntsu(Tile::Suited { suit, value }) => Some((*suit, *value)),
            _ => None,
        })
        .collect();
    
    // Group by suit
    let mut by_suit: HashMap<Suit, Vec<u8>> = HashMap::new();
    for (suit, value) in sequences {
        by_suit.entry(suit).or_default().push(value);
    }
    
    // Check if any suit has 1, 4, 7 (representing 123, 456, 789)
    for values in by_suit.values() {
        if values.contains(&1) && values.contains(&4) && values.contains(&7) {
            return true;
        }
    }
    
    false
}

/// Check for chanta (all groups contain terminal or honor)
fn check_chanta(melds: &[Meld], pair: Tile) -> bool {
    // Pair must be terminal or honor
    if !pair.is_terminal_or_honor() {
        return false;
    }
    
    // All melds must contain terminal or honor
    for meld in melds {
        let has_terminal_or_honor = match meld {
            Meld::Koutsu(t) => t.is_terminal_or_honor(),
            Meld::Shuntsu(Tile::Suited { value, .. }) => {
                // Sequence 123 or 789 contains terminal
                *value == 1 || *value == 7
            }
            Meld::Shuntsu(Tile::Honor(_)) => true, // Can't happen, but handle it
        };
        
        if !has_terminal_or_honor {
            return false;
        }
    }
    
    true
}

/// Check for junchan (all groups contain terminal, no honors)
fn check_junchan(melds: &[Meld], pair: Tile) -> bool {
    // Pair must be terminal (1 or 9), not honor
    let pair_ok = match pair {
        Tile::Suited { value, .. } => value == 1 || value == 9,
        Tile::Honor(_) => false,
    };
    
    if !pair_ok {
        return false;
    }
    
    // All melds must contain terminal (not honor)
    for meld in melds {
        let has_terminal = match meld {
            Meld::Koutsu(Tile::Suited { value, .. }) => *value == 1 || *value == 9,
            Meld::Koutsu(Tile::Honor(_)) => false,
            Meld::Shuntsu(Tile::Suited { value, .. }) => *value == 1 || *value == 7,
            Meld::Shuntsu(Tile::Honor(_)) => false,
        };
        
        if !has_terminal {
            return false;
        }
    }
    
    true
}

/// Check for honitsu (one suit + honors) or chinitsu (one suit only)
fn check_flush_tiles(tiles: &[Tile]) -> Option<Yaku> {
    let mut found_suit: Option<Suit> = None;
    let mut has_honors = false;
    
    for tile in tiles {
        match tile {
            Tile::Suited { suit, .. } => {
                match found_suit {
                    None => found_suit = Some(*suit),
                    Some(s) if s != *suit => return None, // Multiple suits = no flush
                    _ => {}
                }
            }
            Tile::Honor(_) => has_honors = true,
        }
    }
    
    // Must have at least one suited tile for flush
    if found_suit.is_none() {
        return None;
    }
    
    if has_honors {
        Some(Yaku::Honitsu)
    } else {
        Some(Yaku::Chinitsu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_hand, parse_hand_with_aka, to_counts};
    use crate::hand::decompose_hand;
    use crate::context::{GameContext, WinType};

    /// Helper to get yaku for a hand string (no context)
    fn get_yaku(hand: &str) -> Vec<YakuResult> {
        let tiles = parse_hand(hand).unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);
        structures.iter().map(|s| detect_yaku(s)).collect()
    }

    /// Helper to get yaku with context
    fn get_yaku_with_context(hand: &str, context: &GameContext) -> Vec<YakuResult> {
        let tiles = parse_hand(hand).unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);
        structures.iter()
            .map(|s| detect_yaku_with_context(s, &counts, context))
            .collect()
    }

    /// Helper to check if any decomposition has a specific yaku
    fn has_yaku(results: &[YakuResult], yaku: Yaku) -> bool {
        results.iter().any(|r| r.yaku_list.contains(&yaku))
    }

    // ===== Basic Yaku Tests (no context) =====

    #[test]
    fn test_tanyao() {
        let results = get_yaku("234m345p456567s88p");
        assert!(has_yaku(&results, Yaku::Tanyao));
    }

    #[test]
    fn test_no_tanyao_with_terminals() {
        let results = get_yaku("123m456p789s11122z");
        assert!(!has_yaku(&results, Yaku::Tanyao));
    }

    #[test]
    fn test_toitoi() {
        let results = get_yaku("111m222p333s44455z");
        assert!(has_yaku(&results, Yaku::Toitoi));
    }

    #[test]
    fn test_iipeikou() {
        let results = get_yaku("112233m456p789s55z");
        assert!(has_yaku(&results, Yaku::Iipeikou));
    }

    #[test]
    fn test_ryanpeikou() {
        let results = get_yaku("112233m112233p55s");
        assert!(has_yaku(&results, Yaku::Ryanpeikou));
        assert!(!has_yaku(&results, Yaku::Iipeikou));
    }

    #[test]
    fn test_yakuhai_dragon() {
        let results = get_yaku("123m456p789s55566z");
        assert!(has_yaku(&results, Yaku::Yakuhai(Honor::White)));
    }

    #[test]
    fn test_sanshoku() {
        let results = get_yaku("123m123p123s11122z");
        assert!(has_yaku(&results, Yaku::SanshokuDoujun));
    }

    #[test]
    fn test_ittsu() {
        let results = get_yaku("123456789m111p22z");
        assert!(has_yaku(&results, Yaku::Ittsu));
    }

    #[test]
    fn test_chiitoitsu() {
        let results = get_yaku("1122m3344p5566s77z");
        assert!(has_yaku(&results, Yaku::Chiitoitsu));
    }

    #[test]
    fn test_honitsu() {
        let results = get_yaku("123456789m11177z");
        assert!(has_yaku(&results, Yaku::Honitsu));
        assert!(!has_yaku(&results, Yaku::Chinitsu));
    }

    #[test]
    fn test_chinitsu() {
        let results = get_yaku("11123456789999m");
        assert!(has_yaku(&results, Yaku::Chinitsu));
        assert!(!has_yaku(&results, Yaku::Honitsu));
    }

    #[test]
    fn test_chanta() {
        let results = get_yaku("123m789p999s11177z");
        assert!(has_yaku(&results, Yaku::Chanta));
    }

    #[test]
    fn test_junchan() {
        let results = get_yaku("123789m111p99999s");
        assert!(has_yaku(&results, Yaku::Junchan));
        assert!(!has_yaku(&results, Yaku::Chanta));
    }

    #[test]
    fn test_multiple_yaku() {
        let results = get_yaku("223344m567p678s55p");
        assert!(has_yaku(&results, Yaku::Tanyao));
        assert!(has_yaku(&results, Yaku::Iipeikou));
        
        let best = results.iter().max_by_key(|r| r.total_han).unwrap();
        assert!(best.total_han >= 2);
    }

    #[test]
    fn test_complex_hand_best_interpretation() {
        let results = get_yaku("111222333m11155z");
        assert!(has_yaku(&results, Yaku::Toitoi));
    }

    // ===== Context-Aware Yaku Tests =====

    #[test]
    fn test_riichi() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .riichi();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Riichi));
    }

    #[test]
    fn test_double_riichi() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .double_riichi();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::DoubleRiichi));
        assert!(!has_yaku(&results, Yaku::Riichi));
    }

    #[test]
    fn test_ippatsu() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .riichi()
            .ippatsu();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Riichi));
        assert!(has_yaku(&results, Yaku::Ippatsu));
    }

    #[test]
    fn test_menzen_tsumo() {
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East);
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::MenzenTsumo));
    }

    #[test]
    fn test_no_menzen_tsumo_when_open() {
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .open();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::MenzenTsumo));
    }

    #[test]
    fn test_wind_yakuhai_round_wind() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South);
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Yakuhai(Honor::East)));
    }

    #[test]
    fn test_wind_yakuhai_seat_wind() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South);
        let results = get_yaku_with_context("123m456p789s22233z", &context);
        assert!(has_yaku(&results, Yaku::Yakuhai(Honor::South)));
    }

    #[test]
    fn test_double_wind_yakuhai() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East);
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        let east_yakuhai_count = results.iter()
            .flat_map(|r| &r.yaku_list)
            .filter(|y| **y == Yaku::Yakuhai(Honor::East))
            .count();
        assert!(east_yakuhai_count >= 2);
    }

    #[test]
    fn test_non_value_wind_no_yakuhai() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South);
        let results = get_yaku_with_context("123m456p789s33344z", &context);
        assert!(!has_yaku(&results, Yaku::Yakuhai(Honor::West)));
    }

    #[test]
    fn test_no_iipeikou_when_open() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .open();
        let results = get_yaku_with_context("112233m456p789s55z", &context);
        assert!(!has_yaku(&results, Yaku::Iipeikou));
    }

    #[test]
    fn test_open_hand_han_reduction() {
        let context_closed = GameContext::new(WinType::Ron, Honor::East, Honor::East);
        let context_open = GameContext::new(WinType::Ron, Honor::East, Honor::East).open();
        
        let results_closed = get_yaku_with_context("123456789m11177z", &context_closed);
        let results_open = get_yaku_with_context("123456789m11177z", &context_open);
        
        let closed_han = results_closed.iter().map(|r| r.total_han).max().unwrap_or(0);
        let open_han = results_open.iter().map(|r| r.total_han).max().unwrap_or(0);
        
        assert!(closed_han > open_han, "Closed should have more han than open");
    }

    // ===== Dora Tests =====

    #[test]
    fn test_dora_counting() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_dora(vec![Tile::suited(Suit::Man, 1)]);
        
        let tiles = parse_hand("222m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);
        let result = detect_yaku_with_context(&structures[0], &counts, &context);
        
        assert_eq!(result.dora_count, 3);
    }

    #[test]
    fn test_akadora_counting() {
        let parsed = parse_hand_with_aka("123m406p789s11122z").unwrap();
        let counts = to_counts(&parsed.tiles);
        let structures = decompose_hand(&counts);
        
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_aka(parsed.aka_count);
        
        let result = detect_yaku_with_context(&structures[0], &counts, &context);
        assert_eq!(result.dora_count, 1);
    }

    #[test]
    fn test_total_han_with_dora() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_dora(vec![Tile::suited(Suit::Man, 1)]);
        
        let tiles = parse_hand("222m345p456567s88p").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);
        let result = detect_yaku_with_context(&structures[0], &counts, &context);
        
        assert!(has_yaku(&[result.clone()], Yaku::Tanyao));
        assert_eq!(result.dora_count, 3);
        assert_eq!(result.total_han_with_dora(), result.total_han + 3);
    }

    // ===== Situational Yaku Tests =====

    #[test]
    fn test_rinshan_kaihou() {
        // Win on kan replacement tile (must be tsumo)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .rinshan();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::RinshanKaihou));
        assert!(has_yaku(&results, Yaku::MenzenTsumo)); // Also gets menzen tsumo
    }

    #[test]
    fn test_rinshan_requires_tsumo() {
        // Rinshan with ron should not count (impossible in real game, but test logic)
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .rinshan();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::RinshanKaihou));
    }

    #[test]
    fn test_haitei_raoyue() {
        // Tsumo on last drawable tile
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .last_tile();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::HaiteiRaoyue));
        assert!(!has_yaku(&results, Yaku::HouteiRaoyui));
    }

    #[test]
    fn test_houtei_raoyui() {
        // Ron on last discard
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .last_tile();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::HouteiRaoyui));
        assert!(!has_yaku(&results, Yaku::HaiteiRaoyue));
    }

    #[test]
    fn test_tenhou() {
        // Dealer wins on initial deal
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .tenhou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Tenhou));
        assert!(results[0].is_yakuman);
        assert_eq!(results[0].total_han, 13); // Yakuman = 13 han
    }

    #[test]
    fn test_tenhou_requires_dealer() {
        // Non-dealer cannot get tenhou
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .tenhou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Tenhou));
    }

    #[test]
    fn test_tenhou_requires_tsumo() {
        // Tenhou must be tsumo
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .tenhou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Tenhou));
    }

    #[test]
    fn test_chiihou() {
        // Non-dealer wins on first self-draw
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .chiihou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Chiihou));
        assert!(results[0].is_yakuman);
    }

    #[test]
    fn test_chiihou_not_for_dealer() {
        // Dealer cannot get chiihou (they get tenhou instead)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .chiihou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Chiihou));
    }

    #[test]
    fn test_yakuman_overrides_regular_yaku() {
        // With tenhou, should only have yakuman, no regular yaku
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .tenhou()
            .riichi();  // Even with riichi set, yakuman overrides
        let results = get_yaku_with_context("234m345p456567s88p", &context); // This would be tanyao normally
        
        assert!(has_yaku(&results, Yaku::Tenhou));
        assert!(!has_yaku(&results, Yaku::Tanyao)); // No tanyao with yakuman
        assert!(!has_yaku(&results, Yaku::Riichi)); // No riichi with yakuman
        assert!(!has_yaku(&results, Yaku::MenzenTsumo)); // No menzen tsumo with yakuman
    }

    #[test]
    fn test_yakuman_dora_ignored() {
        // Yakuman don't count dora (already at limit)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .tenhou()
            .with_dora(vec![Tile::suited(Suit::Man, 1)]); // 2m would be dora
        
        let tiles = parse_hand("222m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);
        let result = detect_yaku_with_context(&structures[0], &counts, &context);
        
        assert!(result.is_yakuman);
        assert_eq!(result.total_han, 13);
        assert_eq!(result.total_han_with_dora(), 13); // Still 13, dora not added
    }
}
