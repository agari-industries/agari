//! Display utilities for pretty-printing mahjong tiles and hands.
//!
//! Supports both Unicode mahjong characters (🀇🀈🀉...) and ASCII fallback.

use crate::tile::{Tile, Suit, Honor};
use crate::hand::{HandStructure, Meld};

/// Unicode Mahjong tile characters
/// Man (Characters): 🀇🀈🀉🀊🀋🀌🀍🀎🀏
/// Pin (Dots):       🀙🀚🀛🀜🀝🀞🀟🀠🀡
/// Sou (Bamboo):     🀐🀑🀒🀓🀔🀕🀖🀗🀘
/// Winds:           🀀🀁🀂🀃 (E S W N)
/// Dragons:         🀆🀅🀄 (White Green Red)

/// Get the Unicode character for a tile
pub fn tile_to_unicode(tile: &Tile) -> char {
    match tile {
        Tile::Suited { suit, value } => {
            let base = match suit {
                Suit::Man => 0x1F007, // 🀇 = 1-man
                Suit::Pin => 0x1F019, // 🀙 = 1-pin
                Suit::Sou => 0x1F010, // 🀐 = 1-sou
            };
            char::from_u32(base + (*value as u32) - 1).unwrap_or('?')
        }
        Tile::Honor(honor) => {
            match honor {
                Honor::East => '🀀',
                Honor::South => '🀁',
                Honor::West => '🀂',
                Honor::North => '🀃',
                Honor::White => '🀆',
                Honor::Green => '🀅',
                Honor::Red => '🀄',
            }
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
        Tile::Honor(honor) => {
            match honor {
                Honor::East => "E".to_string(),
                Honor::South => "S".to_string(),
                Honor::West => "W".to_string(),
                Honor::North => "N".to_string(),
                Honor::White => "Wh".to_string(),
                Honor::Green => "Gr".to_string(),
                Honor::Red => "Rd".to_string(),
            }
        }
    }
}

/// Format a vector of tiles as Unicode characters
pub fn tiles_to_unicode(tiles: &[Tile]) -> String {
    tiles.iter().map(tile_to_unicode).collect()
}

/// Format a vector of tiles as ASCII
pub fn tiles_to_ascii(tiles: &[Tile]) -> String {
    // Group tiles by suit for compact notation
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
                    // Flush previous suit
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
                // Flush pending suited tiles first
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
    
    // Flush remaining suited tiles
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
    
    // Add honors
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

/// Format a meld for display
pub fn format_meld(meld: &Meld, use_unicode: bool) -> String {
    match meld {
        Meld::Shuntsu(start) => {
            if let Tile::Suited { suit, value } = start {
                let tiles = vec![
                    Tile::suited(*suit, *value),
                    Tile::suited(*suit, *value + 1),
                    Tile::suited(*suit, *value + 2),
                ];
                if use_unicode {
                    tiles_to_unicode(&tiles)
                } else {
                    format!("[{}{}{}{}]", value, value + 1, value + 2, 
                        match suit { Suit::Man => 'm', Suit::Pin => 'p', Suit::Sou => 's' })
                }
            } else {
                "???".to_string()
            }
        }
        Meld::Koutsu(tile) => {
            let tiles = vec![*tile, *tile, *tile];
            if use_unicode {
                tiles_to_unicode(&tiles)
            } else {
                format!("[{}{}{}]", 
                    tile_to_ascii(tile), 
                    tile_to_ascii(tile), 
                    tile_to_ascii(tile))
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
                sorted_pairs.iter()
                    .map(|t| format!("{}{}", tile_to_unicode(t), tile_to_unicode(t)))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else {
                sorted_pairs.iter()
                    .map(|t| format!("[{}{}]", tile_to_ascii(t), tile_to_ascii(t)))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
        HandStructure::Standard { melds, pair } => {
            let mut parts: Vec<String> = melds.iter()
                .map(|m| format_meld(m, use_unicode))
                .collect();
            
            // Add pair
            if use_unicode {
                parts.push(format!("{}{}", tile_to_unicode(pair), tile_to_unicode(pair)));
            } else {
                parts.push(format!("[{}{}]", tile_to_ascii(pair), tile_to_ascii(pair)));
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
        assert_eq!(tile_to_unicode(&Tile::suited(Suit::Man, 1)), '🀇');
        assert_eq!(tile_to_unicode(&Tile::suited(Suit::Man, 9)), '🀏');
        assert_eq!(tile_to_unicode(&Tile::suited(Suit::Pin, 1)), '🀙');
        assert_eq!(tile_to_unicode(&Tile::suited(Suit::Sou, 1)), '🀐');
        assert_eq!(tile_to_unicode(&Tile::honor(Honor::East)), '🀀');
        assert_eq!(tile_to_unicode(&Tile::honor(Honor::Red)), '🀄');
    }
    
    #[test]
    fn test_tiles_to_unicode() {
        let tiles = vec![
            Tile::suited(Suit::Man, 1),
            Tile::suited(Suit::Man, 2),
            Tile::suited(Suit::Man, 3),
        ];
        assert_eq!(tiles_to_unicode(&tiles), "🀇🀈🀉");
    }
    
    #[test]
    fn test_format_meld_unicode() {
        let seq = Meld::Shuntsu(Tile::suited(Suit::Man, 1));
        assert_eq!(format_meld(&seq, true), "🀇🀈🀉");
        
        let triplet = Meld::Koutsu(Tile::honor(Honor::East));
        assert_eq!(format_meld(&triplet, true), "🀀🀀🀀");
    }
}
