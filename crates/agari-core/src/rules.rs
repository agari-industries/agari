//! Scoring rules: the game variant and optional local yaku.

use serde::{Deserialize, Serialize};

use crate::context::indicator_to_dora;
use crate::tile::{Honor, Suit, Tile};

/// Which rules apply when scoring.
///
/// The default scores standard four-player agari rules. `variant` picks
/// the game (`Rules::sanma()` for three players), and each other switch
/// turns on one local yaku; organization presets (EMA, WRC) belong here
/// as constructors once checked against their rulebooks.
///
/// ```
/// use agari::rules::{Renhou, Rules};
///
/// let rules = Rules::default()
///     .open_riichi(true)
///     .renhou(Some(Renhou::Mangan));
/// assert!(rules.open_riichi);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct Rules {
    /// Open riichi scores one han more than riichi or double riichi.
    pub open_riichi: bool,
    /// What renhou is worth, or `None` to not score it.
    pub renhou: Option<Renhou>,
    /// Four-player or three-player mahjong.
    pub variant: Variant,
}

impl Rules {
    /// Three-player rules with no local yaku.
    pub fn sanma() -> Self {
        Rules {
            variant: Variant::Sanma,
            ..Rules::default()
        }
    }

    /// The dora a given indicator points to under these rules.
    pub fn dora_for(&self, indicator: Tile) -> Tile {
        // 2m-8m are not in the sanma wall, so 1m points to 9m.
        if self.variant == Variant::Sanma && indicator == Tile::suited(Suit::Man, 1) {
            return Tile::suited(Suit::Man, 9);
        }
        indicator_to_dora(indicator)
    }

    /// How many players sit at the table.
    pub fn players(&self) -> u32 {
        match self.variant {
            Variant::Yonma => 4,
            Variant::Sanma => 3,
        }
    }

    /// Whether this tile is in the wall.
    pub fn in_wall(&self, tile: Tile) -> bool {
        !(self.variant == Variant::Sanma
            && matches!(
                tile,
                Tile::Suited {
                    suit: Suit::Man,
                    value: 2..=8
                }
            ))
    }

    /// Whether players may call chi.
    pub fn allows_chi(&self) -> bool {
        self.variant != Variant::Sanma
    }

    /// Whether Norths can be pulled as nukidora.
    pub fn allows_nukidora(&self) -> bool {
        self.variant == Variant::Sanma
    }

    /// Whether this wind is in play: a player sits in its seat, and only
    /// seated winds are round winds.
    pub fn has_wind(&self, wind: Honor) -> bool {
        !(self.variant == Variant::Sanma && wind == Honor::North)
    }

    /// Builder-style: turn open riichi on or off
    pub fn open_riichi(mut self, on: bool) -> Self {
        self.open_riichi = on;
        self
    }

    /// Builder-style: set what renhou is worth, or `None` to turn it off
    pub fn renhou(mut self, value: Option<Renhou>) -> Self {
        self.renhou = value;
        self
    }
}

/// Which game is being scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Variant {
    /// Four-player mahjong.
    #[default]
    Yonma,
    /// Three-player mahjong: no 2m-8m and no chi.
    Sanma,
}

/// What renhou (a non-dealer's ron before their first draw) is worth.
/// Rulesets disagree, so the value is a choice rather than a switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Renhou {
    /// Mangan, without other yaku or dora, unless the hand scores more
    /// without renhou (EMA, WRC, JPML).
    Mangan,
    /// A yakuman, which stacks with other yakuman.
    Yakuman,
    /// A yaku of this many han that combines with other yaku and dora.
    /// At least 1, as 0 scores no renhou; 13 or more scores as a counted
    /// yakuman.
    Han(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_without_variant_deserialize_as_yonma() {
        let rules: Rules = serde_json::from_str(r#"{"open_riichi":true}"#).unwrap();
        assert_eq!(rules.variant, Variant::Yonma);
        assert!(rules.open_riichi);
    }
}
