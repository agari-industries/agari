//! Agari - Riichi Mahjong Hand Scoring Calculator
//!
//! A command-line tool for calculating the score of a Riichi Mahjong hand.

use std::collections::HashSet;
use std::process;

use clap::Parser;
use colored::Colorize;
use serde::Serialize;

use agari::{
    context::{GameContext, WinType},
    display::{format_structure, honor_name, tile_to_ascii, tile_to_unicode},
    hand::{HandStructure, decompose_hand, decompose_hand_with_melds},
    parse::{TileCounts, parse_hand_with_aka, to_counts, validate_hand, validate_hand_with_melds},
    scoring::{ScoreLevel, ScoringResult, calculate_score},
    shanten::{ShantenType, calculate_shanten, calculate_ukeire},
    tile::{Honor, Suit, Tile},
    yaku::{Yaku, YakuResult, detect_yaku_with_context},
};

const AFTER_HELP: &str = r#"HAND FORMAT:
    Standard notation: numbers followed by suit letter
    m = Man (Characters), p = Pin (Dots), s = Sou (Bamboo), z = Honors
    Honors (numeric): 1z=East, 2z=South, 3z=West, 4z=North, 5z=White, 6z=Green, 7z=Red
    Honors (letters): e=East, s=South, w=West, n=North, wh=White, g=Green, r=Red
    Red fives: Use 0 instead of 5 (e.g., 0m = red 5-man)

    Called melds (kans, pons, chis):
    [1111m]  = Closed kan (ankan) of 1-man
    (1111m)  = Open kan (daiminkan) of 1-man
    (111m)   = Open triplet (pon) of 1-man
    (123m)   = Open sequence (chi) of 1-2-3 man
    (eee)    = Open triplet (pon) of East wind
    [rrrr]   = Closed kan of Red dragon

EXAMPLES:
    agari 123m456p789s11122z              Basic hand
    agari 123m456p789seeenn               Same hand with letter notation for honors
    agari 123m456p789s11122z -t           Tsumo win
    agari 123m456p789s11122z -r           With riichi
    agari 123m456p789s11122z -w 2m -t     Won on 2-man by tsumo
    agari 123m456p789seeenn -w e -t       Won on East by tsumo (letter notation)
    agari 123m456p789s11122z -d 1m        With dora indicator 1m (2m is dora)
    agari 123m456p789seeenn -d e,n        Dora indicators with letter notation
    agari 234567m234567p22s -w 5p -t      Pinfu tanyao
    agari "[1111m]222333m555p11z" -t      Hand with closed kan (15 tiles)
    agari "[1111m](2222p)345678s11z" -t   Hand with closed + open kan (16 tiles)
    agari "123m456p789s(rrr)whwh" -w wh   Open pon of Red dragon, White pair"#;

#[derive(Parser)]
#[command(name = "agari")]
#[command(about = "Riichi Mahjong Hand Scoring Calculator")]
#[command(after_help = AFTER_HELP)]
struct Args {
    /// Hand notation (e.g., 123m456p789s11122z)
    hand: String,

    /// Winning tile (e.g., 2m, 5z)
    #[arg(short = 'w', long = "win")]
    winning_tile: Option<String>,

    /// Win by self-draw (default: ron)
    #[arg(short, long)]
    tsumo: bool,

    /// Hand is open (has called tiles)
    #[arg(short, long)]
    open: bool,

    /// Riichi declared
    #[arg(short, long)]
    riichi: bool,

    /// Double riichi (first turn)
    #[arg(long)]
    double_riichi: bool,

    /// Ippatsu (win within one turn of riichi)
    #[arg(long)]
    ippatsu: bool,

    /// Round wind: e/s/w/n
    #[arg(long, default_value = "e")]
    round: String,

    /// Seat wind: e/s/w/n
    #[arg(long, default_value = "e")]
    seat: String,

    /// Dora indicators (comma-separated: 1m,5z)
    #[arg(short, long)]
    dora: Option<String>,

    /// Ura dora indicators (with riichi only)
    #[arg(long)]
    ura: Option<String>,

    /// Win on last tile (Haitei/Houtei)
    #[arg(long)]
    last_tile: bool,

    /// Win on kan replacement tile
    #[arg(long)]
    rinshan: bool,

    /// Ron on another player's added kan
    #[arg(long)]
    chankan: bool,

    /// Dealer's first draw win
    #[arg(long)]
    tenhou: bool,

    /// Non-dealer's first draw win
    #[arg(long)]
    chiihou: bool,

    /// Calculate shanten (tiles from tenpai) instead of score
    #[arg(long)]
    shanten: bool,

    /// Show ukeire (tile acceptance) with shanten
    #[arg(long)]
    ukeire: bool,

    /// Use ASCII output instead of Unicode
    #[arg(long)]
    ascii: bool,

    /// Show all possible interpretations
    #[arg(long)]
    all: bool,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

// JSON output structures
#[derive(Serialize)]
struct JsonOutput {
    hand: String,
    context: JsonContext,
    interpretations: Vec<JsonInterpretation>,
}

#[derive(Serialize)]
struct JsonContext {
    win_type: String,
    round_wind: String,
    seat_wind: String,
    is_dealer: bool,
    is_open: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    riichi: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    double_riichi: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    ippatsu: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    dora_indicators: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ura_dora_indicators: Vec<String>,
    #[serde(skip_serializing_if = "is_zero")]
    akadora: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    winning_tile: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    last_tile: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    rinshan: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    chankan: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    tenhou: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    chiihou: bool,
}

#[derive(Serialize)]
struct JsonInterpretation {
    structure: String,
    yaku: Vec<JsonYaku>,
    dora: JsonDora,
    han: u8,
    fu: u8,
    score_level: String,
    payment: JsonPayment,
    #[serde(skip_serializing_if = "Option::is_none")]
    fu_breakdown: Option<JsonFuBreakdown>,
}

#[derive(Serialize)]
struct JsonYaku {
    name: String,
    han: u8,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    yakuman: bool,
}

#[derive(Serialize)]
struct JsonDora {
    #[serde(skip_serializing_if = "is_zero")]
    regular: u8,
    #[serde(skip_serializing_if = "is_zero")]
    ura: u8,
    #[serde(skip_serializing_if = "is_zero")]
    aka: u8,
    total: u8,
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

#[derive(Serialize)]
struct JsonPayment {
    total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_discarder: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_dealer: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_non_dealer: Option<u32>,
}

#[derive(Serialize)]
struct JsonFuBreakdown {
    base: u8,
    #[serde(skip_serializing_if = "is_zero")]
    menzen_ron: u8,
    #[serde(skip_serializing_if = "is_zero")]
    tsumo: u8,
    #[serde(skip_serializing_if = "is_zero")]
    melds: u8,
    #[serde(skip_serializing_if = "is_zero")]
    pair: u8,
    #[serde(skip_serializing_if = "is_zero")]
    wait: u8,
    raw: u8,
    rounded: u8,
}

#[derive(Serialize)]
struct JsonShantenOutput {
    shanten: i8,
    description: String,
    best_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ukeire: Option<JsonUkeire>,
}

#[derive(Serialize)]
struct JsonUkeire {
    tile_count: usize,
    total_available: u8,
    tiles: Vec<JsonUkeireTile>,
}

#[derive(Serialize)]
struct JsonUkeireTile {
    tile: String,
    available: u8,
}

/// Infer the best winning tile when none is specified.
/// Tries each unique tile in the hand and returns the results with the context
/// that produces the highest score.
fn infer_best_winning_tile(
    structures: &[HandStructure],
    all_tiles_counts: &TileCounts,
    base_context: GameContext,
    tiles: &[Tile],
) -> (Vec<(HandStructure, YakuResult, ScoringResult)>, GameContext) {
    // Get unique tiles in the hand
    let unique_tiles: HashSet<Tile> = tiles.iter().copied().collect();

    let mut best_results: Vec<(HandStructure, YakuResult, ScoringResult)> = Vec::new();
    let mut best_context = base_context.clone();
    let mut best_score: Option<(u32, u8, u8)> = None; // (payment, han, -fu for comparison)

    for winning_tile in unique_tiles {
        let context = base_context.clone().with_winning_tile(winning_tile);

        for structure in structures {
            let yaku_result = detect_yaku_with_context(structure, all_tiles_counts, &context);
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
                best_results.clear();
            }

            // If this matches the best score, add to results
            if Some(current) == best_score {
                best_results.push((structure.clone(), yaku_result, score));
            }
        }
    }

    // If no results found (shouldn't happen), fall back to no winning tile
    if best_results.is_empty() {
        let results: Vec<_> = structures
            .iter()
            .map(|s| {
                let yaku_result = detect_yaku_with_context(s, all_tiles_counts, &base_context);
                let score = calculate_score(s, &yaku_result, &base_context);
                (s.clone(), yaku_result, score)
            })
            .collect();
        (results, base_context)
    } else {
        (best_results, best_context)
    }
}

fn main() {
    let args = Args::parse();

    // Configure color output
    // Respects NO_COLOR env var automatically, but --no-color flag overrides
    if args.no_color {
        colored::control::set_override(false);
    }

    // Extract arguments
    let shanten_mode = args.shanten || args.ukeire;
    let ukeire_mode = args.ukeire;
    let riichi = args.riichi || args.double_riichi;

    // Parse the hand
    let parsed = match parse_hand_with_aka(&args.hand) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "❌ Error parsing hand:".red().bold(), e);
            process::exit(1);
        }
    };

    // Check if hand has called melds (kans, pons, chis)
    let has_called_melds = !parsed.called_melds.is_empty();

    // For shanten mode, we don't require exactly 14 tiles
    // (13 tiles for tenpai calculation is common)
    if !shanten_mode {
        // Validate hand size for scoring
        if has_called_melds {
            if let Err(e) = validate_hand_with_melds(&parsed) {
                eprintln!("{} {}", "❌ Invalid hand:".red().bold(), e);
                process::exit(1);
            }
        } else {
            if let Err(e) = validate_hand(&parsed.tiles) {
                eprintln!("{} {}", "❌ Invalid hand:".red().bold(), e);
                process::exit(1);
            }
        }
    } else {
        // For shanten, allow 1-14 tiles (not counting melds)
        let tile_count = parsed.tiles.len();
        if tile_count < 1 || tile_count > 14 {
            eprintln!(
                "{} expected 1-14 tiles, got {}",
                "❌ Invalid hand:".red().bold(),
                tile_count
            );
            process::exit(1);
        }
    }

    // If hand has open melds, mark hand as open
    let has_open_melds = parsed.called_melds.iter().any(|m| m.meld.is_open());

    // Parse winds
    let round_wind = match parse_wind(&args.round) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{} {}", "❌".red().bold(), e);
            process::exit(1);
        }
    };

    let seat_wind = match parse_wind(&args.seat) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{} {}", "❌".red().bold(), e);
            process::exit(1);
        }
    };

    // Parse dora indicators
    let dora_indicators = match args.dora.as_ref().map(|s| parse_tile_list(s)).transpose() {
        Ok(d) => d.unwrap_or_default(),
        Err(e) => {
            eprintln!("{} {}", "❌ Error parsing dora:".red().bold(), e);
            process::exit(1);
        }
    };

    let ura_indicators = match args.ura.as_ref().map(|s| parse_tile_list(s)).transpose() {
        Ok(u) => u.unwrap_or_default(),
        Err(e) => {
            eprintln!("{} {}", "❌ Error parsing ura dora:".red().bold(), e);
            process::exit(1);
        }
    };

    // Check for riichi-dependent options used without riichi, and riichi with open hands
    for warning in validate_riichi_dependencies(
        riichi,
        !ura_indicators.is_empty(),
        args.ippatsu,
        has_open_melds,
        args.open,
    ) {
        eprintln!("{} {}", "⚠️  Warning:".yellow().bold(), warning);
    }

    // Parse winning tile
    let winning_tile = match args
        .winning_tile
        .as_ref()
        .map(|s| parse_single_tile(s))
        .transpose()
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{} {}", "❌ Error parsing winning tile:".red().bold(), e);
            process::exit(1);
        }
    };

    // Build game context
    let win_type = if args.tsumo {
        WinType::Tsumo
    } else {
        WinType::Ron
    };
    let mut context = GameContext::new(win_type, round_wind, seat_wind)
        .with_dora(dora_indicators)
        .with_ura_dora(ura_indicators)
        .with_aka(parsed.aka_count);

    // If winning tile is specified, use it; otherwise we'll infer it later
    let explicit_winning_tile = winning_tile;
    if let Some(wt) = explicit_winning_tile {
        context = context.with_winning_tile(wt);
    }

    if args.open || has_open_melds {
        context = context.open();
    }

    if args.double_riichi {
        context = context.double_riichi();
    } else if riichi {
        context = context.riichi();
    }

    if args.ippatsu {
        context = context.ippatsu();
    }

    if args.last_tile {
        context = context.last_tile();
    }

    if args.rinshan {
        context = context.rinshan();
    }

    if args.chankan {
        context = context.chankan();
    }

    if args.tenhou {
        context = context.tenhou();
    }

    if args.chiihou {
        context = context.chiihou();
    }

    // Convert to tile counts (for hand decomposition)
    let counts = to_counts(&parsed.tiles);

    // For dora counting, we need ALL tiles including those in called melds
    let all_tiles_counts = {
        let mut all_tiles = parsed.tiles.clone();
        for called_meld in &parsed.called_melds {
            all_tiles.extend(&called_meld.tiles);
        }
        to_counts(&all_tiles)
    };

    let use_unicode = !args.ascii;

    // Shanten mode: calculate shanten and optionally ukeire
    if shanten_mode {
        if args.json {
            print_shanten_json(&counts, ukeire_mode);
        } else {
            print_header(use_unicode);
            print_shanten(&counts, ukeire_mode, use_unicode);
            print_footer(use_unicode);
        }
        return;
    }

    // Decompose the hand
    let structures = if has_called_melds {
        // Extract the Meld objects from CalledMeld
        let called_melds: Vec<_> = parsed
            .called_melds
            .iter()
            .map(|cm| cm.meld.clone())
            .collect();
        decompose_hand_with_melds(&counts, &called_melds)
    } else {
        decompose_hand(&counts)
    };

    if structures.is_empty() {
        eprintln!(
            "{}",
            "❌ This hand has no valid winning structure.".red().bold()
        );
        process::exit(1);
    }

    // Score each decomposition
    // Note: We use all_tiles_counts for yaku detection to properly count dora
    // in called melds (pons, chis, kans)
    //
    // If no winning tile was specified, we need to infer the best one.
    // Try each unique tile in the hand and pick the one that maximizes score.
    let (mut results, context) = if explicit_winning_tile.is_none() {
        infer_best_winning_tile(&structures, &all_tiles_counts, context, &parsed.tiles)
    } else {
        let results: Vec<_> = structures
            .iter()
            .map(|s| {
                let yaku_result = detect_yaku_with_context(s, &all_tiles_counts, &context);
                let score = calculate_score(s, &yaku_result, &context);
                (s.clone(), yaku_result, score)
            })
            .collect();
        (results, context)
    };

    // Sort by score (highest first)
    // When payment is the same (e.g., both yakuman), prefer:
    // 1. Higher han (more yaku = better hand)
    // 2. Lower fu (better technique / cleaner hand)
    results.sort_by(|a, b| {
        b.2.payment
            .total
            .cmp(&a.2.payment.total)
            .then_with(|| b.2.han.cmp(&a.2.han))
            .then_with(|| a.2.fu.total.cmp(&b.2.fu.total))
    });

    // Filter to best interpretation only (unless --all)
    let results_to_show: Vec<_> = if args.all {
        results.iter().map(|(s, y, sc)| (s, y, sc)).collect()
    } else {
        results
            .iter()
            .take(1)
            .map(|(s, y, sc)| (s, y, sc))
            .collect()
    };

    // JSON output mode
    if args.json {
        let interpretations: Vec<JsonInterpretation> = results_to_show
            .iter()
            .map(|&(structure, yaku_result, score)| {
                let yaku_list: Vec<JsonYaku> = yaku_result
                    .yaku_list
                    .iter()
                    .map(|y| JsonYaku {
                        name: yaku_name(y).to_string(),
                        han: if context.is_open {
                            y.han_open().unwrap_or(0)
                        } else {
                            y.han()
                        },
                        yakuman: y.is_yakuman(),
                    })
                    .collect();

                let fu_breakdown = if score.fu.total != 25
                    && score.fu.total != 20
                    && score.fu.breakdown.raw_total > 20
                {
                    Some(JsonFuBreakdown {
                        base: 20,
                        menzen_ron: score.fu.breakdown.menzen_ron,
                        tsumo: score.fu.breakdown.tsumo,
                        melds: score.fu.breakdown.melds,
                        pair: score.fu.breakdown.pair,
                        wait: score.fu.breakdown.wait,
                        raw: score.fu.breakdown.raw_total,
                        rounded: score.fu.total,
                    })
                } else {
                    None
                };

                JsonInterpretation {
                    structure: format_structure(structure, false),
                    yaku: yaku_list,
                    dora: JsonDora {
                        regular: yaku_result.regular_dora,
                        ura: yaku_result.ura_dora,
                        aka: yaku_result.aka_dora,
                        total: yaku_result.dora_count,
                    },
                    han: score.han,
                    fu: score.fu.total,
                    score_level: if score.is_counted_yakuman {
                        "Counted Yakuman".to_string()
                    } else {
                        score.score_level.name().to_string()
                    },
                    payment: JsonPayment {
                        total: score.payment.total,
                        from_discarder: score.payment.from_discarder,
                        from_dealer: score.payment.from_dealer,
                        from_non_dealer: score.payment.from_non_dealer,
                    },
                    fu_breakdown,
                }
            })
            .collect();

        let json_context = JsonContext {
            win_type: match context.win_type {
                WinType::Tsumo => "tsumo".to_string(),
                WinType::Ron => "ron".to_string(),
            },
            round_wind: honor_name(&context.round_wind).to_string(),
            seat_wind: honor_name(&context.seat_wind).to_string(),
            is_dealer: context.is_dealer(),
            is_open: context.is_open,
            riichi: context.is_riichi,
            double_riichi: context.is_double_riichi,
            ippatsu: context.is_ippatsu,
            dora_indicators: context
                .dora_indicators
                .iter()
                .map(|t| format!("{}", t))
                .collect(),
            ura_dora_indicators: context
                .ura_dora_indicators
                .iter()
                .map(|t| format!("{}", t))
                .collect(),
            akadora: parsed.aka_count,
            winning_tile: context.winning_tile.map(|t| format!("{}", t)),
            last_tile: context.is_last_tile,
            rinshan: context.is_rinshan,
            chankan: context.is_chankan,
            tenhou: context.is_tenhou,
            chiihou: context.is_chiihou,
        };

        let output = JsonOutput {
            hand: args.hand.clone(),
            context: json_context,
            interpretations,
        };

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return;
    }

    // Display results (human-readable)
    print_header(use_unicode);

    for (i, &(structure, yaku_result, score)) in results_to_show.iter().enumerate() {
        if i > 0 {
            println!("\n{}", "─".repeat(50));
        }

        if results_to_show.len() > 1 {
            println!("\n📋 Interpretation {}", i + 1);
        }

        print_hand(structure, use_unicode);
        print_context(&context, &parsed, use_unicode);
        print_yaku(yaku_result, &context);
        print_score(score);
    }

    print_footer(use_unicode);
}

fn parse_wind(s: &str) -> Result<Honor, String> {
    match s.to_lowercase().as_str() {
        "e" | "east" | "1" => Ok(Honor::East),
        "s" | "south" | "2" => Ok(Honor::South),
        "w" | "west" | "3" => Ok(Honor::West),
        "n" | "north" | "4" => Ok(Honor::North),
        _ => Err(format!("Invalid wind: {}. Use e/s/w/n", s)),
    }
}

/// Validate that riichi-dependent options are used with riichi,
/// and that riichi is not used with open hands.
/// Returns a list of warning messages for any invalid combinations.
fn validate_riichi_dependencies(
    riichi: bool,
    has_ura_dora: bool,
    ippatsu: bool,
    has_open_melds: bool,
    open_flag: bool,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if has_ura_dora && !riichi {
        warnings.push(
            "Ura dora (--ura) specified without riichi. Ura dora only apply when winning with riichi.".to_string()
        );
    }

    if ippatsu && !riichi {
        warnings.push(
            "Ippatsu (--ippatsu) specified without riichi. Ippatsu only applies when winning within one turn of riichi.".to_string()
        );
    }

    if riichi && has_open_melds {
        warnings.push(
            "Riichi specified but hand has open melds. Riichi requires a closed hand (menzen). Use [...] for closed kans instead of (...).".to_string()
        );
    }

    if riichi && open_flag {
        warnings.push(
            "Both --riichi and --open specified. Riichi requires a closed hand (menzen)."
                .to_string(),
        );
    }

    warnings
}

fn parse_single_tile(s: &str) -> Result<Tile, String> {
    let s = s.trim().to_lowercase();

    // Check for honor tile letter notation first
    // Winds: e, s, w, n (east, south, west, north)
    // Dragons: wh (white), g (green), r (red)
    match s.as_str() {
        "e" | "east" => return Ok(Tile::honor(Honor::East)),
        "s" | "south" => return Ok(Tile::honor(Honor::South)),
        "w" | "west" => return Ok(Tile::honor(Honor::West)),
        "n" | "north" => return Ok(Tile::honor(Honor::North)),
        "wh" | "white" | "haku" => return Ok(Tile::honor(Honor::White)),
        "g" | "green" | "hatsu" => return Ok(Tile::honor(Honor::Green)),
        "r" | "red" | "chun" => return Ok(Tile::honor(Honor::Red)),
        _ => {}
    }

    // Standard notation: digit + suit (e.g., "5m", "1z")
    // Must be exactly 2 characters
    if s.len() < 2 {
        return Err(format!("Tile notation too short: {}", s));
    }
    if s.len() > 2 {
        return Err(format!(
            "Expected a single tile, got '{}'. Use -d/--dora for multiple tiles.",
            s
        ));
    }

    let value_char = s.chars().next().unwrap();
    let suit_char = s.chars().last().unwrap();

    let value = match value_char.to_digit(10) {
        Some(v) if v >= 1 && v <= 9 => v as u8,
        Some(0) => 5, // Red five
        _ => return Err(format!("Invalid tile value: {}", value_char)),
    };

    match suit_char {
        'm' => Ok(Tile::suited(Suit::Man, value)),
        'p' => Ok(Tile::suited(Suit::Pin, value)),
        's' => Ok(Tile::suited(Suit::Sou, value)),
        'z' => {
            let honor = match value {
                1 => Honor::East,
                2 => Honor::South,
                3 => Honor::West,
                4 => Honor::North,
                5 => Honor::White,
                6 => Honor::Green,
                7 => Honor::Red,
                _ => return Err(format!("Invalid honor: {}z", value)),
            };
            Ok(Tile::honor(honor))
        }
        _ => Err(format!("Invalid suit: {}", suit_char)),
    }
}

/// Try to parse an honor tile from letter notation at the given position.
/// Returns Some((Honor, chars_consumed)) if successful, None otherwise.
/// Supports: e/E (east), s/S (south), w/W (west), n/N (north)
///           wh/Wh/WH (white), g/G (green), r/R (red)
fn try_parse_honor_letter(chars: &[char], pos: usize) -> Option<(Honor, usize)> {
    if pos >= chars.len() {
        return None;
    }

    let ch = chars[pos].to_ascii_lowercase();

    // Check for two-character "wh" (white dragon) first to avoid conflict with "w" (west)
    if ch == 'w' && pos + 1 < chars.len() && chars[pos + 1].to_ascii_lowercase() == 'h' {
        return Some((Honor::White, 2));
    }

    // Single letter honors
    match ch {
        'e' => Some((Honor::East, 1)),
        's' => Some((Honor::South, 1)),
        'w' => Some((Honor::West, 1)),
        'n' => Some((Honor::North, 1)),
        'g' => Some((Honor::Green, 1)),
        'r' => Some((Honor::Red, 1)),
        _ => None,
    }
}

fn parse_tile_list(s: &str) -> Result<Vec<Tile>, String> {
    if s.is_empty() {
        return Ok(vec![]);
    }

    let mut tiles = Vec::new();

    // Split by comma first, then parse each part
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        // Parse the part character by character, handling mixed notation like "2pg" (2p + green)
        let chars: Vec<char> = part.chars().collect();
        let mut pos = 0;

        while pos < chars.len() {
            // Try to parse as honor letter first (e.g., "e", "wh", "g", "r")
            if let Some((honor, consumed)) = try_parse_honor_letter(&chars, pos) {
                tiles.push(Tile::honor(honor));
                pos += consumed;
                continue;
            }

            // Try to parse as numeric tile notation (digits followed by suit)
            // Collect consecutive digits
            let digit_start = pos;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }

            if pos > digit_start && pos < chars.len() {
                let suit_char = chars[pos].to_ascii_lowercase();
                if "mpsz".contains(suit_char) {
                    // Parse digits with this suit
                    for i in digit_start..pos {
                        let single = format!("{}{}", chars[i], suit_char);
                        tiles.push(parse_single_tile(&single)?);
                    }
                    pos += 1; // consume the suit character
                    continue;
                } else {
                    // Digits followed by non-suit character - error
                    return Err(format!(
                        "Invalid tile notation: digits not followed by suit (m/p/s/z) in '{}'",
                        part
                    ));
                }
            } else if pos > digit_start {
                // Digits at end of string without suit
                return Err(format!(
                    "Invalid tile notation: trailing digits without suit in '{}'",
                    part
                ));
            }

            // Unknown character
            return Err(format!(
                "Invalid character '{}' in tile list '{}'",
                chars[pos], part
            ));
        }
    }

    Ok(tiles)
}

fn print_header(use_unicode: bool) {
    if use_unicode {
        println!(
            "\n{}",
            "╔══════════════════════════════════════════════════════════╗".cyan()
        );
        println!(
            "{}",
            "║            AGARI - Mahjong Score Calculator              ║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════╝".cyan()
        );
    } else {
        println!(
            "\n{}",
            "============================================================".cyan()
        );
        println!(
            "{}",
            "             AGARI - Mahjong Score Calculator"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "============================================================".cyan()
        );
    }
}

fn print_footer(use_unicode: bool) {
    if use_unicode {
        println!(
            "\n{}\n",
            "══════════════════════════════════════════════════════════════".cyan()
        );
    } else {
        println!(
            "\n{}\n",
            "============================================================".cyan()
        );
    }
}

fn print_hand(structure: &HandStructure, use_unicode: bool) {
    println!("\n{}", "📋 Hand Structure:".yellow().bold());
    println!("   {}", format_structure(structure, use_unicode));
}

fn print_context(context: &GameContext, parsed: &agari::parse::ParsedHand, use_unicode: bool) {
    let format_tile = |t: &Tile| -> String {
        if use_unicode {
            tile_to_unicode(t)
        } else {
            format!("{} ", tile_to_ascii(t))
        }
    };

    println!("\n{}", "🎮 Game Context:".yellow().bold());

    let win_str = match context.win_type {
        WinType::Tsumo => "Tsumo (self-draw)".green(),
        WinType::Ron => "Ron (discard)".blue(),
    };
    println!("   {}: {}", "Win Type".dimmed(), win_str);

    println!(
        "   {}: {}",
        "Round Wind".dimmed(),
        honor_name(&context.round_wind)
    );
    println!(
        "   {}: {}",
        "Seat Wind".dimmed(),
        honor_name(&context.seat_wind)
    );

    if context.is_dealer() {
        println!("   {}: {}", "Position".dimmed(), "Dealer (Oya)".magenta());
    }

    if context.is_open {
        println!(
            "   {}: {}",
            "Hand State".dimmed(),
            "Open (called tiles)".yellow()
        );
    } else {
        println!(
            "   {}: {}",
            "Hand State".dimmed(),
            "Closed (Menzen)".green()
        );
    }

    if context.is_riichi {
        if context.is_double_riichi {
            println!(
                "   {}: {}",
                "Riichi".dimmed(),
                "Double Riichi ⚡⚡".cyan().bold()
            );
        } else {
            println!("   {}: {}", "Riichi".dimmed(), "Yes ⚡".cyan().bold());
        }
        if context.is_ippatsu {
            println!("   {}: {}", "Ippatsu".dimmed(), "Yes 💫".cyan());
        }
    }

    if !context.dora_indicators.is_empty() {
        let dora_str: String = context.dora_indicators.iter().map(format_tile).collect();
        println!("   {}: {}", "Dora Indicators".dimmed(), dora_str.trim());
    }

    if context.is_riichi && !context.ura_dora_indicators.is_empty() {
        let ura_str: String = context
            .ura_dora_indicators
            .iter()
            .map(format_tile)
            .collect();
        println!("   {}: {}", "Ura Dora".dimmed(), ura_str.trim());
    }

    if parsed.aka_count > 0 {
        println!(
            "   {}: {}",
            "Red Fives (Akadora)".dimmed(),
            parsed.aka_count.to_string().red().bold()
        );
    }

    if let Some(wt) = context.winning_tile {
        println!("   {}: {}", "Winning Tile".dimmed(), format_tile(&wt));
    }
}

fn print_yaku(yaku_result: &agari::yaku::YakuResult, context: &GameContext) {
    println!("\n{}", "🏆 Yaku:".yellow().bold());

    if yaku_result.yaku_list.is_empty() {
        println!("   {}", "⚠️  No yaku! This hand cannot win.".red().bold());
        return;
    }

    for yaku in &yaku_result.yaku_list {
        let han = if context.is_open {
            yaku.han_open().unwrap_or(0)
        } else {
            yaku.han()
        };

        let name = yaku_name(yaku);
        let han_str = format!("({} han)", han);

        if yaku.is_yakuman() {
            println!(
                "   {} {} {} {}",
                "•".green(),
                name.green().bold(),
                han_str.green(),
                "🌟".to_string()
            );
        } else {
            println!("   {} {} {}", "•".white(), name.white(), han_str.dimmed());
        }
    }

    // Display dora breakdown
    if yaku_result.regular_dora > 0 {
        println!(
            "   {} {} {}",
            "•".white(),
            "Dora".white(),
            format!("({} han)", yaku_result.regular_dora).dimmed()
        );
    }
    if yaku_result.ura_dora > 0 {
        println!(
            "   {} {} {}",
            "•".white(),
            "Ura Dora".white(),
            format!("({} han)", yaku_result.ura_dora).dimmed()
        );
    }
    if yaku_result.aka_dora > 0 {
        println!(
            "   {} {} {}",
            "•".white(),
            "Red Fives (Akadora)".white(),
            format!("({} han)", yaku_result.aka_dora).dimmed()
        );
    }
}

fn print_score(score: &ScoringResult) {
    println!("\n{}", "💰 Score:".yellow().bold());

    // Han and Fu
    println!(
        "   {} {} / {} {}",
        score.han.to_string().bright_white().bold(),
        "han".dimmed(),
        score.fu.total.to_string().bright_white().bold(),
        "fu".dimmed()
    );

    // Score level
    if score.score_level != ScoreLevel::Normal {
        let level_emoji = match score.score_level {
            ScoreLevel::Mangan => "🔥",
            ScoreLevel::Haneman => "🔥🔥",
            ScoreLevel::Baiman => "🔥🔥🔥",
            ScoreLevel::Sanbaiman => "💎",
            ScoreLevel::Yakuman => "👑",
            ScoreLevel::DoubleYakuman => "👑👑",
            ScoreLevel::Normal => "",
        };
        let level_name = if score.is_counted_yakuman {
            "Counted Yakuman"
        } else {
            score.score_level.name()
        };
        let colored_level = match score.score_level {
            ScoreLevel::Mangan => level_name.yellow().bold(),
            ScoreLevel::Haneman => level_name.yellow().bold(),
            ScoreLevel::Baiman => level_name.bright_yellow().bold(),
            ScoreLevel::Sanbaiman => level_name.magenta().bold(),
            ScoreLevel::Yakuman | ScoreLevel::DoubleYakuman => level_name.bright_magenta().bold(),
            ScoreLevel::Normal => level_name.normal(),
        };
        println!("   {} {}", level_emoji, colored_level);
    }

    // Payment box
    println!();
    println!("   {}", "┌─────────────────────────────────────┐".green());
    println!(
        "   {}  {}: {:>6} {}               {}",
        "│".green(),
        "TOTAL".green().bold(),
        score.payment.total.to_string().bright_white().bold(),
        "points".green(),
        "│".green()
    );
    println!("   {}", "└─────────────────────────────────────┘".green());

    if let Some(from_discarder) = score.payment.from_discarder {
        println!(
            "   {}: {} from discarder",
            "Ron".blue(),
            from_discarder.to_string().bright_white()
        );
    } else if score.is_dealer {
        if let Some(from_each) = score.payment.from_non_dealer {
            println!(
                "   {}: {} all (×3 players)",
                "Tsumo".green(),
                from_each.to_string().bright_white()
            );
        }
    } else if let (Some(from_dealer), Some(from_non_dealer)) =
        (score.payment.from_dealer, score.payment.from_non_dealer)
    {
        println!(
            "   {}: {} / {} (dealer / non-dealer)",
            "Tsumo".green(),
            from_dealer.to_string().bright_white(),
            from_non_dealer.to_string().bright_white()
        );
    }

    // Fu breakdown (only if interesting)
    if score.fu.total != 25 && score.fu.total != 20 && score.fu.breakdown.raw_total > 20 {
        println!("\n   {}:", "Fu breakdown".dimmed());
        println!("     {}: {}", "Base".dimmed(), "20");
        if score.fu.breakdown.menzen_ron > 0 {
            println!(
                "     {}: {}",
                "Menzen Ron".dimmed(),
                format!("+{}", score.fu.breakdown.menzen_ron)
            );
        }
        if score.fu.breakdown.tsumo > 0 {
            println!(
                "     {}: {}",
                "Tsumo".dimmed(),
                format!("+{}", score.fu.breakdown.tsumo)
            );
        }
        if score.fu.breakdown.melds > 0 {
            println!(
                "     {}: {}",
                "Melds".dimmed(),
                format!("+{}", score.fu.breakdown.melds)
            );
        }
        if score.fu.breakdown.pair > 0 {
            println!(
                "     {}: {}",
                "Pair".dimmed(),
                format!("+{}", score.fu.breakdown.pair)
            );
        }
        if score.fu.breakdown.wait > 0 {
            println!(
                "     {}: {}",
                "Wait".dimmed(),
                format!("+{}", score.fu.breakdown.wait)
            );
        }
        println!(
            "     {}: {} → {}: {}",
            "Raw".dimmed(),
            score.fu.breakdown.raw_total,
            "Rounded".dimmed(),
            score.fu.total
        );
    }
}

fn print_shanten(counts: &agari::parse::TileCounts, show_ukeire: bool, use_unicode: bool) {
    let result = calculate_shanten(counts);

    println!("\n{}", "📊 Shanten Analysis:".yellow().bold());

    // Shanten value with description
    let shanten_desc = match result.shanten {
        -1 => "Complete hand (Agari)".to_string(),
        0 => "Tenpai (ready to win)".to_string(),
        1 => "Iishanten (1 away from tenpai)".to_string(),
        2 => "Ryanshanten (2 away from tenpai)".to_string(),
        n => format!("{}-shanten ({} away from tenpai)", n, n),
    };

    let shanten_emoji = match result.shanten {
        -1 => "🎉",
        0 => "🎯",
        1 => "📈",
        _ => "📊",
    };

    let colored_shanten = match result.shanten {
        -1 => result.shanten.to_string().green().bold(),
        0 => result.shanten.to_string().cyan().bold(),
        1 => result.shanten.to_string().yellow().bold(),
        _ => result.shanten.to_string().white().bold(),
    };

    println!(
        "   {} {}: {} - {}",
        shanten_emoji,
        "Shanten".dimmed(),
        colored_shanten,
        shanten_desc
    );

    // Best hand type
    let type_name = match result.best_type {
        ShantenType::Standard => "Standard (4 melds + 1 pair)",
        ShantenType::Chiitoitsu => "Chiitoitsu (7 pairs)",
        ShantenType::Kokushi => "Kokushi (13 orphans)",
    };
    println!("   {}: {}", "Best shape".dimmed(), type_name);

    // Ukeire (tile acceptance)
    if show_ukeire && result.shanten >= 0 {
        let ukeire = calculate_ukeire(counts);

        println!("\n{}", "🀄 Ukeire (Tile Acceptance):".yellow().bold());

        if ukeire.tiles.is_empty() {
            println!("   {}", "No tiles improve this hand.".dimmed());
        } else {
            println!(
                "   {} tiles improve the hand ({} total):",
                ukeire.tiles.len().to_string().bright_white().bold(),
                ukeire.total_count.to_string().bright_white().bold()
            );
            println!();

            // Group tiles by type for nicer display
            let mut tile_strs: Vec<String> = Vec::new();
            for ut in &ukeire.tiles {
                let tile_str = if use_unicode {
                    tile_to_unicode(&ut.tile)
                } else {
                    format!("{}", ut.tile)
                };
                tile_strs.push(format!(
                    "{}×{}",
                    tile_str.trim(),
                    ut.available.to_string().dimmed()
                ));
            }

            // Print in rows of ~8 tiles
            for chunk in tile_strs.chunks(8) {
                println!("   {}", chunk.join("  "));
            }
        }
    } else if show_ukeire && result.shanten == -1 {
        println!("\n   Hand is already complete - no tiles needed.");
    }
}

fn print_shanten_json(counts: &agari::parse::TileCounts, show_ukeire: bool) {
    let result = calculate_shanten(counts);

    let shanten_desc = match result.shanten {
        -1 => "Complete hand (Agari)".to_string(),
        0 => "Tenpai (ready to win)".to_string(),
        1 => "Iishanten (1 away from tenpai)".to_string(),
        2 => "Ryanshanten (2 away from tenpai)".to_string(),
        n => format!("{}-shanten ({} away from tenpai)", n, n),
    };

    let type_name = match result.best_type {
        ShantenType::Standard => "Standard (4 melds + 1 pair)",
        ShantenType::Chiitoitsu => "Chiitoitsu (7 pairs)",
        ShantenType::Kokushi => "Kokushi (13 orphans)",
    };

    let ukeire_data = if show_ukeire && result.shanten >= 0 {
        let ukeire = calculate_ukeire(counts);
        Some(JsonUkeire {
            tile_count: ukeire.tiles.len(),
            total_available: ukeire.total_count,
            tiles: ukeire
                .tiles
                .iter()
                .map(|ut| JsonUkeireTile {
                    tile: format!("{}", ut.tile),
                    available: ut.available,
                })
                .collect(),
        })
    } else {
        None
    };

    let output = JsonShantenOutput {
        shanten: result.shanten,
        description: shanten_desc,
        best_type: type_name.to_string(),
        ukeire: ukeire_data,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn yaku_name(yaku: &Yaku) -> &'static str {
    match yaku {
        Yaku::Riichi => "Riichi",
        Yaku::Ippatsu => "Ippatsu",
        Yaku::MenzenTsumo => "Menzen Tsumo",
        Yaku::Tanyao => "Tanyao (All Simples)",
        Yaku::Pinfu => "Pinfu",
        Yaku::Iipeikou => "Iipeikou (Pure Double Sequence)",
        Yaku::Yakuhai(h) => match h {
            Honor::East => "Yakuhai: East Wind",
            Honor::South => "Yakuhai: South Wind",
            Honor::West => "Yakuhai: West Wind",
            Honor::North => "Yakuhai: North Wind",
            Honor::White => "Yakuhai: White Dragon (Haku)",
            Honor::Green => "Yakuhai: Green Dragon (Hatsu)",
            Honor::Red => "Yakuhai: Red Dragon (Chun)",
        },
        Yaku::RinshanKaihou => "Rinshan Kaihou (After Kan)",
        Yaku::Chankan => "Chankan (Robbing the Kan)",
        Yaku::HaiteiRaoyue => "Haitei Raoyue (Last Tile Draw)",
        Yaku::HouteiRaoyui => "Houtei Raoyui (Last Tile Discard)",
        Yaku::DoubleRiichi => "Double Riichi",
        Yaku::Toitoi => "Toitoi (All Triplets)",
        Yaku::SanshokuDoujun => "Sanshoku Doujun (Mixed Triple Sequence)",
        Yaku::SanshokuDoukou => "Sanshoku Doukou (Triple Triplets)",
        Yaku::Ittsu => "Ittsu (Pure Straight)",
        Yaku::Chiitoitsu => "Chiitoitsu (Seven Pairs)",
        Yaku::Chanta => "Chanta (Outside Hand)",
        Yaku::SanAnkou => "San Ankou (Three Concealed Triplets)",
        Yaku::SanKantsu => "San Kantsu (Three Kans)",
        Yaku::Honroutou => "Honroutou (All Terminals and Honors)",
        Yaku::Shousangen => "Shousangen (Little Three Dragons)",
        Yaku::Honitsu => "Honitsu (Half Flush)",
        Yaku::Junchan => "Junchan (Terminals in All Groups)",
        Yaku::Ryanpeikou => "Ryanpeikou (Twice Pure Double Sequence)",
        Yaku::Chinitsu => "Chinitsu (Full Flush)",

        // Yakuman
        Yaku::Tenhou => "Tenhou (Heavenly Hand)",
        Yaku::Chiihou => "Chiihou (Earthly Hand)",
        Yaku::KokushiMusou => "Kokushi Musou (Thirteen Orphans)",
        Yaku::Suuankou => "Suuankou (Four Concealed Triplets)",
        Yaku::Daisangen => "Daisangen (Big Three Dragons)",
        Yaku::Shousuushii => "Shousuushii (Little Four Winds)",
        Yaku::Daisuushii => "Daisuushii (Big Four Winds)",
        Yaku::Tsuuiisou => "Tsuuiisou (All Honors)",
        Yaku::Chinroutou => "Chinroutou (All Terminals)",
        Yaku::Ryuuiisou => "Ryuuiisou (All Green)",
        Yaku::ChuurenPoutou => "Chuuren Poutou (Nine Gates)",

        // Double Yakuman
        Yaku::Kokushi13Wait => "Kokushi Musou 13-wait",
        Yaku::SuuankouTanki => "Suuankou Tanki",
        Yaku::JunseiChuurenPoutou => "Junsei Chuuren Poutou",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== parse_single_tile tests =====

    #[test]
    fn test_parse_single_tile_standard_notation() {
        // Standard numeric notation still works
        assert_eq!(parse_single_tile("1m").unwrap(), Tile::suited(Suit::Man, 1));
        assert_eq!(parse_single_tile("5p").unwrap(), Tile::suited(Suit::Pin, 5));
        assert_eq!(parse_single_tile("9s").unwrap(), Tile::suited(Suit::Sou, 9));
        assert_eq!(parse_single_tile("1z").unwrap(), Tile::honor(Honor::East));
        assert_eq!(parse_single_tile("7z").unwrap(), Tile::honor(Honor::Red));
    }

    #[test]
    fn test_parse_single_tile_rejects_multiple_tiles() {
        // Should reject input that looks like multiple tiles
        assert!(parse_single_tile("1m2m").is_err());
        assert!(parse_single_tile("123m").is_err());
        assert!(parse_single_tile("1m5p").is_err());
    }

    #[test]
    fn test_parse_single_tile_wind_letters() {
        assert_eq!(parse_single_tile("e").unwrap(), Tile::honor(Honor::East));
        assert_eq!(parse_single_tile("s").unwrap(), Tile::honor(Honor::South));
        assert_eq!(parse_single_tile("w").unwrap(), Tile::honor(Honor::West));
        assert_eq!(parse_single_tile("n").unwrap(), Tile::honor(Honor::North));
    }

    #[test]
    fn test_parse_single_tile_wind_letters_uppercase() {
        assert_eq!(parse_single_tile("E").unwrap(), Tile::honor(Honor::East));
        assert_eq!(parse_single_tile("S").unwrap(), Tile::honor(Honor::South));
        assert_eq!(parse_single_tile("W").unwrap(), Tile::honor(Honor::West));
        assert_eq!(parse_single_tile("N").unwrap(), Tile::honor(Honor::North));
    }

    #[test]
    fn test_parse_single_tile_dragon_letters() {
        assert_eq!(parse_single_tile("wh").unwrap(), Tile::honor(Honor::White));
        assert_eq!(parse_single_tile("g").unwrap(), Tile::honor(Honor::Green));
        assert_eq!(parse_single_tile("r").unwrap(), Tile::honor(Honor::Red));
    }

    #[test]
    fn test_parse_single_tile_dragon_letters_uppercase() {
        assert_eq!(parse_single_tile("WH").unwrap(), Tile::honor(Honor::White));
        assert_eq!(parse_single_tile("Wh").unwrap(), Tile::honor(Honor::White));
        assert_eq!(parse_single_tile("G").unwrap(), Tile::honor(Honor::Green));
        assert_eq!(parse_single_tile("R").unwrap(), Tile::honor(Honor::Red));
    }

    #[test]
    fn test_parse_single_tile_verbose_names() {
        assert_eq!(parse_single_tile("east").unwrap(), Tile::honor(Honor::East));
        assert_eq!(
            parse_single_tile("south").unwrap(),
            Tile::honor(Honor::South)
        );
        assert_eq!(parse_single_tile("west").unwrap(), Tile::honor(Honor::West));
        assert_eq!(
            parse_single_tile("north").unwrap(),
            Tile::honor(Honor::North)
        );
        assert_eq!(
            parse_single_tile("white").unwrap(),
            Tile::honor(Honor::White)
        );
        assert_eq!(
            parse_single_tile("green").unwrap(),
            Tile::honor(Honor::Green)
        );
        assert_eq!(parse_single_tile("red").unwrap(), Tile::honor(Honor::Red));
        assert_eq!(
            parse_single_tile("haku").unwrap(),
            Tile::honor(Honor::White)
        );
        assert_eq!(
            parse_single_tile("hatsu").unwrap(),
            Tile::honor(Honor::Green)
        );
        assert_eq!(parse_single_tile("chun").unwrap(), Tile::honor(Honor::Red));
    }

    // ===== parse_tile_list tests =====

    #[test]
    fn test_parse_tile_list_standard_notation() {
        // Standard grouped notation
        let tiles = parse_tile_list("35z").unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::honor(Honor::West));
        assert_eq!(tiles[1], Tile::honor(Honor::White));
    }

    #[test]
    fn test_parse_tile_list_comma_separated() {
        let tiles = parse_tile_list("1m,5p,9s").unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0], Tile::suited(Suit::Man, 1));
        assert_eq!(tiles[1], Tile::suited(Suit::Pin, 5));
        assert_eq!(tiles[2], Tile::suited(Suit::Sou, 9));
    }

    #[test]
    fn test_parse_tile_list_honor_letter_sequence() {
        // Honor letter sequence without commas: "wwh" -> West, White
        let tiles = parse_tile_list("wwh").unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::honor(Honor::West));
        assert_eq!(tiles[1], Tile::honor(Honor::White));
    }

    #[test]
    fn test_parse_tile_list_honor_letter_sequence_whe() {
        // "whe" -> White, East
        let tiles = parse_tile_list("whe").unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::honor(Honor::White));
        assert_eq!(tiles[1], Tile::honor(Honor::East));
    }

    #[test]
    fn test_parse_tile_list_all_winds() {
        let tiles = parse_tile_list("eswn").unwrap();
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0], Tile::honor(Honor::East));
        assert_eq!(tiles[1], Tile::honor(Honor::South));
        assert_eq!(tiles[2], Tile::honor(Honor::West));
        assert_eq!(tiles[3], Tile::honor(Honor::North));
    }

    #[test]
    fn test_parse_tile_list_all_dragons() {
        let tiles = parse_tile_list("whgr").unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0], Tile::honor(Honor::White));
        assert_eq!(tiles[1], Tile::honor(Honor::Green));
        assert_eq!(tiles[2], Tile::honor(Honor::Red));
    }

    #[test]
    fn test_parse_tile_list_complex_west_white_sequence() {
        // "wwwwhwh" -> West, West, West, White, White
        let tiles = parse_tile_list("wwwwhwh").unwrap();
        assert_eq!(tiles.len(), 5);
        assert_eq!(tiles[0], Tile::honor(Honor::West));
        assert_eq!(tiles[1], Tile::honor(Honor::West));
        assert_eq!(tiles[2], Tile::honor(Honor::West));
        assert_eq!(tiles[3], Tile::honor(Honor::White));
        assert_eq!(tiles[4], Tile::honor(Honor::White));
    }

    #[test]
    fn test_parse_tile_list_wwhwwwh() {
        // "wwhwwwh" -> West, White, West, West, White
        let tiles = parse_tile_list("wwhwwwh").unwrap();
        assert_eq!(tiles.len(), 5);
        assert_eq!(tiles[0], Tile::honor(Honor::West));
        assert_eq!(tiles[1], Tile::honor(Honor::White));
        assert_eq!(tiles[2], Tile::honor(Honor::West));
        assert_eq!(tiles[3], Tile::honor(Honor::West));
        assert_eq!(tiles[4], Tile::honor(Honor::White));
    }

    #[test]
    fn test_parse_tile_list_mixed_comma_and_letter() {
        // Can mix comma-separated with letter notation
        let tiles = parse_tile_list("e,wh,1m").unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0], Tile::honor(Honor::East));
        assert_eq!(tiles[1], Tile::honor(Honor::White));
        assert_eq!(tiles[2], Tile::suited(Suit::Man, 1));
    }

    #[test]
    fn test_parse_tile_list_empty() {
        let tiles = parse_tile_list("").unwrap();
        assert!(tiles.is_empty());
    }

    #[test]
    fn test_parse_tile_list_single_honor_letter() {
        let tiles = parse_tile_list("e").unwrap();
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0], Tile::honor(Honor::East));
    }

    #[test]
    fn test_parse_tile_list_uppercase_honors() {
        let tiles = parse_tile_list("ESWN").unwrap();
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0], Tile::honor(Honor::East));
        assert_eq!(tiles[1], Tile::honor(Honor::South));
        assert_eq!(tiles[2], Tile::honor(Honor::West));
        assert_eq!(tiles[3], Tile::honor(Honor::North));
    }

    #[test]
    fn test_parse_tile_list_mixed_suited_and_honor() {
        // "2pg" -> 2-pin + green dragon
        let tiles = parse_tile_list("2pg").unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::suited(Suit::Pin, 2));
        assert_eq!(tiles[1], Tile::honor(Honor::Green));
    }

    #[test]
    fn test_parse_tile_list_mixed_honor_and_suited() {
        // "e5m" -> east + 5-man
        let tiles = parse_tile_list("e5m").unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0], Tile::honor(Honor::East));
        assert_eq!(tiles[1], Tile::suited(Suit::Man, 5));
    }

    #[test]
    fn test_parse_tile_list_complex_mixed() {
        // "1mwh9s" -> 1-man + white + 9-sou
        let tiles = parse_tile_list("1mwh9s").unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0], Tile::suited(Suit::Man, 1));
        assert_eq!(tiles[1], Tile::honor(Honor::White));
        assert_eq!(tiles[2], Tile::suited(Suit::Sou, 9));
    }

    #[test]
    fn test_parse_tile_list_multiple_suited_then_honor() {
        // "19mr" -> 1-man, 9-man + red dragon
        let tiles = parse_tile_list("19mr").unwrap();
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0], Tile::suited(Suit::Man, 1));
        assert_eq!(tiles[1], Tile::suited(Suit::Man, 9));
        assert_eq!(tiles[2], Tile::honor(Honor::Red));
    }

    // ===== validate_riichi_dependencies tests =====

    #[test]
    fn test_validate_riichi_deps_no_warnings_when_valid() {
        // With riichi, ura dora and ippatsu are fine (closed hand)
        let warnings = validate_riichi_dependencies(true, true, true, false, false);
        assert!(warnings.is_empty());

        // Without riichi but also without ura/ippatsu is fine
        let warnings = validate_riichi_dependencies(false, false, false, false, false);
        assert!(warnings.is_empty());

        // Riichi with only ura dora
        let warnings = validate_riichi_dependencies(true, true, false, false, false);
        assert!(warnings.is_empty());

        // Riichi with only ippatsu
        let warnings = validate_riichi_dependencies(true, false, true, false, false);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_validate_riichi_deps_warns_ura_without_riichi() {
        let warnings = validate_riichi_dependencies(false, true, false, false, false);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Ura dora"));
        assert!(warnings[0].contains("without riichi"));
    }

    #[test]
    fn test_validate_riichi_deps_warns_ippatsu_without_riichi() {
        let warnings = validate_riichi_dependencies(false, false, true, false, false);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Ippatsu"));
        assert!(warnings[0].contains("without riichi"));
    }

    #[test]
    fn test_validate_riichi_deps_warns_both_without_riichi() {
        let warnings = validate_riichi_dependencies(false, true, true, false, false);
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.contains("Ura dora")));
        assert!(warnings.iter().any(|w| w.contains("Ippatsu")));
    }

    #[test]
    fn test_validate_riichi_deps_warns_riichi_with_open_melds() {
        let warnings = validate_riichi_dependencies(true, false, false, true, false);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("open melds"));
        assert!(warnings[0].contains("closed hand"));
    }

    #[test]
    fn test_validate_riichi_deps_warns_riichi_with_open_flag() {
        let warnings = validate_riichi_dependencies(true, false, false, false, true);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("--riichi"));
        assert!(warnings[0].contains("--open"));
    }

    #[test]
    fn test_validate_riichi_deps_warns_riichi_with_both_open() {
        // Both open melds and --open flag
        let warnings = validate_riichi_dependencies(true, false, false, true, true);
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.contains("open melds")));
        assert!(warnings.iter().any(|w| w.contains("--open")));
    }

    #[test]
    fn test_validate_riichi_deps_no_warning_open_without_riichi() {
        // Open hand without riichi is perfectly valid
        let warnings = validate_riichi_dependencies(false, false, false, true, false);
        assert!(warnings.is_empty());

        let warnings = validate_riichi_dependencies(false, false, false, false, true);
        assert!(warnings.is_empty());
    }
}
