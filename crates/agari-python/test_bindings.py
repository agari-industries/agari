"""Smoke tests for agari_core Python bindings."""

import agari_core


def test_complete_hand():
    # 123m 456p 789s 11z + winning 1z (complete hand = shanten -1)
    # Tiles: 1m2m3m 4p5p6p 7s8s9s 1z1z1z1z  (but we need 14 tiles for a complete hand)
    # Let's use: 123m 456m 789m 11p 234s = 14 tiles
    hand = [0] * 34
    hand[0] = 1  # 1m
    hand[1] = 1  # 2m
    hand[2] = 1  # 3m
    hand[3] = 1  # 4m
    hand[4] = 1  # 5m
    hand[5] = 1  # 6m
    hand[6] = 1  # 7m
    hand[7] = 1  # 8m
    hand[8] = 1  # 9m
    hand[9] = 2  # 1p (pair)
    hand[18] = 1  # 1s
    hand[19] = 1  # 2s
    hand[20] = 1  # 3s
    shanten = agari_core.calculate_shanten(hand, 0)
    assert shanten == -1, f"Expected -1 (complete), got {shanten}"
    print(f"Complete hand shanten: {shanten}")


def test_tenpai_hand():
    # 123m 456m 789m 1p 234s = 13 tiles, waiting on 1p pair
    hand = [0] * 34
    hand[0] = 1  # 1m
    hand[1] = 1  # 2m
    hand[2] = 1  # 3m
    hand[3] = 1  # 4m
    hand[4] = 1  # 5m
    hand[5] = 1  # 6m
    hand[6] = 1  # 7m
    hand[7] = 1  # 8m
    hand[8] = 1  # 9m
    hand[9] = 1  # 1p
    hand[18] = 1  # 1s
    hand[19] = 1  # 2s
    hand[20] = 1  # 3s
    shanten = agari_core.calculate_shanten(hand, 0)
    assert shanten == 0, f"Expected 0 (tenpai), got {shanten}"
    print(f"Tenpai hand shanten: {shanten}")


def test_ukeire():
    # Same tenpai hand as above
    hand = [0] * 34
    hand[0] = 1  # 1m
    hand[1] = 1  # 2m
    hand[2] = 1  # 3m
    hand[3] = 1  # 4m
    hand[4] = 1  # 5m
    hand[5] = 1  # 6m
    hand[6] = 1  # 7m
    hand[7] = 1  # 8m
    hand[8] = 1  # 9m
    hand[9] = 1  # 1p
    hand[18] = 1  # 1s
    hand[19] = 1  # 2s
    hand[20] = 1  # 3s
    visible = [0] * 34
    shanten, waits = agari_core.calculate_ukeire(hand, 0, visible)
    assert shanten == 0, f"Expected shanten 0, got {shanten}"
    assert waits > 0, f"Expected waits > 0, got {waits}"
    print(f"Ukeire: shanten={shanten}, waits={waits}")


def test_compute_riichi_features():
    hand = [0] * 34
    hand[0] = 1  # 1m
    hand[1] = 1  # 2m
    hand[2] = 1  # 3m
    hand[3] = 1  # 4m
    hand[4] = 1  # 5m
    hand[5] = 1  # 6m
    hand[6] = 1  # 7m
    hand[7] = 1  # 8m
    hand[8] = 1  # 9m
    hand[9] = 1  # 1p
    hand[18] = 1  # 1s
    hand[19] = 1  # 2s
    hand[20] = 1  # 3s
    visible = [0] * 34
    tenpai, shanten_norm, waits_norm = agari_core.compute_riichi_features(hand, 0, visible)
    assert tenpai == 1.0, f"Expected tenpai=1.0, got {tenpai}"
    assert shanten_norm == 0.0, f"Expected shanten_norm=0.0, got {shanten_norm}"
    assert 0.0 < waits_norm <= 1.0, f"Expected 0 < waits_norm <= 1, got {waits_norm}"
    print(f"Features: tenpai={tenpai}, shanten_norm={shanten_norm}, waits_norm={waits_norm}")


def test_batch():
    hand = [0] * 34
    hand[0] = 1
    hand[1] = 1
    hand[2] = 1
    hand[3] = 1
    hand[4] = 1
    hand[5] = 1
    hand[6] = 1
    hand[7] = 1
    hand[8] = 1
    hand[9] = 1
    hand[18] = 1
    hand[19] = 1
    hand[20] = 1
    visible = [0] * 34

    single = agari_core.compute_riichi_features(hand, 0, visible)
    batch = agari_core.batch_compute_riichi_features([hand, hand], [0, 0], [visible, visible])
    assert len(batch) == 2
    assert batch[0] == single
    assert batch[1] == single
    print(f"Batch: {batch}")


def test_valid_chi_combinations():
    # 0 combos: hand has no partners for 1m (index 0)
    hand = [0] * 34
    hand[5] = 1  # 6m
    combos = agari_core.valid_chi_combinations(hand, 0)
    assert combos == [], f"Expected [], got {combos}"
    print(f"Chi 0 combos: {combos}")

    # 1 combo: 2m+3m in hand, discard 1m → (1, 2)
    hand = [0] * 34
    hand[1] = 1  # 2m
    hand[2] = 1  # 3m
    combos = agari_core.valid_chi_combinations(hand, 0)
    assert combos == [(1, 2)], f"Expected [(1, 2)], got {combos}"
    print(f"Chi 1 combo (low end 1m): {combos}")

    # 1 combo: 7m+8m in hand, discard 9m (index 8) → (6, 7)
    hand = [0] * 34
    hand[6] = 1  # 7m
    hand[7] = 1  # 8m
    combos = agari_core.valid_chi_combinations(hand, 8)
    assert combos == [(6, 7)], f"Expected [(6, 7)], got {combos}"
    print(f"Chi 1 combo (high end 9m): {combos}")

    # 2 combos: 1m+2m+4m in hand, discard 3m (index 2)
    # High: (0, 1), Low: (3, 4)? No, d=2, val=2. High: d-2=0, d-1=1 → (0,1). Mid: d-1=1, d+1=3 → need hand[1]>0 and hand[3]>0. hand[3]=0 so no. Low: d+1=3, d+2=4 → need hand[3]>0 and hand[4]>0. No.
    # Let's use: 1m,2m,4m in hand, discard 3m. High=(0,1), Mid needs hand[1] and hand[3]. hand[3]=4m index is 3, yes! So Mid=(1,3).
    hand = [0] * 34
    hand[0] = 1  # 1m
    hand[1] = 1  # 2m
    hand[3] = 1  # 4m
    combos = agari_core.valid_chi_combinations(hand, 2)
    assert combos == [(0, 1), (1, 3)], f"Expected [(0, 1), (1, 3)], got {combos}"
    print(f"Chi 2 combos: {combos}")

    # 3 combos: 3m,4m,6m,7m in hand, discard 5m (index 4)
    # High: d-2=2, d-1=3 → hand[2]=0, no. Need hand[2]. Let's add it.
    # 3m(2),4m(3),6m(5),7m(6) in hand, discard 5m(4). val=4.
    # High: (2,3) → hand[2]>0 and hand[3]>0 → yes
    # Mid: (3,5) → hand[3]>0 and hand[5]>0 → yes
    # Low: (5,6) → hand[5]>0 and hand[6]>0 → yes
    hand = [0] * 34
    hand[2] = 1  # 3m
    hand[3] = 1  # 4m
    hand[5] = 1  # 6m
    hand[6] = 1  # 7m
    combos = agari_core.valid_chi_combinations(hand, 4)
    assert combos == [(2, 3), (3, 5), (5, 6)], f"Expected [(2,3),(3,5),(5,6)], got {combos}"
    print(f"Chi 3 combos: {combos}")

    # Honor tile: discard East (27) → empty
    hand = [0] * 34
    hand[27] = 3
    combos = agari_core.valid_chi_combinations(hand, 27)
    assert combos == [], f"Expected [] for honor, got {combos}"
    print(f"Chi honor tile: {combos}")


def test_shanten_after_chi():
    # Tenpai hand: 123m 456m 789m 1p 234s (shanten 0, waiting on 1p pair)
    # Chi 1s(18) from opponent using 2s(19)+3s(20), then discard something
    # After chi: remove 2s,3s from hand. Hand now has 123m 456m 789m 1p + meld 123s.
    # That's 10 tiles with 1 meld. Need to discard 1 to get to 9 tiles + 1 meld (tenpai).
    # Best discard should keep tenpai (shanten 0) or possibly complete if lucky.
    hand = [0] * 34
    for i in range(9):
        hand[i] = 1  # 1-9m
    hand[9] = 1  # 1p
    hand[18] = 1  # 1s
    hand[19] = 1  # 2s
    hand[20] = 1  # 3s
    # Before chi, shanten is 0
    before = agari_core.calculate_shanten(hand, 0)
    assert before == 0, f"Expected 0 before chi, got {before}"

    # Chi with combo (19, 20) = 2s, 3s; discarded tile = 18 (1s)
    # After removing 2s,3s: hand has 123m 456m 789m 1p 1s → 10 tiles, 1 meld
    # Best discard: drop 1s to keep 123m 456m 789m 1p (9 tiles, 1 meld) → shanten 0 (wait on 1p)
    after = agari_core.shanten_after_chi(hand, 18, (19, 20), 0)
    assert after <= before, f"Expected shanten <= {before} after good chi, got {after}"
    print(f"Shanten after good chi: {before} -> {after}")

    # Bad chi: hand with no good discard after chi
    # Simple hand: 1m,3m,5m,7m,9m,2p,4p,6p,8p,1s,3s,5s,7s (13 tiles, all isolated)
    bad_hand = [0] * 34
    for i in [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24]:
        bad_hand[i] = 1
    before_bad = agari_core.calculate_shanten(bad_hand, 0)
    # Chi 2m(1) with combo (0, 2) = 1m, 3m
    after_bad = agari_core.shanten_after_chi(bad_hand, 1, (0, 2), 0)
    print(f"Shanten after bad chi: {before_bad} -> {after_bad}")
    # With a scattered hand, chi shouldn't magically help much
    assert after_bad >= before_bad - 1, f"Unexpectedly good shanten after bad chi"


def test_shanten_after_kan():
    # Open kan (type 0): hand has 3 of tile, opponent discards 4th
    # Hand: 111m 456m 789m 1p 234s = 13+2 extra 1m = 15? No, hand has counts.
    # Let's use: hand has 3x1m, 456m, 789m, 1p → 3+3+3+1 = 10 tiles with 1 meld already
    # Actually for simplicity: hand with 3x East(27), rest forming melds
    # 123m 456m 789m EEE + 1 meld already? Let's keep it simple.
    hand = [0] * 34
    hand[0] = 1; hand[1] = 1; hand[2] = 1  # 123m
    hand[3] = 1; hand[4] = 1; hand[5] = 1  # 456m
    hand[6] = 1; hand[7] = 1; hand[8] = 1  # 789m
    hand[27] = 3  # EEE
    hand[9] = 1   # 1p
    # 13 tiles, 0 melds. Shanten = 0 (waiting on 1p pair)
    before = agari_core.calculate_shanten(hand, 0)
    assert before == 0, f"Expected 0 before kan, got {before}"

    # Open kan on East: remove 3, melds becomes 1. Hand: 123m 456m 789m 1p (10 tiles, 1 meld)
    after_open = agari_core.shanten_after_kan(hand, 27, 0, 0)
    print(f"Open kan: {before} -> {after_open}")
    assert after_open == 0, f"Expected 0 after open kan, got {after_open}"

    # Closed kan (type 1): need 4 of a tile in hand
    hand2 = [0] * 34
    hand2[0] = 1; hand2[1] = 1; hand2[2] = 1  # 123m
    hand2[3] = 1; hand2[4] = 1; hand2[5] = 1  # 456m
    hand2[27] = 4  # EEEE
    hand2[9] = 1   # 1p
    # 10 tiles, 0 melds (but with 4 of East it's like having a quad in hand)
    after_closed = agari_core.shanten_after_kan(hand2, 27, 1, 0)
    print(f"Closed kan: shanten = {after_closed}")

    # Added kan (type 2): have existing pon (counted as a meld), add 1 more
    hand3 = [0] * 34
    hand3[0] = 1; hand3[1] = 1; hand3[2] = 1  # 123m
    hand3[3] = 1; hand3[4] = 1; hand3[5] = 1  # 456m
    hand3[6] = 1; hand3[7] = 1; hand3[8] = 1  # 789m
    hand3[27] = 1  # E (the 4th East, adding to existing pon)
    # 10 tiles, 1 meld (pon of East). Kakan removes 1 East, stays at 1 meld.
    before3 = agari_core.calculate_shanten(hand3, 1)
    after_added = agari_core.shanten_after_kan(hand3, 27, 2, 1)
    print(f"Added kan: {before3} -> {after_added}")


def test_is_permanent_furiten():
    # Tenpai hand waiting on 1p (index 9): 123m 456m 789m 1p 234s
    hand = [0] * 34
    for i in range(9):
        hand[i] = 1  # 1-9m
    hand[9] = 1   # 1p
    hand[18] = 1  # 1s
    hand[19] = 1  # 2s
    hand[20] = 1  # 3s
    assert agari_core.calculate_shanten(hand, 0) == 0

    # Furiten: 1p (index 9) is in own discards
    furiten = agari_core.is_permanent_furiten(hand, [9], 0)
    assert furiten is True, f"Expected furiten=True, got {furiten}"
    print(f"Furiten with winning tile in discards: {furiten}")

    # Not furiten: discards don't contain winning tile
    clean = agari_core.is_permanent_furiten(hand, [27, 28, 29], 0)
    assert clean is False, f"Expected furiten=False, got {clean}"
    print(f"Furiten with clean discards: {clean}")

    # Non-tenpai hand: should return False
    bad_hand = [0] * 34
    bad_hand[0] = 1; bad_hand[2] = 1; bad_hand[4] = 1
    bad_hand[9] = 1; bad_hand[11] = 1; bad_hand[13] = 1
    bad_hand[18] = 1; bad_hand[20] = 1; bad_hand[22] = 1
    bad_hand[27] = 1; bad_hand[28] = 1; bad_hand[29] = 1; bad_hand[30] = 1
    not_tenpai = agari_core.is_permanent_furiten(bad_hand, [0, 2, 4, 9], 0)
    assert not_tenpai is False, f"Expected False for non-tenpai, got {not_tenpai}"
    print(f"Furiten non-tenpai: {not_tenpai}")


if __name__ == "__main__":
    test_complete_hand()
    test_tenpai_hand()
    test_ukeire()
    test_compute_riichi_features()
    test_batch()
    test_valid_chi_combinations()
    test_shanten_after_chi()
    test_shanten_after_kan()
    test_is_permanent_furiten()
    print("\nAll tests passed!")
