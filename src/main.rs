//! Agari - Riichi Mahjong Hand Scoring Calculator
//!
//! A command-line tool for calculating the score of a Riichi Mahjong hand.

use std::env;
use std::process;

use agari::{
    context::{GameContext, WinType},
    display::{format_structure, honor_name, tile_to_unicode},
    hand::decompose_hand,
    parse::{parse_hand_with_aka, to_counts, validate_hand},
    scoring::{ScoreLevel, ScoringResult, calculate_score},
    tile::{Honor, Suit, Tile},
    yaku::{Yaku, detect_yaku_with_context},
};

const HELP: &str = r#"
╔════════════════════════════════════════════════════════════════════════════╗
║                     AGARI - Mahjong Score Calculator                       ║
╚════════════════════════════════════════════════════════════════════════════╝

USAGE:
    agari <HAND> [OPTIONS]

HAND FORMAT:
    Standard notation: numbers followed by suit letter
    m = Man (Characters), p = Pin (Dots), s = Sou (Bamboo), z = Honors
    Honors: 1z=East, 2z=South, 3z=West, 4z=North, 5z=White, 6z=Green, 7z=Red
    Red fives: Use 0 instead of 5 (e.g., 0m = red 5-man)

EXAMPLES:
    agari 123m456p789s11122z              Basic hand
    agari 123m456p789s11122z -t           Tsumo win
    agari 123m456p789s11122z -r           With riichi
    agari 123m456p789s11122z -w 2m -t     Won on 2-man by tsumo
    agari 123m456p789s11122z -d 1m        With dora indicator 1m (2m is dora)
    agari 234567m234567p22s -w 5p -t      Pinfu tanyao

OPTIONS:
    -w, --win <TILE>      Winning tile (e.g., 2m, 5z)
    -t, --tsumo           Win by self-draw (default: ron)
    -o, --open            Hand is open (has called tiles)
    -r, --riichi          Riichi declared
    --double-riichi       Double riichi (first turn)
    --ippatsu             Ippatsu (win within one turn of riichi)

    --round <WIND>        Round wind: e/s/w/n (default: e)
    --seat <WIND>         Seat wind: e/s/w/n (default: e)

    -d, --dora <TILES>    Dora indicators (comma-separated: 1m,5z)
    --ura <TILES>         Ura dora indicators (with riichi only)

    --last-tile           Win on last tile (Haitei/Houtei)
    --rinshan             Win on kan replacement tile
    --chankan             Ron on another player's added kan
    --tenhou              Dealer's first draw win
    --chiihou             Non-dealer's first draw win

    --ascii               Use ASCII output instead of Unicode
    --all                 Show all possible interpretations
    -h, --help            Show this help message
"#;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("{}", HELP);
        process::exit(0);
    }

    // Parse arguments
    let mut hand_str: Option<String> = None;
    let mut winning_tile_str: Option<String> = None;
    let mut tsumo = false;
    let mut open = false;
    let mut riichi = false;
    let mut double_riichi = false;
    let mut ippatsu = false;
    let mut round_wind_str = "e".to_string();
    let mut seat_wind_str = "e".to_string();
    let mut dora_str: Option<String> = None;
    let mut ura_str: Option<String> = None;
    let mut last_tile = false;
    let mut rinshan = false;
    let mut chankan = false;
    let mut tenhou = false;
    let mut chiihou = false;
    let mut ascii = false;
    let mut show_all = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                process::exit(0);
            }
            "-t" | "--tsumo" => tsumo = true,
            "-o" | "--open" => open = true,
            "-r" | "--riichi" => riichi = true,
            "--double-riichi" => {
                double_riichi = true;
                riichi = true;
            }
            "--ippatsu" => ippatsu = true,
            "--last-tile" => last_tile = true,
            "--rinshan" => rinshan = true,
            "--chankan" => chankan = true,
            "--tenhou" => tenhou = true,
            "--chiihou" => chiihou = true,
            "--ascii" => ascii = true,
            "--all" => show_all = true,
            "-w" | "--win" => {
                i += 1;
                if i < args.len() {
                    winning_tile_str = Some(args[i].clone());
                }
            }
            "--round" => {
                i += 1;
                if i < args.len() {
                    round_wind_str = args[i].clone();
                }
            }
            "--seat" => {
                i += 1;
                if i < args.len() {
                    seat_wind_str = args[i].clone();
                }
            }
            "-d" | "--dora" => {
                i += 1;
                if i < args.len() {
                    dora_str = Some(args[i].clone());
                }
            }
            "--ura" => {
                i += 1;
                if i < args.len() {
                    ura_str = Some(args[i].clone());
                }
            }
            _ => {
                if !arg.starts_with('-') && hand_str.is_none() {
                    hand_str = Some(arg.clone());
                } else if arg.starts_with('-') {
                    eprintln!("❌ Unknown option: {}", arg);
                    process::exit(1);
                }
            }
        }
        i += 1;
    }

    let hand_str = match hand_str {
        Some(h) => h,
        None => {
            eprintln!("❌ No hand provided. Use -h for help.");
            process::exit(1);
        }
    };

    // Parse the hand
    let parsed = match parse_hand_with_aka(&hand_str) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Error parsing hand: {}", e);
            process::exit(1);
        }
    };

    // Validate hand size
    if let Err(e) = validate_hand(&parsed.tiles) {
        eprintln!("❌ Invalid hand: {}", e);
        process::exit(1);
    }

    // Parse winds
    let round_wind = match parse_wind(&round_wind_str) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("❌ {}", e);
            process::exit(1);
        }
    };

    let seat_wind = match parse_wind(&seat_wind_str) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("❌ {}", e);
            process::exit(1);
        }
    };

    // Parse dora indicators
    let dora_indicators = match dora_str.as_ref().map(|s| parse_tile_list(s)).transpose() {
        Ok(d) => d.unwrap_or_default(),
        Err(e) => {
            eprintln!("❌ Error parsing dora: {}", e);
            process::exit(1);
        }
    };

    let ura_indicators = match ura_str.as_ref().map(|s| parse_tile_list(s)).transpose() {
        Ok(u) => u.unwrap_or_default(),
        Err(e) => {
            eprintln!("❌ Error parsing ura dora: {}", e);
            process::exit(1);
        }
    };

    // Parse winning tile
    let winning_tile = match winning_tile_str
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
    let win_type = if tsumo { WinType::Tsumo } else { WinType::Ron };
    let mut context = GameContext::new(win_type, round_wind, seat_wind)
        .with_dora(dora_indicators)
        .with_ura_dora(ura_indicators)
        .with_aka(parsed.aka_count);

    if let Some(wt) = winning_tile {
        context = context.with_winning_tile(wt);
    }

    if open {
        context = context.open();
    }

    if double_riichi {
        context = context.double_riichi();
    } else if riichi {
        context = context.riichi();
    }

    if ippatsu {
        context = context.ippatsu();
    }

    if last_tile {
        context = context.last_tile();
    }

    if rinshan {
        context = context.rinshan();
    }

    if chankan {
        context = context.chankan();
    }

    if tenhou {
        context = context.tenhou();
    }

    if chiihou {
        context = context.chiihou();
    }

    // Decompose the hand
    let counts = to_counts(&parsed.tiles);
    let structures = decompose_hand(&counts);

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
    let results_to_show: &[_] = if show_all { &results } else { &results[..1] };

    // Display results
    let use_unicode = !ascii;

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
