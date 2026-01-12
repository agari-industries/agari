use std::collections::HashMap;
use crate::tile::{Tile, Suit, Honor};

pub type TileCounts = HashMap<Tile, u8>;

/// Result of parsing a hand, including red five (akadora) count
#[derive(Debug, Clone)]
pub struct ParsedHand {
    pub tiles: Vec<Tile>,
    pub aka_count: u8,  // Number of red fives (0m, 0p, 0s)
}

/// Parse a hand string into tiles.
/// Red fives use '0' notation: 0m = red 5m, 0p = red 5p, 0s = red 5s
pub fn parse_hand(input: &str) -> Result<Vec<Tile>, String> {
    Ok(parse_hand_with_aka(input)?.tiles)
}

/// Parse a hand string, also tracking red five count
pub fn parse_hand_with_aka(input: &str) -> Result<ParsedHand, String> {
    let mut tiles = Vec::new();
    let mut aka_count = 0u8;
    // Store (digit, is_red) pairs
    let mut pending: Vec<(u8, bool)> = Vec::new();

    for ch in input.chars() {
        match ch {
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
            'p' => {
                for &(n, is_red) in &pending {
                    tiles.push(Tile::suited(Suit::Pin, n));
                    if is_red {
                        aka_count += 1;
                    }
                }
                pending.clear();
            }
            's' => {
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

            _ => return Err(format!("Unexpected character: {}", ch)),
        }
    }

    if !pending.is_empty() {
        return Err("Trailing numbers without suit suffix".to_string());
    }

    Ok(ParsedHand { tiles, aka_count })
}

pub fn to_counts(tiles: &[Tile]) -> TileCounts {
    let mut counts = HashMap::new();
    for &tile in tiles {
        *counts.entry(tile).or_insert(0) += 1;
    }
    counts
}

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
}
