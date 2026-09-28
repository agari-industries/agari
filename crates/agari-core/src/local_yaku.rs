//! Local yaku. The standard detector hands its result here, and nothing
//! changes unless a switch in [`Rules`](crate::rules::Rules) is on.

use crate::context::GameContext;
use crate::yaku::{Yaku, YakuResult};

pub(crate) fn apply(mut result: YakuResult, context: &GameContext) -> YakuResult {
    if context.rules.open_riichi && context.is_open_riichi {
        open_riichi(&mut result);
    }
    result
}

/// Open riichi is one han on top of riichi or double riichi. The standard
/// pass only awards those to closed hands that are not yakuman, so
/// upgrading the entry it already made keeps the same limits.
fn open_riichi(result: &mut YakuResult) {
    for yaku in &mut result.yaku_list {
        let open = match yaku {
            Yaku::Riichi => Yaku::OpenRiichi,
            Yaku::DoubleRiichi => Yaku::OpenDoubleRiichi,
            _ => continue,
        };
        *yaku = open;
        result.total_han += 1;
        return;
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{GameContext, WinType};
    use crate::hand::decompose_hand;
    use crate::parse::{parse_hand, to_counts};
    use crate::rules::Rules;
    use crate::scoring::{ScoringResult, calculate_score};
    use crate::tile::{Honor, Suit, Tile};
    use crate::yaku::{Yaku, YakuResult, detect_yaku_with_context};

    /// Score every decomposition and keep the one that pays most, as callers do
    fn score(hand: &str, context: &GameContext) -> (YakuResult, ScoringResult) {
        let counts = to_counts(&parse_hand(hand).unwrap());
        decompose_hand(&counts)
            .iter()
            .map(|s| {
                let yaku = detect_yaku_with_context(s, &counts, context);
                let score = calculate_score(s, &yaku, context);
                (yaku, score)
            })
            .max_by_key(|(_, score)| score.payment.total)
            .unwrap()
    }

    // Riichi is the only yaku: West triplet, South pair, ron on 1m
    const RIICHI_ONLY: &str = "123m456p789s33322z";

    fn ron() -> GameContext {
        GameContext::new(WinType::Ron, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 1))
    }

    fn open_riichi_rules() -> Rules {
        Rules::default().open_riichi(true)
    }

    #[test]
    fn open_riichi_off_by_default() {
        let (yaku, _) = score(RIICHI_ONLY, &ron().open_riichi());
        assert_eq!(yaku.yaku_list, vec![Yaku::Riichi]);
        assert_eq!(yaku.total_han, 1);
    }

    #[test]
    fn open_riichi_is_two_han() {
        let context = ron().open_riichi().with_rules(open_riichi_rules());
        let (yaku, _) = score(RIICHI_ONLY, &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::OpenRiichi]);
        assert_eq!(yaku.total_han, 2);
    }

    #[test]
    fn open_double_riichi_is_three_han() {
        let context = ron()
            .double_riichi()
            .open_riichi()
            .with_rules(open_riichi_rules());
        let (yaku, _) = score(RIICHI_ONLY, &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::OpenDoubleRiichi]);
        assert_eq!(yaku.total_han, 3);
    }

    #[test]
    fn open_riichi_requires_riichi() {
        let mut context = ron().with_rules(open_riichi_rules());
        context.is_open_riichi = true;
        let (yaku, _) = score(RIICHI_ONLY, &context);
        assert!(yaku.yaku_list.is_empty());
        assert_eq!(yaku.total_han, 0);
    }

    #[test]
    fn open_riichi_rejected_for_open_hand() {
        let context = ron()
            .with_winning_tile(Tile::suited(Suit::Man, 2))
            .open()
            .open_riichi()
            .with_rules(open_riichi_rules());
        let (yaku, _) = score("234m345p456s678s88p", &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::Tanyao]);
        assert_eq!(yaku.total_han, 1);
    }

    #[test]
    fn open_riichi_not_added_to_yakuman() {
        let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
            .with_winning_tile(Tile::suited(Suit::Man, 1))
            .open_riichi()
            .with_rules(open_riichi_rules());
        let (yaku, score) = score("111m333p555s66677z", &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::Suuankou]);
        assert_eq!(score.payment.total, 32000);
    }

    #[test]
    fn is_local_marks_only_local_yaku() {
        assert!(Yaku::OpenRiichi.is_local());
        assert!(Yaku::OpenDoubleRiichi.is_local());
        assert!(!Yaku::Riichi.is_local());
        assert!(!Yaku::Tenhou.is_local());
    }
}
