//! Fu (minipoints) calculation and final score/payout computation.
//!
//! In Riichi Mahjong, the final score is determined by:
//! 1. Han (doubles) from yaku
//! 2. Fu (minipoints) from hand composition
//! 3. Whether the winner is dealer or not
//! 4. Whether the win was by tsumo or ron

use serde::{Deserialize, Serialize};

use crate::context::{GameContext, WinType};
use crate::hand::{HandStructure, Meld};
use crate::tile::{Honor, Tile};
use crate::wait::{best_wait_type_for_scoring, is_pinfu};
use crate::yaku::YakuResult;

/// Score limit levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScoreLevel {
    /// Below mangan - use fu calculation
    Normal,
    /// 5 han (or 3 han 70+ fu, or 4 han 40+ fu)
    Mangan,
    /// 6-7 han
    Haneman,
    /// 8-10 han
    Baiman,
    /// 11-12 han
    Sanbaiman,
    /// 13+ han (or yakuman)
    Yakuman,
    /// Double yakuman (26+ han or 2 yakuman)
    DoubleYakuman,
}

impl ScoreLevel {
    /// Basic points for this score level (before dealer/tsumo multipliers)
    pub fn basic_points(&self) -> u32 {
        match self {
            ScoreLevel::Normal => 0, // Calculated from fu
            ScoreLevel::Mangan => 2000,
            ScoreLevel::Haneman => 3000,
            ScoreLevel::Baiman => 4000,
            ScoreLevel::Sanbaiman => 6000,
            ScoreLevel::Yakuman => 8000,
            ScoreLevel::DoubleYakuman => 16000,
        }
    }

    /// Display name for this level
    pub fn name(&self) -> &'static str {
        match self {
            ScoreLevel::Normal => "",
            ScoreLevel::Mangan => "Mangan",
            ScoreLevel::Haneman => "Haneman",
            ScoreLevel::Baiman => "Baiman",
            ScoreLevel::Sanbaiman => "Sanbaiman",
            ScoreLevel::Yakuman => "Yakuman",
            ScoreLevel::DoubleYakuman => "Double Yakuman",
        }
    }
}

/// Result of fu calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuResult {
    /// Total fu (rounded up to nearest 10, except 25 for chiitoitsu)
    pub total: u8,
    /// Breakdown of fu components for display
    pub breakdown: FuBreakdown,
}

/// Detailed breakdown of fu components
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FuBreakdown {
    pub base: u8,       // Always 20
    pub menzen_ron: u8, // +10 for closed hand ron
    pub tsumo: u8,      // +2 for tsumo (except pinfu)
    pub melds: u8,      // Fu from triplets/kans
    pub pair: u8,       // Fu from yakuhai pair
    pub wait: u8,       // Fu from wait type
    pub raw_total: u8,  // Sum before rounding
}

/// Payment structure for a winning hand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Total points won
    pub total: u32,
    /// If tsumo: what each non-dealer pays (None for ron)
    pub from_non_dealer: Option<u32>,
    /// If tsumo: what dealer pays (None for ron, or if winner is dealer)
    pub from_dealer: Option<u32>,
    /// If ron: what the discarder pays (None for tsumo)
    pub from_discarder: Option<u32>,
}

/// Complete scoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
    pub fu: FuResult,
    pub han: u8,
    pub score_level: ScoreLevel,
    pub basic_points: u32,
    pub payment: Payment,
    pub is_dealer: bool,
    /// True when yakuman-level score is reached through accumulated han (13+)
    /// rather than through actual yakuman yaku patterns
    pub is_counted_yakuman: bool,
}

// ============================================================================
// Fu Calculation
// ============================================================================

/// Calculate fu for a hand.
///
/// # Arguments
/// * `structure` - The hand structure (decomposition)
/// * `context` - Game context (win type, winds, etc.)
///
/// # Returns
/// FuResult with total fu and breakdown
pub fn calculate_fu(structure: &HandStructure, context: &GameContext) -> FuResult {
    match structure {
        HandStructure::Chiitoitsu { .. } => {
            // Chiitoitsu is always exactly 25 fu (no rounding)
            FuResult {
                total: 25,
                breakdown: FuBreakdown {
                    base: 25,
                    ..Default::default()
                },
            }
        }

        HandStructure::Kokushi { .. } => {
            // Kokushi is yakuman, fu doesn't matter but return 30
            FuResult {
                total: 30,
                breakdown: FuBreakdown {
                    base: 30,
                    ..Default::default()
                },
            }
        }

        HandStructure::Standard { melds, pair } => calculate_standard_fu(melds, *pair, context),
    }
}

/// Calculate fu for a standard hand (4 melds + pair)
fn calculate_standard_fu(melds: &[Meld], pair: Tile, context: &GameContext) -> FuResult {
    let mut breakdown = FuBreakdown {
        base: 20,
        ..Default::default()
    };

    // Check for pinfu + tsumo (special case: exactly 20 fu, no rounding)
    let winning_tile = context.winning_tile;
    let is_pinfu_hand = winning_tile
        .map(|wt| {
            is_pinfu(
                &HandStructure::Standard {
                    melds: melds.to_vec(),
                    pair,
                },
                wt,
                context,
            )
        })
        .unwrap_or(false);

    if is_pinfu_hand && context.win_type == WinType::Tsumo {
        // Pinfu + Tsumo = exactly 20 fu, no additional fu, no rounding
        return FuResult {
            total: 20,
            breakdown: FuBreakdown {
                base: 20,
                ..Default::default()
            },
        };
    }

    // Menzen Ron: +10 fu for closed hand winning by ron
    if !context.is_open && context.win_type == WinType::Ron {
        breakdown.menzen_ron = 10;
    }

    // Tsumo: +2 fu (but NOT for pinfu)
    if context.win_type == WinType::Tsumo && !is_pinfu_hand {
        breakdown.tsumo = 2;
    }

    // Meld fu (accounting for ron-completed triplets)
    // When winning by ron on a TRUE shanpon wait, the triplet completed by the
    // winning tile is treated as "open" for fu purposes, because the final
    // tile came from another player's discard.
    //
    // However, if the winning tile could also complete a sequence (nobetan pattern),
    // then it's NOT a pure shanpon wait, and the triplet remains closed for fu.
    for meld in melds {
        breakdown.melds += meld_fu_with_context(meld, melds, context);
    }

    // Pair fu (yakuhai pairs)
    breakdown.pair = pair_fu(pair, context);

    // Wait fu
    // If Pinfu is awarded, wait must be ryanmen (0 fu) - use that interpretation
    // Otherwise, use the highest fu wait type for maximum scoring
    if let Some(wt) = winning_tile {
        if is_pinfu_hand {
            // Pinfu requires ryanmen, which is 0 fu
            breakdown.wait = 0;
        } else if let Some(wait_type) = best_wait_type_for_scoring(
            &HandStructure::Standard {
                melds: melds.to_vec(),
                pair,
            },
            wt,
        ) {
            breakdown.wait = wait_type.fu();
        }
    }

    // Calculate raw total
    breakdown.raw_total = breakdown.base
        + breakdown.menzen_ron
        + breakdown.tsumo
        + breakdown.melds
        + breakdown.pair
        + breakdown.wait;

    // Round up to nearest 10
    let total = round_up_to_10(breakdown.raw_total);

    // Special case: open hand with no fu beyond base = 30 fu minimum
    // (An open hand with all sequences and no yakuhai pair is still 30 fu)
    let total = if context.is_open && total < 30 {
        30
    } else {
        total
    };

    FuResult { total, breakdown }
}

/// Calculate fu for a single meld, accounting for game context
///
/// When winning by ron on a TRUE shanpon wait, the triplet completed by the
/// winning tile is treated as "open" for fu purposes, because the final
/// tile came from another player's discard.
///
/// However, if the winning tile could also complete a sequence in the hand
/// (nobetan pattern like 11123 waiting on 1 or 4), then it's NOT a pure
/// shanpon wait. In nobetan, the hand has alternative interpretations where
/// the winning tile completes a sequence rather than a triplet, so the
/// triplet should remain "closed" for fu purposes.
///
/// Fu values for triplets (koutsu):
/// - Simple (2-8): 2 open, 4 closed
/// - Terminal/Honor (1,9,honors): 4 open, 8 closed
///
/// Fu values for kans:
/// - Simple (2-8): 8 open, 16 closed
/// - Terminal/Honor (1,9,honors): 16 open, 32 closed
fn meld_fu_with_context(meld: &Meld, all_melds: &[Meld], context: &GameContext) -> u8 {
    match meld {
        Meld::Shuntsu(_, _) => 0, // Sequences give no fu

        Meld::Koutsu(tile, is_meld_open) => {
            let is_terminal_or_honor = tile.is_terminal_or_honor();

            // Base: 2 fu for simple triplet, 4 for terminal/honor triplet
            // Double for closed
            let base = if is_terminal_or_honor { 4 } else { 2 };

            // Check if this triplet was completed by ron
            let is_ron_on_this_tile =
                context.win_type == WinType::Ron && context.winning_tile == Some(*tile);

            // Only treat as "open" if it's a TRUE shanpon wait.
            // If the winning tile also appears in a CLOSED sequence (nobetan pattern),
            // then the triplet remains closed for fu purposes.
            // Open/called sequences don't count - they were already complete before the wait.
            let is_true_shanpon =
                is_ron_on_this_tile && !winning_tile_in_closed_sequence(*tile, all_melds);

            if *is_meld_open || is_true_shanpon {
                base // Open fu
            } else {
                base * 2 // Closed fu
            }
        }

        Meld::Kan(tile, kan_type) => {
            let is_terminal_or_honor = tile.is_terminal_or_honor();

            // Kan fu is 4x the triplet fu
            // Simple: 8 open, 16 closed
            // Terminal/Honor: 16 open, 32 closed
            let base = if is_terminal_or_honor { 16 } else { 8 };

            // Note: Kans cannot be completed by ron (you can't ron a kan),
            // so we only check the kan type
            if kan_type.is_open() {
                base
            } else {
                base * 2
            }
        }
    }
}

/// Check if a tile appears in any CLOSED sequence in the hand.
/// Used to detect nobetan patterns where the winning tile could complete
/// either a triplet or a sequence.
///
/// Only closed sequences count because open/called sequences were already
/// complete before the wait - they don't represent alternative interpretations
/// of the waiting shape.
fn winning_tile_in_closed_sequence(tile: Tile, melds: &[Meld]) -> bool {
    // Honor tiles can never be in sequences
    let (suit, value) = match tile {
        Tile::Suited { suit, value } => (suit, value),
        Tile::Honor(_) => return false,
    };

    for meld in melds {
        // Only check CLOSED sequences (is_open = false)
        if let Meld::Shuntsu(start_tile, is_open) = meld {
            if *is_open {
                continue; // Skip open/called sequences
            }
            if let Tile::Suited {
                suit: seq_suit,
                value: start_val,
            } = start_tile
            {
                // Check if tile is part of this sequence (start, start+1, start+2)
                if *seq_suit == suit && value >= *start_val && value <= start_val + 2 {
                    return true;
                }
            }
        }
    }
    false
}

/// Calculate fu for a single meld (without context, used in tests)
///
/// Fu values for triplets (koutsu):
/// - Simple (2-8): 2 open, 4 closed
/// - Terminal/Honor (1,9,honors): 4 open, 8 closed
///
/// Fu values for kans:
/// - Simple (2-8): 8 open, 16 closed
/// - Terminal/Honor (1,9,honors): 16 open, 32 closed
#[cfg(test)]
fn meld_fu(meld: &Meld) -> u8 {
    match meld {
        Meld::Shuntsu(_, _) => 0, // Sequences give no fu

        Meld::Koutsu(tile, is_meld_open) => {
            let is_terminal_or_honor = tile.is_terminal_or_honor();

            // Base: 2 fu for simple triplet, 4 for terminal/honor triplet
            // Double for closed
            let base = if is_terminal_or_honor { 4 } else { 2 };

            if *is_meld_open {
                base
            } else {
                base * 2
            }
        }

        Meld::Kan(tile, kan_type) => {
            let is_terminal_or_honor = tile.is_terminal_or_honor();

            // Kan fu is 4x the triplet fu
            // Simple: 8 open, 16 closed
            // Terminal/Honor: 16 open, 32 closed
            let base = if is_terminal_or_honor { 16 } else { 8 };

            if kan_type.is_open() {
                base
            } else {
                base * 2
            }
        }
    }
}

/// Calculate fu for the pair
fn pair_fu(pair: Tile, context: &GameContext) -> u8 {
    match pair {
        Tile::Honor(honor) => {
            match honor {
                // Dragons always give 2 fu
                Honor::White | Honor::Green | Honor::Red => 2,

                // Winds give 2 fu if they're value winds
                // Double wind (both round and seat) gives 2 fu (some rules say 4)
                wind => {
                    let mut fu = 0;
                    if wind == context.round_wind {
                        fu += 2;
                    }
                    if wind == context.seat_wind {
                        fu += 2;
                    }
                    fu
                }
            }
        }
        Tile::Suited { .. } => 0, // Suited pairs give no fu
    }
}

/// Round up to the nearest 10
fn round_up_to_10(value: u8) -> u8 {
    ((value + 9) / 10) * 10
}

// ============================================================================
// Score Calculation
// ============================================================================

/// Determine the score level based on han and fu
pub fn determine_score_level(han: u8, fu: u8, is_yakuman: bool) -> ScoreLevel {
    if is_yakuman {
        if han >= 26 {
            ScoreLevel::DoubleYakuman
        } else {
            ScoreLevel::Yakuman
        }
    } else if han >= 13 {
        ScoreLevel::Yakuman // Counted yakuman (kazoe yakuman)
    } else if han >= 11 {
        ScoreLevel::Sanbaiman
    } else if han >= 8 {
        ScoreLevel::Baiman
    } else if han >= 6 {
        ScoreLevel::Haneman
    } else if han >= 5 {
        ScoreLevel::Mangan
    } else if han == 4 && fu >= 40 {
        ScoreLevel::Mangan
    } else if han == 3 && fu >= 70 {
        ScoreLevel::Mangan
    } else {
        ScoreLevel::Normal
    }
}

/// Calculate basic points from han and fu
///
/// Basic formula: fu × 2^(han+2)
/// Capped at 2000 (mangan)
pub fn calculate_basic_points(han: u8, fu: u8, is_yakuman: bool) -> u32 {
    let level = determine_score_level(han, fu, is_yakuman);

    if level != ScoreLevel::Normal {
        return level.basic_points();
    }

    // Normal calculation: fu × 2^(han+2)
    let basic = (fu as u32) * 2u32.pow((han + 2) as u32);

    // Cap at mangan (2000)
    basic.min(2000)
}

/// Calculate final payment based on basic points, dealer status, and win type
pub fn calculate_payment(basic_points: u32, is_dealer: bool, win_type: WinType) -> Payment {
    match win_type {
        WinType::Tsumo => {
            if is_dealer {
                // Dealer tsumo: each non-dealer pays basic × 2
                let from_each = round_up_to_100(basic_points * 2);
                Payment {
                    total: from_each * 3,
                    from_non_dealer: Some(from_each),
                    from_dealer: None, // Dealer is the winner
                    from_discarder: None,
                }
            } else {
                // Non-dealer tsumo: dealer pays basic × 2, others pay basic × 1
                let from_dealer = round_up_to_100(basic_points * 2);
                let from_non_dealer = round_up_to_100(basic_points);
                Payment {
                    total: from_dealer + (from_non_dealer * 2),
                    from_non_dealer: Some(from_non_dealer),
                    from_dealer: Some(from_dealer),
                    from_discarder: None,
                }
            }
        }
        WinType::Ron => {
            // Ron: discarder pays everything
            let multiplier = if is_dealer { 6 } else { 4 };
            let from_discarder = round_up_to_100(basic_points * multiplier);
            Payment {
                total: from_discarder,
                from_non_dealer: None,
                from_dealer: None,
                from_discarder: Some(from_discarder),
            }
        }
    }
}

/// Round up to the nearest 100
fn round_up_to_100(value: u32) -> u32 {
    ((value + 99) / 100) * 100
}

// ============================================================================
// Complete Scoring
// ============================================================================

/// Calculate complete score for a hand
///
/// # Arguments
/// * `structure` - Hand decomposition
/// * `yaku_result` - Result from yaku detection (includes han and dora)
/// * `context` - Game context
///
/// # Returns
/// Complete scoring result with fu, han, level, and payment
pub fn calculate_score(
    structure: &HandStructure,
    yaku_result: &YakuResult,
    context: &GameContext,
) -> ScoringResult {
    // Calculate fu
    let fu = calculate_fu(structure, context);

    // Get total han (yaku + dora)
    let han = yaku_result.total_han_with_dora();

    // Determine score level
    let score_level = determine_score_level(han, fu.total, yaku_result.is_yakuman);

    // Calculate basic points
    let basic_points = calculate_basic_points(han, fu.total, yaku_result.is_yakuman);

    // Calculate payment
    let is_dealer = context.is_dealer();
    let payment = calculate_payment(basic_points, is_dealer, context.win_type);

    // Counted yakuman: reached yakuman level (13+ han) without actual yakuman yaku
    let is_counted_yakuman = (score_level == ScoreLevel::Yakuman
        || score_level == ScoreLevel::DoubleYakuman)
        && !yaku_result.is_yakuman;

    ScoringResult {
        fu,
        han,
        score_level,
        basic_points,
        payment,
        is_dealer,
        is_counted_yakuman,
    }
}

/// Format a scoring result for display
pub fn format_score(result: &ScoringResult, yaku_result: &YakuResult) -> String {
    let mut output = String::new();

    // Yaku list
    output.push_str("Yaku:\n");
    for yaku in &yaku_result.yaku_list {
        let han = yaku.han();
        output.push_str(&format!("  • {:?} ({} han)\n", yaku, han));
    }

    // Dora breakdown
    if yaku_result.regular_dora > 0 {
        output.push_str(&format!("  • Dora ({} han)\n", yaku_result.regular_dora));
    }
    if yaku_result.ura_dora > 0 {
        output.push_str(&format!("  • Ura Dora ({} han)\n", yaku_result.ura_dora));
    }
    if yaku_result.aka_dora > 0 {
        output.push_str(&format!(
            "  • Red Fives (Akadora) ({} han)\n",
            yaku_result.aka_dora
        ));
    }

    // Han and Fu
    output.push_str(&format!("\n{} han / {} fu\n", result.han, result.fu.total));

    // Score level (if applicable)
    if result.score_level != ScoreLevel::Normal {
        output.push_str(&format!("{}\n", result.score_level.name()));
    }

    // Payment
    output.push_str(&format!("\nTotal: {} points\n", result.payment.total));

    if let Some(from_discarder) = result.payment.from_discarder {
        output.push_str(&format!("Ron: {} from discarder\n", from_discarder));
    } else {
        if result.is_dealer {
            if let Some(from_each) = result.payment.from_non_dealer {
                output.push_str(&format!("Tsumo: {} all\n", from_each));
            }
        } else {
            if let (Some(from_dealer), Some(from_non_dealer)) =
                (result.payment.from_dealer, result.payment.from_non_dealer)
            {
                output.push_str(&format!("Tsumo: {}/{}\n", from_dealer, from_non_dealer));
            }
        }
    }

    output
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hand::decompose_hand;
    use crate::parse::{parse_hand, to_counts};
    use crate::tile::Suit;
    use crate::yaku::detect_yaku_with_context;

    // ===== Helper Functions =====

    fn score_hand(hand: &str, context: &GameContext) -> Vec<ScoringResult> {
        let tiles = parse_hand(hand).unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        structures
            .iter()
            .map(|s| {
                let yaku_result = detect_yaku_with_context(s, &counts, context);
                calculate_score(s, &yaku_result, context)
            })
            .collect()
    }

    fn best_score(results: &[ScoringResult]) -> &ScoringResult {
        results
            .iter()
            .max_by(|a, b| {
                a.payment
                    .total
                    .cmp(&b.payment.total)
                    .then_with(|| a.han.cmp(&b.han))
                    .then_with(|| b.fu.total.cmp(&a.fu.total))
            })
            .unwrap()
    }

    // ===== Fu Calculation Tests =====

    #[test]
    fn test_fu_chiitoitsu() {
        // Chiitoitsu is always 25 fu
        let tiles = parse_hand("1122m3344p5566s77z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let chiitoi = structures
            .iter()
            .find(|s| matches!(s, HandStructure::Chiitoitsu { .. }))
            .unwrap();

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        let fu = calculate_fu(chiitoi, &context);

        assert_eq!(fu.total, 25);
    }

    #[test]
    fn test_fu_pinfu_tsumo() {
        // Pinfu + Tsumo = exactly 20 fu
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 4)); // Ryanmen wait

        let tiles = parse_hand("123456m789p234s55p").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        // Find a structure that qualifies for pinfu
        let fu_results: Vec<_> = structures
            .iter()
            .map(|s| calculate_fu(s, &context))
            .collect();

        // At least one should be 20 fu (pinfu tsumo)
        assert!(fu_results.iter().any(|f| f.total == 20));
    }

    #[test]
    fn test_fu_menzen_ron() {
        // Closed hand ron = +10 fu
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        let tiles = parse_hand("234m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        // Base 20 + Menzen Ron 10 + triplet fu = at least 30
        assert!(fu.total >= 30);
        assert_eq!(fu.breakdown.menzen_ron, 10);
    }

    #[test]
    fn test_fu_tsumo_bonus() {
        // Tsumo = +2 fu (non-pinfu hand)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::honor(Honor::East));

        let tiles = parse_hand("123m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        assert_eq!(fu.breakdown.tsumo, 2);
    }

    #[test]
    fn test_fu_triplet_simple_open() {
        // Open triplet of simples = 2 fu
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .open()
            .with_winning_tile(Tile::suited(Suit::Man, 5));

        // 555m (simple triplet) + sequences
        let tiles = parse_hand("555m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        // Should have 2 fu from the simple triplet (open)
        // Plus 8 fu from terminal/honor triplet (111z, but this is closed...
        // Wait, in an open hand, are the non-called melds closed?
        // Yes! Only the called melds are open. But for simplicity we're treating
        // all melds as having the same open/closed status as the hand.
        // This is a simplification - proper implementation would track each meld.
        assert!(fu.breakdown.melds >= 2);
    }

    #[test]
    fn test_fu_triplet_terminal_closed() {
        // Closed triplet of terminals = 8 fu
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 2));

        // 111m (terminal triplet, closed) + sequences
        let tiles = parse_hand("111234m456p789s22z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        // Closed terminal triplet = 8 fu
        assert!(fu.breakdown.melds >= 8);
    }

    #[test]
    fn test_fu_yakuhai_pair() {
        // Dragon pair = 2 fu
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        // 234m 456p 789s 111z 55z - pair of white dragons (5z)
        let tiles = parse_hand("234m456p789s11155z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        assert!(!structures.is_empty(), "Should have valid decomposition");
        let fu = calculate_fu(&structures[0], &context);

        assert_eq!(fu.breakdown.pair, 2);
    }

    #[test]
    fn test_fu_double_wind_pair() {
        // Pair of double wind (both round and seat) = 4 fu
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        // 234m 456p 789s 222z 11z - pair of east (both round and seat wind)
        let tiles = parse_hand("234m456p789s22211z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        assert!(!structures.is_empty(), "Should have valid decomposition");
        let fu = calculate_fu(&structures[0], &context);

        // Double wind = 4 fu (2 for round wind + 2 for seat wind)
        assert_eq!(fu.breakdown.pair, 4);
    }

    #[test]
    fn test_fu_wait_kanchan() {
        // Kanchan wait = 2 fu
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 3)); // Middle of 234

        let tiles = parse_hand("234m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        assert_eq!(fu.breakdown.wait, 2);
    }

    #[test]
    fn test_fu_ron_completed_triplet_simple() {
        // When winning by ron on a shanpon wait, the triplet completed by
        // the winning tile should be treated as "open" for fu purposes.
        // Simple triplet completed by ron = 2 fu (not 4 fu)

        // Hand: 222m 678m 444p 666p 11z - shanpon wait on 2m/1z
        // Winning on 2m by ron
        // Note: Using West round wind so the East (1z) pair doesn't add fu
        let context = GameContext::new(WinType::Ron, Honor::West, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 2));

        let tiles = parse_hand("222678m444666p11z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        // 222m completed by ron = 2 fu (open)
        // 444p closed = 4 fu
        // 666p closed = 4 fu
        // 11z pair = 0 fu (not yakuhai since round=West, seat=South)
        // Base 20 + Menzen Ron 10 + melds (2+4+4) = 40 fu
        assert_eq!(fu.breakdown.melds, 10); // 2 + 4 + 4
        assert_eq!(fu.breakdown.pair, 0);
        assert_eq!(fu.total, 40);
    }

    #[test]
    fn test_fu_ron_completed_triplet_honor() {
        // Honor triplet completed by ron = 4 fu (not 8 fu)

        // Hand: 66p 456s 444z with open melds (222s) (555z)
        // Winning on 4z (North) by ron
        use crate::hand::decompose_hand_with_melds;
        use crate::parse::parse_hand_with_aka;

        let parsed = parse_hand_with_aka("66p456s444z(222s)(555z)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();
        let structures = decompose_hand_with_melds(&counts, &called_melds);

        let context = GameContext::new(WinType::Ron, Honor::South, Honor::South)
            .open()
            .with_winning_tile(Tile::honor(Honor::North));

        let fu = calculate_fu(&structures[0], &context);

        // (222s) open simple = 2 fu
        // (555z) open honor = 4 fu
        // 444z completed by ron = 4 fu (open honor, not 8 fu closed)
        // Base 20 + melds (2+4+4) = 30 fu
        assert_eq!(fu.breakdown.melds, 10); // 2 + 4 + 4
        assert_eq!(fu.total, 30);
    }

    #[test]
    fn test_fu_tsumo_triplet_stays_closed() {
        // When winning by tsumo, triplets should remain closed
        // Simple triplet by tsumo = 4 fu (closed)

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 2));

        let tiles = parse_hand("222678m444666p11z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);

        let fu = calculate_fu(&structures[0], &context);

        // 222m by tsumo = 4 fu (closed)
        // 444p closed = 4 fu
        // 666p closed = 4 fu
        // Base 20 + Tsumo 2 + melds (4+4+4) = 34 → 40 fu
        assert_eq!(fu.breakdown.melds, 12); // 4 + 4 + 4
        assert_eq!(fu.total, 40);
    }

    #[test]
    fn test_fu_nobetan_triplet_stays_closed() {
        // Nobetan pattern: 11123 waiting on 1 or 4
        // When winning on 1 by ron, the triplet 111 should remain CLOSED
        // because the 1 also appears in the sequence 123 (not a true shanpon).
        //
        // Hand: 99m 111123p 789s (222z) - won on 1p by ron
        // This is the exact case from the Tenhou validation mismatch.
        use crate::hand::decompose_hand_with_melds;
        use crate::parse::parse_hand_with_aka;

        let parsed = parse_hand_with_aka("99m111123p789s(222z)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();
        let structures = decompose_hand_with_melds(&counts, &called_melds);

        // Round wind South, Seat wind North - (222z) is South = yakuhai
        let context = GameContext::new(WinType::Ron, Honor::South, Honor::North)
            .open()
            .with_winning_tile(Tile::suited(Suit::Pin, 1));

        let fu = calculate_fu(&structures[0], &context);

        // 111p is NOT a true shanpon (1p also in 123p sequence) = 8 fu (closed terminal)
        // (222z) open honor = 4 fu
        // Base 20 + melds (8+4) = 32 → 40 fu
        assert_eq!(fu.breakdown.melds, 12); // 8 + 4
        assert_eq!(fu.total, 40);
    }

    #[test]
    fn test_fu_nobetan_open_sequence_does_not_count() {
        // When the winning tile appears in an OPEN (called) sequence,
        // that should NOT trigger the nobetan exception.
        // Open sequences were completed before the wait was established.
        //
        // Hand: 99m 111p 456s 789s (123p) - ron on 1p
        // The 1p appears in both 111p triplet AND (123p) open chi.
        // But since (123p) is open, it doesn't count as nobetan.
        // The 111p triplet should be treated as "opened by ron" (shanpon wait).
        use crate::hand::decompose_hand_with_melds;
        use crate::parse::parse_hand_with_aka;

        let parsed = parse_hand_with_aka("99m111p456s789s(123p)").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();
        let structures = decompose_hand_with_melds(&counts, &called_melds);

        let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
            .open()
            .with_winning_tile(Tile::suited(Suit::Pin, 1));

        let fu = calculate_fu(&structures[0], &context);

        // 111p is a TRUE shanpon (open 123p doesn't count) = 4 fu (open terminal)
        // (123p) open sequence = 0 fu
        // 456s closed sequence = 0 fu
        // 789s closed sequence = 0 fu
        // 99m pair = 0 fu
        // Base 20 + melds 4 = 24 → 30 fu
        assert_eq!(fu.breakdown.melds, 4); // 111p open terminal triplet = 4 fu
        assert_eq!(fu.total, 30);
    }

    #[test]
    fn test_fu_rounding() {
        // Fu should round up to nearest 10
        assert_eq!(round_up_to_10(22), 30);
        assert_eq!(round_up_to_10(30), 30);
        assert_eq!(round_up_to_10(31), 40);
        assert_eq!(round_up_to_10(25), 30); // But chiitoitsu stays 25
    }

    // ===== Kan Fu Tests =====

    #[test]
    fn test_fu_kan_simple_open() {
        use crate::hand::{KanType, Meld};

        // Open kan of simples = 8 fu
        let kan = Meld::kan(Tile::suited(Suit::Man, 5), KanType::Open);
        assert_eq!(meld_fu(&kan), 8);

        // Added kan (shouminkan) is also open = 8 fu
        let added_kan = Meld::kan(Tile::suited(Suit::Pin, 3), KanType::Added);
        assert_eq!(meld_fu(&added_kan), 8);
    }

    #[test]
    fn test_fu_kan_simple_closed() {
        use crate::hand::{KanType, Meld};

        // Closed kan of simples = 16 fu
        let kan = Meld::kan(Tile::suited(Suit::Sou, 7), KanType::Closed);
        assert_eq!(meld_fu(&kan), 16);
    }

    #[test]
    fn test_fu_kan_terminal_open() {
        use crate::hand::{KanType, Meld};

        // Open kan of terminals = 16 fu
        let kan = Meld::kan(Tile::suited(Suit::Man, 1), KanType::Open);
        assert_eq!(meld_fu(&kan), 16);

        let kan_9 = Meld::kan(Tile::suited(Suit::Pin, 9), KanType::Added);
        assert_eq!(meld_fu(&kan_9), 16);
    }

    #[test]
    fn test_fu_kan_terminal_closed() {
        use crate::hand::{KanType, Meld};

        // Closed kan of terminals = 32 fu
        let kan = Meld::kan(Tile::suited(Suit::Sou, 1), KanType::Closed);
        assert_eq!(meld_fu(&kan), 32);
    }

    #[test]
    fn test_fu_kan_honor_open() {
        use crate::hand::{KanType, Meld};
        use crate::tile::Honor;

        // Open kan of honors = 16 fu
        let kan = Meld::kan(Tile::honor(Honor::East), KanType::Open);
        assert_eq!(meld_fu(&kan), 16);

        let dragon_kan = Meld::kan(Tile::honor(Honor::White), KanType::Added);
        assert_eq!(meld_fu(&dragon_kan), 16);
    }

    #[test]
    fn test_fu_kan_honor_closed() {
        use crate::hand::{KanType, Meld};
        use crate::tile::Honor;

        // Closed kan of honors = 32 fu
        let kan = Meld::kan(Tile::honor(Honor::Red), KanType::Closed);
        assert_eq!(meld_fu(&kan), 32);

        let wind_kan = Meld::kan(Tile::honor(Honor::North), KanType::Closed);
        assert_eq!(meld_fu(&wind_kan), 32);
    }

    #[test]
    fn test_fu_comparison_triplet_vs_kan() {
        use crate::hand::{KanType, Meld};

        // Kan fu should be 4x the equivalent triplet fu
        let simple_tile = Tile::suited(Suit::Man, 5);
        let terminal_tile = Tile::suited(Suit::Pin, 1);

        // Open simple: triplet 2, kan 8
        let triplet_open = Meld::Koutsu(simple_tile, true);
        let kan_open = Meld::kan(simple_tile, KanType::Open);
        assert_eq!(meld_fu(&kan_open), meld_fu(&triplet_open) * 4);

        // Closed simple: triplet 4, kan 16
        let triplet_closed = Meld::koutsu(simple_tile);
        let kan_closed = Meld::kan(simple_tile, KanType::Closed);
        assert_eq!(meld_fu(&kan_closed), meld_fu(&triplet_closed) * 4);

        // Open terminal: triplet 4, kan 16
        let triplet_term_open = Meld::Koutsu(terminal_tile, true);
        let kan_term_open = Meld::kan(terminal_tile, KanType::Open);
        assert_eq!(meld_fu(&kan_term_open), meld_fu(&triplet_term_open) * 4);

        // Closed terminal: triplet 8, kan 32
        let triplet_term_closed = Meld::koutsu(terminal_tile);
        let kan_term_closed = Meld::kan(terminal_tile, KanType::Closed);
        assert_eq!(meld_fu(&kan_term_closed), meld_fu(&triplet_term_closed) * 4);
    }

    // ===== Score Level Tests =====

    #[test]
    fn test_score_level_mangan() {
        assert_eq!(determine_score_level(5, 30, false), ScoreLevel::Mangan);
        assert_eq!(determine_score_level(4, 40, false), ScoreLevel::Mangan);
        assert_eq!(determine_score_level(3, 70, false), ScoreLevel::Mangan);
    }

    #[test]
    fn test_score_level_haneman() {
        assert_eq!(determine_score_level(6, 30, false), ScoreLevel::Haneman);
        assert_eq!(determine_score_level(7, 30, false), ScoreLevel::Haneman);
    }

    #[test]
    fn test_score_level_baiman() {
        assert_eq!(determine_score_level(8, 30, false), ScoreLevel::Baiman);
        assert_eq!(determine_score_level(10, 30, false), ScoreLevel::Baiman);
    }

    #[test]
    fn test_score_level_sanbaiman() {
        assert_eq!(determine_score_level(11, 30, false), ScoreLevel::Sanbaiman);
        assert_eq!(determine_score_level(12, 30, false), ScoreLevel::Sanbaiman);
    }

    #[test]
    fn test_score_level_yakuman() {
        assert_eq!(determine_score_level(13, 30, false), ScoreLevel::Yakuman);
        assert_eq!(determine_score_level(13, 30, true), ScoreLevel::Yakuman);
    }

    // ===== Basic Points Tests =====

    #[test]
    fn test_basic_points_simple() {
        // 1 han 30 fu = 30 × 2^3 = 240
        assert_eq!(calculate_basic_points(1, 30, false), 240);

        // 2 han 30 fu = 30 × 2^4 = 480
        assert_eq!(calculate_basic_points(2, 30, false), 480);

        // 3 han 30 fu = 30 × 2^5 = 960
        assert_eq!(calculate_basic_points(3, 30, false), 960);

        // 4 han 30 fu = 30 × 2^6 = 1920
        assert_eq!(calculate_basic_points(4, 30, false), 1920);
    }

    #[test]
    fn test_basic_points_mangan_cap() {
        // 4 han 40 fu = mangan = 2000
        assert_eq!(calculate_basic_points(4, 40, false), 2000);

        // 5 han = mangan = 2000
        assert_eq!(calculate_basic_points(5, 30, false), 2000);
    }

    #[test]
    fn test_basic_points_limits() {
        assert_eq!(calculate_basic_points(6, 30, false), 3000); // Haneman
        assert_eq!(calculate_basic_points(8, 30, false), 4000); // Baiman
        assert_eq!(calculate_basic_points(11, 30, false), 6000); // Sanbaiman
        assert_eq!(calculate_basic_points(13, 30, false), 8000); // Yakuman
    }

    // ===== Payment Tests =====

    #[test]
    fn test_payment_dealer_tsumo() {
        // Dealer tsumo mangan: 4000 all (× 3 = 12000)
        let payment = calculate_payment(2000, true, WinType::Tsumo);

        assert_eq!(payment.from_non_dealer, Some(4000));
        assert_eq!(payment.from_dealer, None);
        assert_eq!(payment.total, 12000);
    }

    #[test]
    fn test_payment_non_dealer_tsumo() {
        // Non-dealer tsumo mangan: 4000/2000
        let payment = calculate_payment(2000, false, WinType::Tsumo);

        assert_eq!(payment.from_dealer, Some(4000));
        assert_eq!(payment.from_non_dealer, Some(2000));
        assert_eq!(payment.total, 8000);
    }

    #[test]
    fn test_payment_dealer_ron() {
        // Dealer ron mangan: 12000
        let payment = calculate_payment(2000, true, WinType::Ron);

        assert_eq!(payment.from_discarder, Some(12000));
        assert_eq!(payment.total, 12000);
    }

    #[test]
    fn test_payment_non_dealer_ron() {
        // Non-dealer ron mangan: 8000
        let payment = calculate_payment(2000, false, WinType::Ron);

        assert_eq!(payment.from_discarder, Some(8000));
        assert_eq!(payment.total, 8000);
    }

    #[test]
    fn test_payment_rounding() {
        // Payments round up to nearest 100
        // 1 han 30 fu non-dealer ron = 240 × 4 = 960 → 1000
        let payment = calculate_payment(240, false, WinType::Ron);
        assert_eq!(payment.from_discarder, Some(1000));
    }

    // ===== Complete Scoring Tests =====

    #[test]
    fn test_complete_score_riichi_tsumo() {
        // Riichi + Menzen Tsumo = 2 han
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .riichi()
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        let results = score_hand("234m456p789s11122z", &context);
        let best = best_score(&results);

        // Should have at least 2 han (riichi + menzen tsumo)
        assert!(best.han >= 2);
        assert!(best.payment.total > 0);
    }

    #[test]
    fn test_complete_score_pinfu_tsumo() {
        // Pinfu + Tsumo = 2 han, 20 fu
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 4)); // Ryanmen

        let results = score_hand("123456m789p234s55p", &context);
        let best = best_score(&results);

        // Should be 20 fu (pinfu tsumo special case)
        assert_eq!(best.fu.total, 20);
        // Should have 2 han (pinfu + menzen tsumo)
        assert_eq!(best.han, 2);
    }

    #[test]
    fn test_complete_score_haneman() {
        // Chinitsu (6 han closed) + Menzen Tsumo (1 han) = 7 han = Haneman
        // Using a hand without Ittsu to avoid extra han
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 5));

        // 111m 234m 345m 555m 99m - chinitsu without ittsu
        let results = score_hand("111234345555m99m", &context);
        let best = best_score(&results);

        // Chinitsu closed = 6 han + menzen tsumo = 1 han = 7 han = Haneman
        assert_eq!(best.score_level, ScoreLevel::Haneman);
        assert_eq!(best.basic_points, 3000);
    }

    #[test]
    fn test_complete_score_yakuman() {
        // Tenhou = Yakuman
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .tenhou()
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        let results = score_hand("234m456p789s11122z", &context);
        let best = best_score(&results);

        assert_eq!(best.score_level, ScoreLevel::Yakuman);
        assert_eq!(best.han, 13);
        // Dealer yakuman tsumo = 16000 all = 48000 total
        assert_eq!(best.payment.total, 48000);
    }

    #[test]
    fn test_common_scores() {
        // Test some common score patterns

        // 1 han 30 fu non-dealer ron = 240 × 4 = 960 → 1000
        let payment = calculate_payment(calculate_basic_points(1, 30, false), false, WinType::Ron);
        assert_eq!(payment.total, 1000);

        // 2 han 30 fu non-dealer ron = 480 × 4 = 1920 → 2000
        let payment = calculate_payment(calculate_basic_points(2, 30, false), false, WinType::Ron);
        assert_eq!(payment.total, 2000);

        // 3 han 40 fu non-dealer ron = 1280 × 4 = 5120 → 5200
        let payment = calculate_payment(calculate_basic_points(3, 40, false), false, WinType::Ron);
        assert_eq!(payment.total, 5200);

        // 4 han 30 fu non-dealer ron = 1920 × 4 = 7680 → 7700 (NOT mangan, need 40+ fu)
        let payment = calculate_payment(calculate_basic_points(4, 30, false), false, WinType::Ron);
        assert_eq!(payment.total, 7700);

        // 4 han 40 fu non-dealer ron = mangan = 8000
        let payment = calculate_payment(calculate_basic_points(4, 40, false), false, WinType::Ron);
        assert_eq!(payment.total, 8000);
    }

    #[test]
    fn test_format_score() {
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .riichi()
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        let tiles = parse_hand("234m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        let structures = decompose_hand(&counts);
        let yaku_result = detect_yaku_with_context(&structures[0], &counts, &context);
        let score_result = calculate_score(&structures[0], &yaku_result, &context);

        let formatted = format_score(&score_result, &yaku_result);

        // Should contain key information
        assert!(formatted.contains("Riichi"));
        assert!(formatted.contains("han"));
        assert!(formatted.contains("fu"));
        assert!(formatted.contains("Total:"));
    }

    // ===== Kan Scoring Tests =====

    #[test]
    fn test_hand_with_closed_kan_fu() {
        use crate::hand::decompose_hand_with_melds;
        use crate::parse::parse_hand_with_aka;

        // Hand: [1111m] 222m 333m 555p 11z (15 tiles with closed kan)
        let parsed = parse_hand_with_aka("[1111m]222333m555p11z").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();

        let structures = decompose_hand_with_melds(&counts, &called_melds);
        assert!(!structures.is_empty());

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_winning_tile(Tile::suited(Suit::Man, 2));

        let fu = calculate_fu(&structures[0], &context);

        // Closed terminal kan = 32 fu
        // Closed simple triplet 222m = 4 fu
        // Closed simple triplet 333m = 4 fu
        // Closed simple triplet 555p = 4 fu
        // Double wind pair 11z = 4 fu
        // Tsumo = 2 fu
        // Base = 20 fu
        // Total = 32 + 4 + 4 + 4 + 4 + 2 + 20 = 70 fu
        assert_eq!(fu.total, 70);
    }

    #[test]
    fn test_hand_with_open_kan_fu() {
        use crate::hand::decompose_hand_with_melds;
        use crate::parse::parse_hand_with_aka;

        // Hand: (5555m) 123p 456p 789s 11z (15 tiles with open kan)
        let parsed = parse_hand_with_aka("(5555m)123456p789s11z").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();

        let structures = decompose_hand_with_melds(&counts, &called_melds);
        assert!(!structures.is_empty());

        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .open()
            .with_winning_tile(Tile::suited(Suit::Pin, 3));

        let fu = calculate_fu(&structures[0], &context);

        // Open simple kan = 8 fu
        // Verify the kan contributes the right fu
        assert_eq!(fu.breakdown.melds, 8);
        // Total fu will include base 20 + kan 8 + wait fu, rounded up
        assert!(fu.total >= 30);
    }

    #[test]
    fn test_hand_with_honor_kan() {
        use crate::hand::decompose_hand_with_melds;
        use crate::parse::parse_hand_with_aka;

        // Hand: [5555z] 123m 456p 789s 11z (15 tiles with closed dragon kan)
        let parsed = parse_hand_with_aka("[5555z]123m456p789s11z").unwrap();
        let counts = to_counts(&parsed.tiles);
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();

        let structures = decompose_hand_with_melds(&counts, &called_melds);
        assert!(!structures.is_empty());

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_winning_tile(Tile::suited(Suit::Man, 2));

        let fu = calculate_fu(&structures[0], &context);

        // Closed honor kan = 32 fu
        // Sequences = 0 fu each
        // Double wind pair = 4 fu
        // Tsumo = 2 fu
        // Base = 20 fu
        // Total = 32 + 0 + 0 + 0 + 4 + 2 + 20 = 58 -> rounded to 60 fu
        assert_eq!(fu.total, 60);
    }

    // ========================================================================
    // Counted Yakuman Tests
    // ========================================================================

    #[test]
    fn test_counted_yakuman_chinitsu_ryanpeikou() {
        // 22334455667799s - Chinitsu + Ryanpeikou + Pinfu + Riichi + Tsumo
        // This reaches 13+ han without actual yakuman yaku
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .riichi()
            .with_winning_tile(Tile::suited(Suit::Sou, 2));

        let results = score_hand("22334455667799s", &context);
        let best = best_score(&results);

        // Riichi(1) + Menzen Tsumo(1) + Pinfu(1) + Ryanpeikou(3) + Chinitsu(6) = 12 han
        // With ippatsu or dora it would be 13+, but even at 12 han it's sanbaiman
        // Let's verify the hand structure is correct
        assert!(best.han >= 12);
        assert!(!best.is_counted_yakuman); // 12 han is Sanbaiman, not Yakuman
    }

    #[test]
    fn test_counted_yakuman_with_dora() {
        // Same hand but with dora to push it to 13+ han
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .riichi()
            .ippatsu()
            .with_winning_tile(Tile::suited(Suit::Sou, 2))
            .with_dora(vec![Tile::suited(Suit::Sou, 1)]); // 2s is dora

        let results = score_hand("22334455667799s", &context);
        let best = best_score(&results);

        // Riichi(1) + Ippatsu(1) + Menzen Tsumo(1) + Pinfu(1) + Ryanpeikou(3) + Chinitsu(6) + Dora(2) = 15 han
        assert!(best.han >= 13);
        assert_eq!(best.score_level, ScoreLevel::Yakuman);
        assert!(best.is_counted_yakuman); // Reached yakuman through counting, not yakuman yaku
    }

    #[test]
    fn test_true_yakuman_kokushi() {
        // Kokushi Musou - a true yakuman
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_winning_tile(Tile::suited(Suit::Man, 1));

        let results = score_hand("19m19p19s12345677z", &context);
        let best = best_score(&results);

        assert_eq!(best.score_level, ScoreLevel::Yakuman);
        assert!(!best.is_counted_yakuman); // True yakuman, not counted
    }

    #[test]
    fn test_true_yakuman_suuankou() {
        // Suuankou - four concealed triplets (true yakuman)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_winning_tile(Tile::Honor(Honor::White));

        let results = score_hand("111222333m444p55z", &context);
        let best = best_score(&results);

        assert_eq!(best.score_level, ScoreLevel::Yakuman);
        assert!(!best.is_counted_yakuman); // True yakuman, not counted
    }

    #[test]
    fn test_true_yakuman_tenhou() {
        // Tenhou - dealer's first draw win (true yakuman)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .tenhou()
            .with_winning_tile(Tile::suited(Suit::Man, 4));

        let results = score_hand("234m456p789s11122z", &context);
        let best = best_score(&results);

        assert_eq!(best.score_level, ScoreLevel::Yakuman);
        assert!(!best.is_counted_yakuman); // True yakuman, not counted
    }

    #[test]
    fn test_not_counted_yakuman_below_13_han() {
        // A high-scoring hand that doesn't reach yakuman level
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .riichi()
            .with_winning_tile(Tile::suited(Suit::Man, 5));

        // Chinitsu (6) + Riichi (1) + Menzen Tsumo (1) = 8 han = Baiman
        let results = score_hand("123345567789m55m", &context);
        let best = best_score(&results);

        assert!(best.han < 13);
        assert_eq!(best.score_level, ScoreLevel::Baiman);
        assert!(!best.is_counted_yakuman);
    }

    // ========================================================================
    // Interpretation Preference Tests
    // ========================================================================

    #[test]
    fn test_prefer_ryanpeikou_over_chiitoitsu() {
        // Hand: 22334455667799s can be interpreted as:
        // - Chiitoitsu (7 pairs): 2 han
        // - Ryanpeikou + Pinfu (234s 234s 567s 567s + 99s): 4 han
        //
        // When both reach the same payment (e.g., both yakuman at 13+ han),
        // the higher han interpretation should be preferred.
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .riichi()
            .ippatsu()
            .with_winning_tile(Tile::suited(Suit::Sou, 2))
            .with_dora(vec![Tile::suited(Suit::Sou, 1)]); // 2s is dora

        let results = score_hand("22334455667799s", &context);
        let best = best_score(&results);

        // Should pick Ryanpeikou interpretation (higher han)
        // Ryanpeikou: Riichi(1) + Ippatsu(1) + Tsumo(1) + Pinfu(1) + Ryanpeikou(3) + Chinitsu(6) + Dora(2) = 15 han
        // Chiitoitsu: Riichi(1) + Ippatsu(1) + Tsumo(1) + Chiitoitsu(2) + Chinitsu(6) + Dora(2) = 13 han
        assert!(
            best.han >= 15,
            "Expected 15+ han for Ryanpeikou, got {}",
            best.han
        );

        // Both are yakuman, but Ryanpeikou has more han
        assert_eq!(best.score_level, ScoreLevel::Yakuman);
    }

    #[test]
    fn test_chiitoitsu_vs_ryanpeikou_different_scores() {
        // When interpretations have different payment totals,
        // the higher payment wins regardless of han count.
        //
        // Without enough dora/yaku to both reach yakuman,
        // Ryanpeikou (3 han) + Pinfu (1 han) = 4 han beats Chiitoitsu (2 han)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 2));

        let results = score_hand("22334455667799s", &context);
        let best = best_score(&results);

        // Ryanpeikou + Pinfu + Chinitsu + Menzen Tsumo = 3 + 1 + 6 + 1 = 11 han (Sanbaiman)
        // Chiitoitsu + Chinitsu + Menzen Tsumo = 2 + 6 + 1 = 9 han (Baiman)
        // Ryanpeikou interpretation should win with higher payment
        assert!(
            best.han >= 11,
            "Expected 11+ han for Ryanpeikou, got {}",
            best.han
        );
        assert_eq!(best.score_level, ScoreLevel::Sanbaiman);
    }

    // ===== Winning Tile Inference Tests =====
    //
    // These tests verify that choosing different winning tiles affects scoring
    // correctly, which is the foundation for the winning tile inference feature.
    // The actual inference logic is in main.rs, but these tests ensure the
    // scoring module correctly handles different winning tiles.

    #[test]
    fn test_winning_tile_affects_pinfu_eligibility() {
        // Hand: 334455m334455p66s (Ryanpeikou shape)
        // With winning tile on 3m (ryanmen wait): Pinfu applies, 20 fu
        // With winning tile on 6s (tanki wait): Pinfu does NOT apply, 30 fu

        // Test with ryanmen wait (3m) - should get Pinfu
        let context_ryanmen = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 3));
        let results_ryanmen = score_hand("334455m334455p66s", &context_ryanmen);
        let best_ryanmen = best_score(&results_ryanmen);

        // Pinfu + Menzen Tsumo + Tanyao + Ryanpeikou = 1 + 1 + 1 + 3 = 6 han, 20 fu
        assert_eq!(
            best_ryanmen.fu.total, 20,
            "Ryanmen wait should give 20 fu (Pinfu)"
        );
        assert_eq!(best_ryanmen.han, 6, "Should have 6 han with Pinfu");

        // Test with tanki wait (6s) - should NOT get Pinfu
        let context_tanki = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 6));
        let results_tanki = score_hand("334455m334455p66s", &context_tanki);
        let best_tanki = best_score(&results_tanki);

        // Menzen Tsumo + Tanyao + Ryanpeikou = 1 + 1 + 3 = 5 han, 30 fu (no Pinfu)
        assert_eq!(
            best_tanki.fu.total, 30,
            "Tanki wait should give 30 fu (no Pinfu)"
        );
        assert_eq!(best_tanki.han, 5, "Should have 5 han without Pinfu");
    }

    #[test]
    fn test_winning_tile_affects_payment_with_pinfu() {
        // Same hand as above, but verify the payment difference
        // Ryanmen (6 han 20 fu) vs Tanki (5 han 30 fu)
        //
        // Valid winning tiles in 334455m334455p66s:
        // - Ryanmen: 3m, 5m, 3p, 5p (edge of 345 sequences)
        // - Kanchan: 4m, 4p (middle of sequences)
        // - Tanki: 6s (pair wait)

        let context_ryanmen = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 3)); // 3m gives ryanmen (2-5 wait on 345)
        let results_ryanmen = score_hand("334455m334455p66s", &context_ryanmen);
        let best_ryanmen = best_score(&results_ryanmen);

        let context_tanki = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 6));
        let results_tanki = score_hand("334455m334455p66s", &context_tanki);
        let best_tanki = best_score(&results_tanki);

        // 6 han (Haneman) pays more than 5 han (Mangan)
        assert!(
            best_ryanmen.payment.total > best_tanki.payment.total,
            "Ryanmen interpretation ({} points) should pay more than tanki ({} points)",
            best_ryanmen.payment.total,
            best_tanki.payment.total
        );

        // Verify score levels
        assert_eq!(best_ryanmen.score_level, ScoreLevel::Haneman);
        assert_eq!(best_tanki.score_level, ScoreLevel::Mangan);
    }

    #[test]
    fn test_winning_tile_ryanpeikou_multiple_ryanmen_options() {
        // Hand: 334455m334455p66s
        // Valid winning tiles (must be in hand):
        // - Ryanmen: 3m, 5m, 3p, 5p (give Pinfu)
        // - Kanchan: 4m, 4p (no Pinfu)
        // - Tanki: 6s (no Pinfu)
        //
        // All ryanmen options should give the same score (Pinfu applies)

        let ryanmen_tiles = [
            Tile::suited(Suit::Man, 3),
            Tile::suited(Suit::Man, 5),
            Tile::suited(Suit::Pin, 3),
            Tile::suited(Suit::Pin, 5),
        ];

        let mut ryanmen_scores = Vec::new();
        for tile in &ryanmen_tiles {
            let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
                .with_winning_tile(*tile);
            let results = score_hand("334455m334455p66s", &context);
            let best = best_score(&results);
            ryanmen_scores.push((tile, best.han, best.fu.total, best.payment.total));
        }

        // All ryanmen waits should give the same han and fu
        let first = &ryanmen_scores[0];
        for (tile, han, fu, payment) in &ryanmen_scores {
            assert_eq!(
                *han, first.1,
                "Tile {:?} gave {} han, expected {} han",
                tile, han, first.1
            );
            assert_eq!(
                *fu, first.2,
                "Tile {:?} gave {} fu, expected {} fu",
                tile, fu, first.2
            );
            assert_eq!(
                *payment, first.3,
                "Tile {:?} gave {} payment, expected {} payment",
                tile, payment, first.3
            );
        }

        // All should have Pinfu (20 fu)
        assert_eq!(first.2, 20, "Ryanmen wait should give 20 fu");
        // All should have 6 han (Menzen Tsumo + Tanyao + Pinfu + Ryanpeikou)
        assert_eq!(first.1, 6, "Ryanmen wait should give 6 han");
    }

    #[test]
    fn test_winning_tile_inference_chooses_best_for_mahjong_soul_hand() {
        // This is the exact hand from the MahjongSoul screenshot that prompted
        // the winning tile inference feature:
        // Hand: 440566m334405p66s (with red fives represented as 0)
        // Note: We test with regular 5s since the scoring module doesn't track aka
        //
        // The actual hand is: 445566m334455p66s
        // Valid winning tiles:
        // - Ryanmen: 4m, 6m, 3p, 5p (give Pinfu)
        // - Kanchan: 5m, 4p (no Pinfu)
        // - Tanki: 6s (no Pinfu)
        //
        // With winning tile 4m or 6m (ryanmen on 456m): Pinfu applies
        // With winning tile 6s (tanki): Pinfu does NOT apply

        let context_good = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .riichi()
            .with_winning_tile(Tile::suited(Suit::Man, 4)); // Ryanmen on 3-6 for 456
        let results_good = score_hand("445566m334455p66s", &context_good);
        let best_good = best_score(&results_good);

        let context_bad = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .riichi()
            .with_winning_tile(Tile::suited(Suit::Sou, 6)); // Tanki
        let results_bad = score_hand("445566m334455p66s", &context_bad);
        let best_bad = best_score(&results_bad);

        // Good interpretation: Riichi + Menzen Tsumo + Pinfu + Tanyao + Ryanpeikou
        // = 1 + 1 + 1 + 1 + 3 = 7 han, 20 fu
        assert_eq!(best_good.fu.total, 20);
        assert_eq!(best_good.han, 7);
        assert_eq!(best_good.score_level, ScoreLevel::Haneman);

        // Bad interpretation: Riichi + Menzen Tsumo + Tanyao + Ryanpeikou (no Pinfu)
        // = 1 + 1 + 1 + 3 = 6 han, 30 fu
        assert_eq!(best_bad.fu.total, 30);
        assert_eq!(best_bad.han, 6);
        assert_eq!(best_bad.score_level, ScoreLevel::Haneman);

        // Both are Haneman, but 7 han 20 fu pays more than 6 han 30 fu
        // Actually both pay the same (Haneman flat rate), but han count differs
        // The inference should still prefer 7 han for display purposes
        assert!(
            best_good.han > best_bad.han,
            "Good inference ({} han) should have more han than bad ({} han)",
            best_good.han,
            best_bad.han
        );
    }

    #[test]
    fn test_winning_tile_no_inference_needed_for_explicit_tile() {
        // When a winning tile is explicitly specified, use it even if
        // a different tile would give a better score

        // Hand that could be Pinfu with ryanmen or not with tanki
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 6)); // Tanki - suboptimal

        let results = score_hand("334455m334455p66s", &context);
        let best = best_score(&results);

        // Should use the explicit tanki wait, giving 30 fu (no Pinfu)
        assert_eq!(best.fu.total, 30);
        assert_eq!(best.han, 5); // No Pinfu
    }

    #[test]
    fn test_winning_tile_affects_wait_fu() {
        // Test that different waits give different fu values
        // Hand: 234567m234567p22s
        // Win on 2s: tanki (pair) wait = 2 fu for wait
        // Win on 4m: ryanmen wait = 0 fu for wait
        // Win on 5m: kanchan wait = 2 fu for wait (if 3-5 interpretation)

        // Tanki wait on 2s
        let context_tanki = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 2));
        let results_tanki = score_hand("234567m234567p22s", &context_tanki);
        let best_tanki = best_score(&results_tanki);

        // Ryanmen wait on 7m (completing 567m, waiting on 4-7)
        let context_ryanmen = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 7));
        let results_ryanmen = score_hand("234567m234567p22s", &context_ryanmen);
        let best_ryanmen = best_score(&results_ryanmen);

        // Both should be valid, but tanki has +2 fu for wait
        // Ryanmen with all sequences and non-yakuhai pair = Pinfu eligible
        // Tanki = not Pinfu eligible

        // Ryanmen should give Pinfu (lower fu, higher han)
        assert!(
            best_ryanmen.han >= best_tanki.han,
            "Ryanmen should have >= han ({} vs {})",
            best_ryanmen.han,
            best_tanki.han
        );
    }

    #[test]
    fn test_winning_tile_chinitsu_ryanpeikou_best_inference() {
        // Full flush Ryanpeikou: 22334455667788s
        // Multiple possible winning tiles, but only some give Pinfu
        // Win on 2s or 8s: ryanmen wait (Pinfu applies)
        // Win on 5s: could be kanchan or ryanmen depending on interpretation

        // Ryanmen on 8s
        let context_8s = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 8));
        let results_8s = score_hand("22334455667788s", &context_8s);
        let best_8s = best_score(&results_8s);

        // Ryanmen on 2s
        let context_2s = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 2));
        let results_2s = score_hand("22334455667788s", &context_2s);
        let best_2s = best_score(&results_2s);

        // Both should give Pinfu + Ryanpeikou + Chinitsu + Menzen Tsumo + Tanyao
        // = 1 + 3 + 6 + 1 + 1 = 12 han, 20 fu
        assert_eq!(best_8s.fu.total, 20, "8s ryanmen should give 20 fu");
        assert_eq!(best_2s.fu.total, 20, "2s ryanmen should give 20 fu");
        assert_eq!(best_8s.han, best_2s.han, "Both should have same han");

        // Should be Sanbaiman (11-12 han)
        assert_eq!(best_8s.score_level, ScoreLevel::Sanbaiman);
    }

    #[test]
    fn test_winning_tile_with_no_winning_tile_set() {
        // When no winning tile is set, Pinfu cannot be determined
        // This tests the baseline behavior that the inference feature improves upon

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South);
        // Note: No with_winning_tile call

        let results = score_hand("334455m334455p66s", &context);
        let best = best_score(&results);

        // Without winning tile, Pinfu cannot be awarded (can't verify ryanmen wait)
        // Should still get: Menzen Tsumo + Tanyao + Ryanpeikou = 5 han
        // Fu will include tsumo bonus since Pinfu isn't awarded
        assert_eq!(
            best.han, 5,
            "Without winning tile, should get 5 han (no Pinfu)"
        );
        assert_eq!(best.fu.total, 30, "Without winning tile, should get 30 fu");
    }

    #[test]
    fn test_winning_tile_inference_with_dora() {
        // Test that winning tile inference works correctly with dora
        // The winning tile itself might be dora

        // Hand: 234567m234567p22s
        // With dora indicator 6m, the dora tile is 7m
        // 234567m = 234 + 567 sequences, one 7m
        // Yaku: Menzen Tsumo + Pinfu + Tanyao + Dora 1 = 1 + 1 + 1 + 1 = 4 han

        let context_7m = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 7)) // Ryanmen wait
            .with_dora(vec![Tile::suited(Suit::Man, 6)]); // 7m is dora

        let results = score_hand("234567m234567p22s", &context_7m);
        let best = best_score(&results);

        // Should include dora from the 7m tiles
        // Yaku: Menzen Tsumo (1) + Pinfu (1) + Tanyao (1) + Dora 1 = 4 han
        assert!(
            best.han >= 4,
            "Should have at least 4 han with dora, got {}",
            best.han
        );
        assert_eq!(best.fu.total, 20, "Should have 20 fu with Pinfu");
    }

    #[test]
    fn test_winning_tile_shanpon_vs_ryanmen_ambiguous() {
        // Hand: 111222333m456p77s
        // This hand has triplets, so Pinfu is impossible regardless of wait
        // But it tests that winning tile affects other scoring aspects

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 1));

        let results = score_hand("111222333m456p77s", &context);
        let best = best_score(&results);

        // Has triplets, so no Pinfu
        // Should have: Menzen Tsumo + Sanankou (possibly) + other yaku
        assert!(
            best.fu.total >= 30,
            "Should have at least 30 fu with triplets"
        );
    }

    #[test]
    fn test_winning_tile_seven_pairs_always_tanki() {
        // Chiitoitsu (seven pairs) is always tanki wait, so winning tile
        // doesn't affect whether it's tanki - it's always tanki

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 1));

        let results = score_hand("1133m2255p4477s11z", &context);

        // Find the Chiitoitsu interpretation
        let chiitoitsu_result = results.iter().find(|r| r.fu.total == 25);
        assert!(
            chiitoitsu_result.is_some(),
            "Should have a Chiitoitsu interpretation with 25 fu"
        );

        let chii = chiitoitsu_result.unwrap();
        assert_eq!(chii.fu.total, 25, "Chiitoitsu always has 25 fu");
    }

    #[test]
    fn test_winning_tile_inference_prefers_higher_payment() {
        // When two interpretations tie on han, prefer lower fu (but both rounded same)
        // When payment is equal, prefer higher han

        // This tests the scoring comparison logic that inference relies on
        // 3m is a ryanmen wait (Pinfu applies), 6s is a tanki wait (no Pinfu)
        let context_good = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 3)); // Ryanmen
        let results_good = score_hand("334455m334455p66s", &context_good);
        let best_good = best_score(&results_good);

        let context_bad = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 6)); // Tanki
        let results_bad = score_hand("334455m334455p66s", &context_bad);
        let best_bad = best_score(&results_bad);

        // The "good" interpretation should be preferred (more han, more payment)
        // Ryanmen: 6 han 20 fu (Haneman = 12000)
        // Tanki: 5 han 30 fu (Mangan = 8000)
        assert!(
            best_good.payment.total > best_bad.payment.total,
            "Good interpretation ({} points) should pay more than bad ({} points)",
            best_good.payment.total,
            best_bad.payment.total
        );
        assert!(
            best_good.han > best_bad.han,
            "Good interpretation ({} han) should have more han than bad ({} han)",
            best_good.han,
            best_bad.han
        );
    }

    #[test]
    fn test_winning_tile_kanchan_vs_ryanmen() {
        // Test that kanchan wait (middle tile) doesn't give Pinfu
        // Hand: 334455m334455p66s
        // Win on 4m: kanchan wait (waiting for middle of 345)
        // Win on 3m: ryanmen wait (waiting for 2-5 on 345)

        let context_kanchan = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 4)); // Kanchan
        let results_kanchan = score_hand("334455m334455p66s", &context_kanchan);
        let best_kanchan = best_score(&results_kanchan);

        let context_ryanmen = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 3)); // Ryanmen
        let results_ryanmen = score_hand("334455m334455p66s", &context_ryanmen);
        let best_ryanmen = best_score(&results_ryanmen);

        // Kanchan should NOT get Pinfu
        assert_eq!(best_kanchan.fu.total, 30, "Kanchan should give 30 fu");
        assert_eq!(best_kanchan.han, 5, "Kanchan should give 5 han (no Pinfu)");

        // Ryanmen should get Pinfu
        assert_eq!(best_ryanmen.fu.total, 20, "Ryanmen should give 20 fu");
        assert_eq!(
            best_ryanmen.han, 6,
            "Ryanmen should give 6 han (with Pinfu)"
        );
    }

    #[test]
    fn test_all_wait_types_for_ryanpeikou_hand() {
        // Comprehensive test of all valid winning tiles for 334455m334455p66s
        // This documents which tiles give which wait types

        let hand = "334455m334455p66s";

        // Ryanmen waits (Pinfu applies): 3m, 5m, 3p, 5p
        for tile in [
            Tile::suited(Suit::Man, 3),
            Tile::suited(Suit::Man, 5),
            Tile::suited(Suit::Pin, 3),
            Tile::suited(Suit::Pin, 5),
        ] {
            let context =
                GameContext::new(WinType::Tsumo, Honor::East, Honor::South).with_winning_tile(tile);
            let results = score_hand(hand, &context);
            let best = best_score(&results);
            assert_eq!(
                best.fu.total, 20,
                "Tile {:?} should be ryanmen (20 fu), got {} fu",
                tile, best.fu.total
            );
            assert_eq!(
                best.han, 6,
                "Tile {:?} should give 6 han with Pinfu, got {} han",
                tile, best.han
            );
        }

        // Kanchan waits (no Pinfu): 4m, 4p
        for tile in [Tile::suited(Suit::Man, 4), Tile::suited(Suit::Pin, 4)] {
            let context =
                GameContext::new(WinType::Tsumo, Honor::East, Honor::South).with_winning_tile(tile);
            let results = score_hand(hand, &context);
            let best = best_score(&results);
            assert_eq!(
                best.fu.total, 30,
                "Tile {:?} should be kanchan (30 fu), got {} fu",
                tile, best.fu.total
            );
            assert_eq!(
                best.han, 5,
                "Tile {:?} should give 5 han without Pinfu, got {} han",
                tile, best.han
            );
        }

        // Tanki wait (no Pinfu): 6s
        let tile = Tile::suited(Suit::Sou, 6);
        let context =
            GameContext::new(WinType::Tsumo, Honor::East, Honor::South).with_winning_tile(tile);
        let results = score_hand(hand, &context);
        let best = best_score(&results);
        assert_eq!(
            best.fu.total, 30,
            "Tile {:?} should be tanki (30 fu), got {} fu",
            tile, best.fu.total
        );
        assert_eq!(
            best.han, 5,
            "Tile {:?} should give 5 han without Pinfu, got {} han",
            tile, best.han
        );
    }

    #[test]
    fn test_inference_should_maximize_score_not_just_han() {
        // Edge case: ensure inference prefers higher payment, not just higher han
        // In most cases more han = more payment, but verify the logic is correct
        //
        // Hand: 234567m234567p22s
        // All winning tiles (2m, 3m, 4m, 5m, 6m, 7m, 2p, ..., 2s) should be tested
        // and the one with highest payment should be selected

        // Test that ryanmen wins over tanki when both are valid
        let context_ryanmen = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 7)); // Ryanmen on 567
        let results_ryanmen = score_hand("234567m234567p22s", &context_ryanmen);
        let best_ryanmen = best_score(&results_ryanmen);

        let context_tanki = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 2)); // Tanki on pair
        let results_tanki = score_hand("234567m234567p22s", &context_tanki);
        let best_tanki = best_score(&results_tanki);

        // Ryanmen should give Pinfu and thus higher payment
        assert!(
            best_ryanmen.payment.total >= best_tanki.payment.total,
            "Ryanmen ({}) should pay >= tanki ({})",
            best_ryanmen.payment.total,
            best_tanki.payment.total
        );
    }

    #[test]
    fn test_inference_handles_hand_with_only_tanki_wait() {
        // Hand that can ONLY be completed with tanki wait (seven pairs)
        // Chiitoitsu always has tanki wait, no ryanmen possible
        //
        // Hand: 1133m2255p4477s11z
        // Only valid wait is tanki on one of the pairs

        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 1));
        let results = score_hand("1133m2255p4477s11z", &context);

        // Should find Chiitoitsu interpretation
        let has_chiitoitsu = results.iter().any(|r| r.fu.total == 25);
        assert!(has_chiitoitsu, "Should have Chiitoitsu interpretation");
    }

    #[test]
    fn test_inference_with_multiple_suits_and_honors() {
        // More complex hand with honors to test inference doesn't break
        // Hand: 123m456p789s11122z
        // Multiple possible winning tiles, but some give yakuhai bonus

        let context_1z = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_winning_tile(Tile::honor(Honor::East)); // East wind triplet
        let results_1z = score_hand("123m456p789s11122z", &context_1z);
        let best_1z = best_score(&results_1z);

        // Should have double yakuhai (round + seat wind)
        assert!(
            best_1z.han >= 3,
            "Should have at least 3 han with double wind yakuhai"
        );
    }

    #[test]
    fn test_inference_ron_vs_tsumo_different_optimal() {
        // The optimal winning tile might differ between ron and tsumo
        // because of menzen tsumo bonus and fu differences
        //
        // Hand: 234567m234567p22s
        // Ron: Pinfu only (no menzen tsumo)
        // Tsumo: Pinfu + Menzen Tsumo

        let context_ron = GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 7)); // Ryanmen
        let results_ron = score_hand("234567m234567p22s", &context_ron);
        let best_ron = best_score(&results_ron);

        let context_tsumo = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 7)); // Ryanmen
        let results_tsumo = score_hand("234567m234567p22s", &context_tsumo);
        let best_tsumo = best_score(&results_tsumo);

        // Tsumo should have one more han (menzen tsumo)
        assert_eq!(
            best_tsumo.han,
            best_ron.han + 1,
            "Tsumo ({}) should have 1 more han than ron ({})",
            best_tsumo.han,
            best_ron.han
        );
    }

    #[test]
    fn test_inference_preserves_dora_count() {
        // Verify that inference doesn't affect dora counting
        // The winning tile might itself be dora
        //
        // Hand: 234567m234567p22s with dora indicator 1s (2s is dora)
        // Winning on 2s adds dora, but 2s is tanki (no Pinfu)
        // Winning on 7m is ryanmen (Pinfu) but no dora from win tile

        let context_dora_win = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Sou, 2))
            .with_dora(vec![Tile::suited(Suit::Sou, 1)]); // 2s is dora
        let results_dora = score_hand("234567m234567p22s", &context_dora_win);
        let best_dora = best_score(&results_dora);

        let context_no_dora_win = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 7))
            .with_dora(vec![Tile::suited(Suit::Sou, 1)]); // 2s is dora
        let results_no_dora = score_hand("234567m234567p22s", &context_no_dora_win);
        let best_no_dora = best_score(&results_no_dora);

        // Both should have 2 dora (the pair 22s)
        // But the tanki win has +2 fu for wait, no Pinfu
        // The ryanmen win has Pinfu, 20 fu
        //
        // Dora win: Menzen Tsumo (1) + Tanyao (1) + Dora 2 = 4 han, 30 fu
        // No dora from win: Menzen Tsumo (1) + Pinfu (1) + Tanyao (1) + Dora 2 = 5 han, 20 fu
        //
        // The ryanmen (no dora from win tile) should be better overall
        assert!(
            best_no_dora.payment.total >= best_dora.payment.total,
            "Ryanmen+Pinfu ({}) should pay >= tanki+dora ({}) due to higher han",
            best_no_dora.payment.total,
            best_dora.payment.total
        );
    }
}
