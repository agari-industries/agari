use crate::hand::{KanType, Meld};
use crate::tile::{Honor, Suit, Tile};
use std::collections::HashMap;

pub type TileCounts = HashMap<Tile, u8>;

/// A called meld (kan, pon, or chi) that was declared in the hand notation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalledMeld {
    pub meld: Meld,
    pub tiles: Vec<Tile>,
}

/// Result of parsing a hand, including red five (akadora) count and called melds
#[derive(Debug, Clone)]
pub struct ParsedHand {
    pub tiles: Vec<Tile>,              // Tiles in hand (not in called melds)
    pub aka_count: u8,                 // Number of red fives (0m, 0p, 0s)
    pub called_melds: Vec<CalledMeld>, // Kans and other called melds
}

/// Parse a hand string into tiles.
/// Red fives use '0' notation: 0m = red 5m, 0p = red 5p, 0s = red 5s
pub fn parse_hand(input: &str) -> Result<Vec<Tile>, String> {
    let parsed = parse_hand_with_aka(input)?;
    // Combine hand tiles with tiles from called melds
    let mut all_tiles = parsed.tiles;
    for called in &parsed.called_melds {
        all_tiles.extend(&called.tiles);
    }
    Ok(all_tiles)
}

/// Parse a hand string, also tracking red five count and called melds
///
/// Notation:
/// - Regular tiles: 123m456p789s1234z
/// - Red fives: 0m, 0p, 0s
/// - Closed kan (ankan): [1111m] or [5555z]
/// - Open kan (daiminkan/shouminkan): (1111m) or (5555z)
/// - Open triplet (pon): (111m) or (555z)
/// - Open sequence (chi): (123m)
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

pub fn parse_hand_with_aka(input: &str) -> Result<ParsedHand, String> {
    let mut tiles = Vec::new();
    let mut aka_count = 0u8;
    let mut called_melds = Vec::new();
    // Store (digit, is_red) pairs
    let mut pending: Vec<(u8, bool)> = Vec::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        match ch {
            '[' | '(' => {
                // Start of a called meld
                let is_closed = ch == '[';
                let close_char = if is_closed { ']' } else { ')' };

                // Find the closing bracket
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && chars[end] != close_char {
                    end += 1;
                }

                if end >= chars.len() {
                    return Err(format!("Unclosed bracket starting at position {}", i));
                }

                // Parse the meld content
                let meld_str: String = chars[start..end].iter().collect();
                let (meld, meld_tiles, meld_aka) = parse_meld(&meld_str, is_closed)?;

                called_melds.push(CalledMeld {
                    meld,
                    tiles: meld_tiles,
                });
                aka_count += meld_aka;

                i = end + 1;
                continue;
            }

            ']' | ')' => {
                return Err(format!(
                    "Unexpected closing bracket '{}' at position {}",
                    ch, i
                ));
            }

            '1'..='9' => {
                let digit = ch.to_digit(10).unwrap() as u8;
                pending.push((digit, false));
            }

            '0' => {
                // Red five - treat as 5 but mark as aka
                pending.push((5, true));
            }

            'm' => {
                for &(n, is_red) in &pending {
                    tiles.push(Tile::suited(Suit::Man, n));
                    if is_red {
                        aka_count += 1;
                    }
                }
                pending.clear();
            }
            'p' if !pending.is_empty() => {
                for &(n, is_red) in &pending {
                    tiles.push(Tile::suited(Suit::Pin, n));
                    if is_red {
                        aka_count += 1;
                    }
                }
                pending.clear();
            }
            's' if !pending.is_empty() => {
                for &(n, is_red) in &pending {
                    tiles.push(Tile::suited(Suit::Sou, n));
                    if is_red {
                        aka_count += 1;
                    }
                }
                pending.clear();
            }

            'z' => {
                for &(n, is_red) in &pending {
                    if is_red {
                        return Err("Red fives (0) cannot be used with honors (z)".to_string());
                    }
                    let honor = match n {
                        1 => Honor::East,
                        2 => Honor::South,
                        3 => Honor::West,
                        4 => Honor::North,
                        5 => Honor::White,
                        6 => Honor::Green,
                        7 => Honor::Red,
                        _ => return Err(format!("Invalid honor number: {}", n)),
                    };
                    tiles.push(Tile::honor(honor));
                }
                pending.clear();
            }

            ' ' | '\t' | '\n' => {}

            // Try honor letter notation (e, s, w, n, wh, g, r)
            _ => {
                // First, check if there are pending digits - they need a suit
                if !pending.is_empty() {
                    return Err(format!(
                        "Unexpected character '{}' - pending digits need a suit (m/p/s/z)",
                        ch
                    ));
                }

                // Try to parse as honor letter
                if let Some((honor, consumed)) = try_parse_honor_letter(&chars, i) {
                    tiles.push(Tile::honor(honor));
                    i += consumed;
                    continue;
                }

                return Err(format!("Unexpected character: {}", ch));
            }
        }
        i += 1;
    }

    if !pending.is_empty() {
        return Err("Trailing numbers without suit suffix".to_string());
    }

    Ok(ParsedHand {
        tiles,
        aka_count,
        called_melds,
    })
}

/// Parse a meld string (contents inside brackets)
/// Returns (Meld, tiles, aka_count)
/// Supports both numeric notation (e.g., "111z") and letter notation for honors (e.g., "eee")
fn parse_meld(meld_str: &str, is_closed: bool) -> Result<(Meld, Vec<Tile>, u8), String> {
    let chars: Vec<char> = meld_str.chars().collect();

    if chars.is_empty() {
        return Err("Empty meld".to_string());
    }

    // First, try to parse as honor letter notation (e.g., "eee", "rrr", "wwww")
    // This is detected by checking if all characters are honor letters
    let mut honor_tiles: Vec<Honor> = Vec::new();
    let mut i = 0;
    let mut is_honor_notation = true;

    while i < chars.len() && is_honor_notation {
        if let Some((honor, consumed)) = try_parse_honor_letter(&chars, i) {
            honor_tiles.push(honor);
            i += consumed;
        } else {
            is_honor_notation = false;
        }
    }

    // If we consumed all characters as honors, use honor notation
    if is_honor_notation && i == chars.len() && !honor_tiles.is_empty() {
        let tiles: Vec<Tile> = honor_tiles.iter().map(|&h| Tile::honor(h)).collect();
        let tile_count = tiles.len();

        // Determine the meld type
        let meld = match tile_count {
            4 => {
                let first = tiles[0];
                if !tiles.iter().all(|&t| t == first) {
                    return Err("Kan must have 4 identical tiles".to_string());
                }
                let kan_type = if is_closed {
                    KanType::Closed
                } else {
                    KanType::Open
                };
                Meld::Kan(first, kan_type)
            }
            3 => {
                let first = tiles[0];
                if tiles.iter().all(|&t| t == first) {
                    if is_closed {
                        Meld::koutsu(first)
                    } else {
                        Meld::koutsu_open(first)
                    }
                } else {
                    return Err(
                        "Honor meld must have 3 identical tiles (no sequences with honors)"
                            .to_string(),
                    );
                }
            }
            _ => return Err(format!("Meld must have 3 or 4 tiles, got {}", tile_count)),
        };

        return Ok((meld, tiles, 0)); // No aka dora for honors
    }

    // Fall back to standard numeric notation (e.g., "111z", "123m")
    // Find the suit character (last character)
    let suit_char = chars[chars.len() - 1];
    let suit = match suit_char {
        'm' => Some(Suit::Man),
        'p' => Some(Suit::Pin),
        's' => Some(Suit::Sou),
        'z' => None, // Honor
        _ => return Err(format!("Invalid suit in meld: {}", suit_char)),
    };

    // Parse the numbers
    let mut values: Vec<(u8, bool)> = Vec::new(); // (value, is_red)
    for &ch in &chars[..chars.len() - 1] {
        match ch {
            '1'..='9' => {
                let digit = ch.to_digit(10).unwrap() as u8;
                values.push((digit, false));
            }
            '0' => {
                // Red five
                if suit.is_none() {
                    return Err("Red fives cannot be used with honors".to_string());
                }
                values.push((5, true));
            }
            _ => return Err(format!("Invalid character in meld: {}", ch)),
        }
    }

    let tile_count = values.len();
    let mut aka_count = 0u8;

    // Create the tiles
    let tiles: Vec<Tile> = values
        .iter()
        .map(|&(val, is_red)| {
            if is_red {
                aka_count += 1;
            }
            if let Some(s) = suit {
                Tile::suited(s, val)
            } else {
                // Honor
                let honor = match val {
                    1 => Honor::East,
                    2 => Honor::South,
                    3 => Honor::West,
                    4 => Honor::North,
                    5 => Honor::White,
                    6 => Honor::Green,
                    7 => Honor::Red,
                    _ => panic!("Invalid honor value: {}", val),
                };
                Tile::honor(honor)
            }
        })
        .collect();

    // Determine the meld type
    let meld = match tile_count {
        4 => {
            // Kan - all 4 tiles must be the same
            let first = tiles[0];
            if !tiles.iter().all(|&t| t == first) {
                return Err("Kan must have 4 identical tiles".to_string());
            }
            let kan_type = if is_closed {
                KanType::Closed
            } else {
                KanType::Open
            };
            Meld::Kan(first, kan_type)
        }
        3 => {
            let first = tiles[0];
            if tiles.iter().all(|&t| t == first) {
                // Triplet (pon)
                if is_closed {
                    Meld::koutsu(first)
                } else {
                    Meld::koutsu_open(first)
                }
            } else if suit.is_some() {
                // Sequence (chi) - must be consecutive
                let mut sorted_values: Vec<u8> = values.iter().map(|(v, _)| *v).collect();
                sorted_values.sort();
                if sorted_values[1] == sorted_values[0] + 1
                    && sorted_values[2] == sorted_values[1] + 1
                {
                    let start_tile = Tile::suited(suit.unwrap(), sorted_values[0]);
                    if is_closed {
                        Meld::shuntsu(start_tile)
                    } else {
                        Meld::shuntsu_open(start_tile)
                    }
                } else {
                    return Err("Sequence must have 3 consecutive tiles".to_string());
                }
            } else {
                return Err("Invalid 3-tile meld".to_string());
            }
        }
        _ => return Err(format!("Meld must have 3 or 4 tiles, got {}", tile_count)),
    };

    Ok((meld, tiles, aka_count))
}

pub fn to_counts(tiles: &[Tile]) -> TileCounts {
    let mut counts = HashMap::new();
    for &tile in tiles {
        *counts.entry(tile).or_insert(0) += 1;
    }
    counts
}

/// Validate a hand for scoring (must be exactly 14 tiles, with kans counting as 3)
pub fn validate_hand(tiles: &[Tile]) -> Result<(), String> {
    if tiles.len() != 14 {
        return Err(format!("Hand must have 14 tiles, got {}", tiles.len()));
    }

    let counts = to_counts(tiles);
    for (tile, count) in &counts {
        if *count > 4 {
            return Err(format!("Tile {:?} appears {} times (max 4)", tile, count));
        }
    }

    Ok(())
}

/// Validate a hand with called melds
/// Each kan adds 1 extra tile (4 tiles instead of 3), so:
/// - 0 kans: 14 tiles
/// - 1 kan: 15 tiles
/// - 2 kans: 16 tiles
/// - 3 kans: 17 tiles
/// - 4 kans: 18 tiles
pub fn validate_hand_with_melds(parsed: &ParsedHand) -> Result<(), String> {
    let kan_count = parsed
        .called_melds
        .iter()
        .filter(|m| matches!(m.meld, Meld::Kan(_, _)))
        .count();

    let total_tiles = parsed.tiles.len()
        + parsed
            .called_melds
            .iter()
            .map(|m| m.tiles.len())
            .sum::<usize>();

    let expected = 14 + kan_count;

    if total_tiles != expected {
        return Err(format!(
            "Hand with {} kan(s) must have {} tiles, got {}",
            kan_count, expected, total_tiles
        ));
    }

    // Check that no tile appears more than 4 times
    let mut all_tiles = parsed.tiles.clone();
    for called in &parsed.called_melds {
        all_tiles.extend(&called.tiles);
    }

    let counts = to_counts(&all_tiles);
    for (tile, count) in &counts {
        if *count > 4 {
            return Err(format!("Tile {:?} appears {} times (max 4)", tile, count));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_hand() {
        let tiles = parse_hand("123m456p789s11z").unwrap();
        assert_eq!(tiles.len(), 11);
        assert_eq!(tiles[0], Tile::suited(Suit::Man, 1));
        assert_eq!(tiles[9], Tile::honor(Honor::East));
    }

    #[test]
    fn parse_full_hand() {
        let tiles = parse_hand("123m456p789s11222z").unwrap();
        assert_eq!(tiles.len(), 14);
    }

    #[test]
    fn parse_invalid_honor() {
        let result = parse_hand("89z");
        assert!(result.is_err());
    }

    #[test]
    fn parse_trailing_numbers() {
        let result = parse_hand("123");
        assert!(result.is_err());
    }

    #[test]
    fn validate_correct_hand() {
        let tiles = parse_hand("123m456p789s11222z").unwrap();
        assert!(validate_hand(&tiles).is_ok());
    }

    #[test]
    fn validate_wrong_count() {
        let tiles = parse_hand("123m456p789s11z").unwrap();
        assert!(validate_hand(&tiles).is_err());
    }

    #[test]
    fn validate_too_many_copies() {
        let tiles = parse_hand("11111m456p789s11z").unwrap();
        assert!(validate_hand(&tiles).is_err());
    }

    // ===== Red Five (Akadora) Tests =====

    #[test]
    fn parse_red_five_manzu() {
        let result = parse_hand_with_aka("0m").unwrap();
        assert_eq!(result.tiles.len(), 1);
        assert_eq!(result.tiles[0], Tile::suited(Suit::Man, 5));
        assert_eq!(result.aka_count, 1);
    }

    #[test]
    fn parse_red_five_all_suits() {
        let result = parse_hand_with_aka("0m0p0s").unwrap();
        assert_eq!(result.tiles.len(), 3);
        assert_eq!(result.tiles[0], Tile::suited(Suit::Man, 5));
        assert_eq!(result.tiles[1], Tile::suited(Suit::Pin, 5));
        assert_eq!(result.tiles[2], Tile::suited(Suit::Sou, 5));
        assert_eq!(result.aka_count, 3);
    }

    #[test]
    fn parse_mixed_red_and_regular_fives() {
        // Hand with both red 5m and regular 5m
        let result = parse_hand_with_aka("50m").unwrap();
        assert_eq!(result.tiles.len(), 2);
        assert_eq!(result.tiles[0], Tile::suited(Suit::Man, 5));
        assert_eq!(result.tiles[1], Tile::suited(Suit::Man, 5));
        assert_eq!(result.aka_count, 1); // Only one is red
    }

    #[test]
    fn parse_hand_with_red_five() {
        // Full hand with a red 5p
        let result = parse_hand_with_aka("123m406p789s11122z").unwrap();
        assert_eq!(result.tiles.len(), 14);
        assert_eq!(result.aka_count, 1);
        // The 0 should have been parsed as 5p
        assert_eq!(result.tiles[4], Tile::suited(Suit::Pin, 5));
    }

    #[test]
    fn parse_red_zero_with_honor_fails() {
        let result = parse_hand_with_aka("0z");
        assert!(result.is_err());
    }

    // ===== Kan Notation Tests =====

    #[test]
    fn parse_closed_kan() {
        let result = parse_hand_with_aka("[1111m]").unwrap();
        assert_eq!(result.tiles.len(), 0); // Tiles are in the meld
        assert_eq!(result.called_melds.len(), 1);

        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles.len(), 4);
        assert!(matches!(meld.meld, Meld::Kan(_, KanType::Closed)));
    }

    #[test]
    fn parse_open_kan() {
        let result = parse_hand_with_aka("(1111m)").unwrap();
        assert_eq!(result.called_melds.len(), 1);

        let meld = &result.called_melds[0];
        assert!(matches!(meld.meld, Meld::Kan(_, KanType::Open)));
    }

    #[test]
    fn parse_honor_kan() {
        let result = parse_hand_with_aka("[5555z]").unwrap();
        assert_eq!(result.called_melds.len(), 1);

        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles[0], Tile::honor(Honor::White));
        assert!(matches!(meld.meld, Meld::Kan(_, KanType::Closed)));
    }

    #[test]
    fn parse_hand_with_kan() {
        // Hand with a closed kan: [1111m] 222m 333m 555p 11z
        let result = parse_hand_with_aka("[1111m]222333m555p11z").unwrap();

        // Regular tiles (not in kan)
        assert_eq!(result.tiles.len(), 11); // 222m + 333m + 555p + 11z

        // One kan meld
        assert_eq!(result.called_melds.len(), 1);
        assert_eq!(result.called_melds[0].tiles.len(), 4);

        // Total tiles should be 15
        let total = result.tiles.len() + result.called_melds[0].tiles.len();
        assert_eq!(total, 15);
    }

    #[test]
    fn parse_hand_with_multiple_kans() {
        // Hand with two kans: [1111m] [2222p] 345s 678s 11z
        // 4 + 4 + 3 + 3 + 2 = 16 tiles
        let result = parse_hand_with_aka("[1111m][2222p]345678s11z").unwrap();

        assert_eq!(result.called_melds.len(), 2);
        assert_eq!(result.tiles.len(), 8); // 345s + 678s + 11z

        // Total should be 16 tiles (14 + 2 extra for 2 kans)
        let total: usize = result.tiles.len()
            + result
                .called_melds
                .iter()
                .map(|m| m.tiles.len())
                .sum::<usize>();
        assert_eq!(total, 16);
    }

    #[test]
    fn parse_open_pon() {
        let result = parse_hand_with_aka("(111m)").unwrap();
        assert_eq!(result.called_melds.len(), 1);

        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles.len(), 3);
        assert!(matches!(meld.meld, Meld::Koutsu(_, true))); // open = true
    }

    #[test]
    fn parse_open_chi() {
        let result = parse_hand_with_aka("(123m)").unwrap();
        assert_eq!(result.called_melds.len(), 1);

        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles.len(), 3);
        assert!(matches!(meld.meld, Meld::Shuntsu(_, true))); // open = true
    }

    #[test]
    fn parse_kan_with_red_five() {
        let result = parse_hand_with_aka("[0555m]").unwrap();
        assert_eq!(result.aka_count, 1);
        assert_eq!(result.called_melds.len(), 1);
    }

    #[test]
    fn validate_hand_with_one_kan() {
        let result = parse_hand_with_aka("[1111m]222333m555p11z").unwrap();
        assert!(validate_hand_with_melds(&result).is_ok());
    }

    #[test]
    fn validate_hand_with_two_kans() {
        // [1111m] [2222p] 345s 678s 11z = 16 tiles (14 + 2 for 2 kans)
        let result = parse_hand_with_aka("[1111m][2222p]345678s11z").unwrap();
        assert!(validate_hand_with_melds(&result).is_ok());
    }

    #[test]
    fn invalid_kan_different_tiles() {
        let result = parse_hand_with_aka("[1234m]");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_unclosed_bracket() {
        let result = parse_hand_with_aka("[1111m");
        assert!(result.is_err());
    }

    // ===== Honor Letter Notation Tests =====

    #[test]
    fn parse_honor_letter_winds() {
        // Test all wind letter notations: e, s, w, n
        let result = parse_hand_with_aka("eswn").unwrap();
        assert_eq!(result.tiles.len(), 4);
        assert_eq!(result.tiles[0], Tile::honor(Honor::East));
        assert_eq!(result.tiles[1], Tile::honor(Honor::South));
        assert_eq!(result.tiles[2], Tile::honor(Honor::West));
        assert_eq!(result.tiles[3], Tile::honor(Honor::North));
    }

    #[test]
    fn parse_honor_letter_dragons() {
        // Test dragon letter notations: wh, g, r
        let result = parse_hand_with_aka("whgr").unwrap();
        assert_eq!(result.tiles.len(), 3);
        assert_eq!(result.tiles[0], Tile::honor(Honor::White));
        assert_eq!(result.tiles[1], Tile::honor(Honor::Green));
        assert_eq!(result.tiles[2], Tile::honor(Honor::Red));
    }

    #[test]
    fn parse_honor_letter_mixed_with_suits() {
        // Mix letter honors with suited tiles
        let result = parse_hand_with_aka("123meee").unwrap();
        assert_eq!(result.tiles.len(), 6);
        assert_eq!(result.tiles[0], Tile::suited(Suit::Man, 1));
        assert_eq!(result.tiles[3], Tile::honor(Honor::East));
        assert_eq!(result.tiles[4], Tile::honor(Honor::East));
        assert_eq!(result.tiles[5], Tile::honor(Honor::East));
    }

    #[test]
    fn parse_honor_letter_full_hand() {
        // Full hand using letter notation: 123m456p789seeenn
        let result = parse_hand_with_aka("123m456p789seeenn").unwrap();
        assert_eq!(result.tiles.len(), 14);
        // Check East tiles
        assert_eq!(result.tiles[9], Tile::honor(Honor::East));
        assert_eq!(result.tiles[10], Tile::honor(Honor::East));
        assert_eq!(result.tiles[11], Tile::honor(Honor::East));
        // Check North tiles
        assert_eq!(result.tiles[12], Tile::honor(Honor::North));
        assert_eq!(result.tiles[13], Tile::honor(Honor::North));
    }

    #[test]
    fn parse_honor_letter_white_dragon_disambiguation() {
        // Test that "wh" parses as White, not West + something
        let result = parse_hand_with_aka("whwhwh").unwrap();
        assert_eq!(result.tiles.len(), 3);
        assert!(result.tiles.iter().all(|&t| t == Tile::honor(Honor::White)));
    }

    #[test]
    fn parse_honor_letter_west_vs_white() {
        // w = West, wh = White - ensure disambiguation works
        let result = parse_hand_with_aka("wwwwhwh").unwrap();
        assert_eq!(result.tiles.len(), 5);
        assert_eq!(result.tiles[0], Tile::honor(Honor::West));
        assert_eq!(result.tiles[1], Tile::honor(Honor::West));
        assert_eq!(result.tiles[2], Tile::honor(Honor::West));
        assert_eq!(result.tiles[3], Tile::honor(Honor::White));
        assert_eq!(result.tiles[4], Tile::honor(Honor::White));
    }

    #[test]
    fn parse_honor_letter_uppercase() {
        // Test that uppercase letters also work
        let result = parse_hand_with_aka("ESWN").unwrap();
        assert_eq!(result.tiles.len(), 4);
        assert_eq!(result.tiles[0], Tile::honor(Honor::East));
        assert_eq!(result.tiles[1], Tile::honor(Honor::South));
        assert_eq!(result.tiles[2], Tile::honor(Honor::West));
        assert_eq!(result.tiles[3], Tile::honor(Honor::North));
    }

    #[test]
    fn parse_honor_letter_meld_pon() {
        // Test honor letter notation in called melds: (eee) = pon of East
        let result = parse_hand_with_aka("(eee)").unwrap();
        assert_eq!(result.called_melds.len(), 1);
        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles.len(), 3);
        assert!(meld.tiles.iter().all(|&t| t == Tile::honor(Honor::East)));
        assert!(matches!(meld.meld, Meld::Koutsu(_, true))); // open = true
    }

    #[test]
    fn parse_honor_letter_meld_kan() {
        // Test honor letter notation in kans: [rrrr] = closed kan of Red dragon
        let result = parse_hand_with_aka("[rrrr]").unwrap();
        assert_eq!(result.called_melds.len(), 1);
        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles.len(), 4);
        assert!(meld.tiles.iter().all(|&t| t == Tile::honor(Honor::Red)));
        assert!(matches!(meld.meld, Meld::Kan(_, KanType::Closed)));
    }

    #[test]
    fn parse_honor_letter_meld_white_dragon() {
        // Test white dragon kan with letter notation: [whwhwhwh]
        let result = parse_hand_with_aka("[whwhwhwh]").unwrap();
        assert_eq!(result.called_melds.len(), 1);
        let meld = &result.called_melds[0];
        assert_eq!(meld.tiles.len(), 4);
        assert!(meld.tiles.iter().all(|&t| t == Tile::honor(Honor::White)));
    }

    #[test]
    fn parse_honor_letter_hand_with_meld() {
        // Full hand with letter notation meld: 123m456p789s(rrr)whwh
        let result = parse_hand_with_aka("123m456p789s(rrr)whwh").unwrap();
        assert_eq!(result.tiles.len(), 11); // 9 suited + 2 white
        assert_eq!(result.called_melds.len(), 1);

        // Check white dragon pair
        assert_eq!(result.tiles[9], Tile::honor(Honor::White));
        assert_eq!(result.tiles[10], Tile::honor(Honor::White));

        // Check red dragon pon
        let meld = &result.called_melds[0];
        assert!(meld.tiles.iter().all(|&t| t == Tile::honor(Honor::Red)));
    }

    #[test]
    fn parse_honor_letter_invalid_after_digits() {
        // Digits followed by honor letter should fail (digits need a suit)
        let result = parse_hand_with_aka("123e");
        assert!(result.is_err());
    }
}
