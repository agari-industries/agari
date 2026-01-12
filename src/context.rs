//! Game context for scoring - tracks win conditions, winds, dora, etc.

use crate::tile::{Tile, Honor};
use crate::parse::TileCounts;

/// How the hand was won
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinType {
    /// Won by taking another player's discard
    Ron,
    /// Won by self-draw
    Tsumo,
}

/// Complete game context needed for scoring
#[derive(Debug, Clone)]
pub struct GameContext {
    // === Win condition ===
    pub win_type: WinType,
    
    // === Winds ===
    /// The round wind (East round = East, South round = South, etc.)
    pub round_wind: Honor,
    /// The player's seat wind
    pub seat_wind: Honor,
    
    // === Hand state ===
    /// Whether the hand has called tiles (chi/pon/kan)
    /// A closed hand (menzen) has is_open = false
    pub is_open: bool,
    
    // === Riichi ===
    pub is_riichi: bool,
    pub is_double_riichi: bool,
    pub is_ippatsu: bool,
    
    // === Situational yaku ===
    /// Won on kan replacement tile (rinshan kaihou)
    pub is_rinshan: bool,
    /// Last tile of the game (haitei for tsumo, houtei for ron)
    pub is_last_tile: bool,
    /// Dealer's first draw win (tenhou) - only valid for dealer + tsumo + first draw
    pub is_tenhou: bool,
    /// Non-dealer's first draw win (chiihou) - for future use
    pub is_chiihou: bool,
    
    // === Dora ===
    /// Dora indicators (the tile shown, not the actual dora)
    pub dora_indicators: Vec<Tile>,
    /// Ura dora indicators (revealed only on riichi win)
    pub ura_dora_indicators: Vec<Tile>,
    
    // === Akadora (red fives) ===
    /// Number of red fives in the winning hand
    pub aka_count: u8,
}

impl GameContext {
    /// Create a basic context with minimal info
    pub fn new(win_type: WinType, round_wind: Honor, seat_wind: Honor) -> Self {
        GameContext {
            win_type,
            round_wind,
            seat_wind,
            is_open: false,
            is_riichi: false,
            is_double_riichi: false,
            is_ippatsu: false,
            is_rinshan: false,
            is_last_tile: false,
            is_tenhou: false,
            is_chiihou: false,
            dora_indicators: Vec::new(),
            ura_dora_indicators: Vec::new(),
            aka_count: 0,
        }
    }

    /// Builder-style: set hand as open
    pub fn open(mut self) -> Self {
        self.is_open = true;
        self
    }

    /// Builder-style: set riichi
    pub fn riichi(mut self) -> Self {
        self.is_riichi = true;
        self
    }

    /// Builder-style: set double riichi
    pub fn double_riichi(mut self) -> Self {
        self.is_double_riichi = true;
        self.is_riichi = true;
        self
    }

    /// Builder-style: set ippatsu
    pub fn ippatsu(mut self) -> Self {
        self.is_ippatsu = true;
        self
    }

    /// Builder-style: set rinshan (kan replacement win)
    pub fn rinshan(mut self) -> Self {
        self.is_rinshan = true;
        self
    }

    /// Builder-style: set last tile (haitei/houtei)
    pub fn last_tile(mut self) -> Self {
        self.is_last_tile = true;
        self
    }

    /// Builder-style: set tenhou (dealer first draw win)
    pub fn tenhou(mut self) -> Self {
        self.is_tenhou = true;
        self
    }

    /// Builder-style: set chiihou (non-dealer first draw win)
    pub fn chiihou(mut self) -> Self {
        self.is_chiihou = true;
        self
    }

    /// Builder-style: add dora indicator(s)
    pub fn with_dora(mut self, indicators: Vec<Tile>) -> Self {
        self.dora_indicators = indicators;
        self
    }

    /// Builder-style: add ura dora indicator(s)
    pub fn with_ura_dora(mut self, indicators: Vec<Tile>) -> Self {
        self.ura_dora_indicators = indicators;
        self
    }

    /// Builder-style: set aka (red five) count
    pub fn with_aka(mut self, count: u8) -> Self {
        self.aka_count = count;
        self
    }

    /// Check if this wind is a value wind (round or seat wind)
    pub fn is_value_wind(&self, wind: Honor) -> bool {
        wind == self.round_wind || wind == self.seat_wind
    }
    
    /// Check if hand is closed (menzen)
    pub fn is_closed(&self) -> bool {
        !self.is_open
    }
    
    /// Check if player is dealer (seat wind == East)
    pub fn is_dealer(&self) -> bool {
        self.seat_wind == Honor::East
    }
}

/// Calculate what tile is dora given a dora indicator
/// 
/// Dora indicator -> Actual dora:
/// - Suited: indicator + 1 (wraps 9 -> 1)
/// - Winds: E -> S -> W -> N -> E
/// - Dragons: White -> Green -> Red -> White
pub fn indicator_to_dora(indicator: Tile) -> Tile {
    match indicator {
        Tile::Suited { suit, value } => {
            let next_value = if value == 9 { 1 } else { value + 1 };
            Tile::suited(suit, next_value)
        }
        Tile::Honor(honor) => {
            let next_honor = match honor {
                // Winds cycle: E -> S -> W -> N -> E
                Honor::East => Honor::South,
                Honor::South => Honor::West,
                Honor::West => Honor::North,
                Honor::North => Honor::East,
                // Dragons cycle: White -> Green -> Red -> White
                Honor::White => Honor::Green,
                Honor::Green => Honor::Red,
                Honor::Red => Honor::White,
            };
            Tile::honor(next_honor)
        }
    }
}

/// Count total dora in a hand given the game context
pub fn count_dora(counts: &TileCounts, context: &GameContext) -> u8 {
    let mut total = 0u8;
    
    // Count regular dora
    for indicator in &context.dora_indicators {
        let dora = indicator_to_dora(*indicator);
        total += counts.get(&dora).copied().unwrap_or(0);
    }
    
    // Count ura dora (only if riichi)
    if context.is_riichi {
        for indicator in &context.ura_dora_indicators {
            let dora = indicator_to_dora(*indicator);
            total += counts.get(&dora).copied().unwrap_or(0);
        }
    }
    
    // Add akadora count
    total += context.aka_count;
    
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Suit;
    use crate::parse::{parse_hand, to_counts};

    #[test]
    fn test_indicator_to_dora_suited() {
        // 1m indicator -> 2m dora
        assert_eq!(
            indicator_to_dora(Tile::suited(Suit::Man, 1)),
            Tile::suited(Suit::Man, 2)
        );
        
        // 9p indicator -> 1p dora (wraps)
        assert_eq!(
            indicator_to_dora(Tile::suited(Suit::Pin, 9)),
            Tile::suited(Suit::Pin, 1)
        );
    }

    #[test]
    fn test_indicator_to_dora_winds() {
        assert_eq!(indicator_to_dora(Tile::honor(Honor::East)), Tile::honor(Honor::South));
        assert_eq!(indicator_to_dora(Tile::honor(Honor::South)), Tile::honor(Honor::West));
        assert_eq!(indicator_to_dora(Tile::honor(Honor::West)), Tile::honor(Honor::North));
        assert_eq!(indicator_to_dora(Tile::honor(Honor::North)), Tile::honor(Honor::East));
    }

    #[test]
    fn test_indicator_to_dora_dragons() {
        assert_eq!(indicator_to_dora(Tile::honor(Honor::White)), Tile::honor(Honor::Green));
        assert_eq!(indicator_to_dora(Tile::honor(Honor::Green)), Tile::honor(Honor::Red));
        assert_eq!(indicator_to_dora(Tile::honor(Honor::Red)), Tile::honor(Honor::White));
    }

    #[test]
    fn test_count_dora_simple() {
        // Hand with three 2m tiles, dora indicator is 1m (so 2m is dora)
        let tiles = parse_hand("222m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_dora(vec![Tile::suited(Suit::Man, 1)]);
        
        assert_eq!(count_dora(&counts, &context), 3);
    }

    #[test]
    fn test_count_dora_with_ura() {
        // Hand with 2m and 5p
        let tiles = parse_hand("222m555p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        
        // Dora indicator 1m (dora = 2m), ura indicator 4p (ura = 5p)
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .riichi()
            .with_dora(vec![Tile::suited(Suit::Man, 1)])
            .with_ura_dora(vec![Tile::suited(Suit::Pin, 4)]);
        
        // 3 dora (2m) + 3 ura (5p) = 6
        assert_eq!(count_dora(&counts, &context), 6);
    }

    #[test]
    fn test_count_dora_ura_only_with_riichi() {
        let tiles = parse_hand("222m555p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        
        // Same indicators but NO riichi
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_dora(vec![Tile::suited(Suit::Man, 1)])
            .with_ura_dora(vec![Tile::suited(Suit::Pin, 4)]);
        
        // Only regular dora counts (3), ura doesn't count without riichi
        assert_eq!(count_dora(&counts, &context), 3);
    }

    #[test]
    fn test_count_dora_with_aka() {
        let tiles = parse_hand("123m456p789s11122z").unwrap();
        let counts = to_counts(&tiles);
        
        // No dora indicators, but 2 akadora
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::East)
            .with_aka(2);
        
        assert_eq!(count_dora(&counts, &context), 2);
    }

    #[test]
    fn test_value_wind() {
        let context = GameContext::new(WinType::Ron, Honor::East, Honor::South);
        
        assert!(context.is_value_wind(Honor::East));   // Round wind
        assert!(context.is_value_wind(Honor::South));  // Seat wind
        assert!(!context.is_value_wind(Honor::West));
        assert!(!context.is_value_wind(Honor::North));
    }

    #[test]
    fn test_builder_pattern() {
        let context = GameContext::new(WinType::Tsumo, Honor::South, Honor::West)
            .riichi()
            .ippatsu()
            .with_dora(vec![Tile::suited(Suit::Man, 1)])
            .with_aka(1);
        
        assert!(context.is_riichi);
        assert!(context.is_ippatsu);
        assert_eq!(context.dora_indicators.len(), 1);
        assert_eq!(context.aka_count, 1);
        assert!(context.is_closed());
    }
}
