# Agari v0.8.0 Release Notes

## Overview

This release focuses on improving the web frontend's meld builder functionality and fixing critical bugs in the shanten calculation when called melds (pon, chi, kan) are present.

## New Features

### Web Frontend

- **Meld Builder UI**: Added interactive UI for building called melds (chi, pon, open kan, closed kan/ankan)
  - Click "Chi", "Pon", "Open Kan", or "Closed Kan" buttons to start building a meld
  - Select tiles from the palette to add to the meld
  - Validation ensures proper meld formation (same tiles for pon/kan, sequential tiles for chi)
  
- **Winning Tile Selection**: Click on any tile in your hand to mark it as the winning tile for scoring

- **Hand Notation Display**: The agari-core tile notation (e.g., `1m2m3m(234s)[1111p]`) is now always visible when tiles or melds are present

## Bug Fixes

### Shanten Calculation (agari-core)

- **Fixed incorrect shanten with called melds**: The shanten calculation now properly accounts for the minimum number of tiles required to form a valid tenpai hand
  - Previously, hands like `999m[1111m][1111p][1111s]` (3 tiles + 3 kans) incorrectly showed as "Tenpai"
  - Now correctly calculates that this hand is 1-shanten (needs 1 more tile for a valid wait)
  
- **Tile deficit validation**: Added logic to ensure shanten cannot report tenpai when there aren't enough tiles to form a valid waiting pattern
  - Minimum tiles for tenpai: `13 - 3 * called_melds` (or 1 tile for 4 called melds)

### Web Frontend

- **Fixed kan hand size calculation**: Kans now correctly consume 3 hand slots instead of 4
  - Each meld (pon, chi, or kan) reduces available hand slots by 3
  - This matches real mahjong rules where declaring a kan gives you a replacement draw

## Technical Changes

### agari-core

- Added `calculate_shanten_with_melds(counts, called_melds)` public API
- Refactored internal shanten calculation to track tile deficit
- Updated CLI to use the new meld-aware shanten calculation

### agari-wasm

- WASM bindings now properly pass called meld count to shanten calculation
- Improved winning tile inference for hands with called melds

## Upgrade Notes

This release is backward compatible. The shanten calculation changes may produce different (more accurate) results for hands with called melds that have fewer tiles than expected.

## Expected Shanten Values by Configuration

For reference, here are the minimum tiles needed in hand for tenpai with called melds:

| Called Melds | Min Hand Tiles for Tenpai |
|--------------|---------------------------|
| 0            | 13                        |
| 1            | 10                        |
| 2            | 7                         |
| 3            | 4                         |
| 4            | 1 (tanki wait)            |

Hands with fewer tiles than the minimum will show shanten values reflecting the tile deficit.