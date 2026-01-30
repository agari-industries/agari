# Changelog

All notable changes to this project will be documented in this file.

## [0.10.0] - 2026-01-30

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