//! Local yaku. The standard detector hands its result here, and nothing
//! changes unless a switch in [`Rules`](crate::rules::Rules) is on.

use crate::context::{GameContext, WinType};
use crate::rules::Renhou;
use crate::yaku::{Yaku, YakuResult};

pub(crate) fn apply(mut result: YakuResult, context: &GameContext) -> YakuResult {
    if context.rules.open_riichi && context.is_open_riichi {
        open_riichi(&mut result);
    }
    if let Some(value) = context.rules.renhou
        && value != Renhou::Han(0)
        && context.is_renhou
        && context.win_type == WinType::Ron
        && context.is_closed()
        && !context.is_dealer()
    {
        renhou(&mut result, value);
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

fn renhou(result: &mut YakuResult, value: Renhou) {
    let yaku = Yaku::Renhou(value);
    match value {
        // EMA 2025 4.2.4: renhou is five han with no other yaku or dora.
        // WRC, JPML and EMA score the hand without it when that pays more,
        // which needs a yaku of its own.
        Renhou::Mangan => {
            let scores_more = !result.yaku_list.is_empty()
                && (result.is_yakuman || result.total_han_with_dora() >= 6);
            if !scores_more {
                result.yaku_list = vec![yaku];
                result.total_han = yaku.han();
                result.dora_count = 0;
                result.regular_dora = 0;
                result.ura_dora = 0;
                result.aka_dora = 0;
                result.nuki_dora = 0;
            }
        }
        // Yakuman stack in agari (ScoreLevel::Yakuman counts units), so a
        // yakuman hand gains another; any other hand is replaced by it.
        Renhou::Yakuman => {
            if result.is_yakuman {
                result.yaku_list.push(yaku);
                result.total_han = result.total_han.saturating_add(yaku.han());
            } else {
                result.yaku_list = vec![yaku];
                result.total_han = yaku.han();
                result.is_yakuman = true;
            }
        }
        // Han does not add to a yakuman, which is already worth more
        Renhou::Han(n) => {
            if !result.is_yakuman {
                result.yaku_list.push(yaku);
                result.total_han = result.total_han.saturating_add(n);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{GameContext, WinType};
    use crate::hand::decompose_hand;
    use crate::parse::{parse_hand, to_counts};
    use crate::rules::{Renhou, Rules};
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

    fn renhou_ron(value: Renhou) -> GameContext {
        ron()
            .renhou()
            .with_rules(Rules::default().renhou(Some(value)))
    }

    const ALL_VALUES: [Renhou; 3] = [Renhou::Mangan, Renhou::Yakuman, Renhou::Han(5)];

    // Tanyao with 222m as three dora under a 1m indicator
    const TANYAO_THREE_DORA: &str = "222m345p456s678s88p";

    fn with_man_dora(context: GameContext) -> GameContext {
        context.with_dora(vec![Tile::suited(Suit::Man, 1)])
    }

    #[test]
    fn renhou_off_by_default() {
        let (yaku, _) = score(RIICHI_ONLY, &ron().renhou());
        assert!(yaku.yaku_list.is_empty());
    }

    #[test]
    fn renhou_mangan_without_other_yaku() {
        let (yaku, score) = score(RIICHI_ONLY, &renhou_ron(Renhou::Mangan));
        assert_eq!(yaku.yaku_list, vec![Yaku::Renhou(Renhou::Mangan)]);
        assert_eq!(yaku.total_han, 5);
        assert!(!yaku.is_yakuman);
        assert_eq!(score.payment.total, 8000);
    }

    #[test]
    fn renhou_mangan_replaces_yaku_and_dora_below_haneman() {
        let context =
            with_man_dora(renhou_ron(Renhou::Mangan)).with_winning_tile(Tile::suited(Suit::Pin, 3));
        let (yaku, score) = score(TANYAO_THREE_DORA, &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::Renhou(Renhou::Mangan)]);
        assert_eq!(yaku.dora_count, 0);
        assert_eq!(yaku.total_han_with_dora(), 5);
        assert_eq!(score.payment.total, 8000);
    }

    #[test]
    fn renhou_mangan_ignores_dora_without_a_yaku() {
        // Six dora but no yaku: the hand cannot be scored without renhou
        let context = renhou_ron(Renhou::Mangan)
            .with_dora(vec![Tile::suited(Suit::Man, 9), Tile::suited(Suit::Man, 9)]);
        let (yaku, score) = score("111m456p789s33322z", &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::Renhou(Renhou::Mangan)]);
        assert_eq!(score.payment.total, 8000);
    }

    #[test]
    fn renhou_mangan_keeps_higher_normal_score() {
        let (yaku, score) = score("123345567789m99m", &renhou_ron(Renhou::Mangan));
        assert!(yaku.yaku_list.contains(&Yaku::Chinitsu));
        assert!(!yaku.yaku_list.contains(&Yaku::Renhou(Renhou::Mangan)));
        assert_eq!(score.payment.total, 12000);
    }

    #[test]
    fn renhou_mangan_keeps_yakuman() {
        let context = renhou_ron(Renhou::Mangan).with_winning_tile(Tile::suited(Suit::Man, 9));
        let (yaku, score) = score("11112345678999m", &context);
        assert_eq!(yaku.yaku_list, vec![Yaku::ChuurenPoutou]);
        assert_eq!(score.payment.total, 32000);
    }

    #[test]
    fn renhou_yakuman_scores_yakuman() {
        let (yaku, score) = score(RIICHI_ONLY, &renhou_ron(Renhou::Yakuman));
        assert_eq!(yaku.yaku_list, vec![Yaku::Renhou(Renhou::Yakuman)]);
        assert!(yaku.is_yakuman);
        assert_eq!(score.payment.total, 32000);
    }

    #[test]
    fn renhou_yakuman_stacks_with_yakuman() {
        let context = renhou_ron(Renhou::Yakuman).with_winning_tile(Tile::suited(Suit::Man, 9));
        let (yaku, score) = score("11112345678999m", &context);
        assert_eq!(
            yaku.yaku_list,
            vec![Yaku::ChuurenPoutou, Yaku::Renhou(Renhou::Yakuman)]
        );
        assert_eq!(score.payment.total, 64000);
    }

    #[test]
    fn renhou_han_is_cumulative() {
        let context =
            with_man_dora(renhou_ron(Renhou::Han(5))).with_winning_tile(Tile::suited(Suit::Pin, 3));
        let (yaku, score) = score(TANYAO_THREE_DORA, &context);
        assert_eq!(
            yaku.yaku_list,
            vec![Yaku::Tanyao, Yaku::Renhou(Renhou::Han(5))]
        );
        assert_eq!(yaku.total_han, 6);
        assert_eq!(yaku.total_han_with_dora(), 9);
        assert_eq!(score.payment.total, 16000);
    }

    #[test]
    fn renhou_han_zero_is_off() {
        let (yaku, _) = score(RIICHI_ONLY, &renhou_ron(Renhou::Han(0)));
        assert!(yaku.yaku_list.is_empty());
        assert_eq!(yaku.total_han, 0);
    }

    #[test]
    fn renhou_requires_ron() {
        for value in ALL_VALUES {
            let context = GameContext::new(WinType::Tsumo, Honor::East, Honor::South)
                .with_winning_tile(Tile::suited(Suit::Man, 1))
                .renhou()
                .with_rules(Rules::default().renhou(Some(value)));
            let (yaku, _) = score(RIICHI_ONLY, &context);
            assert_eq!(yaku.yaku_list, vec![Yaku::MenzenTsumo], "{value:?}");
        }
    }

    #[test]
    fn renhou_not_for_dealer() {
        for value in ALL_VALUES {
            let context = GameContext::new(WinType::Ron, Honor::East, Honor::East)
                .with_winning_tile(Tile::suited(Suit::Man, 1))
                .renhou()
                .with_rules(Rules::default().renhou(Some(value)));
            let (yaku, _) = score(RIICHI_ONLY, &context);
            assert!(yaku.yaku_list.is_empty(), "{value:?}");
        }
    }

    #[test]
    fn renhou_rejected_for_open_hand() {
        for value in ALL_VALUES {
            let context = renhou_ron(value).open();
            let (yaku, _) = score(RIICHI_ONLY, &context);
            assert!(yaku.yaku_list.is_empty(), "{value:?}");
        }
    }

    #[test]
    fn is_local_marks_only_local_yaku() {
        assert!(Yaku::OpenRiichi.is_local());
        assert!(Yaku::OpenDoubleRiichi.is_local());
        assert!(Yaku::Renhou(Renhou::Mangan).is_local());
        assert!(!Yaku::Riichi.is_local());
        assert!(!Yaku::Tenhou.is_local());
    }
}
