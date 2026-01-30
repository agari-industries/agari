//! WebAssembly bindings for the Agari Riichi Mahjong scoring engine.
//!
//! This crate provides JavaScript-friendly wrappers around the core Agari library,
//! allowing it to be used in web applications via WebAssembly.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use agari::context::{GameContext, WinType};
use agari::hand::{HandStructure, decompose_hand, decompose_hand_with_melds};
use agari::parse::TileCounts;
use agari::parse::{parse_hand_with_aka, to_counts};
use agari::scoring::{ScoringResult, calculate_score};
use agari::shanten::{ShantenResult, UkeireResult, calculate_shanten_with_melds, calculate_ukeire};
use agari::tile::{Honor, Tile};
use agari::yaku::{Yaku, YakuResult, detect_yaku_with_context};

/// Initialize panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn init() {
    // Panic hook can be added later if needed
}

// ============================================================================
// Request/Response types for JavaScript interop
// ============================================================================

/// Input for scoring a hand from JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRequest {
    /// Hand string in Agari notation (e.g., "123m456p789s11122z")
    pub hand: String,
    /// Winning tile (optional, will be inferred if not provided)
    pub winning_tile: Option<String>,
    /// Whether the win was by self-draw (tsumo)
    pub is_tsumo: bool,
    /// Whether riichi was declared
    pub is_riichi: bool,
    /// Whether double riichi was declared
    pub is_double_riichi: bool,
    /// Whether ippatsu (win within one turn of riichi)
    pub is_ippatsu: bool,
    /// Round wind: "east", "south", "west", "north"
    pub round_wind: String,
    /// Seat wind: "east", "south", "west", "north"
    pub seat_wind: String,
    /// Dora indicator tiles (e.g., ["1m", "5z"])
    pub dora_indicators: Vec<String>,
    /// Ura dora indicator tiles
    pub ura_dora_indicators: Vec<String>,
    /// Whether won on the last tile (haitei/houtei)
    pub is_last_tile: bool,
    /// Whether won on kan replacement tile (rinshan)
    pub is_rinshan: bool,
    /// Whether ron on another player's added kan (chankan)
    pub is_chankan: bool,
    /// Whether tenhou (dealer first draw win)
    pub is_tenhou: bool,
    /// Whether chiihou (non-dealer first draw win)
    pub is_chiihou: bool,
}

/// Scoring result returned to JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResponse {
    pub success: bool,
    pub error: Option<String>,
    pub result: Option<ScoringOutput>,
}

/// Detailed scoring output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringOutput {
    /// List of yaku with their han values
    pub yaku: Vec<YakuInfo>,
    /// Total han (before dora)
    pub han: u8,
    /// Total fu
    pub fu: u8,
    /// Dora breakdown
    pub dora: DoraInfo,
    /// Total han including dora
    pub total_han: u8,
    /// Score level name (e.g., "Mangan", "Haneman")
    pub score_level: String,
    /// Payment information
    pub payment: PaymentInfo,
    /// Whether the player is dealer
    pub is_dealer: bool,
    /// Whether this is a counted yakuman (13+ han)
    pub is_counted_yakuman: bool,
    /// Fu breakdown for display
    pub fu_breakdown: FuBreakdownInfo,
    /// Hand structure description
    pub hand_structure: String,
    /// The inferred winning tile (if not explicitly provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inferred_winning_tile: Option<String>,
}

/// Information about a single yaku
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YakuInfo {
    pub name: String,
    pub han: u8,
    pub is_yakuman: bool,
}

/// Dora count breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoraInfo {
    pub regular: u8,
    pub ura: u8,
    pub aka: u8,
    pub total: u8,
}

/// Payment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInfo {
    pub total: u32,
    pub from_discarder: Option<u32>,
    pub from_dealer: Option<u32>,
    pub from_non_dealer: Option<u32>,
}

/// Fu breakdown for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuBreakdownInfo {
    pub base: u8,
    pub menzen_ron: u8,
    pub tsumo: u8,
    pub melds: u8,
    pub pair: u8,
    pub wait: u8,
    pub raw_total: u8,
    pub rounded: u8,
}

/// Shanten calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShantenResponse {
    pub success: bool,
    pub error: Option<String>,
    pub shanten: Option<i8>,
    pub best_type: Option<String>,
    pub description: Option<String>,
}

/// Ukeire (tile acceptance) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UkeireResponse {
    pub success: bool,
    pub error: Option<String>,
    pub shanten: Option<i8>,
    pub tiles: Option<Vec<UkeireTileInfo>>,
    pub total_count: Option<u8>,
}

/// Single tile in ukeire result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UkeireTileInfo {
    pub tile: String,
    pub available: u8,
}

// ============================================================================
// WASM-exported functions
// ============================================================================

/// Score a mahjong hand
///
/// Takes a JSON-serialized ScoreRequest and returns a JSON-serialized ScoreResponse
#[wasm_bindgen]
pub fn score_hand(request_js: JsValue) -> JsValue {
    let request: ScoreRequest = match serde_wasm_bindgen::from_value(request_js) {
        Ok(r) => r,
        Err(e) => {
            return serde_wasm_bindgen::to_value(&ScoreResponse {
                success: false,
                error: Some(format!("Failed to parse request: {}", e)),
                result: None,
            })
            .unwrap();
        }
    };

    match score_hand_internal(&request) {
        Ok(output) => serde_wasm_bindgen::to_value(&ScoreResponse {
            success: true,
            error: None,
            result: Some(output),
        })
        .unwrap(),
        Err(e) => serde_wasm_bindgen::to_value(&ScoreResponse {
            success: false,
            error: Some(e),
            result: None,
        })
        .unwrap(),
    }
}

/// Calculate shanten for a hand
#[wasm_bindgen]
pub fn calculate_shanten_js(hand: &str) -> JsValue {
    match calculate_shanten_internal(hand) {
        Ok((result, desc)) => serde_wasm_bindgen::to_value(&ShantenResponse {
            success: true,
            error: None,
            shanten: Some(result.shanten),
            best_type: Some(format!("{:?}", result.best_type)),
            description: Some(desc),
        })
        .unwrap(),
        Err(e) => serde_wasm_bindgen::to_value(&ShantenResponse {
            success: false,
            error: Some(e),
            shanten: None,
            best_type: None,
            description: None,
        })
        .unwrap(),
    }
}

/// Calculate ukeire (tile acceptance) for a hand
#[wasm_bindgen]
pub fn calculate_ukeire_js(hand: &str) -> JsValue {
    match calculate_ukeire_internal(hand) {
        Ok(result) => {
            let tiles: Vec<UkeireTileInfo> = result
                .tiles
                .iter()
                .map(|t| UkeireTileInfo {
                    tile: t.tile.to_string(),
                    available: t.available,
                })
                .collect();

            serde_wasm_bindgen::to_value(&UkeireResponse {
                success: true,
                error: None,
                shanten: Some(result.shanten),
                tiles: Some(tiles),
                total_count: Some(result.total_count),
            })
            .unwrap()
        }
        Err(e) => serde_wasm_bindgen::to_value(&UkeireResponse {
            success: false,
            error: Some(e),
            shanten: None,
            tiles: None,
            total_count: None,
        })
        .unwrap(),
    }
}

/// Validate a hand string without scoring
#[wasm_bindgen]
pub fn validate_hand(hand: &str) -> JsValue {
    match parse_hand_with_aka(hand) {
        Ok(_) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": true,
            "error": null
        }))
        .unwrap(),
        Err(e) => serde_wasm_bindgen::to_value(&serde_json::json!({
            "valid": false,
            "error": e.to_string()
        }))
        .unwrap(),
    }
}

// ============================================================================
// Internal implementation functions
// ============================================================================

fn score_hand_internal(request: &ScoreRequest) -> Result<ScoringOutput, String> {
    // Parse the hand
    let parsed = parse_hand_with_aka(&request.hand).map_err(|e| e.to_string())?;
    let counts = to_counts(&parsed.tiles);

    // For dora counting, we need ALL tiles including those in called melds
    let all_tiles_counts = {
        let mut all_tiles = parsed.tiles.clone();
        for called_meld in &parsed.called_melds {
            all_tiles.extend(&called_meld.tiles);
        }
        to_counts(&all_tiles)
    };

    // Parse winds
    let round_wind = parse_wind(&request.round_wind)?;
    let seat_wind = parse_wind(&request.seat_wind)?;

    // Determine win type
    let win_type = if request.is_tsumo {
        WinType::Tsumo
    } else {
        WinType::Ron
    };

    // Check if hand has open melds
    let has_open_melds = parsed.called_melds.iter().any(|m| m.meld.is_open());

    // Build game context
    let mut context = GameContext::new(win_type, round_wind, seat_wind);

    if has_open_melds {
        context = context.open();
    }

    if request.is_riichi {
        context = context.riichi();
    }
    if request.is_double_riichi {
        context = context.double_riichi();
    }
    if request.is_ippatsu {
        context = context.ippatsu();
    }
    if request.is_last_tile {
        context = context.last_tile();
    }
    if request.is_rinshan {
        context = context.rinshan();
    }
    if request.is_chankan {
        context = context.chankan();
    }
    if request.is_tenhou {
        context = context.tenhou();
    }
    if request.is_chiihou {
        context = context.chiihou();
    }

    // Parse dora indicators
    let dora_indicators = parse_tile_list(&request.dora_indicators)?;
    let ura_dora_indicators = parse_tile_list(&request.ura_dora_indicators)?;

    context = context.with_dora(dora_indicators);
    context = context.with_ura_dora(ura_dora_indicators);
    context = context.with_aka(parsed.aka_count);

    // Parse winning tile if provided, otherwise we'll infer it
    let explicit_winning_tile = if let Some(ref wt) = request.winning_tile {
        let tile = parse_single_tile(wt)?;
        context = context.with_winning_tile(tile);
        true
    } else {
        false
    };

    // Decompose the hand
    let structures = if parsed.called_melds.is_empty() {
        decompose_hand(&counts)
    } else {
        let melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();
        decompose_hand_with_melds(&counts, &melds)
    };

    if structures.is_empty() {
        return Err("No valid hand structure found".to_string());
    }

    // If no winning tile was specified, infer the best one by trying all unique tiles
    let (best, context) = if explicit_winning_tile {
        // Use the specified winning tile - score each structure interpretation
        let mut best: Option<(HandStructure, YakuResult, ScoringResult)> = None;

        for structure in &structures {
            let yaku = detect_yaku_with_context(structure, &all_tiles_counts, &context);

            // Skip interpretations with no yaku
            if yaku.yaku_list.is_empty() {
                continue;
            }

            let score = calculate_score(structure, &yaku, &context);

            let is_better = match &best {
                None => true,
                Some((_, _, best_score)) => {
                    score.payment.total > best_score.payment.total
                        || (score.payment.total == best_score.payment.total
                            && score.han > best_score.han)
                }
            };

            if is_better {
                best = Some((structure.clone(), yaku, score));
            }
        }
        (best, context)
    } else {
        // Infer the best winning tile by trying all unique tiles in the hand
        infer_best_winning_tile(&structures, &all_tiles_counts, context, &parsed.tiles)
    };

    let (structure, yaku, score) = best.ok_or("No valid yaku found for this hand")?;

    // Convert to output format
    let yaku_list: Vec<YakuInfo> = yaku
        .yaku_list
        .iter()
        .map(|y| YakuInfo {
            name: yaku_name(y),
            han: if context.is_open {
                y.han_open().unwrap_or(y.han())
            } else {
                y.han()
            },
            is_yakuman: y.is_yakuman(),
        })
        .collect();

    let total_han = yaku.total_han_with_dora();

    // Determine if winning tile was inferred
    let inferred_winning_tile = if !explicit_winning_tile {
        context.winning_tile.map(|t| format!("{}", t))
    } else {
        None
    };

    Ok(ScoringOutput {
        yaku: yaku_list,
        han: yaku.total_han,
        fu: score.fu.total,
        dora: DoraInfo {
            regular: yaku.regular_dora,
            ura: yaku.ura_dora,
            aka: yaku.aka_dora,
            total: yaku.dora_count,
        },
        total_han,
        score_level: score.score_level.name().to_string(),
        payment: PaymentInfo {
            total: score.payment.total,
            from_discarder: score.payment.from_discarder,
            from_dealer: score.payment.from_dealer,
            from_non_dealer: score.payment.from_non_dealer,
        },
        is_dealer: score.is_dealer,
        is_counted_yakuman: score.is_counted_yakuman,
        fu_breakdown: FuBreakdownInfo {
            base: score.fu.breakdown.base,
            menzen_ron: score.fu.breakdown.menzen_ron,
            tsumo: score.fu.breakdown.tsumo,
            melds: score.fu.breakdown.melds,
            pair: score.fu.breakdown.pair,
            wait: score.fu.breakdown.wait,
            raw_total: score.fu.breakdown.raw_total,
            rounded: score.fu.total,
        },
        hand_structure: format_structure(&structure),
        inferred_winning_tile,
    })
}

fn calculate_shanten_internal(hand: &str) -> Result<(ShantenResult, String), String> {
    let parsed = parse_hand_with_aka(hand).map_err(|e| e.to_string())?;
    let counts = to_counts(&parsed.tiles);

    // Count called melds (pon, chi, kan)
    let called_melds = parsed.called_melds.len() as u8;

    let result = calculate_shanten_with_melds(&counts, called_melds);

    let description = match result.shanten {
        -1 => "Complete hand (agari)".to_string(),
        0 => "Tenpai (ready)".to_string(),
        1 => "Iishanten (1 away from tenpai)".to_string(),
        n => format!("{}-shanten ({} away from tenpai)", n, n),
    };

    Ok((result, description))
}

fn calculate_ukeire_internal(hand: &str) -> Result<UkeireResult, String> {
    let parsed = parse_hand_with_aka(hand).map_err(|e| e.to_string())?;
    let counts = to_counts(&parsed.tiles);
    Ok(calculate_ukeire(&counts))
}

// ============================================================================
// Helper functions
// ============================================================================

fn parse_wind(s: &str) -> Result<Honor, String> {
    match s.to_lowercase().as_str() {
        "east" | "e" | "1z" => Ok(Honor::East),
        "south" | "s" | "2z" => Ok(Honor::South),
        "west" | "w" | "3z" => Ok(Honor::West),
        "north" | "n" | "4z" => Ok(Honor::North),
        _ => Err(format!("Invalid wind: {}", s)),
    }
}

fn parse_single_tile(s: &str) -> Result<Tile, String> {
    Tile::try_from(s)
}

/// Infer the best winning tile by trying all unique tiles in the hand
fn infer_best_winning_tile(
    structures: &[HandStructure],
    all_tiles_counts: &TileCounts,
    base_context: GameContext,
    tiles: &[Tile],
) -> (
    Option<(HandStructure, YakuResult, ScoringResult)>,
    GameContext,
) {
    // Get unique tiles in the hand
    let unique_tiles: HashSet<Tile> = tiles.iter().copied().collect();

    let mut best: Option<(HandStructure, YakuResult, ScoringResult)> = None;
    let mut best_context = base_context.clone();
    let mut best_score: Option<(u32, u8, u8)> = None; // (payment, han, 255-fu for comparison)

    for winning_tile in unique_tiles {
        let context = base_context.clone().with_winning_tile(winning_tile);

        for structure in structures {
            let yaku_result = detect_yaku_with_context(structure, all_tiles_counts, &context);

            // Skip interpretations with no yaku
            if yaku_result.yaku_list.is_empty() {
                continue;
            }

            let score = calculate_score(structure, &yaku_result, &context);

            // Compare: prefer higher payment, then higher han, then lower fu
            let current = (score.payment.total, score.han, 255 - score.fu.total);

            let is_better = match best_score {
                None => true,
                Some(best) => current > best,
            };

            if is_better {
                best_score = Some(current);
                best_context = context.clone();
                best = Some((structure.clone(), yaku_result, score));
            }
        }
    }

    (best, best_context)
}

fn parse_tile_list(tiles: &[String]) -> Result<Vec<agari::tile::Tile>, String> {
    tiles.iter().map(|s| parse_single_tile(s)).collect()
}

fn format_structure(structure: &HandStructure) -> String {
    match structure {
        HandStructure::Standard { melds, pair } => {
            let meld_strs: Vec<String> = melds.iter().map(|m| format!("{:?}", m)).collect();
            format!("Standard: {} + pair of {}", meld_strs.join(", "), pair)
        }
        HandStructure::Chiitoitsu { pairs } => {
            let pair_strs: Vec<String> = pairs.iter().map(|p| p.to_string()).collect();
            format!("Chiitoitsu: {}", pair_strs.join(", "))
        }
        HandStructure::Kokushi { pair } => {
            format!("Kokushi Musou (pair: {})", pair)
        }
    }
}

fn yaku_name(yaku: &Yaku) -> String {
    match yaku {
        Yaku::Riichi => "Riichi".to_string(),
        Yaku::Ippatsu => "Ippatsu".to_string(),
        Yaku::MenzenTsumo => "Menzen Tsumo".to_string(),
        Yaku::Tanyao => "Tanyao".to_string(),
        Yaku::Pinfu => "Pinfu".to_string(),
        Yaku::Iipeikou => "Iipeikou".to_string(),
        Yaku::Yakuhai(honor) => {
            let name = match honor {
                Honor::East => "East",
                Honor::South => "South",
                Honor::West => "West",
                Honor::North => "North",
                Honor::White => "Haku",
                Honor::Green => "Hatsu",
                Honor::Red => "Chun",
            };
            format!("Yakuhai ({})", name)
        }
        Yaku::RinshanKaihou => "Rinshan Kaihou".to_string(),
        Yaku::Chankan => "Chankan".to_string(),
        Yaku::HaiteiRaoyue => "Haitei Raoyue".to_string(),
        Yaku::HouteiRaoyui => "Houtei Raoyui".to_string(),
        Yaku::DoubleRiichi => "Double Riichi".to_string(),
        Yaku::Toitoi => "Toitoi".to_string(),
        Yaku::SanshokuDoujun => "Sanshoku Doujun".to_string(),
        Yaku::SanshokuDoukou => "Sanshoku Doukou".to_string(),
        Yaku::Ittsu => "Ittsu".to_string(),
        Yaku::Chiitoitsu => "Chiitoitsu".to_string(),
        Yaku::Chanta => "Chanta".to_string(),
        Yaku::SanAnkou => "San Ankou".to_string(),
        Yaku::SanKantsu => "San Kantsu".to_string(),
        Yaku::Honroutou => "Honroutou".to_string(),
        Yaku::Shousangen => "Shousangen".to_string(),
        Yaku::Honitsu => "Honitsu".to_string(),
        Yaku::Junchan => "Junchan".to_string(),
        Yaku::Ryanpeikou => "Ryanpeikou".to_string(),
        Yaku::Chinitsu => "Chinitsu".to_string(),
        Yaku::Tenhou => "Tenhou".to_string(),
        Yaku::Chiihou => "Chiihou".to_string(),
        Yaku::KokushiMusou => "Kokushi Musou".to_string(),
        Yaku::Suuankou => "Suuankou".to_string(),
        Yaku::Daisangen => "Daisangen".to_string(),
        Yaku::Shousuushii => "Shousuushii".to_string(),
        Yaku::Daisuushii => "Daisuushii".to_string(),
        Yaku::Tsuuiisou => "Tsuuiisou".to_string(),
        Yaku::Chinroutou => "Chinroutou".to_string(),
        Yaku::Ryuuiisou => "Ryuuiisou".to_string(),
        Yaku::ChuurenPoutou => "Chuuren Poutou".to_string(),
        Yaku::Kokushi13Wait => "Kokushi 13-Wait".to_string(),
        Yaku::SuuankouTanki => "Suuankou Tanki".to_string(),
        Yaku::JunseiChuurenPoutou => "Junsei Chuuren Poutou".to_string(),
    }
}

// ============================================================================
// Tests - WASM-specific functionality only
// (Scoring/yaku/shanten logic is tested in agari-core)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use agari::shanten::ShantenType;

    // ========================================================================
    // Helper functions for tests
    // ========================================================================

    fn make_request(hand: &str) -> ScoreRequest {
        ScoreRequest {
            hand: hand.to_string(),
            winning_tile: None,
            is_tsumo: false,
            is_riichi: false,
            is_double_riichi: false,
            is_ippatsu: false,
            round_wind: "east".to_string(),
            seat_wind: "east".to_string(),
            dora_indicators: vec![],
            ura_dora_indicators: vec![],
            is_last_tile: false,
            is_rinshan: false,
            is_chankan: false,
            is_tenhou: false,
            is_chiihou: false,
        }
    }

    // ========================================================================
    // Request/Response integration tests
    // (Tests the WASM binding layer, not the scoring logic)
    // ========================================================================

    #[test]
    fn test_score_request_success() {
        let mut request = make_request("234m345p456s678m66p");
        request.winning_tile = Some("6p".to_string());

        let result = score_hand_internal(&request);

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.payment.total > 0);
        assert!(!output.yaku.is_empty());
    }

    #[test]
    fn test_score_request_invalid_hand() {
        let request = make_request("123m456p"); // Too few tiles

        let result = score_hand_internal(&request);

        assert!(result.is_err());
    }

    #[test]
    fn test_score_request_invalid_wind() {
        let mut request = make_request("234m345p456s678m66p");
        request.winning_tile = Some("6p".to_string());
        request.round_wind = "invalid".to_string();

        let result = score_hand_internal(&request);

        assert!(result.is_err());
    }

    #[test]
    fn test_score_request_with_dora_indicators() {
        let mut request = make_request("234m345p456s678m66p");
        request.winning_tile = Some("6p".to_string());
        request.dora_indicators = vec!["5p".to_string()]; // 6p is dora

        let result = score_hand_internal(&request).unwrap();

        assert_eq!(result.dora.regular, 2); // Two 6p tiles
    }

    #[test]
    fn test_score_request_with_ura_dora() {
        let mut request = make_request("234m345p456s678m66p");
        request.winning_tile = Some("6p".to_string());
        request.is_riichi = true;
        request.ura_dora_indicators = vec!["5p".to_string()];

        let result = score_hand_internal(&request).unwrap();

        assert_eq!(result.dora.ura, 2);
    }

    #[test]
    fn test_score_request_with_open_melds() {
        let mut request = make_request("234m55p(111z)(222z)(333z)");
        request.winning_tile = Some("5p".to_string());

        let result = score_hand_internal(&request);

        assert!(result.is_ok());
    }

    // ========================================================================
    // Inferred winning tile tests (WASM-specific feature)
    // ========================================================================

    #[test]
    fn test_inferred_winning_tile_set_when_not_provided() {
        let request = make_request("123m456p789s234m55s");

        let result = score_hand_internal(&request).unwrap();

        assert!(result.inferred_winning_tile.is_some());
    }

    #[test]
    fn test_inferred_winning_tile_none_when_provided() {
        let mut request = make_request("234m345p456s678m66p");
        request.winning_tile = Some("6p".to_string());

        let result = score_hand_internal(&request).unwrap();

        assert!(result.inferred_winning_tile.is_none());
    }

    #[test]
    fn test_inferred_winning_tile_maximizes_score() {
        // Hand where ryanmen wait gives pinfu (higher score than tanki)
        let request = make_request("123m456p789s234m55s");

        let result = score_hand_internal(&request).unwrap();

        // Should infer the tile that gives pinfu
        assert!(result.yaku.iter().any(|y| y.name == "Pinfu"));
    }

    // ========================================================================
    // Shanten API tests
    // ========================================================================

    #[test]
    fn test_shanten_api_success() {
        let (result, desc) = calculate_shanten_internal("123m456p789s234m55p").unwrap();

        assert_eq!(result.shanten, -1);
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_shanten_api_with_melds() {
        let (result, _) = calculate_shanten_internal("123m5p(111z)(222z)(333z)").unwrap();

        assert_eq!(result.shanten, 0);
    }

    #[test]
    fn test_shanten_api_returns_best_type() {
        let (result, _) = calculate_shanten_internal("1133557799m11p3s").unwrap();

        assert_eq!(result.best_type, ShantenType::Chiitoitsu);
    }

    #[test]
    fn test_shanten_api_invalid_hand() {
        let result = calculate_shanten_internal("invalid");

        assert!(result.is_err());
    }

    // ========================================================================
    // Ukeire API tests
    // ========================================================================

    #[test]
    fn test_ukeire_api_success() {
        let result = calculate_ukeire_internal("123m456p789s234m5p").unwrap();

        assert_eq!(result.shanten, 0);
        assert!(!result.tiles.is_empty());
        assert!(result.total_count > 0);
    }

    #[test]
    fn test_ukeire_api_complete_hand() {
        let result = calculate_ukeire_internal("123m456p789s234m55p").unwrap();

        assert_eq!(result.shanten, -1);
        assert!(result.tiles.is_empty());
    }

    // ========================================================================
    // parse_wind tests (WASM-specific helper)
    // ========================================================================

    #[test]
    fn test_parse_wind_full_names() {
        assert_eq!(parse_wind("east").unwrap(), Honor::East);
        assert_eq!(parse_wind("south").unwrap(), Honor::South);
        assert_eq!(parse_wind("west").unwrap(), Honor::West);
        assert_eq!(parse_wind("north").unwrap(), Honor::North);
    }

    #[test]
    fn test_parse_wind_abbreviations() {
        assert_eq!(parse_wind("e").unwrap(), Honor::East);
        assert_eq!(parse_wind("s").unwrap(), Honor::South);
        assert_eq!(parse_wind("w").unwrap(), Honor::West);
        assert_eq!(parse_wind("n").unwrap(), Honor::North);
    }

    #[test]
    fn test_parse_wind_tile_notation() {
        assert_eq!(parse_wind("1z").unwrap(), Honor::East);
        assert_eq!(parse_wind("2z").unwrap(), Honor::South);
        assert_eq!(parse_wind("3z").unwrap(), Honor::West);
        assert_eq!(parse_wind("4z").unwrap(), Honor::North);
    }

    #[test]
    fn test_parse_wind_case_insensitive() {
        assert_eq!(parse_wind("EAST").unwrap(), Honor::East);
        assert_eq!(parse_wind("East").unwrap(), Honor::East);
    }

    #[test]
    fn test_parse_wind_invalid() {
        assert!(parse_wind("invalid").is_err());
        assert!(parse_wind("5z").is_err());
    }

    // ========================================================================
    // yaku_name tests (WASM-specific display helper)
    // ========================================================================

    #[test]
    fn test_yaku_name_basic() {
        assert_eq!(yaku_name(&Yaku::Riichi), "Riichi");
        assert_eq!(yaku_name(&Yaku::Tanyao), "Tanyao");
        assert_eq!(yaku_name(&Yaku::Pinfu), "Pinfu");
    }

    #[test]
    fn test_yaku_name_yakuhai_winds() {
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::East)), "Yakuhai (East)");
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::South)), "Yakuhai (South)");
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::West)), "Yakuhai (West)");
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::North)), "Yakuhai (North)");
    }

    #[test]
    fn test_yaku_name_yakuhai_dragons() {
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::White)), "Yakuhai (Haku)");
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::Green)), "Yakuhai (Hatsu)");
        assert_eq!(yaku_name(&Yaku::Yakuhai(Honor::Red)), "Yakuhai (Chun)");
    }

    #[test]
    fn test_yaku_name_yakuman() {
        assert_eq!(yaku_name(&Yaku::KokushiMusou), "Kokushi Musou");
        assert_eq!(yaku_name(&Yaku::Suuankou), "Suuankou");
        assert_eq!(yaku_name(&Yaku::Daisangen), "Daisangen");
    }

    // ========================================================================
    // format_structure tests (WASM-specific display helper)
    // ========================================================================

    #[test]
    fn test_format_structure_standard() {
        use agari::hand::{HandStructure, Meld};
        use agari::tile::{Suit, Tile};

        let structure = HandStructure::Standard {
            melds: vec![
                Meld::Shuntsu(Tile::suited(Suit::Man, 1), false),
                Meld::Shuntsu(Tile::suited(Suit::Pin, 4), false),
                Meld::Shuntsu(Tile::suited(Suit::Sou, 7), false),
                Meld::Shuntsu(Tile::suited(Suit::Man, 2), false),
            ],
            pair: Tile::suited(Suit::Pin, 5),
        };

        let formatted = format_structure(&structure);

        assert!(formatted.contains("Standard"));
    }

    #[test]
    fn test_format_structure_chiitoitsu() {
        use agari::hand::HandStructure;
        use agari::tile::{Suit, Tile};

        let structure = HandStructure::Chiitoitsu {
            pairs: vec![
                Tile::suited(Suit::Man, 1),
                Tile::suited(Suit::Man, 3),
                Tile::suited(Suit::Man, 5),
                Tile::suited(Suit::Man, 7),
                Tile::suited(Suit::Man, 9),
                Tile::suited(Suit::Pin, 1),
                Tile::suited(Suit::Sou, 3),
            ],
        };

        let formatted = format_structure(&structure);

        assert!(formatted.contains("Chiitoitsu"));
    }

    #[test]
    fn test_format_structure_kokushi() {
        use agari::hand::HandStructure;
        use agari::tile::{Suit, Tile};

        let structure = HandStructure::Kokushi {
            pair: Tile::suited(Suit::Man, 1),
        };

        let formatted = format_structure(&structure);

        assert!(formatted.contains("Kokushi"));
    }
}
