//! Display utilities for pretty-printing mahjong tiles and hands.
//!
//! Supports both Unicode mahjong characters (🀇🀈🀉...) and ASCII fallback.

use crate::hand::{HandStructure, KanType, Meld};
use crate::parse::ParsedHand;
use crate::tile::{Honor, KOKUSHI_TILES, Suit, Tile};

/// Get the Unicode character for a tile with a trailing space for better rendering.
pub fn tile_to_unicode(tile: &Tile) -> String {
    match tile {
        Tile::Suited { suit, value } => {
            let base = match suit {
                Suit::Man => 0x1F007, // 🀇 = 1-man
                Suit::Pin => 0x1F019, // 🀙 = 1-pin
                Suit::Sou => 0x1F010, // 🀐 = 1-sou
            };
            let c = char::from_u32(base + (*value as u32) - 1).unwrap_or('?');
            format!("{c} ")
        }
        Tile::Honor(honor) => {
            let s = match honor {
                Honor::East => "🀀 ",
                Honor::South => "🀁 ",
                Honor::West => "🀂 ",
                Honor::North => "🀃 ",
                Honor::White => "🀆 ",
                Honor::Green => "🀅 ",
                Honor::Red => "🀄︎ ", // Includes variation selector + space
            };
            s.to_string()
        }
    }
}

/// Get a colored ASCII representation of a tile
pub fn tile_to_ascii(tile: &Tile) -> String {
    match tile {
        Tile::Suited { suit, value } => {
            let s = match suit {
                Suit::Man => 'm',
                Suit::Pin => 'p',
                Suit::Sou => 's',
            };
            format!("{}{}", value, s)
        }
        Tile::Honor(honor) => match honor {
            Honor::East => "E".to_string(),
            Honor::South => "S".to_string(),
            Honor::West => "W".to_string(),
            Honor::North => "N".to_string(),
            Honor::White => "Wh".to_string(),
            Honor::Green => "Gr".to_string(),
            Honor::Red => "Rd".to_string(),
        },
    }
}

/// Format a slice of tiles as Unicode characters
pub fn tiles_to_unicode(tiles: &[Tile]) -> String {
    tiles.iter().map(tile_to_unicode).collect()
}

/// Format a slice of tiles as ASCII
pub fn tiles_to_ascii(tiles: &[Tile]) -> String {
    let mut result = String::new();
    let mut current_suit: Option<Suit> = None;
    let mut pending_values: Vec<u8> = Vec::new();
    let mut honors: Vec<&Honor> = Vec::new();

    for tile in tiles {
        match tile {
            Tile::Suited { suit, value } => {
                if current_suit == Some(*suit) {
                    pending_values.push(*value);
                } else {
                    if let Some(s) = current_suit {
                        for v in &pending_values {
                            result.push_str(&v.to_string());
                        }
                        result.push(match s {
                            Suit::Man => 'm',
                            Suit::Pin => 'p',
                            Suit::Sou => 's',
                        });
                    }
                    pending_values.clear();
                    pending_values.push(*value);
                    current_suit = Some(*suit);
                }
            }
            Tile::Honor(h) => {
                if let Some(s) = current_suit {
                    for v in &pending_values {
                        result.push_str(&v.to_string());
                    }
                    result.push(match s {
                        Suit::Man => 'm',
                        Suit::Pin => 'p',
                        Suit::Sou => 's',
                    });
                    pending_values.clear();
                    current_suit = None;
                }
                honors.push(h);
            }
        }
    }

    if let Some(s) = current_suit {
        for v in &pending_values {
            result.push_str(&v.to_string());
        }
        result.push(match s {
            Suit::Man => 'm',
            Suit::Pin => 'p',
            Suit::Sou => 's',
        });
    }

    if !honors.is_empty() {
        for h in &honors {
            let n = match h {
                Honor::East => '1',
                Honor::South => '2',
                Honor::West => '3',
                Honor::North => '4',
                Honor::White => '5',
                Honor::Green => '6',
                Honor::Red => '7',
            };
            result.push(n);
        }
        result.push('z');
    }

    result
}

/// Format a ParsedHand to normalized notation string (standard numeric notation)
/// This produces machine-readable output suitable for JSON, using notation like "123m456p789s11144z"
pub fn format_hand_normalized(parsed: &ParsedHand) -> String {
    let mut result = String::new();

    // Format called melds first (they appear at the start in the original notation)
    for called in &parsed.called_melds {
        let bracket = match &called.meld {
            Meld::Kan(_, KanType::Closed) => ('[', ']'),
            _ => ('(', ')'),
        };

        result.push(bracket.0);
        result.push_str(&tiles_to_ascii(&called.tiles));
        result.push(bracket.1);
    }

    // Then format the hand tiles
    result.push_str(&tiles_to_ascii(&parsed.tiles));

    result
}

/// Format a tile to compact notation (e.g., "1m", "5z")
fn tile_to_notation(tile: &Tile) -> (String, char) {
    match tile {
        Tile::Suited { suit, value } => {
            let suit_char = match suit {
                Suit::Man => 'm',
                Suit::Pin => 'p',
                Suit::Sou => 's',
            };
            (value.to_string(), suit_char)
        }
        Tile::Honor(honor) => {
            let value = match honor {
                Honor::East => '1',
                Honor::South => '2',
                Honor::West => '3',
                Honor::North => '4',
                Honor::White => '5',
                Honor::Green => '6',
                Honor::Red => '7',
            };
            (value.to_string(), 'z')
        }
    }
}

/// Format a meld using standard numeric notation (e.g., [123m], [111z])
/// This is used for machine-readable JSON output
fn format_meld_normalized(meld: &Meld) -> String {
    match meld {
        Meld::Shuntsu(start, _is_open) => {
            if let Tile::Suited { suit, value } = start {
                let suit_char = match suit {
                    Suit::Man => 'm',
                    Suit::Pin => 'p',
                    Suit::Sou => 's',
                };
                format!("[{}{}{}{}]", value, value + 1, value + 2, suit_char)
            } else {
                "???".to_string()
            }
        }
        Meld::Koutsu(tile, _is_open) => {
            let (val, suit) = tile_to_notation(tile);
            format!("[{}{}{}{}]", val, val, val, suit)
        }
        Meld::Kan(tile, _kan_type) => {
            let (val, suit) = tile_to_notation(tile);
            format!("[{}{}{}{}{}]", val, val, val, val, suit)
        }
    }
}

/// Format a hand structure using standard numeric notation
/// This produces machine-readable output suitable for JSON, e.g., "[123m] [456p] [789s] [111z] [44z]"
pub fn format_structure_normalized(structure: &HandStructure) -> String {
    match structure {
        HandStructure::Chiitoitsu { pairs } => {
            let mut sorted_pairs = pairs.clone();
            sorted_pairs.sort();

            sorted_pairs
                .iter()
                .map(|t| {
                    let (val, suit) = tile_to_notation(t);
                    format!("[{}{}{}]", val, val, suit)
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
        HandStructure::Kokushi { pair } => {
            let mut tiles: Vec<Tile> = KOKUSHI_TILES.to_vec();
            tiles.sort();

            let tile_strs: Vec<String> = tiles
                .iter()
                .map(|t| {
                    let (val, suit) = tile_to_notation(t);
                    if t == pair {
                        format!("[{}{}{}]", val, val, suit)
                    } else {
                        format!("[{}{}]", val, suit)
                    }
                })
                .collect();
            tile_strs.join(" ")
        }
        HandStructure::Standard { melds, pair } => {
            let mut parts: Vec<String> = melds.iter().map(|m| format_meld_normalized(m)).collect();

            let (val, suit) = tile_to_notation(pair);
            parts.push(format!("[{}{}{}]", val, val, suit));

            parts.join(" ")
        }
    }
}

/// Format a meld for display
pub fn format_meld(meld: &Meld, use_unicode: bool) -> String {
    match meld {
        Meld::Shuntsu(start, _is_open) => {
            if let Tile::Suited { suit, value } = start {
                let tiles = [
                    Tile::suited(*suit, *value),
                    Tile::suited(*suit, *value + 1),
                    Tile::suited(*suit, *value + 2),
                ];
                if use_unicode {
                    tiles_to_unicode(&tiles)
                } else {
                    format!(
                        "[{}{}{}{}]",
                        value,
                        value + 1,
                        value + 2,
                        match suit {
                            Suit::Man => 'm',
                            Suit::Pin => 'p',
                            Suit::Sou => 's',
                        }
                    )
                }
            } else {
                "???".to_string()
            }
        }
        Meld::Koutsu(tile, _is_open) => {
            let tiles = [*tile, *tile, *tile];
            if use_unicode {
                tiles_to_unicode(&tiles)
            } else {
                let ascii = tile_to_ascii(tile);
                format!("[{ascii}{ascii}{ascii}]")
            }
        }
        Meld::Kan(tile, _kan_type) => {
            let tiles = [*tile, *tile, *tile, *tile];
            if use_unicode {
                tiles_to_unicode(&tiles)
            } else {
                let ascii = tile_to_ascii(tile);
                format!("[{ascii}{ascii}{ascii}{ascii}]")
            }
        }
    }
}

/// Format a hand structure for display
pub fn format_structure(structure: &HandStructure, use_unicode: bool) -> String {
    match structure {
        HandStructure::Chiitoitsu { pairs } => {
            let mut sorted_pairs = pairs.clone();
            sorted_pairs.sort();

            if use_unicode {
                sorted_pairs
                    .iter()
                    .map(|t| {
                        let uni = tile_to_unicode(t);
                        format!("{uni}{uni}")
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                sorted_pairs
                    .iter()
                    .map(|t| {
                        let ascii = tile_to_ascii(t);
                        format!("[{ascii}{ascii}]")
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
        HandStructure::Kokushi { pair } => {
            // Display all 13 kokushi tiles, with the pair tile shown twice
            let mut tiles: Vec<Tile> = KOKUSHI_TILES.to_vec();
            tiles.sort();

            if use_unicode {
                let tile_strs: Vec<String> = tiles
                    .iter()
                    .map(|t| {
                        if t == pair {
                            // Show pair tile twice
                            format!("{}{}", tile_to_unicode(t), tile_to_unicode(t))
                        } else {
                            tile_to_unicode(t)
                        }
                    })
                    .collect();
                tile_strs.join(" ")
            } else {
                let tile_strs: Vec<String> = tiles
                    .iter()
                    .map(|t| {
                        let ascii = tile_to_ascii(t);
                        if t == pair {
                            format!("[{ascii}{ascii}]")
                        } else {
                            format!("[{ascii}]")
                        }
                    })
                    .collect();
                tile_strs.join(" ")
            }
        }
        HandStructure::Standard { melds, pair } => {
            let mut parts: Vec<String> =
                melds.iter().map(|m| format_meld(m, use_unicode)).collect();

            if use_unicode {
                let uni = tile_to_unicode(pair);
                parts.push(format!("{uni}{uni}"));
            } else {
                let ascii = tile_to_ascii(pair);
                parts.push(format!("[{ascii}{ascii}]"));
            }

            parts.join(" ")
        }
    }
}

/// Get honor name for display
pub fn honor_name(honor: &Honor) -> &'static str {
    match honor {
        Honor::East => "East",
        Honor::South => "South",
        Honor::West => "West",
        Honor::North => "North",
        Honor::White => "White Dragon",
        Honor::Green => "Green Dragon",
        Honor::Red => "Red Dragon",
    }
}

/// Suit name for display
pub fn suit_name(suit: &Suit) -> &'static str {
    match suit {
        Suit::Man => "Man (Characters)",
        Suit::Pin => "Pin (Dots)",
        Suit::Sou => "Sou (Bamboo)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_to_unicode() {
        assert_eq!(tile_to_unicode(&Tile::suited(Suit::Man, 1)), "🀇 ");
        assert_eq!(tile_to_unicode(&Tile::suited(Suit::Man, 9)), "🀏 ");
        assert_eq!(tile_to_unicode(&Tile::honor(Honor::East)), "🀀 ");
        assert_eq!(tile_to_unicode(&Tile::honor(Honor::Red)), "🀄︎ ");
    }

    #[test]
    fn test_tiles_to_unicode() {
        let tiles = [
            Tile::suited(Suit::Man, 1),
            Tile::suited(Suit::Man, 2),
            Tile::suited(Suit::Man, 3),
        ];
        assert_eq!(tiles_to_unicode(&tiles), "🀇 🀈 🀉 ");
    }
}
