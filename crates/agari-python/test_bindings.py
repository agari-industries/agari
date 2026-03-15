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


if __name__ == "__main__":
    test_complete_hand()
    test_tenpai_hand()
    test_ukeire()
    test_compute_riichi_features()
    test_batch()
    print("\nAll tests passed!")
