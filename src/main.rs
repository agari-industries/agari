//! Agari - Riichi Mahjong Hand Scoring Calculator
//!
//! A command-line tool for calculating the score of a Riichi Mahjong hand.

use std::process;

use clap::Parser;

use agari::{
    context::{GameContext, WinType},
    display::{format_structure, honor_name, tile_to_unicode},
    hand::{decompose_hand, decompose_hand_with_melds},
    parse::{parse_hand_with_aka, to_counts, validate_hand, validate_hand_with_melds},
    scoring::{ScoreLevel, ScoringResult, calculate_score},
    shanten::{ShantenType, calculate_shanten, calculate_ukeire},
    tile::{Honor, Suit, Tile},
    yaku::{Yaku, detect_yaku_with_context},
};

const AFTER_HELP: &str = r#"HAND FORMAT:
    Standard notation: numbers followed by suit letter
    m = Man (Characters), p = Pin (Dots), s = Sou (Bamboo), z = Honors
    Honors: 1z=East, 2z=South, 3z=West, 4z=North, 5z=White, 6z=Green, 7z=Red
    Red fives: Use 0 instead of 5 (e.g., 0m = red 5-man)

    Called melds (kans, pons, chis):
    [1111m]  = Closed kan (ankan) of 1-man
    (1111m)  = Open kan (daiminkan) of 1-man
    (111m)   = Open triplet (pon) of 1-man
    (123m)   = Open sequence (chi) of 1-2-3 man

EXAMPLES:
    agari 123m456p789s11122z              Basic hand
    agari 123m456p789s11122z -t           Tsumo win
    agari 123m456p789s11122z -r           With riichi
    agari 123m456p789s11122z -w 2m -t     Won on 2-man by tsumo
    agari 123m456p789s11122z -d 1m        With dora indicator 1m (2m is dora)
    agari 234567m234567p22s -w 5p -t      Pinfu tanyao
    agari "[1111m]222333m555p11z" -t      Hand with closed kan (15 tiles)
    agari "[1111m](2222p)345678s11z" -t   Hand with closed + open kan (16 tiles)"#;

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
}

fn main() {
    let args = Args::parse();

    // Extract arguments
    let shanten_mode = args.shanten || args.ukeire;
    let ukeire_mode = args.ukeire;
    let riichi = args.riichi || args.double_riichi;

    // Parse the hand
    let parsed = match parse_hand_with_aka(&args.hand) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Error parsing hand: {}", e);
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
                eprintln!("❌ Invalid hand: {}", e);
                process::exit(1);
            }
        } else {
            if let Err(e) = validate_hand(&parsed.tiles) {
                eprintln!("❌ Invalid hand: {}", e);
                process::exit(1);
            }
        }
    } else {
        // For shanten, allow 1-14 tiles (not counting melds)
        let tile_count = parsed.tiles.len();
        if tile_count < 1 || tile_count > 14 {
            eprintln!("❌ Invalid hand: expected 1-14 tiles, got {}", tile_count);
            process::exit(1);
        }
    }

    // If hand has open melds, mark hand as open
    let has_open_melds = parsed.called_melds.iter().any(|m| m.meld.is_open());

    // Parse winds
    let round_wind = match parse_wind(&args.round) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("❌ {}", e);
            process::exit(1);
        }
    };

    let seat_wind = match parse_wind(&args.seat) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("❌ {}", e);
            process::exit(1);
        }
    };

    // Parse dora indicators
    let dora_indicators = match args.dora.as_ref().map(|s| parse_tile_list(s)).transpose() {
        Ok(d) => d.unwrap_or_default(),
        Err(e) => {
            eprintln!("❌ Error parsing dora: {}", e);
            process::exit(1);
        }
    };

    let ura_indicators = match args.ura.as_ref().map(|s| parse_tile_list(s)).transpose() {
        Ok(u) => u.unwrap_or_default(),
        Err(e) => {
            eprintln!("❌ Error parsing ura dora: {}", e);
            process::exit(1);
        }
    };

    // Parse winning tile
    let winning_tile = match args
        .winning_tile
        .as_ref()
        .map(|s| parse_single_tile(s))
        .transpose()
    {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Error parsing winning tile: {}", e);
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

    if let Some(wt) = winning_tile {
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

    // Convert to tile counts
    let counts = to_counts(&parsed.tiles);
    let use_unicode = !args.ascii;

    // Shanten mode: calculate shanten and optionally ukeire
    if shanten_mode {
        print_header(use_unicode);
        print_shanten(&counts, ukeire_mode, use_unicode);
        print_footer(use_unicode);
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
        eprintln!("❌ This hand has no valid winning structure.");
        process::exit(1);
    }

    // Score each decomposition
    let mut results: Vec<_> = structures
        .iter()
        .map(|s| {
            let yaku_result = detect_yaku_with_context(s, &counts, &context);
            let score = calculate_score(s, &yaku_result, &context);
            (s, yaku_result, score)
        })
        .collect();

    // Sort by score (highest first)
    results.sort_by(|a, b| b.2.payment.total.cmp(&a.2.payment.total));

    // Filter to best interpretation only (unless --all)
    let results_to_show: &[_] = if args.all { &results } else { &results[..1] };

    // Display results
    print_header(use_unicode);

    for (i, (structure, yaku_result, score)) in results_to_show.iter().enumerate() {
        if i > 0 {
            println!("\n{}", "─".repeat(50));
        }

        if results_to_show.len() > 1 {
            println!("\n📋 Interpretation {}", i + 1);
        }

        print_hand(structure, use_unicode);
        print_context(&context, &parsed);
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

fn parse_single_tile(s: &str) -> Result<Tile, String> {
    let s = s.trim();
    if s.len() < 2 {
        return Err(format!("Tile notation too short: {}", s));
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

fn parse_tile_list(s: &str) -> Result<Vec<Tile>, String> {
    if s.is_empty() {
        return Ok(vec![]);
    }

    s.split(',')
        .map(|part| parse_single_tile(part.trim()))
        .collect()
}

fn print_header(use_unicode: bool) {
    if use_unicode {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║            AGARI - Mahjong Score Calculator              ║");
        println!("╚══════════════════════════════════════════════════════════╝");
    } else {
        println!("\n============================================================");
        println!("             AGARI - Mahjong Score Calculator");
        println!("============================================================");
    }
}

fn print_footer(use_unicode: bool) {
    if use_unicode {
        println!("\n══════════════════════════════════════════════════════════════\n");
    } else {
        println!("\n============================================================\n");
    }
}

fn print_hand(structure: &agari::hand::HandStructure, use_unicode: bool) {
    println!("\n📋 Hand Structure:");
    println!("   {}", format_structure(structure, use_unicode));
}

fn print_context(context: &GameContext, parsed: &agari::parse::ParsedHand) {
    println!("\n🎮 Game Context:");

    let win_str = match context.win_type {
        WinType::Tsumo => "Tsumo (self-draw)",
        WinType::Ron => "Ron (discard)",
    };
    println!("   Win Type: {}", win_str);

    println!("   Round Wind: {}", honor_name(&context.round_wind));
    println!("   Seat Wind: {}", honor_name(&context.seat_wind));

    if context.is_dealer() {
        println!("   Position: Dealer (Oya)");
    }

    if context.is_open {
        println!("   Hand State: Open (called tiles)");
    } else {
        println!("   Hand State: Closed (Menzen)");
    }

    if context.is_riichi {
        if context.is_double_riichi {
            println!("   Riichi: Double Riichi ⚡⚡");
        } else {
            println!("   Riichi: Yes ⚡");
        }
        if context.is_ippatsu {
            println!("   Ippatsu: Yes 💫");
        }
    }

    if !context.dora_indicators.is_empty() {
        let dora_str: String = context
            .dora_indicators
            .iter()
            .map(|t| format!("{} ", tile_to_unicode(t)))
            .collect();
        println!("   Dora Indicators: {}", dora_str.trim());
    }

    if context.is_riichi && !context.ura_dora_indicators.is_empty() {
        let ura_str: String = context
            .ura_dora_indicators
            .iter()
            .map(|t| format!("{} ", tile_to_unicode(t)))
            .collect();
        println!("   Ura Dora: {}", ura_str.trim());
    }

    if parsed.aka_count > 0 {
        println!("   Red Fives (Akadora): {}", parsed.aka_count);
    }

    if let Some(wt) = context.winning_tile {
        println!("   Winning Tile: {}", tile_to_unicode(&wt));
    }
}

fn print_yaku(yaku_result: &agari::yaku::YakuResult, context: &GameContext) {
    println!("\n🏆 Yaku:");

    if yaku_result.yaku_list.is_empty() {
        println!("   ⚠️  No yaku! This hand cannot win.");
        return;
    }

    for yaku in &yaku_result.yaku_list {
        let han = if context.is_open {
            yaku.han_open().unwrap_or(0)
        } else {
            yaku.han()
        };

        let name = yaku_name(yaku);
        let yakuman_marker = if yaku.is_yakuman() { " 🌟" } else { "" };

        println!("   • {} ({} han){}", name, han, yakuman_marker);
    }

    if yaku_result.dora_count > 0 {
        println!("   • Dora ({} han)", yaku_result.dora_count);
    }
}

fn print_score(score: &ScoringResult) {
    println!("\n💰 Score:");

    // Han and Fu
    println!("   {} han / {} fu", score.han, score.fu.total);

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
        println!("   {} {}", level_emoji, score.score_level.name());
    }

    // Payment box
    println!();
    println!("   ┌─────────────────────────────────────┐");
    println!(
        "   │  TOTAL: {:>6} points               │",
        score.payment.total
    );
    println!("   └─────────────────────────────────────┘");

    if let Some(from_discarder) = score.payment.from_discarder {
        println!("   Ron: {} from discarder", from_discarder);
    } else if score.is_dealer {
        if let Some(from_each) = score.payment.from_non_dealer {
            println!("   Tsumo: {} all (×3 players)", from_each);
        }
    } else if let (Some(from_dealer), Some(from_non_dealer)) =
        (score.payment.from_dealer, score.payment.from_non_dealer)
    {
        println!(
            "   Tsumo: {} / {} (dealer / non-dealer)",
            from_dealer, from_non_dealer
        );
    }

    // Fu breakdown (only if interesting)
    if score.fu.total != 25 && score.fu.total != 20 && score.fu.breakdown.raw_total > 20 {
        println!("\n   Fu breakdown:");
        println!("     Base: 20");
        if score.fu.breakdown.menzen_ron > 0 {
            println!("     Menzen Ron: +{}", score.fu.breakdown.menzen_ron);
        }
        if score.fu.breakdown.tsumo > 0 {
            println!("     Tsumo: +{}", score.fu.breakdown.tsumo);
        }
        if score.fu.breakdown.melds > 0 {
            println!("     Melds: +{}", score.fu.breakdown.melds);
        }
        if score.fu.breakdown.pair > 0 {
            println!("     Pair: +{}", score.fu.breakdown.pair);
        }
        if score.fu.breakdown.wait > 0 {
            println!("     Wait: +{}", score.fu.breakdown.wait);
        }
        println!(
            "     Raw: {} → Rounded: {}",
            score.fu.breakdown.raw_total, score.fu.total
        );
    }
}

fn print_shanten(counts: &agari::parse::TileCounts, show_ukeire: bool, use_unicode: bool) {
    let result = calculate_shanten(counts);

    println!("\n📊 Shanten Analysis:");

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

    println!(
        "   {} Shanten: {} - {}",
        shanten_emoji, result.shanten, shanten_desc
    );

    // Best hand type
    let type_name = match result.best_type {
        ShantenType::Standard => "Standard (4 melds + 1 pair)",
        ShantenType::Chiitoitsu => "Chiitoitsu (7 pairs)",
        ShantenType::Kokushi => "Kokushi (13 orphans)",
    };
    println!("   Best shape: {}", type_name);

    // Ukeire (tile acceptance)
    if show_ukeire && result.shanten >= 0 {
        let ukeire = calculate_ukeire(counts);

        println!("\n🀄 Ukeire (Tile Acceptance):");

        if ukeire.tiles.is_empty() {
            println!("   No tiles improve this hand.");
        } else {
            println!(
                "   {} tiles improve the hand ({} total):",
                ukeire.tiles.len(),
                ukeire.total_count
            );
            println!();

            // Group tiles by type for nicer display
            let mut tile_strs: Vec<String> = Vec::new();
            for ut in &ukeire.tiles {
                let tile_str = if use_unicode {
                    format!("{} ", tile_to_unicode(&ut.tile))
                } else {
                    format!("{}", ut.tile)
                };
                tile_strs.push(format!("{}×{}", tile_str.trim(), ut.available));
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
