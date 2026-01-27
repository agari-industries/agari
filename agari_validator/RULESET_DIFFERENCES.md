# Ruleset Differences: Agari vs Tenhou

This document tracks known differences between Agari's scoring rules and Tenhou's rules. When validating against Tenhou historical data, mismatches caused by these differences are **not bugs** - they are intentional rule variations.

## How to Use This Document

When the validator reports a mismatch, check if it falls into one of these categories before investigating as a bug. You can use the patterns below to help identify ruleset-based mismatches.

---

## Known Differences

### 1. Suuankou Tanki (四暗刻単騎) - Double Yakuman vs Single Yakuman

**Agari behavior:** Scores Suuankou Tanki as **Double Yakuman (64,000 points for non-dealer ron)**

**Tenhou behavior:** Scores Suuankou Tanki as **Single Yakuman (32,000 points for non-dealer ron)**

**Description:**
Suuankou (Four Concealed Triplets) with a tanki (single tile) wait is considered a harder wait than the standard shanpon (dual pair) wait. Some rulesets award double yakuman for this achievement, while Tenhou uses single yakuman.

**How to identify:**
- Hand has 4 closed triplets/kans + pair
- Winning tile completes the pair (tanki wait)
- Agari shows 64,000 points, Tenhou shows 32,000 points
- Agari reports "Suuankou Tanki (26 han)" or "Double Yakuman"

**Example:**
```
agari 666m444p33399s[4444z] -w 9s --round e --seat s -d 3z,5p
Agari:  80 fu, 26 han, 64000 pts (Double Yakuman)
Tenhou: 32000 pts (Single Yakuman)
```

---

### 2. Junsei Chuuren Poutou (純正九蓮宝燈) - Double Yakuman vs Single Yakuman

**Agari behavior:** Scores Junsei Chuuren (9-sided wait) as **Double Yakuman**

**Tenhou behavior:** Scores Junsei Chuuren as **Single Yakuman**

**Description:**
Chuuren Poutou (Nine Gates) with a 9-sided wait (1112345678999 waiting on any tile in the suit) is the "pure" form. Some rulesets award double yakuman, while Tenhou uses single yakuman.

**How to identify:**
- Hand is Chuuren Poutou (1112345678999 in one suit)
- Any tile in that suit completes the hand (9-sided wait)
- Point difference is 32,000 (single vs double yakuman)

---

### 3. Kokushi Musou 13-wait (国士無双十三面待ち) - Double Yakuman vs Single Yakuman

**Agari behavior:** Scores Kokushi 13-wait as **Double Yakuman**

**Tenhou behavior:** Scores Kokushi 13-wait as **Single Yakuman**

**Description:**
Kokushi Musou (Thirteen Orphans) with a 13-sided wait (holding one of each terminal/honor, waiting on any) is the rarest form. Some rulesets award double yakuman.

**How to identify:**
- Hand is Kokushi (all terminals and honors with one pair)
- The hand was waiting on any of the 13 terminal/honor tiles
- Point difference is 32,000 (single vs double yakuman)

---

## Future Considerations

### Potential Configuration Options

To better match Tenhou's rules, Agari could add command-line flags:

```
--tenhou-rules          Use Tenhou-compatible yakuman scoring
--no-double-yakuman     Disable all double yakuman scoring
```

### Adding New Differences

When you discover a new ruleset difference during validation:

1. Verify it's truly a ruleset difference (not a bug)
2. Add an entry to this document with:
   - Clear description of both behaviors
   - How to identify the pattern
   - Example command and outputs
3. Consider whether Agari should support configurable rules

---

## Known Validator Limitations

### Hand State Tracking Errors

In rare cases (approximately 1 in 8,000+), the validator may extract a hand that has the correct tile count but cannot form a valid mahjong structure. This typically manifests as an "ERROR: This hand has no valid winning structure" from Agari.

**Causes:**
- Complex meld interactions (chankan, rinshan)
- Hidden tile logs (tiles shown as "?" from other players' perspectives)
- Edge cases in mjai event ordering

**Identification:**
- Agari reports "no valid winning structure"
- The hand tiles don't form valid melds

**Resolution:**
These are validator limitations, not Agari bugs. The errors can be safely ignored when the error rate is < 0.1% of validated hands.

---

## Validation Tips

When running the validator:

1. **Export mismatches** to JSON for analysis:
   ```bash
   python agari_validator.py /path/to/data --export-mismatches mismatches.json
   ```

2. **Check yakuman hands** - if points differ by exactly 32,000 and involve yakuman, it's likely a double-yakuman ruleset difference

3. **Verify with Agari directly** - run the command shown in the mismatch output to see full details:
   ```bash
   agari [hand] [flags]
   ```
