# Changelog

All notable changes to this project will be documented in this file.

## [0.18.0]

### Added

- **Shareable URL Support**: Generate shareable URLs that encode the full hand configuration for the web calculator
- **Crates.io Publishing**: Package is now available on crates.io with `cargo install agari`

### Fixed

- **Ukeire Analysis with Called Melds**: Fixed ukeire (tile acceptance) producing bogus results for hands with open melds — previously showed nearly all 34 tile types as improving tiles instead of the correct few
- **Red Five in Shared URLs**: Preserved red five (aka dora) notation in the winning tile when generating shareable URLs
- **Crates.io Publish Job**: Fixed cargo-dist custom publish job configuration for crates.io releases

### Changed

- **GitHub Username Migration**: Updated repository references from `sullenb` to `rysb-dev`

## [0.17.0]

### Changed

- **GitHub Username Migration**: Updated all repository URLs and references from `ryblogs` to `sullenb`
  - Updated Cargo.toml repository and homepage URLs
  - Updated GitHub Actions workflows for deploy and release
  - Updated Homebrew tap reference to `sullenb/homebrew-tap`
  - Updated all installation instructions in README

### Added

- **Web UI Enhancements**:
  - Dark tile theme with toggle option
  - Improved score result layout
  - Custom domain support (agari.org)

### Fixed

- **Mobile UX**: Prevented double-tap-to-zoom on mobile tile interactions

### Documentation

- Enhanced README with images and web frontend information
- Updated live demo URL to agari.org

## [0.16.0]

### Added

- **Japanese Localization (Web UI)**: Full Japanese (日本語) language support for the web interface
  - Type-safe i18n system with English and Japanese translations
  - Language switcher in header with persistent preference (localStorage)
  - Auto-detection of browser language preference
  - All UI strings, yaku names, and score levels translated
  - Translations for: 満貫, 跳満, 倍満, 三倍満, 役満, etc.

- **Suu Kantsu (四槓子) Yakuman**: Added the missing Four Kans yakuman
  - Detects when hand contains exactly 4 kans (open or closed)
  - Awards yakuman (13 han) - can stack with Suuankou for double yakuman
  - Added CLI display: "Suu Kantsu (Four Kans)"
  - Added WASM support and i18n translations

### Fixed

- **Yaku Translation Mapping**: Fixed yaku names not translating in Japanese locale
  - Updated yakuNameMap to match actual WASM backend output format
  - WASM uses shorter names (e.g., "Sanshoku Doujun") vs CLI verbose names
  - Made translation helpers reactive to locale changes

## [0.15.0]

### Changed

- **Web UI Redesign**: Complete redesign of the web interface with a modern linear design language
  - Boxy, sharp-cornered design with no border-radius
  - 1px borders for depth instead of shadows
  - Modern dark color scheme (`#09090b` base, `#3b82f6` accent)
  - Monospace typography for values, badges, and code elements
  - Grid-based panel layout with 1px gap separators
  - Flat hover states using border color changes
  - Updated all components: Tile, TilePalette, ContextOptions, ScoreResult, DoraPicker

### Fixed

- **Validator CLI**: Simplified CLI by removing `--files` parameter; validator now always uses bundled test files

## [0.14.0]

### Added

- **CI Test Workflow**: Added GitHub Actions workflow to run tests on all PRs and pushes to main
- **CI Clippy Lint Check**: Added clippy linting to CI pipeline, enforcing zero warnings
- **Regression Tests**: Added tests for sequences-first triplet extraction in shanten calculation

### Changed

- **Clippy 2024 Compliance**: Comprehensive clippy fixes across the codebase (PR #10)
  - Modernized iterator patterns and range checks
  - Replaced manual implementations with idiomatic Rust
  - Simplified conditional expressions and closures

### Fixed

- **Deploy Workflow on Forks**: Fixed GitHub Pages deploy workflow failing on forks by adding repository owner check

### New Contributors

- **@DrCheeseFace** made their first contribution in PR #10 — thank you! 🎉

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