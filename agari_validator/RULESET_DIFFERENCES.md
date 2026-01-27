# Ruleset Differences: Agari vs Tenhou

This document tracks known differences between Agari's scoring rules and Tenhou's rules. When validating against Tenhou historical data, mismatches caused by these differences are **not bugs** - they are intentional rule variations.

## How to Use This Document

When the validator reports a mismatch, check if it falls into one of these categories before investigating as a bug. You can use the patterns below to help identify ruleset-based mismatches.

---

## Known Differences

### 1. Suuankou Tanki Tsumo (四暗刻単騎) - Double Yakuman vs Single Yakuman

**Agari behavior:** Scores Suuankou Tanki **by tsumo** as **Double Yakuman**

**Tenhou behavior:** Scores Suuankou Tanki as **Single Yakuman** regardless of win type

**Description:**
Suuankou (Four Concealed Triplets) with a tanki (single tile) wait won by tsumo is considered the pinnacle achievement. Agari awards double yakuman for this specific case, while Tenhou uses single yakuman.

Note: As of commit `480b841`, Agari correctly awards **single yakuman** for Suuankou won by **ron** (matching Tenhou), since the ron tile technically "opens" the last triplet. Only tsumo tanki receives double yakuman in Agari.

**How to identify:**
- Hand has 4 closed triplets/kans + pair
- Winning tile completes the pair (tanki wait)
- Win is by **tsumo** (not ron)
- Point difference is 32,000 (single vs double yakuman)

**Example:**
```
agari 111m222p333s44455z -w 5z -t
Agari:  Double Yakuman (96,000 dealer tsumo)
Tenhou: Single Yakuman (48,000 dealer tsumo)
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

## Resolved Issues (Previously Listed as Differences)

The following issues were previously thought to be ruleset differences but were actually bugs that have been fixed:

### Suuankou Ron (Fixed in commit 480b841)
Previously Agari awarded double yakuman for Suuankou tanki even when won by ron. This was incorrect - ron on the pair technically "opens" one of the triplets from the opponent's perspective. Now Agari correctly awards single yakuman for Suuankou ron, matching Tenhou.

### Kokushi 13-wait Detection (Fixed in commit 415860c)
Previously Kokushi 13-wait detection had edge cases that could cause incorrect scoring. The detection logic has been corrected.

### Fu Calculation for Nobetan Patterns (Fixed in commit c963216)
Ron-completed triplets in nobetan patterns (e.g., 11123 waiting on 1 or 4) were incorrectly scored as "open" triplets. Now only true shanpon waits treat the ron-completed triplet as open.

### Akadora in Non-Aka Games (Fixed in commit 569d93e)
The MJAI converter marks tiles as red fives based on Tenhou's tile ID encoding, even when akadora is disabled in the game rules. The validator now strips red five markings when `aka_flag: false`.

### Chankan and Rinshan Detection (Fixed in commit 26c6ab9)
Edge cases in chankan (robbing a kan) and rinshan (kan replacement draw) detection have been corrected.

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

In rare cases, the validator may extract a hand that has the correct tile count but cannot form a valid mahjong structure. This typically manifests as an "ERROR: This hand has no valid winning structure" from Agari.

**Causes:**
- Complex meld interactions (chankan, rinshan)
- Hidden tile logs (tiles shown as "?" from other players' perspectives)
- Edge cases in MJAI event ordering

**Identification:**
- Agari reports "no valid winning structure"
- The hand tiles don't form valid melds

**Resolution:**
These are validator limitations, not Agari bugs. The errors can be safely ignored when the error rate is low.

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
