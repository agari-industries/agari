# Changelog

All notable changes to this project will be documented in this file.

## [0.13.1]

### Fixed

- **Tile Texture Rendering**: Embedded Front.svg background directly into all tile SVGs for proper texture/3D effect rendering
- **Dora Picker Exhausted Tiles**: Tiles that have been fully used in the hand are now disabled in the dora/ura dora picker, preventing selection of unavailable tiles
- **WIN Badge Overlap**: Moved the "WIN" label from top-right to bottom-center of tiles to avoid overlap with the remove button
- **TypeScript Warning**: Fixed "Cannot find name 'process'" warning in vite.config.ts by using Vite's built-in `mode` parameter

### Changed

- **Tile Component Simplification**: Removed redundant tile background layering in Tile.svelte since backgrounds are now embedded in SVGs

## [0.13.0]

### Fixed

- **Riichi Options for Open Hands**: Riichi, Double Riichi, and Ippatsu options are now disabled when the hand has open melds (chi, pon, or open kan)
  - Options are automatically unchecked if they were selected before adding an open meld
  - Shows notice: "🔓 Open hand — Riichi not available"

### Added

- **Live Demo Link**: Added prominent link to the web interface at the top of README.md

## [0.12.0]

### Fixed

- **Red Five Duplicate in Meld Builder**: Fixed bug where users could add multiple red fives (aka dora) of the same suit in pon/chi/kan meld builder (e.g., 0p, 0p, 0p)
  - Meld builder tiles now included in `tileCounts` so the palette updates in real-time
  - Properly checks red five availability using separate `red5m`/`red5p`/`red5s` count tracking
- **Invalid Chi Meld Sequences**: Fixed bug where chi meld builder allowed non-consecutive tiles (e.g., 5p5p5p or 2m4m6m)
  - Added sequence validation that only allows tiles forming valid 3-consecutive runs
  - Invalid tiles are now visually grayed out and unclickable in the palette

### Changed

- **Improved Meld Builder UX**: For pon/kan/ankan melds, after selecting the first tile, all other tile types are now visually disabled in the palette (previously only enforced in code)

## [0.11.0]

### Added

- **Inferred Winning Tile Display**: When scoring a hand without explicitly selecting a winning tile, the frontend now shows which tile was automatically inferred and highlights it in the hand
  - Yellow info banner: "💡 Winning tile inferred as [tile]. Click a tile in your hand to select explicitly."
  - Auto-selects the inferred tile in the hand display with the "WIN" badge
  - Handles red five matching (inferred "5m" correctly matches red "0m")
- **WASM Test Suite**: Added 27 focused tests for `agari-wasm` covering the WASM binding layer
  - Request/response integration tests
  - Inferred winning tile functionality
  - Shanten and ukeire API wrappers
  - Helper functions (`parse_wind`, `yaku_name`, `format_structure`)

### Changed

- `ScoringOutput` in WASM bindings now includes optional `inferred_winning_tile` field

## [0.10.0]

### Added

- **New Tile Graphics**: Replaced programmatic SVG tiles with beautiful riichi mahjong tile artwork from [FluffyStuff/riichi-mahjong-tiles](https://github.com/FluffyStuff/riichi-mahjong-tiles) (public domain CC0)
- **Visual Dora Picker**: Replaced text dropdown for dora/ura-dora selection with an intuitive visual tile picker modal
- **Improved Tile Removal UX**: Added visible × button on hand tiles for easier removal (appears on hover for desktop, always visible on mobile/touch devices)
- Dora indicator section now expanded by default for better discoverability

### Fixed

- **Tile Count Tracking for Melds**: Adding chi/pon/kan melds now correctly decreases the remaining tile count indicators
- **Red Five Tracking**: Red 5 tiles (aka-dora) now correctly show 1 remaining instead of 4, since there's only one red five per suit
- **Dora Indicator Red Five Handling**: Fixed dora/ura-dora selection and tracking for red fives using 0m/0p/0s notation

### Changed

- Red fives now appear at the end of each suit row in the dora picker (matching the main tile palette layout)
- Increased horizontal spacing between tiles in the "Build Your Hand" tile selection area for better visual clarity
- Removed "Right-click to remove" hint text (now using visible × buttons instead)

## [0.9.0] - 2026-01-29

### Fixed

- Handle red five (0m/0p/0s) notation in dora selectors
- Count dora in called melds (kans) correctly
- Add red 5s to dora selector dropdowns

## [0.8.0] - 2026-01-28

### Fixed

- Shanten calculation with called melds
- Improved meld builder UI

## [0.7.0] - 2026-01-27

### Added

- Web frontend with Svelte
- WASM bindings for browser usage
- Interactive hand builder UI