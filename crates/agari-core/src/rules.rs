//! Optional scoring rules.

use serde::{Deserialize, Serialize};

/// Which optional rules apply when scoring.
///
/// The default scores standard agari rules. Each switch turns on one
/// local yaku; organization presets (EMA, WRC) belong here as
/// constructors once checked against their rulebooks.
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
}

impl Rules {
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
