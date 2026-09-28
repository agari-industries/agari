//! Optional scoring rules.

use serde::{Deserialize, Serialize};

/// Which optional rules apply when scoring.
///
/// The default scores standard agari rules. Each switch turns on one
/// local yaku; organization presets (EMA, WRC) belong here as
/// constructors once checked against their rulebooks.
///
/// ```
/// use agari::rules::Rules;
///
/// let rules = Rules::default().open_riichi(true);
/// assert!(rules.open_riichi);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
#[non_exhaustive]
pub struct Rules {
    /// Open riichi scores one han more than riichi or double riichi.
    pub open_riichi: bool,
}

impl Rules {
    /// Builder-style: turn open riichi on or off
    pub fn open_riichi(mut self, on: bool) -> Self {
        self.open_riichi = on;
        self
    }
}
