use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Man, // Manzu
    Pin, // Pinzu
    Sou, // Souzu
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Honor {
    // Winds
    East,
    South,
    West,
    North,
    // Dragons
    White,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Tile {
    Suited { suit: Suit, value: u8 }, // value will be 1..9
    Honor(Honor),
}

impl Tile {
    /// Create a suited tile (e.g., 5-man)
    pub fn suited(suit: Suit, value: u8) -> Self {
        Tile::Suited { suit, value }
    }

    /// Create an honor tile
    pub fn honor(honor: Honor) -> Self {
        Tile::Honor(honor)
    }

    /// Is this a simple tile (2-8 of any suit)?
    pub fn is_simple(&self) -> bool {
        match self {
            Tile::Suited { value, .. } => *value >= 2 && *value <= 8,
            Tile::Honor(_) => false,
        }
    }

    /// Is this a terminal (1 or 9) or honor?
    pub fn is_terminal_or_honor(&self) -> bool {
        match self {
            Tile::Suited { value, .. } => *value == 1 || *value == 9,
            Tile::Honor(_) => true,
        }
    }
}

impl TryFrom<&str> for Tile {
    type Error = String;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        // Convert the string to a list of characters
        let chars: Vec<char> = input.chars().collect();

        // Basic validation: "1m" or "ew" are 2 characters
        if chars.len() != 2 {
            return Err(format!("Invalid tile format: {}", input));
        }

        let value_char = chars[0];
        let type_char = chars[1];

        // TASK: Use a 'match' statement here to handle type_char
        // 'm' -> Suit::Man
        // 'p' -> Suit::Pin
        // 's' -> Suit::Sou
        // 'z' -> This is usually how honors are represented (1z = East, etc.)

        match type_char {
            'm' | 'p' | 's' => {
                let suit = match type_char {
                    'm' => Suit::Man,
                    'p' => Suit::Pin,
                    _ => Suit::Sou,
                };

                // Convert value_char to a digit (1-9)
                let val = value_char.to_digit(10).ok_or("Not a digit")? as u8;

                if val < 1 || val > 9 {
                    return Err("Suited tiles must be 1-9".to_string());
                }

                Ok(Tile::Suited { suit, value: val })
            }
            'z' => {
                // Map 1-7 to the Honor enum variants
                // 1=East, 2=South, 3=West, 4=North, 5=White, 6=Green, 7=Red
                // 1. Create a variable for the specific Honor variant
                let val_digit = value_char.to_digit(10).ok_or("Not a digit")? as u8;
                // 2. Use a match on val_digit (1 through 7)
                let honor_variant = match val_digit {
                    1 => Honor::East,
                    2 => Honor::South,
                    3 => Honor::West,
                    4 => Honor::North,
                    5 => Honor::White,
                    6 => Honor::Green,
                    7 => Honor::Red,
                    // ... and so on ...
                    _ => return Err("Out of bounds for Honors".into()),
                };

                // 3. Wrap that variant into the Tile enum and return it successfully
                Ok(Tile::Honor(honor_variant))
            }
            _ => Err(format!("Unknown suit: {}", type_char)),
        }
    }
}

// The "Pretty Printer"
// This lets you use println!("{}", tile) instead of println!("{:?}", tile)
impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tile::Suited { suit, value } => {
                let s = match suit {
                    Suit::Man => 'm',
                    Suit::Pin => 'p',
                    Suit::Sou => 's',
                };
                write!(f, "{}{}", value, s)
            }
            Tile::Honor(h) => {
                let v = match h {
                    Honor::East => 1,
                    Honor::South => 2,
                    Honor::West => 3,
                    Honor::North => 4,
                    Honor::White => 5,
                    Honor::Green => 6,
                    Honor::Red => 7,
                };
                write!(f, "{}z", v)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tiles() {
        // Using the new helper methods
        let five_man = Tile::suited(Suit::Man, 5);
        let red_dragon = Tile::honor(Honor::Red);

        println!("{:?}", five_man);
        println!("{:?}", red_dragon);

        assert_eq!(five_man, Tile::suited(Suit::Man, 5));
    }

    #[test]
    fn tile_properties() {
        // Simples: 2-8 of any suit
        assert!(Tile::suited(Suit::Pin, 5).is_simple());
        assert!(!Tile::suited(Suit::Pin, 1).is_simple());
        assert!(!Tile::suited(Suit::Pin, 9).is_simple());
        assert!(!Tile::honor(Honor::East).is_simple());

        // Terminals and honors
        assert!(Tile::suited(Suit::Sou, 1).is_terminal_or_honor());
        assert!(Tile::suited(Suit::Sou, 9).is_terminal_or_honor());
        assert!(Tile::honor(Honor::White).is_terminal_or_honor());
        assert!(!Tile::suited(Suit::Man, 5).is_terminal_or_honor());
    }
}
