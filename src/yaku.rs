//! Yaku (scoring pattern) detection for Riichi Mahjong hands.

use crate::context::{GameContext, WinType, count_dora_detailed};
use crate::hand::{HandStructure, Meld};
use crate::parse::TileCounts;
use crate::tile::{Honor, Suit, Tile};
use crate::wait::is_pinfu;
use std::collections::HashMap;

/// Represents a scoring pattern (yaku)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Yaku {
    // === 1 han ===
    Riichi,         // Declared riichi (closed only)
    Ippatsu,        // Win within 1 turn of riichi
    MenzenTsumo,    // Self-draw with closed hand
    Tanyao,         // All simples (no terminals or honors)
    Pinfu,          // All sequences, valueless pair, two-sided wait
    Iipeikou,       // Two identical sequences
    Yakuhai(Honor), // Triplet of dragons or value winds
    RinshanKaihou,  // Win on kan replacement tile
    Chankan,        // Ron on another player's added kan
    HaiteiRaoyue,   // Tsumo on last drawable tile
    HouteiRaoyui,   // Ron on last discard

    // === 2 han ===
    DoubleRiichi,   // Riichi on first turn
    Toitoi,         // All triplets
    SanshokuDoujun, // Same sequence in all 3 suits
    SanshokuDoukou, // Same triplet in all 3 suits
    Ittsu,          // 1-9 straight in one suit
    Chiitoitsu,     // Seven pairs
    Chanta,         // All groups contain terminal or honor
    SanAnkou,       // Three concealed triplets
    SanKantsu,      // Three kans (quads)
    Honroutou,      // All terminals and honors
    Shousangen,     // Small three dragons (2 dragon triplets + dragon pair)

    // === 3 han ===
    Honitsu,    // Half flush (one suit + honors)
    Junchan,    // All groups contain terminal (no honors)
    Ryanpeikou, // Two pairs of identical sequences

    // === 6 han ===
    Chinitsu, // Full flush (one suit only)

    // === Yakuman (limit hands) ===
    Tenhou,        // Dealer wins on initial deal
    Chiihou,       // Non-dealer wins on first draw
    KokushiMusou,  // Thirteen orphans
    Suuankou,      // Four concealed triplets
    Daisangen,     // Big three dragons
    Shousuushii,   // Little four winds
    Daisuushii,    // Big four winds
    Tsuuiisou,     // All honors
    Chinroutou,    // All terminals
    Ryuuiisou,     // All green
    ChuurenPoutou, // Nine gates

    // === Double Yakuman ===
    Kokushi13Wait,       // Kokushi with 13-sided wait
    SuuankouTanki,       // Suuankou with tanki wait
    JunseiChuurenPoutou, // Pure nine gates (9-sided wait)
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
            Yaku::Chankan => 1,
            Yaku::HaiteiRaoyue => 1,
            Yaku::HouteiRaoyui => 1,

            // 2 han
            Yaku::DoubleRiichi => 2,
            Yaku::Toitoi => 2,
            Yaku::SanshokuDoujun => 2,
            Yaku::SanshokuDoukou => 2,
            Yaku::Ittsu => 2,
            Yaku::Chiitoitsu => 2,
            Yaku::Chanta => 2,
            Yaku::SanAnkou => 2,
            Yaku::SanKantsu => 2,
            Yaku::Honroutou => 2,
            Yaku::Shousangen => 2,

            // 3 han
            Yaku::Honitsu => 3,
            Yaku::Junchan => 3,
            Yaku::Ryanpeikou => 3,

            // 6 han
            Yaku::Chinitsu => 6,

            // Yakuman (13 han equivalent)
            Yaku::Tenhou => 13,
            Yaku::Chiihou => 13,
            Yaku::KokushiMusou => 13,
            Yaku::Suuankou => 13,
            Yaku::Daisangen => 13,
            Yaku::Shousuushii => 13,
            Yaku::Daisuushii => 13,
            Yaku::Tsuuiisou => 13,
            Yaku::Chinroutou => 13,
            Yaku::Ryuuiisou => 13,
            Yaku::ChuurenPoutou => 13,

            // Double Yakuman (26 han equivalent)
            Yaku::Kokushi13Wait => 26,
            Yaku::SuuankouTanki => 26,
            Yaku::JunseiChuurenPoutou => 26,
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
            Yaku::Tenhou => None,
            Yaku::Chiihou => None,
            Yaku::Suuankou => None,
            Yaku::SuuankouTanki => None,
            Yaku::ChuurenPoutou => None,
            Yaku::JunseiChuurenPoutou => None,
            Yaku::KokushiMusou => None,
            Yaku::Kokushi13Wait => None,

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
            Yaku::Chankan => Some(1),
            Yaku::HaiteiRaoyue => Some(1),
            Yaku::HouteiRaoyui => Some(1),
            Yaku::Toitoi => Some(2),
            Yaku::SanshokuDoukou => Some(2),
            Yaku::Chiitoitsu => Some(2), // Can't actually be open, but logically same
            Yaku::SanAnkou => Some(2),
            Yaku::SanKantsu => Some(2),
            Yaku::Honroutou => Some(2),
            Yaku::Shousangen => Some(2),

            // Yakuman that can be open
            Yaku::Daisangen => Some(13),
            Yaku::Shousuushii => Some(13),
            Yaku::Daisuushii => Some(13),
            Yaku::Tsuuiisou => Some(13),
            Yaku::Chinroutou => Some(13),
            Yaku::Ryuuiisou => Some(13),
        }
    }

    /// Check if this yaku is valid when the hand is open
    pub fn valid_when_open(&self) -> bool {
        self.han_open().is_some()
    }

    /// Check if this is a yakuman (limit hand)
    pub fn is_yakuman(&self) -> bool {
        matches!(
            self,
            Yaku::Tenhou
                | Yaku::Chiihou
                | Yaku::KokushiMusou
                | Yaku::Kokushi13Wait
                | Yaku::Suuankou
                | Yaku::SuuankouTanki
                | Yaku::Daisangen
                | Yaku::Shousuushii
                | Yaku::Daisuushii
                | Yaku::Tsuuiisou
                | Yaku::Chinroutou
                | Yaku::Ryuuiisou
                | Yaku::ChuurenPoutou
                | Yaku::JunseiChuurenPoutou
        )
    }
}

/// Result of yaku detection
#[derive(Debug, Clone)]
pub struct YakuResult {
    pub yaku_list: Vec<Yaku>,
    pub total_han: u8,
    pub dora_count: u8,
    /// Breakdown of dora by type (for display purposes)
    pub regular_dora: u8,
    pub ura_dora: u8,
    pub aka_dora: u8,
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
    if context.is_chiihou && context.win_type == WinType::Tsumo && !is_open && !context.is_dealer()
    {
        yaku_list.push(Yaku::Chiihou);
    }

    // === Structure-based Yakuman ===
    match structure {
        HandStructure::Kokushi { pair } => {
            if let Some(winning_tile) = context.winning_tile {
                if winning_tile == *pair {
                    // 13-wait (junsei kokushi): player had all 13 different terminals/honors,
                    // waiting on any of them. The winning tile becomes the pair.
                    yaku_list.push(Yaku::Kokushi13Wait);
                } else {
                    // Regular kokushi: player had a pair already, waiting for the missing tile
                    yaku_list.push(Yaku::KokushiMusou);
                }
            } else {
                yaku_list.push(Yaku::KokushiMusou);
            }
        }

        HandStructure::Chiitoitsu { pairs } => {
            // Tsuuiisou (all honors) in chiitoitsu
            if pairs.iter().all(|t| t.is_honor()) {
                yaku_list.push(Yaku::Tsuuiisou);
            }
            // Chinroutou (all terminals) in chiitoitsu
            else if pairs.iter().all(|t| t.is_terminal()) {
                yaku_list.push(Yaku::Chinroutou);
            }
        }

        HandStructure::Standard { melds, pair } => {
            // Suuankou (Four Concealed Triplets)
            if let Some(yaku) = check_suuankou(melds, *pair, context) {
                yaku_list.push(yaku);
            }

            // Daisangen (Big Three Dragons)
            if check_daisangen(melds) {
                yaku_list.push(Yaku::Daisangen);
            }

            // Shousuushii / Daisuushii (Four Winds)
            if let Some(yaku) = check_four_winds(melds, *pair) {
                yaku_list.push(yaku);
            }

            // Tsuuiisou (All Honors)
            if check_tsuuiisou(melds, *pair) {
                yaku_list.push(Yaku::Tsuuiisou);
            }

            // Chinroutou (All Terminals)
            if check_chinroutou(melds, *pair) {
                yaku_list.push(Yaku::Chinroutou);
            }

            // Ryuuiisou (All Green)
            if check_ryuuiisou(melds, *pair) {
                yaku_list.push(Yaku::Ryuuiisou);
            }

            // Chuuren Poutou (Nine Gates) - closed only
            if !is_open {
                if let Some(yaku) = check_chuuren_poutou(counts, context) {
                    yaku_list.push(yaku);
                }
            }
        }
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

        // Chankan (ron on another player's added kan - must be ron)
        if context.is_chankan && context.win_type == WinType::Ron {
            yaku_list.push(Yaku::Chankan);
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
            HandStructure::Kokushi { .. } => {
                // Kokushi is yakuman, handled above
            }

            HandStructure::Chiitoitsu { pairs } => {
                yaku_list.push(Yaku::Chiitoitsu);

                if pairs.iter().all(|t| t.is_simple()) {
                    yaku_list.push(Yaku::Tanyao);
                }

                // Honroutou in chiitoitsu
                if pairs.iter().all(|t| t.is_terminal_or_honor()) {
                    yaku_list.push(Yaku::Honroutou);
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

                // Pinfu (requires winning tile to be set)
                if let Some(winning_tile) = context.winning_tile {
                    if is_pinfu(structure, winning_tile, context) {
                        yaku_list.push(Yaku::Pinfu);
                    }
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
                if melds.iter().all(|m| m.is_triplet_or_kan()) {
                    yaku_list.push(Yaku::Toitoi);
                }

                // Sanshoku doujun
                if check_sanshoku(melds) {
                    yaku_list.push(Yaku::SanshokuDoujun);
                }

                // Sanshoku doukou (same triplet in all 3 suits)
                if check_sanshoku_doukou(melds) {
                    yaku_list.push(Yaku::SanshokuDoukou);
                }

                // Ittsu
                if check_ittsu(melds) {
                    yaku_list.push(Yaku::Ittsu);
                }

                // Chanta (but not junchan)
                if check_chanta(melds, *pair) && !check_junchan(melds, *pair) {
                    yaku_list.push(Yaku::Chanta);
                }

                // San Ankou (three concealed triplets)
                // Note: Closed kans count as concealed triplets for san ankou
                {
                    // First, check if the winning tile could complete a sequence in this hand.
                    // If it can, then triplets matching the winning tile remain concealed
                    // (the player could have won on the sequence instead).
                    let winning_tile_completes_sequence = if let Some(wt) = context.winning_tile {
                        melds.iter().any(|m| {
                            if let Meld::Shuntsu(start, _) = m {
                                // Check if winning tile is part of this sequence
                                if let (
                                    Tile::Suited {
                                        suit: ws,
                                        value: wv,
                                    },
                                    Tile::Suited {
                                        suit: ss,
                                        value: sv,
                                    },
                                ) = (wt, start)
                                {
                                    ws == *ss && wv >= *sv && wv <= sv + 2
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        })
                    } else {
                        false
                    };

                    let mut concealed_triplets = 0;
                    for meld in melds {
                        match meld {
                            Meld::Koutsu(tile, is_open_meld) => {
                                // A triplet is concealed if:
                                // 1. It's not an open pon
                                // 2. For ron, the winning tile did NOT complete this triplet,
                                //    OR the winning tile could have completed a sequence instead
                                if !is_open_meld {
                                    if context.win_type == WinType::Tsumo {
                                        concealed_triplets += 1;
                                    } else if let Some(wt) = context.winning_tile {
                                        if *tile != wt || winning_tile_completes_sequence {
                                            concealed_triplets += 1;
                                        }
                                    }
                                }
                            }
                            Meld::Kan(_, kan_type) => {
                                // Closed kans count as concealed triplets
                                if !kan_type.is_open() {
                                    concealed_triplets += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    if concealed_triplets == 3 {
                        yaku_list.push(Yaku::SanAnkou);
                    }
                }

                // San Kantsu (three kans)
                {
                    let kan_count = melds
                        .iter()
                        .filter(|m| matches!(m, Meld::Kan(_, _)))
                        .count();
                    if kan_count == 3 {
                        yaku_list.push(Yaku::SanKantsu);
                    }
                }

                // Honroutou (all terminals and honors)
                if check_honroutou(melds, *pair) {
                    yaku_list.push(Yaku::Honroutou);
                }

                // Shousangen (small three dragons)
                if check_shousangen(melds, *pair) {
                    yaku_list.push(Yaku::Shousangen);
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
        yaku_list.iter().filter_map(|y| y.han_open()).sum()
    } else {
        yaku_list.iter().map(|y| y.han()).sum()
    };

    // Count dora with breakdown
    let dora = count_dora_detailed(counts, context);

    YakuResult {
        yaku_list,
        total_han,
        dora_count: dora.total(),
        regular_dora: dora.regular,
        ura_dora: dora.ura,
        aka_dora: dora.aka,
        is_yakuman,
    }
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
            Meld::Koutsu(t, _) => {
                tiles.push(*t);
                tiles.push(*t);
                tiles.push(*t);
            }
            Meld::Shuntsu(t, _) => {
                tiles.push(*t);
                if let Tile::Suited { suit, value } = t {
                    tiles.push(Tile::suited(*suit, value + 1));
                    tiles.push(Tile::suited(*suit, value + 2));
                }
            }
            Meld::Kan(t, _) => {
                tiles.push(*t);
                tiles.push(*t);
                tiles.push(*t);
                tiles.push(*t);
            }
        }
    }

    tiles
}

/// Check for iipeikou (2 identical sequences) or ryanpeikou (2 pairs of identical sequences)
fn check_peikou(melds: &[Meld]) -> Option<Yaku> {
    let sequences: Vec<_> = melds
        .iter()
        .filter_map(|m| match m {
            Meld::Shuntsu(t, _) => Some(*t),
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
/// Kans also count for yakuhai
fn check_yakuhai(melds: &[Meld], context: &GameContext) -> Vec<Yaku> {
    let mut result = Vec::new();

    for meld in melds {
        // Get the honor tile from triplet or kan
        let honor = match meld {
            Meld::Koutsu(Tile::Honor(h), _) => Some(h),
            Meld::Kan(Tile::Honor(h), _) => Some(h),
            _ => None,
        };

        if let Some(honor) = honor {
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
    let sequences: Vec<(Suit, u8)> = melds
        .iter()
        .filter_map(|m| match m {
            Meld::Shuntsu(Tile::Suited { suit, value }, _) => Some((*suit, *value)),
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
    let sequences: Vec<(Suit, u8)> = melds
        .iter()
        .filter_map(|m| match m {
            Meld::Shuntsu(Tile::Suited { suit, value }, _) => Some((*suit, *value)),
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

    // Chanta requires at least one sequence (otherwise it's honroutou)
    let has_sequence = melds.iter().any(|m| matches!(m, Meld::Shuntsu(_, _)));
    if !has_sequence {
        return false;
    }

    // All melds must contain terminal or honor
    for meld in melds {
        let has_terminal_or_honor = match meld {
            Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_terminal_or_honor(),
            Meld::Shuntsu(Tile::Suited { value, .. }, _) => {
                // Sequence 123 or 789 contains terminal
                *value == 1 || *value == 7
            }
            Meld::Shuntsu(Tile::Honor(_), _) => true, // Can't happen, but handle it
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

    // Junchan requires at least one sequence (otherwise it's chinroutou)
    let has_sequence = melds.iter().any(|m| matches!(m, Meld::Shuntsu(_, _)));
    if !has_sequence {
        return false;
    }

    // All melds must contain terminal (not honor)
    for meld in melds {
        let has_terminal = match meld {
            Meld::Koutsu(Tile::Suited { value, .. }, _)
            | Meld::Kan(Tile::Suited { value, .. }, _) => *value == 1 || *value == 9,
            Meld::Koutsu(Tile::Honor(_), _) | Meld::Kan(Tile::Honor(_), _) => false,
            Meld::Shuntsu(Tile::Suited { value, .. }, _) => *value == 1 || *value == 7,
            Meld::Shuntsu(Tile::Honor(_), _) => false,
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

// ============================================================================
// Yakuman Helper Functions
// ============================================================================

/// Check for Suuankou (Four Concealed Triplets)
/// Closed kans count as concealed triplets for suuankou
fn check_suuankou(melds: &[Meld], _pair: Tile, context: &GameContext) -> Option<Yaku> {
    // For ron, we need to know the winning tile to determine if suuankou is valid
    // If no winning tile is set for ron, we can't determine suuankou
    if context.win_type == WinType::Ron && context.winning_tile.is_none() {
        return None;
    }

    // Count concealed triplets and closed kans (must have 4 total, no sequences)
    let mut concealed_triplet_count = 0;

    for meld in melds {
        match meld {
            Meld::Koutsu(tile, is_open) => {
                if !is_open {
                    // For ron, check if this triplet was completed by the winning tile
                    if context.win_type == WinType::Ron {
                        if let Some(wt) = context.winning_tile {
                            if *tile == wt {
                                // This triplet was completed by ron, not concealed
                                continue;
                            }
                        }
                    }
                    concealed_triplet_count += 1;
                }
            }
            Meld::Kan(_, kan_type) => {
                // Closed kans count as concealed triplets
                if !kan_type.is_open() {
                    concealed_triplet_count += 1;
                }
            }
            Meld::Shuntsu(_, _) => {
                // Has a sequence, can't be suuankou
                return None;
            }
        }
    }

    if concealed_triplet_count != 4 {
        return None;
    }

    // Tenhou treats all Suuankou as single yakuman, regardless of wait type or win method.
    // Some rulesets award double yakuman for Suuankou Tanki (tsumo on pair wait),
    // but we follow Tenhou's rules for compatibility.
    Some(Yaku::Suuankou)
}

/// Check for Daisangen (Big Three Dragons)
/// Kans also count for daisangen
fn check_daisangen(melds: &[Meld]) -> bool {
    let dragon_triplets = melds
        .iter()
        .filter(|m| match m {
            Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_dragon(),
            _ => false,
        })
        .count();

    dragon_triplets == 3
}

/// Check for Shousuushii or Daisuushii
/// Kans also count for four winds
fn check_four_winds(melds: &[Meld], pair: Tile) -> Option<Yaku> {
    let wind_triplets: Vec<Honor> = melds
        .iter()
        .filter_map(|m| {
            let honor = match m {
                Meld::Koutsu(Tile::Honor(h), _) | Meld::Kan(Tile::Honor(h), _) => Some(h),
                _ => None,
            };
            if let Some(honor) = honor {
                if matches!(
                    honor,
                    Honor::East | Honor::South | Honor::West | Honor::North
                ) {
                    return Some(*honor);
                }
            }
            None
        })
        .collect();

    if wind_triplets.len() == 4 {
        return Some(Yaku::Daisuushii);
    }

    if wind_triplets.len() == 3 {
        // Check if pair is the fourth wind
        if let Tile::Honor(honor) = pair {
            if matches!(
                honor,
                Honor::East | Honor::South | Honor::West | Honor::North
            ) {
                if !wind_triplets.contains(&honor) {
                    return Some(Yaku::Shousuushii);
                }
            }
        }
    }

    None
}

/// Check for Tsuuiisou (All Honors)
fn check_tsuuiisou(melds: &[Meld], pair: Tile) -> bool {
    if !pair.is_honor() {
        return false;
    }

    melds.iter().all(|m| match m {
        Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_honor(),
        Meld::Shuntsu(_, _) => false, // Sequences can't be honors
    })
}

/// Check for Chinroutou (All Terminals)
fn check_chinroutou(melds: &[Meld], pair: Tile) -> bool {
    if !pair.is_terminal() {
        return false;
    }

    melds.iter().all(|m| match m {
        Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_terminal(),
        Meld::Shuntsu(_, _) => false, // Sequences can't be all terminals
    })
}

/// Check for Ryuuiisou (All Green)
fn check_ryuuiisou(melds: &[Meld], pair: Tile) -> bool {
    if !pair.is_green() {
        return false;
    }

    melds.iter().all(|m| match m {
        Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_green(),
        Meld::Shuntsu(start, _) => {
            // Only valid green sequence is 234s
            matches!(
                start,
                Tile::Suited {
                    suit: Suit::Sou,
                    value: 2
                }
            )
        }
    })
}

/// Check for Chuuren Poutou (Nine Gates)
fn check_chuuren_poutou(counts: &TileCounts, context: &GameContext) -> Option<Yaku> {
    // Must be closed
    if context.is_open {
        return None;
    }

    // Find the suit (must be single suit, no honors)
    let mut suit: Option<Suit> = None;
    for tile in counts.keys() {
        match tile {
            Tile::Suited { suit: s, .. } => {
                if suit.is_none() {
                    suit = Some(*s);
                } else if suit != Some(*s) {
                    return None; // Multiple suits
                }
            }
            Tile::Honor(_) => return None, // Has honors
        }
    }

    let suit = suit?;

    // Check the pattern: 1112345678999 + any one tile
    // Required counts: 1:3+, 2:1+, 3:1+, 4:1+, 5:1+, 6:1+, 7:1+, 8:1+, 9:3+
    // Total should be 14

    let mut total = 0u8;
    let mut pattern_match = true;
    let mut extra_tile: Option<u8> = None;

    for value in 1..=9 {
        let tile = Tile::suited(suit, value);
        let count = counts.get(&tile).copied().unwrap_or(0);
        total += count;

        let required = if value == 1 || value == 9 { 3 } else { 1 };

        if count < required {
            pattern_match = false;
            break;
        }

        // Track extra tile (the +1 that makes it 14)
        if count > required {
            if extra_tile.is_some() && count > required + 1 {
                pattern_match = false;
                break;
            }
            extra_tile = Some(value);
        }
    }

    if !pattern_match || total != 14 {
        return None;
    }

    // Check for junsei (pure) nine gates - 9-sided wait
    // This happens when winning tile could be any of 1-9
    if let Some(winning_tile) = context.winning_tile {
        if let Tile::Suited {
            value: wv,
            suit: ws,
        } = winning_tile
        {
            if ws == suit {
                // If the extra tile equals winning tile, check if removing it
                // leaves exactly the base pattern
                if extra_tile == Some(wv) {
                    // This is the pure 9-sided wait
                    return Some(Yaku::JunseiChuurenPoutou);
                }
            }
        }
    }

    Some(Yaku::ChuurenPoutou)
}

/// Check for Sanshoku Doukou (same triplet value in all three suits)
/// Kans also count for sanshoku doukou
fn check_sanshoku_doukou(melds: &[Meld]) -> bool {
    let triplets: Vec<(Suit, u8)> = melds
        .iter()
        .filter_map(|m| match m {
            Meld::Koutsu(Tile::Suited { suit, value }, _)
            | Meld::Kan(Tile::Suited { suit, value }, _) => Some((*suit, *value)),
            _ => None,
        })
        .collect();

    // Check if there's a value that appears in all three suits as triplets
    for value in 1..=9 {
        let mut has_man = false;
        let mut has_pin = false;
        let mut has_sou = false;

        for (suit, v) in &triplets {
            if *v == value {
                match suit {
                    Suit::Man => has_man = true,
                    Suit::Pin => has_pin = true,
                    Suit::Sou => has_sou = true,
                }
            }
        }

        if has_man && has_pin && has_sou {
            return true;
        }
    }

    false
}

/// Check for Honroutou (all terminals and honors, no sequences)
fn check_honroutou(melds: &[Meld], pair: Tile) -> bool {
    if !pair.is_terminal_or_honor() {
        return false;
    }

    melds.iter().all(|m| match m {
        Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_terminal_or_honor(),
        Meld::Shuntsu(_, _) => false, // No sequences allowed
    })
}

/// Check for Shousangen (small three dragons)
/// Kans also count for shousangen
fn check_shousangen(melds: &[Meld], pair: Tile) -> bool {
    // Need exactly 2 dragon triplets/kans and dragon pair
    let dragon_triplets = melds
        .iter()
        .filter(|m| match m {
            Meld::Koutsu(t, _) | Meld::Kan(t, _) => t.is_dragon(),
            _ => false,
        })
        .count();

    let pair_is_dragon = pair.is_dragon();

    dragon_triplets == 2 && pair_is_dragon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{GameContext, WinType};
    use crate::hand::decompose_hand;
    use crate::parse::{parse_hand, to_counts};

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
        structures
            .iter()
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
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East).riichi();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Riichi));
    }

    #[test]
    fn test_double_riichi() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East).double_riichi();
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
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East).open();
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
        let east_yakuhai_count = results
            .iter()
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
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East).open();
        let results = get_yaku_with_context("112233m456p789s55z", &context);
        assert!(!has_yaku(&results, Yaku::Iipeikou));
    }

    #[test]
    fn test_open_hand_han_reduction() {
        let context_closed = GameContext::new(WinType::Ron, Honor::East, Honor::East);
        let context_open = GameContext::new(WinType::Ron, Honor::East, Honor::East).open();

        let results_closed = get_yaku_with_context("123456789m11177z", &context_closed);
        let results_open = get_yaku_with_context("123456789m11177z", &context_open);

        let closed_han = results_closed
            .iter()
            .map(|r| r.total_han)
            .max()
            .unwrap_or(0);
        let open_han = results_open.iter().map(|r| r.total_han).max().unwrap_or(0);

        assert!(
            closed_han > open_han,
            "Closed should have more han than open"
        );
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
        let parsed = crate::parse::parse_hand_with_aka("123m406p789s11122z").unwrap();
        let counts = to_counts(&parsed.tiles);
        let structures = decompose_hand(&counts);

        let context =
            GameContext::new(WinType::Ron, Honor::East, Honor::East).with_aka(parsed.aka_count);

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
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East).rinshan();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::RinshanKaihou));
        assert!(has_yaku(&results, Yaku::MenzenTsumo)); // Also gets menzen tsumo
    }

    #[test]
    fn test_rinshan_requires_tsumo() {
        // Rinshan with ron should not count (impossible in real game, but test logic)
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East).rinshan();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::RinshanKaihou));
    }

    #[test]
    fn test_chankan() {
        // Win by robbing another player's added kan (must be ron)
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South).chankan();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Chankan));

        // Verify it's 1 han
        let yaku_result = &results[0];
        let chankan_han = yaku_result
            .yaku_list
            .iter()
            .find(|y| **y == Yaku::Chankan)
            .map(|y| y.han())
            .unwrap_or(0);
        assert_eq!(chankan_han, 1);
    }

    #[test]
    fn test_chankan_requires_ron() {
        // Chankan must be ron (you're robbing someone else's kan)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South).chankan();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Chankan));
    }

    #[test]
    fn test_haitei_raoyue() {
        // Tsumo on last drawable tile
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East).last_tile();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::HaiteiRaoyue));
        assert!(!has_yaku(&results, Yaku::HouteiRaoyui));
    }

    #[test]
    fn test_houtei_raoyui() {
        // Ron on last discard
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East).last_tile();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::HouteiRaoyui));
        assert!(!has_yaku(&results, Yaku::HaiteiRaoyue));
    }

    #[test]
    fn test_tenhou() {
        // Dealer wins on initial deal
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East).tenhou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Tenhou));
        assert!(results[0].is_yakuman);
        assert_eq!(results[0].total_han, 13); // Yakuman = 13 han
    }

    #[test]
    fn test_tenhou_requires_dealer() {
        // Non-dealer cannot get tenhou
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South).tenhou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Tenhou));
    }

    #[test]
    fn test_tenhou_requires_tsumo() {
        // Tenhou must be tsumo
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East).tenhou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Tenhou));
    }

    #[test]
    fn test_chiihou() {
        // Non-dealer wins on first self-draw
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South).chiihou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(has_yaku(&results, Yaku::Chiihou));
        assert!(results[0].is_yakuman);
    }

    #[test]
    fn test_chiihou_not_for_dealer() {
        // Dealer cannot get chiihou (they get tenhou instead)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East).chiihou();
        let results = get_yaku_with_context("123m456p789s11122z", &context);
        assert!(!has_yaku(&results, Yaku::Chiihou));
    }

    #[test]
    fn test_yakuman_overrides_regular_yaku() {
        // With tenhou, should only have yakuman, no regular yaku
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .tenhou()
            .riichi(); // Even with riichi set, yakuman overrides
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

    // ===== Pinfu Tests =====

    #[test]
    fn test_pinfu_basic() {
        // All sequences, non-yakuhai pair, ryanmen wait
        // 123m 456m 789p 234s 55p - won on 4s (from 23s wait - ryanmen)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 4));
        let results = get_yaku_with_context("123456m789p234s55p", &context);

        assert!(has_yaku(&results, Yaku::Pinfu), "Should have pinfu");
        assert!(
            has_yaku(&results, Yaku::MenzenTsumo),
            "Should have menzen tsumo"
        );
    }

    #[test]
    fn test_pinfu_with_tanyao() {
        // Pinfu + Tanyao combination
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 5));
        let results = get_yaku_with_context("234567m234p345s66p", &context);

        assert!(has_yaku(&results, Yaku::Pinfu));
        assert!(has_yaku(&results, Yaku::Tanyao));
    }

    #[test]
    fn test_pinfu_fails_with_triplet() {
        // Has a triplet, can't be pinfu
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::honor(Honor::White));
        let results = get_yaku_with_context("123m456p789s11155z", &context);

        assert!(!has_yaku(&results, Yaku::Pinfu));
    }

    #[test]
    fn test_pinfu_fails_with_dragon_pair() {
        // Dragon pair = yakuhai pair, not pinfu
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 4));
        let results = get_yaku_with_context("123m456m789p234s55z", &context);

        assert!(
            !has_yaku(&results, Yaku::Pinfu),
            "Dragon pair means no pinfu"
        );
    }

    #[test]
    fn test_pinfu_fails_with_value_wind_pair() {
        // Seat wind pair = yakuhai pair, not pinfu
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 4));
        let results = get_yaku_with_context("123m456m789p234s22z", &context);

        assert!(
            !has_yaku(&results, Yaku::Pinfu),
            "Seat wind pair means no pinfu"
        );
    }

    #[test]
    fn test_pinfu_ok_with_non_value_wind_pair() {
        // West wind pair when seat is South, round is East = not yakuhai
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 4));
        let results = get_yaku_with_context("123m456m789p234s33z", &context);

        assert!(
            has_yaku(&results, Yaku::Pinfu),
            "Non-value wind pair allows pinfu"
        );
    }

    #[test]
    fn test_pinfu_fails_with_kanchan_wait() {
        // All sequences, good pair, but kanchan wait (won on middle tile 3s from 24s)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 3));
        let results = get_yaku_with_context("123m456m789p234s55p", &context);

        assert!(
            !has_yaku(&results, Yaku::Pinfu),
            "Kanchan wait means no pinfu"
        );
    }

    #[test]
    fn test_pinfu_fails_when_open() {
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .open()
            .with_winning_tile(Tile::suited(Suit::Sou, 4));
        let results = get_yaku_with_context("123m456m789p234s55p", &context);

        assert!(!has_yaku(&results, Yaku::Pinfu), "Open hand can't be pinfu");
    }

    #[test]
    fn test_pinfu_no_winning_tile_no_pinfu() {
        // Without winning tile set, pinfu cannot be detected
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        // Note: no .with_winning_tile()
        let results = get_yaku_with_context("123m456m789p234s55p", &context);

        assert!(
            !has_yaku(&results, Yaku::Pinfu),
            "No winning tile = no pinfu detection"
        );
    }

    #[test]
    fn test_san_ankou_ron_invalidation() {
        // Hand: 111m 222p 333s 456s 77z
        let hand_str = "111m222p333s456s77z";
        let winning_tile = Tile::suited(Suit::Sou, 3);

        // 1. Tsumo Case - All triplets remain concealed
        let context_tsumo = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_winning_tile(winning_tile);
        let results_tsumo = get_yaku_with_context(hand_str, &context_tsumo);
        assert!(has_yaku(&results_tsumo, Yaku::SanAnkou));

        // 2. Ron Case (Completing triplet) - The 333s triplet is now "open"
        let context_ron = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_winning_tile(winning_tile);
        let results_ron = get_yaku_with_context(hand_str, &context_ron);
        assert!(!has_yaku(&results_ron, Yaku::SanAnkou));

        // 3. Ron Case (Tanki wait on pair) - All three triplets stay concealed
        let pair_tile = Tile::honor(Honor::Red);
        let context_tanki =
            GameContext::new(WinType::Ron, Honor::East, Honor::East).with_winning_tile(pair_tile);
        let results_tanki = get_yaku_with_context(hand_str, &context_tanki);
        assert!(has_yaku(&results_tanki, Yaku::SanAnkou));
    }

    #[test]
    fn test_san_ankou_ron_with_sequence_alternative() {
        // When ron tile could complete EITHER a triplet OR a sequence,
        // the player can declare the sequence win, keeping the triplet concealed.
        //
        // Hand: 111m 222p 333s 345s 77z - ron on 3s
        // The 3s could complete 333s (triplet) OR 345s (sequence).
        // If interpreted as completing the sequence, all three triplets stay concealed.
        let hand_str = "111m222p333345s77z";
        let winning_tile = Tile::suited(Suit::Sou, 3);

        // Ron on 3s - could be shanpon (333s) OR sequence (345s)
        // Since 345s is a valid sequence containing 3s, the 333s triplet stays concealed
        let context_ron = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_winning_tile(winning_tile);
        let results_ron = get_yaku_with_context(hand_str, &context_ron);

        // Sanankou should be awarded because the player can declare the 345s sequence win
        assert!(has_yaku(&results_ron, Yaku::SanAnkou));
    }

    #[test]
    fn test_san_ankou_ron_no_sequence_alternative() {
        // When ron tile can ONLY complete a triplet (no sequence alternative),
        // that triplet is "opened" and doesn't count as concealed.
        //
        // Hand: 111m 222p 333s 678s 77z - ron on 3s
        // The 3s can only complete 333s (triplet), not 678s.
        let hand_str = "111m222p333s678s77z";
        let winning_tile = Tile::suited(Suit::Sou, 3);

        let context_ron = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_winning_tile(winning_tile);
        let results_ron = get_yaku_with_context(hand_str, &context_ron);

        // Sanankou should NOT be awarded - only 2 concealed triplets remain
        assert!(!has_yaku(&results_ron, Yaku::SanAnkou));
    }
}
