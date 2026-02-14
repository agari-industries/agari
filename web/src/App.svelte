<script lang="ts">
    import { onMount } from "svelte";
    import {
        initAgari,
        scoreHand,
        calculateShanten,
        type ScoreRequest,
        type ScoringOutput,
        type ShantenResponse,
        ALL_TILES,
    } from "./lib/agari";
    import { t } from "./lib/i18n";
    import TilePalette from "./lib/components/TilePalette.svelte";
    import ContextOptions from "./lib/components/ContextOptions.svelte";
    import ScoreResult from "./lib/components/ScoreResult.svelte";
    import Tile from "./lib/components/Tile.svelte";
    import DoraPicker from "./lib/components/DoraPicker.svelte";
    import LanguageSwitcher from "./lib/components/LanguageSwitcher.svelte";
    import TileThemeSwitcher from "./lib/components/TileThemeSwitcher.svelte";
    import {
        serializeToUrl,
        deserializeFromUrl,
        clearUrlParams,
    } from "./lib/urlState";

    // ============================================================================
    // State
    // ============================================================================

    interface TileEntry {
        tile: string;
        isRed?: boolean;
        id: number;
    }

    interface Meld {
        type: "chi" | "pon" | "kan" | "ankan";
        tiles: TileEntry[];
        id: number;
    }

    let wasmLoaded = $state(false);
    let wasmError = $state<string | null>(null);

    // Hand state
    let handTiles = $state<TileEntry[]>([]);
    let nextTileId = $state(0);

    // Winning tile selection
    let selectedWinningTileIndex = $state<number | null>(null);

    // Melds (called tiles)
    let melds = $state<Meld[]>([]);
    let nextMeldId = $state(0);
    let showMeldBuilder = $state(false);
    let meldBuilderType = $state<"chi" | "pon" | "kan" | "ankan">("pon");
    let meldBuilderTiles = $state<TileEntry[]>([]);

    // Dora state
    let doraIndicators = $state<TileEntry[]>([]);
    let uraDoraIndicators = $state<TileEntry[]>([]);
    let nextDoraId = $state(0);
    let showDoraSection = $state(true);
    let showDoraPicker = $state(false);
    let showUraDoraPicker = $state(false);

    // Context options
    let isTsumo = $state(false);
    let isRiichi = $state(false);
    let isDoubleRiichi = $state(false);
    let isIppatsu = $state(false);
    let roundWind = $state<"east" | "south" | "west" | "north">("east");
    let seatWind = $state<"east" | "south" | "west" | "north">("east");
    let isLastTile = $state(false);
    let isRinshan = $state(false);
    let isChankan = $state(false);
    let isTenhou = $state(false);
    let isChiihou = $state(false);

    // Results
    let scoreResult = $state<ScoringOutput | null>(null);
    let scoreError = $state<string | null>(null);
    let shantenResult = $state<ShantenResponse | null>(null);
    let isCalculating = $state(false);

    // Mode
    let mode = $state<"score" | "shanten">("score");

    // Share
    let shareTooltip = $state<string | null>(null);

    // ============================================================================
    // Derived state
    // ============================================================================

    // Type for tile counts including red five tracking
    type TileCountsType = Record<string, number>;

    // Calculate remaining tiles
    const tileCounts: TileCountsType = $derived.by(() => {
        const counts: TileCountsType = {};
        for (const tile of ALL_TILES) {
            counts[tile] = 4;
        }
        // Red fives: only 1 of each exists (they come from the pool of 4 regular 5s)
        // We track red fives separately - when a red 5 is used, it reduces both
        // the red 5 count (max 1) AND the regular 5 count (since it's one of the 4 fives)
        const redFiveCounts: Record<string, number> = {
            "5m": 1,
            "5p": 1,
            "5s": 1,
        };

        // Subtract hand tiles
        for (const entry of handTiles) {
            if (counts[entry.tile] !== undefined) {
                counts[entry.tile]--;
            }
            // Track red five usage
            if (entry.isRed && redFiveCounts[entry.tile] !== undefined) {
                redFiveCounts[entry.tile]--;
            }
        }
        // Subtract meld tiles
        for (const meld of melds) {
            for (const entry of meld.tiles) {
                if (counts[entry.tile] !== undefined) {
                    counts[entry.tile]--;
                }
                // Track red five usage
                if (entry.isRed && redFiveCounts[entry.tile] !== undefined) {
                    redFiveCounts[entry.tile]--;
                }
            }
        }
        // Subtract meld builder tiles (tiles currently being added to a meld)
        for (const entry of meldBuilderTiles) {
            if (counts[entry.tile] !== undefined) {
                counts[entry.tile]--;
            }
            // Track red five usage
            if (entry.isRed && redFiveCounts[entry.tile] !== undefined) {
                redFiveCounts[entry.tile]--;
            }
        }
        // Subtract dora indicators
        for (const entry of doraIndicators) {
            // Red fives use 0m/0p/0s notation - map to 5m/5p/5s for count tracking
            const countTile = entry.tile.startsWith("0")
                ? "5" + entry.tile[1]
                : entry.tile;
            if (counts[countTile] !== undefined) {
                counts[countTile]--;
            }
            // Track red five usage for dora indicators
            if (
                entry.tile.startsWith("0") &&
                redFiveCounts[countTile] !== undefined
            ) {
                redFiveCounts[countTile]--;
            }
        }
        // Subtract ura dora indicators
        for (const entry of uraDoraIndicators) {
            // Red fives use 0m/0p/0s notation - map to 5m/5p/5s for count tracking
            const countTile = entry.tile.startsWith("0")
                ? "5" + entry.tile[1]
                : entry.tile;
            if (counts[countTile] !== undefined) {
                counts[countTile]--;
            }
            // Track red five usage for ura dora indicators
            if (
                entry.tile.startsWith("0") &&
                redFiveCounts[countTile] !== undefined
            ) {
                redFiveCounts[countTile]--;
            }
        }

        // Add red five counts to the return object (keyed as "red5m", "red5p", "red5s")
        return {
            ...counts,
            red5m: redFiveCounts["5m"],
            red5p: redFiveCounts["5p"],
            red5s: redFiveCounts["5s"],
        };
    });

    // Build hand string
    const handString = $derived.by(() => {
        if (handTiles.length === 0) return "";

        // Group tiles by suit, maintaining order for building the string
        const groups: Record<string, string[]> = { m: [], p: [], s: [], z: [] };

        for (const entry of handTiles) {
            const suit = entry.tile[1];
            const value = entry.isRed ? "0" : entry.tile[0];
            if (groups[suit]) {
                groups[suit].push(value);
            }
        }

        // Build string
        let result = "";
        for (const [suit, values] of Object.entries(groups)) {
            if (values.length > 0) {
                result += values.join("") + suit;
            }
        }
        return result;
    });

    // Count red fives (in hand and melds)
    const akaCount = $derived.by(() => {
        const handAka = handTiles.filter((t) => t.isRed).length;
        const meldAka = melds.reduce(
            (acc, m) => acc + m.tiles.filter((t) => t.isRed).length,
            0,
        );
        return handAka + meldAka;
    });

    // Count tiles in melds (for display purposes)
    const tilesInMelds = $derived(
        melds.reduce((acc, m) => acc + m.tiles.length, 0),
    );

    // Count meld slots used for hand size calculation
    // Each meld (pon/chi/kan) uses 3 "slots" from the hand, because:
    // - Pon/Chi: 3 tiles called
    // - Kan: 4 tiles called, but you draw a replacement tile (+1 to hand)
    // So effectively, all melds reduce hand size by 3
    const meldSlotsUsed = $derived(melds.length * 3);

    // Max tiles in hand based on mode and melds
    const maxHandTiles = $derived(
        mode === "score" ? 14 - meldSlotsUsed : 13 - meldSlotsUsed,
    );

    // Selected winning tile (always use standard notation, e.g. "5s" not "0s")
    const winningTile = $derived.by(() => {
        if (
            selectedWinningTileIndex !== null &&
            handTiles[selectedWinningTileIndex]
        ) {
            const entry = handTiles[selectedWinningTileIndex];
            return entry.tile; // Red-ness is tracked separately, winning tile just identifies which tile
        }
        return undefined;
    });

    // Check if hand has open melds
    const hasOpenMelds = $derived(melds.some((m) => m.type !== "ankan"));

    // Total tiles (hand + melds)
    const totalTiles = $derived(handTiles.length + tilesInMelds);

    // Can calculate score
    const canCalculate = $derived(
        wasmLoaded && totalTiles >= (mode === "score" ? 14 : 1),
    );

    // Compute disabled tiles for meld builder (for chi validation)
    const meldBuilderDisabledTiles: Set<string> = $derived.by(() => {
        if (!showMeldBuilder) return new Set();

        // For pon/kan, after first tile is selected, only that tile is allowed
        if (
            (meldBuilderType === "pon" ||
                meldBuilderType === "kan" ||
                meldBuilderType === "ankan") &&
            meldBuilderTiles.length > 0
        ) {
            const allowedTile = meldBuilderTiles[0].tile;
            const disabled = new Set<string>();
            for (const tile of ALL_TILES) {
                if (tile !== allowedTile) {
                    disabled.add(tile);
                }
            }
            return disabled;
        }

        // For chi, compute which tiles can form a valid sequence
        if (meldBuilderType === "chi") {
            const allowed = getChiAllowedTiles();
            const disabled = new Set<string>();
            for (const tile of ALL_TILES) {
                if (!allowed.has(tile)) {
                    disabled.add(tile);
                }
            }
            return disabled;
        }

        return new Set();
    });

    // Compute disabled tiles for dora pickers (tiles with 0 remaining count)
    const doraDisabledTiles: Set<string> = $derived.by(() => {
        const disabled = new Set<string>();
        for (const tile of ALL_TILES) {
            if (tileCounts[tile] <= 0) {
                disabled.add(tile);
            }
        }
        // Also disable red fives if no red fives remaining
        if (tileCounts["red5m"] <= 0) disabled.add("0m");
        if (tileCounts["red5p"] <= 0) disabled.add("0p");
        if (tileCounts["red5s"] <= 0) disabled.add("0s");
        return disabled;
    });

    // ============================================================================
    // Functions
    // ============================================================================

    // Add tile to hand
    function addTile(tile: string, isRed: boolean = false) {
        if (handTiles.length >= maxHandTiles) return;
        if (tileCounts[tile] <= 0) return;

        handTiles = [...handTiles, { tile, isRed, id: nextTileId++ }];
        recalculateShanten();
    }

    // Remove tile from hand
    function removeTile(index: number) {
        // If we're removing the winning tile, clear the selection
        if (selectedWinningTileIndex === index) {
            selectedWinningTileIndex = null;
        } else if (
            selectedWinningTileIndex !== null &&
            index < selectedWinningTileIndex
        ) {
            // Adjust index if removing a tile before the winning tile
            selectedWinningTileIndex--;
        }
        handTiles = handTiles.filter((_, i) => i !== index);
        recalculateShanten();
    }

    // Select winning tile
    function selectWinningTile(index: number) {
        if (selectedWinningTileIndex === index) {
            selectedWinningTileIndex = null; // Deselect if clicking the same tile
        } else {
            selectedWinningTileIndex = index;
        }
    }

    // Meld builder functions
    function startMeldBuilder(type: "chi" | "pon" | "kan" | "ankan") {
        meldBuilderType = type;
        meldBuilderTiles = [];
        showMeldBuilder = true;
    }

    // Get the numeric value of a tile (handles red fives)
    function getTileValue(tile: string, isRed: boolean = false): number {
        if (isRed) return 5;
        return parseInt(tile[0]);
    }

    // Compute allowed tiles for chi meld builder based on current selection
    function getChiAllowedTiles(): Set<string> {
        const allowed = new Set<string>();

        if (meldBuilderTiles.length === 0) {
            // Any non-honor tile is allowed as the first tile
            for (const suit of ["m", "p", "s"]) {
                for (let i = 1; i <= 9; i++) {
                    allowed.add(`${i}${suit}`);
                }
            }
            return allowed;
        }

        const suit = meldBuilderTiles[0].tile[1];

        if (meldBuilderTiles.length === 1) {
            // Second tile: must be same suit and able to form a sequence with first tile
            const v1 = getTileValue(
                meldBuilderTiles[0].tile,
                meldBuilderTiles[0].isRed,
            );
            // Possible second tiles: v1-2, v1-1, v1+1, v1+2 (that could still form a valid sequence)
            for (let delta = -2; delta <= 2; delta++) {
                if (delta === 0) continue;
                const v2 = v1 + delta;
                if (v2 >= 1 && v2 <= 9) {
                    // Check if v1 and v2 can form part of a valid sequence (3 consecutive tiles exist)
                    const min = Math.min(v1, v2);
                    const max = Math.max(v1, v2);
                    // A valid sequence needs 3 consecutive tiles
                    // If we have 2 tiles, the third must complete the sequence
                    // Possible: min-1 (if >= 1), or max+1 (if <= 9)
                    if (max - min <= 2) {
                        // tiles are close enough to form a sequence
                        // Check if third tile would be valid
                        if (max - min === 1) {
                            // Consecutive: need min-1 or max+1
                            if (min - 1 >= 1 || max + 1 <= 9) {
                                allowed.add(`${v2}${suit}`);
                            }
                        } else if (max - min === 2) {
                            // Gap of 1: middle tile completes it
                            allowed.add(`${v2}${suit}`);
                        }
                    }
                }
            }
            return allowed;
        }

        if (meldBuilderTiles.length === 2) {
            // Third tile: must complete the sequence
            const v1 = getTileValue(
                meldBuilderTiles[0].tile,
                meldBuilderTiles[0].isRed,
            );
            const v2 = getTileValue(
                meldBuilderTiles[1].tile,
                meldBuilderTiles[1].isRed,
            );
            const min = Math.min(v1, v2);
            const max = Math.max(v1, v2);

            if (max - min === 1) {
                // Consecutive tiles, need either end
                if (min - 1 >= 1) allowed.add(`${min - 1}${suit}`);
                if (max + 1 <= 9) allowed.add(`${max + 1}${suit}`);
            } else if (max - min === 2) {
                // Gap of 1, need middle tile
                allowed.add(`${min + 1}${suit}`);
            }
            // If gap > 2, no valid third tile exists
            return allowed;
        }

        return allowed;
    }

    function addTileToMeldBuilder(tile: string, isRed: boolean = false) {
        const maxTiles =
            meldBuilderType === "kan" || meldBuilderType === "ankan" ? 4 : 3;
        if (meldBuilderTiles.length >= maxTiles) return;

        // Check regular tile count
        if (tileCounts[tile] <= 0) return;

        // For red fives, also check the red five count
        if (isRed) {
            const redKey = `red${tile}` as keyof typeof tileCounts;
            if (tileCounts[redKey] <= 0) return;
        }

        // For chi, validate that tile can form a valid sequence
        if (meldBuilderType === "chi") {
            // Honor tiles cannot be in chi
            if (tile[1] === "z") return;

            const allowedTiles = getChiAllowedTiles();
            if (!allowedTiles.has(tile)) return;
        }

        // For pon/kan, tiles must be the same
        if (
            (meldBuilderType === "pon" ||
                meldBuilderType === "kan" ||
                meldBuilderType === "ankan") &&
            meldBuilderTiles.length > 0
        ) {
            const baseTile = meldBuilderTiles[0].tile;
            if (tile !== baseTile) return;
        }

        meldBuilderTiles = [
            ...meldBuilderTiles,
            { tile, isRed, id: nextTileId++ },
        ];
    }

    function removeTileFromMeldBuilder(index: number) {
        meldBuilderTiles = meldBuilderTiles.filter((_, i) => i !== index);
    }

    function confirmMeld() {
        const requiredTiles =
            meldBuilderType === "kan" || meldBuilderType === "ankan" ? 4 : 3;
        if (meldBuilderTiles.length !== requiredTiles) return;

        // For chi, sort tiles
        if (meldBuilderType === "chi") {
            meldBuilderTiles.sort((a, b) => {
                const valA = a.isRed ? 5 : parseInt(a.tile[0]);
                const valB = b.isRed ? 5 : parseInt(b.tile[0]);
                return valA - valB;
            });
        }

        melds = [
            ...melds,
            {
                type: meldBuilderType,
                tiles: [...meldBuilderTiles],
                id: nextMeldId++,
            },
        ];
        showMeldBuilder = false;
        meldBuilderTiles = [];
        recalculateShanten();
    }

    function cancelMeldBuilder() {
        showMeldBuilder = false;
        meldBuilderTiles = [];
    }

    function removeMeld(index: number) {
        melds = melds.filter((_, i) => i !== index);
        recalculateShanten();
    }

    // Add dora indicator
    function addDoraIndicator(tile: string, _isRed: boolean = false) {
        if (doraIndicators.length >= 5) return;
        // For red fives (0m/0p/0s), check both the regular 5 count and red five count
        if (tile.startsWith("0")) {
            const baseTile = "5" + tile[1];
            const redKey = `red${baseTile}` as keyof typeof tileCounts;
            if (tileCounts[baseTile] <= 0 || tileCounts[redKey] <= 0) return;
        } else {
            if (tileCounts[tile] <= 0) return;
        }
        doraIndicators = [...doraIndicators, { tile, id: nextDoraId++ }];
    }

    // Remove dora indicator
    function removeDoraIndicator(index: number) {
        doraIndicators = doraIndicators.filter((_, i) => i !== index);
    }

    // Add ura dora indicator
    function addUraDoraIndicator(tile: string, _isRed: boolean = false) {
        if (uraDoraIndicators.length >= 5) return;
        // For red fives (0m/0p/0s), check both the regular 5 count and red five count
        if (tile.startsWith("0")) {
            const baseTile = "5" + tile[1];
            const redKey = `red${baseTile}` as keyof typeof tileCounts;
            if (tileCounts[baseTile] <= 0 || tileCounts[redKey] <= 0) return;
        } else {
            if (tileCounts[tile] <= 0) return;
        }
        uraDoraIndicators = [...uraDoraIndicators, { tile, id: nextDoraId++ }];
    }

    // Remove ura dora indicator
    function removeUraDoraIndicator(index: number) {
        uraDoraIndicators = uraDoraIndicators.filter((_, i) => i !== index);
    }

    // Clear hand
    function clearHand() {
        handTiles = [];
        melds = [];
        doraIndicators = [];
        uraDoraIndicators = [];
        selectedWinningTileIndex = null;
        scoreResult = null;
        scoreError = null;
        shantenResult = null;
        clearUrlParams();
    }

    // Share hand via URL
    async function shareHand() {
        // Compute winning tile with red five notation for URL
        let urlWinningTile = winningTile;
        if (
            selectedWinningTileIndex !== null &&
            handTiles[selectedWinningTileIndex]?.isRed
        ) {
            urlWinningTile = "0" + handTiles[selectedWinningTileIndex].tile[1];
        }

        const url = serializeToUrl({
            handTiles,
            melds,
            winningTile: urlWinningTile,
            doraIndicators,
            uraDoraIndicators,
            isTsumo,
            isRiichi,
            isDoubleRiichi,
            isIppatsu,
            roundWind,
            seatWind,
            isLastTile,
            isRinshan,
            isChankan,
            isTenhou,
            isChiihou,
        });

        history.replaceState({}, "", url);

        try {
            await navigator.clipboard.writeText(url);
            shareTooltip = $t.copiedToClipboard;
        } catch {
            shareTooltip = $t.copiedToClipboard;
        }

        setTimeout(() => {
            shareTooltip = null;
        }, 2000);
    }

    // Build meld notation string for backend
    // Format: (111m) for open melds (pon/chi/open kan), [1111m] for closed kan (ankan)
    function buildMeldNotation(): string {
        let meldStr = "";
        for (const meld of melds) {
            const tiles = meld.tiles
                .map((t) => (t.isRed ? "0" : t.tile[0]))
                .join("");
            const suit = meld.tiles[0].tile[1];
            if (meld.type === "ankan") {
                meldStr += `[${tiles}${suit}]`; // Closed kan uses square brackets
            } else {
                meldStr += `(${tiles}${suit})`; // Open melds use parentheses
            }
        }
        return meldStr;
    }

    // Calculate shanten in real-time
    function recalculateShanten() {
        if (!wasmLoaded || (handTiles.length === 0 && melds.length === 0)) {
            shantenResult = null;
            return;
        }

        const fullHand = handString + buildMeldNotation();
        const result = calculateShanten(fullHand);
        shantenResult = result;
    }

    // Calculate score
    function calculate() {
        if (!canCalculate) return;

        isCalculating = true;
        scoreError = null;
        scoreResult = null;

        // Convert red five notation (0m/0p/0s) to standard notation (5m/5p/5s) for backend
        const normalizeRedFive = (tile: string): string => {
            if (tile[0] === "0") {
                return "5" + tile[1];
            }
            return tile;
        };

        try {
            const fullHand = handString + buildMeldNotation();
            const request: ScoreRequest = {
                hand: fullHand,
                winning_tile: winningTile,
                is_tsumo: isTsumo,
                is_riichi: hasOpenMelds ? false : isRiichi, // Can't riichi with open hand
                is_double_riichi: hasOpenMelds ? false : isDoubleRiichi,
                is_ippatsu: hasOpenMelds ? false : isIppatsu,
                round_wind: roundWind,
                seat_wind: seatWind,
                dora_indicators: doraIndicators.map((d) =>
                    normalizeRedFive(d.tile),
                ),
                ura_dora_indicators: uraDoraIndicators.map((d) =>
                    normalizeRedFive(d.tile),
                ),
                is_last_tile: isLastTile,
                is_rinshan: isRinshan,
                is_chankan: isChankan,
                is_tenhou: isTenhou,
                is_chiihou: isChiihou,
            };

            const response = scoreHand(request);

            if (response.success && response.result) {
                scoreResult = response.result;
                scoreError = null;

                // If winning tile was inferred, auto-select it in the hand
                if (
                    response.result.inferred_winning_tile &&
                    selectedWinningTileIndex === null
                ) {
                    const inferredTile = response.result.inferred_winning_tile;
                    // Find the first matching tile in hand
                    // Handle red fives: inferred "5m" should match "5m" or red "0m"
                    const matchIndex = handTiles.findIndex((entry) => {
                        if (entry.tile === inferredTile) return true;
                        // Check if inferred is a 5 and entry is a red five (0) of same suit
                        if (
                            inferredTile[0] === "5" &&
                            entry.isRed &&
                            entry.tile[1] === inferredTile[1]
                        )
                            return true;
                        return false;
                    });
                    if (matchIndex !== -1) {
                        selectedWinningTileIndex = matchIndex;
                    }
                }
            } else {
                scoreError = response.error || "Unknown error";
                scoreResult = null;
            }
        } catch (e) {
            scoreError = e instanceof Error ? e.message : "Calculation failed";
            scoreResult = null;
        } finally {
            isCalculating = false;
        }
    }

    // Handle context change
    function handleContextChange() {
        // Could auto-recalculate here if desired
    }

    // ============================================================================
    // Lifecycle
    // ============================================================================

    onMount(async () => {
        try {
            await initAgari();
            wasmLoaded = true;

            // Restore state from URL if present (shared link)
            const urlState = deserializeFromUrl();
            if (urlState) {
                handTiles = urlState.handTiles;
                melds = urlState.melds;
                doraIndicators = urlState.doraIndicators;
                uraDoraIndicators = urlState.uraDoraIndicators;
                isTsumo = urlState.isTsumo;
                isRiichi = urlState.isRiichi;
                isDoubleRiichi = urlState.isDoubleRiichi;
                isIppatsu = urlState.isIppatsu;
                roundWind = urlState.roundWind;
                seatWind = urlState.seatWind;
                isLastTile = urlState.isLastTile;
                isRinshan = urlState.isRinshan;
                isChankan = urlState.isChankan;
                isTenhou = urlState.isTenhou;
                isChiihou = urlState.isChiihou;

                // Update ID counters to avoid collisions
                const maxTileId = Math.max(
                    0,
                    ...handTiles.map((t) => t.id),
                    ...melds.flatMap((m) => m.tiles.map((t) => t.id)),
                    ...doraIndicators.map((t) => t.id),
                    ...uraDoraIndicators.map((t) => t.id),
                );
                nextTileId = maxTileId + 1;
                nextMeldId = melds.length;
                nextDoraId = doraIndicators.length + uraDoraIndicators.length;

                // Select winning tile if specified
                if (urlState.winningTile) {
                    const wt = urlState.winningTile;
                    const matchIndex = handTiles.findIndex((entry) => {
                        if (entry.tile === wt) return true;
                        // Match "0m" from URL against red five entry (tile="5m", isRed=true)
                        if (
                            wt[0] === "0" &&
                            entry.isRed &&
                            entry.tile === "5" + wt[1]
                        )
                            return true;
                        // Match "5m" from URL against red five entry
                        if (
                            wt[0] === "5" &&
                            entry.isRed &&
                            entry.tile[1] === wt[1]
                        )
                            return true;
                        return false;
                    });
                    if (matchIndex !== -1) {
                        selectedWinningTileIndex = matchIndex;
                    }
                }

                recalculateShanten();

                // Auto-calculate score if hand is complete
                const totalRestoredTiles =
                    urlState.handTiles.length +
                    urlState.melds.reduce((acc, m) => acc + m.tiles.length, 0);
                if (totalRestoredTiles >= 14) {
                    calculate();
                }
            }
        } catch (e) {
            wasmError =
                e instanceof Error ? e.message : "Failed to load WASM module";
        }
    });
</script>

<div class="app">
    <header class="header">
        <div class="header-content">
            <h1 class="logo">
                <span class="logo-icon">🀄</span>
                <span class="logo-text">Agari</span>
            </h1>
            <p class="tagline">{$t.tagline}</p>
            <div class="header-settings">
                <TileThemeSwitcher />
                <LanguageSwitcher />
            </div>
        </div>
    </header>

    <main class="main">
        {#if wasmError}
            <div class="error-banner">
                <span>⚠️ {$t.failedToLoad} {wasmError}</span>
            </div>
        {:else if !wasmLoaded}
            <div class="loading-banner">
                <div class="spinner"></div>
                <span>{$t.loadingCalculator}</span>
            </div>
        {:else}
            <div class="calculator-layout">
                <!-- Left Column: Hand Builder -->
                <div class="hand-section">
                    <div class="card">
                        <div class="card-header">
                            <h2>{$t.buildYourHand}</h2>
                            <div class="header-actions">
                                <button
                                    class="btn btn-secondary"
                                    onclick={shareHand}
                                    disabled={handTiles.length === 0}
                                >
                                    {shareTooltip || $t.share}
                                </button>
                                <button
                                    class="btn btn-secondary"
                                    onclick={clearHand}
                                >
                                    {$t.clear}
                                </button>
                            </div>
                        </div>

                        <!-- Tile Palette -->
                        <TilePalette
                            onSelect={showMeldBuilder
                                ? addTileToMeldBuilder
                                : addTile}
                            {tileCounts}
                            showRedFives={true}
                            disabledTiles={meldBuilderDisabledTiles}
                        />

                        <!-- Meld Builder Buttons -->
                        <div class="meld-buttons">
                            <span class="meld-label">{$t.addMeld}</span>
                            <button
                                class="btn btn-sm"
                                onclick={() => startMeldBuilder("chi")}
                                disabled={showMeldBuilder}>{$t.chi}</button
                            >
                            <button
                                class="btn btn-sm"
                                onclick={() => startMeldBuilder("pon")}
                                disabled={showMeldBuilder}>{$t.pon}</button
                            >
                            <button
                                class="btn btn-sm"
                                onclick={() => startMeldBuilder("kan")}
                                disabled={showMeldBuilder}>{$t.openKan}</button
                            >
                            <button
                                class="btn btn-sm"
                                onclick={() => startMeldBuilder("ankan")}
                                disabled={showMeldBuilder}
                                >{$t.closedKan}</button
                            >
                        </div>

                        <!-- Meld Builder Panel -->
                        {#if showMeldBuilder}
                            <div class="meld-builder">
                                <div class="meld-builder-header">
                                    <span
                                        >{#if meldBuilderType === "chi"}{$t.buildingChi}{:else if meldBuilderType === "pon"}{$t.buildingPon}{:else if meldBuilderType === "kan"}{$t.buildingOpenKan}{:else}{$t.buildingClosedKan}{/if}</span
                                    >
                                    <span class="meld-builder-hint">
                                        {#if meldBuilderType === "chi"}
                                            {$t.hintChi}
                                        {:else if meldBuilderType === "pon"}
                                            {$t.hintPon}
                                        {:else}
                                            {$t.hintKan}
                                        {/if}
                                    </span>
                                </div>
                                <div class="meld-builder-tiles">
                                    {#each meldBuilderTiles as entry, index (entry.id)}
                                        <Tile
                                            tile={entry.tile}
                                            red={entry.isRed}
                                            size="md"
                                            onclick={() =>
                                                removeTileFromMeldBuilder(
                                                    index,
                                                )}
                                        />
                                    {/each}
                                    {#each Array((meldBuilderType === "kan" || meldBuilderType === "ankan" ? 4 : 3) - meldBuilderTiles.length) as _}
                                        <div class="meld-placeholder"></div>
                                    {/each}
                                </div>
                                <div class="meld-builder-actions">
                                    <button
                                        class="btn btn-sm btn-secondary"
                                        onclick={cancelMeldBuilder}
                                        >{$t.cancel}</button
                                    >
                                    <button
                                        class="btn btn-sm btn-primary"
                                        onclick={confirmMeld}
                                        disabled={meldBuilderTiles.length !==
                                            (meldBuilderType === "kan" ||
                                            meldBuilderType === "ankan"
                                                ? 4
                                                : 3)}
                                    >
                                        {$t.confirmAddMeld}
                                    </button>
                                </div>
                            </div>
                        {/if}
                    </div>

                    <!-- Melds Display -->
                    {#if melds.length > 0}
                        <div class="card">
                            <div class="melds-display">
                                <h3 class="melds-title">{$t.calledMelds}</h3>
                                <div class="melds-list">
                                    {#each melds as meld, index (meld.id)}
                                        <div class="meld-group">
                                            <span
                                                class="meld-type-badge"
                                                class:open={meld.type !==
                                                    "ankan"}
                                            >
                                                {meld.type === "ankan"
                                                    ? "🔒"
                                                    : "📢"}
                                                {meld.type}
                                            </span>
                                            <div class="meld-tiles">
                                                {#each meld.tiles as entry (entry.id)}
                                                    <Tile
                                                        tile={entry.tile}
                                                        red={entry.isRed}
                                                        size="sm"
                                                    />
                                                {/each}
                                            </div>
                                            <button
                                                class="btn-remove-meld"
                                                onclick={() =>
                                                    removeMeld(index)}>×</button
                                            >
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        </div>
                    {/if}

                    <!-- Hand Display -->
                    <div class="card">
                        <div class="hand-header">
                            <h3>{$t.yourHand}</h3>
                            {#if handTiles.length > 0}
                                <span class="winning-tile-hint"
                                    >{$t.selectWinningTileHint}</span
                                >
                            {/if}
                        </div>
                        <div class="hand-tiles-selectable">
                            {#each handTiles as entry, index (entry.id)}
                                <div class="tile-container">
                                    <button
                                        type="button"
                                        class="tile-wrapper"
                                        class:selected={selectedWinningTileIndex ===
                                            index}
                                        onclick={() => selectWinningTile(index)}
                                    >
                                        <Tile
                                            tile={entry.tile}
                                            red={entry.isRed}
                                            size="md"
                                        />
                                        {#if selectedWinningTileIndex === index}
                                            <div class="winning-badge">
                                                {$t.winBadge}
                                            </div>
                                        {/if}
                                    </button>
                                    <button
                                        type="button"
                                        class="tile-remove-btn"
                                        onclick={(e) => {
                                            e.stopPropagation();
                                            removeTile(index);
                                        }}
                                        aria-label="Remove tile">×</button
                                    >
                                </div>
                            {/each}
                            {#each Array(Math.max(0, maxHandTiles - handTiles.length)) as _}
                                <div class="tile-placeholder"></div>
                            {/each}
                        </div>
                        {#if handTiles.length > 0 || melds.length > 0}
                            <p class="hand-notation">
                                {handString}{buildMeldNotation()}
                            </p>
                        {/if}

                        <!-- Shanten Display -->
                        {#if shantenResult && (handTiles.length > 0 || melds.length > 0)}
                            {#if shantenResult.success}
                                <div class="shanten-display">
                                    {#if shantenResult.shanten === -1}
                                        <span class="shanten-badge complete"
                                            >✓ {$t.complete}</span
                                        >
                                    {:else if shantenResult.shanten === 0}
                                        <span class="shanten-badge tenpai"
                                            >{$t.tenpai}</span
                                        >
                                    {:else}
                                        <span class="shanten-badge"
                                            >{shantenResult.shanten}{$t.shanten}</span
                                        >
                                    {/if}
                                    <span class="shanten-type"
                                        >({shantenResult.best_type})</span
                                    >
                                </div>
                            {:else if shantenResult.error}
                                <div class="shanten-error">
                                    <span class="shanten-error-text"
                                        >Shanten: {shantenResult.error}</span
                                    >
                                </div>
                            {/if}
                        {/if}
                    </div>

                    <!-- Dora Section -->
                    <div class="card">
                        <button
                            class="dora-toggle"
                            onclick={() => (showDoraSection = !showDoraSection)}
                        >
                            <span>{$t.doraIndicators}</span>
                            <span
                                class="toggle-arrow"
                                class:open={showDoraSection}>▼</span
                            >
                        </button>

                        {#if showDoraSection}
                            <div class="dora-content">
                                <div class="dora-row">
                                    <span class="dora-label">{$t.dora}</span>
                                    <div class="dora-tiles">
                                        {#each doraIndicators as entry, index (entry.id)}
                                            <div class="dora-tile-wrapper">
                                                <Tile
                                                    tile={entry.tile}
                                                    size="sm"
                                                    red={entry.tile.startsWith(
                                                        "0",
                                                    )}
                                                />
                                                <button
                                                    type="button"
                                                    class="dora-remove-btn"
                                                    onclick={() =>
                                                        removeDoraIndicator(
                                                            index,
                                                        )}
                                                    aria-label="Remove dora indicator"
                                                    >×</button
                                                >
                                            </div>
                                        {/each}
                                        {#if doraIndicators.length < 5}
                                            <button
                                                type="button"
                                                class="dora-add-btn"
                                                onclick={() =>
                                                    (showDoraPicker = true)}
                                                >{$t.addButton}</button
                                            >
                                        {/if}
                                    </div>
                                </div>

                                {#if isRiichi}
                                    <div class="dora-row">
                                        <span class="dora-label"
                                            >{$t.uraDora}</span
                                        >
                                        <div class="dora-tiles">
                                            {#each uraDoraIndicators as entry, index (entry.id)}
                                                <div class="dora-tile-wrapper">
                                                    <Tile
                                                        tile={entry.tile}
                                                        size="sm"
                                                        red={entry.tile.startsWith(
                                                            "0",
                                                        )}
                                                    />
                                                    <button
                                                        type="button"
                                                        class="dora-remove-btn"
                                                        onclick={() =>
                                                            removeUraDoraIndicator(
                                                                index,
                                                            )}
                                                        aria-label="Remove ura dora indicator"
                                                        >×</button
                                                    >
                                                </div>
                                            {/each}
                                            {#if uraDoraIndicators.length < 5}
                                                <button
                                                    type="button"
                                                    class="dora-add-btn"
                                                    onclick={() =>
                                                        (showUraDoraPicker = true)}
                                                    >{$t.addButton}</button
                                                >
                                            {/if}
                                        </div>
                                    </div>
                                {/if}

                                {#if akaCount > 0}
                                    <div class="aka-display">
                                        {$t.akadoraInHand}
                                        <span class="aka-count">{akaCount}</span
                                        >
                                    </div>
                                {/if}
                            </div>
                        {/if}

                        <!-- Dora Picker Modal -->
                        {#if showDoraPicker}
                            <DoraPicker
                                onSelect={(tile) => {
                                    addDoraIndicator(tile);
                                    showDoraPicker = false;
                                }}
                                onClose={() => (showDoraPicker = false)}
                                disabledTiles={doraDisabledTiles}
                            />
                        {/if}

                        <!-- Ura Dora Picker Modal -->
                        {#if showUraDoraPicker}
                            <DoraPicker
                                onSelect={(tile) => {
                                    addUraDoraIndicator(tile);
                                    showUraDoraPicker = false;
                                }}
                                onClose={() => (showUraDoraPicker = false)}
                                disabledTiles={doraDisabledTiles}
                            />
                        {/if}
                    </div>

                    <!-- Results (moved from right column) -->
                    <div class="card results-card">
                        <h2 class="card-title">{$t.results}</h2>
                        <ScoreResult
                            result={scoreResult}
                            error={scoreError}
                            loading={isCalculating}
                        />
                    </div>
                </div>

                <!-- Right Column: Options & Calculate -->
                <div class="options-section">
                    <!-- Context Options -->
                    <div class="card">
                        <h2 class="card-title">{$t.options}</h2>
                        <ContextOptions
                            bind:isTsumo
                            bind:isRiichi
                            bind:isDoubleRiichi
                            bind:isIppatsu
                            bind:roundWind
                            bind:seatWind
                            bind:isLastTile
                            bind:isRinshan
                            bind:isChankan
                            bind:isTenhou
                            bind:isChiihou
                            {hasOpenMelds}
                            onChange={handleContextChange}
                        />
                    </div>

                    <!-- Calculate Button -->
                    <button
                        class="btn btn-primary btn-calculate"
                        onclick={calculate}
                        disabled={!canCalculate || isCalculating}
                    >
                        {#if isCalculating}
                            {$t.calculating}
                        {:else}
                            {$t.calculateScore}
                        {/if}
                    </button>
                </div>
            </div>
        {/if}
    </main>

    <footer class="footer">
        <p>
            {$t.footerPoweredBy}
            <a
                href="https://github.com/rysb-dev/agari"
                target="_blank"
                rel="noopener">Agari</a
            >
            {$t.footerDescription}
        </p>
    </footer>
</div>

<style>
    .app {
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        background: var(--bg-base);
    }

    /* Header */
    .header {
        padding: var(--space-4) var(--space-6);
        border-bottom: 1px solid var(--border);
        background: var(--bg-surface);
    }

    .header-content {
        max-width: 1400px;
        margin: 0 auto;
        display: flex;
        align-items: center;
        gap: var(--space-4);
    }

    .header-settings {
        display: flex;
        align-items: center;
        gap: var(--space-4);
        margin-left: auto;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        margin: 0;
        font-size: 1.25rem;
        font-weight: 700;
    }

    .logo-icon {
        font-size: 1.5rem;
    }

    .logo-text {
        color: var(--text-primary);
    }

    .tagline {
        margin: 0;
        color: var(--text-muted);
        font-size: 0.8125rem;
        padding-left: var(--space-4);
        border-left: 1px solid var(--border);
    }

    /* Main */
    .main {
        flex: 1;
        padding: var(--space-4);
        max-width: 1400px;
        margin: 0 auto;
        width: 100%;
    }

    .error-banner,
    .loading-banner {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: var(--space-4);
        padding: var(--space-6);
        background: var(--bg-surface);
        border: 1px solid var(--border);
        color: var(--text-secondary);
    }

    .error-banner {
        background: var(--error-muted);
        border-color: var(--error);
        color: var(--error);
    }

    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid var(--border);
        border-top-color: var(--accent);
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    /* Calculator Layout */
    .calculator-layout {
        display: grid;
        grid-template-columns: 1fr 340px;
        gap: 1px;
        background: var(--border);
        border: 1px solid var(--border);
    }

    .hand-section,
    .options-section {
        display: flex;
        flex-direction: column;
        background: var(--bg-base);
    }

    .hand-section {
        gap: 1px;
        background: var(--border);
    }

    .options-section {
        background: var(--bg-surface);
    }

    /* Cards / Panels */
    .card {
        background: var(--bg-surface);
        padding: var(--space-4);
    }

    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--space-4);
        padding-bottom: var(--space-3);
        border-bottom: 1px solid var(--border);
    }

    .header-actions {
        display: flex;
        gap: var(--space-2);
    }

    .card-header h2,
    .card-title {
        margin: 0;
        font-size: 0.6875rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted);
    }

    .card-title {
        margin-bottom: var(--space-4);
        padding-bottom: var(--space-3);
        border-bottom: 1px solid var(--border);
    }

    /* Buttons */
    .btn {
        padding: var(--space-2) var(--space-3);
        font-weight: 500;
        font-size: 0.8125rem;
        cursor: pointer;
        transition: all 0.15s ease;
        border: 1px solid var(--border);
        background: var(--bg-elevated);
        color: var(--text-primary);
    }

    .btn:hover:not(:disabled) {
        background: var(--bg-muted);
        border-color: var(--text-muted);
    }

    .btn-primary {
        background: var(--accent);
        border-color: var(--accent);
        color: white;
    }

    .btn-primary:hover:not(:disabled) {
        background: var(--accent-hover);
        border-color: var(--accent-hover);
    }

    .btn-secondary {
        background: var(--bg-elevated);
        border-color: var(--border);
    }

    .btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-calculate {
        width: 100%;
        padding: var(--space-3);
        font-size: 0.875rem;
        margin-top: var(--space-4);
    }

    .btn-sm {
        padding: var(--space-1) var(--space-2);
        font-size: 0.75rem;
    }

    /* Shanten Display */
    .shanten-display {
        margin-top: var(--space-3);
        padding-top: var(--space-3);
        border-top: 1px solid var(--border);
        display: flex;
        align-items: center;
        gap: var(--space-2);
        font-size: 0.8125rem;
    }

    .shanten-badge {
        padding: var(--space-1) var(--space-2);
        background: var(--bg-elevated);
        border: 1px solid var(--border);
        font-weight: 600;
        font-family: var(--font-mono);
        font-size: 0.75rem;
    }

    .shanten-badge.tenpai {
        background: var(--success-muted);
        border-color: var(--success);
        color: var(--success);
    }

    .shanten-badge.complete {
        background: var(--warning-muted);
        border-color: var(--warning);
        color: var(--warning);
    }

    .shanten-type {
        color: var(--text-muted);
        font-size: 0.75rem;
    }

    /* Dora Section */
    .dora-toggle {
        width: 100%;
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: none;
        border: none;
        color: var(--text-primary);
        font-size: 0.6875rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        cursor: pointer;
        padding: 0;
    }

    .toggle-arrow {
        transition: transform 0.2s ease;
        color: var(--text-muted);
    }

    .toggle-arrow.open {
        transform: rotate(180deg);
    }

    /* Meld buttons */
    .meld-buttons {
        display: flex;
        gap: var(--space-2);
        align-items: center;
        margin-top: var(--space-4);
        padding-top: var(--space-4);
        border-top: 1px solid var(--border);
        flex-wrap: wrap;
    }

    .meld-label {
        font-size: 0.6875rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted);
    }

    /* Meld builder */
    .meld-builder {
        margin-top: var(--space-4);
        padding: var(--space-3);
        background: var(--accent-muted);
        border: 1px solid var(--accent);
    }

    .meld-builder-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--space-3);
        font-weight: 500;
        font-size: 0.8125rem;
    }

    .meld-builder-hint {
        font-size: 0.6875rem;
        color: var(--text-muted);
    }

    .meld-builder-tiles {
        display: flex;
        gap: var(--space-2);
        margin-bottom: var(--space-3);
    }

    .meld-placeholder {
        width: 40px;
        height: 56px;
        border: 1px dashed var(--border);
        background: var(--bg-elevated);
    }

    .meld-builder-actions {
        display: flex;
        gap: var(--space-2);
        justify-content: flex-end;
    }

    /* Melds display */
    .melds-display {
        padding: var(--space-2) 0;
    }

    .melds-title {
        font-size: 0.6875rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        margin-bottom: var(--space-3);
        color: var(--text-muted);
    }

    .melds-list {
        display: flex;
        flex-wrap: wrap;
        gap: var(--space-3);
    }

    .meld-group {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        padding: var(--space-2);
        background: var(--bg-elevated);
        border: 1px solid var(--border);
    }

    .meld-type-badge {
        font-size: 0.625rem;
        padding: var(--space-1) var(--space-2);
        border: 1px solid var(--success);
        background: var(--success-muted);
        color: var(--success);
        text-transform: uppercase;
        font-weight: 600;
        font-family: var(--font-mono);
    }

    .meld-type-badge.open {
        border-color: var(--warning);
        background: var(--warning-muted);
        color: var(--warning);
    }

    .meld-tiles {
        display: flex;
        gap: 2px;
    }

    .btn-remove-meld {
        width: 18px;
        height: 18px;
        border: 1px solid var(--error);
        background: var(--error-muted);
        color: var(--error);
        cursor: pointer;
        font-size: 12px;
        line-height: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s ease;
    }

    .btn-remove-meld:hover {
        background: var(--error);
        color: white;
    }

    /* Hand tiles */
    .hand-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--space-3);
    }

    .hand-header h3 {
        margin: 0;
        font-size: 0.6875rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted);
    }

    .winning-tile-hint {
        font-size: 0.6875rem;
        color: var(--text-muted);
    }

    .hand-tiles-selectable {
        display: flex;
        flex-wrap: wrap;
        gap: var(--space-1);
    }

    .tile-wrapper {
        position: relative;
        cursor: pointer;
        transition: all 0.1s ease;
        background: none;
        border: 2px solid transparent;
        padding: 0;
    }

    .tile-wrapper:hover {
        border-color: var(--text-muted);
    }

    .tile-wrapper.selected {
        border-color: var(--success);
        background: var(--success-muted);
    }

    .winning-badge {
        position: absolute;
        bottom: -6px;
        left: 50%;
        transform: translateX(-50%);
        background: var(--success);
        color: var(--text-inverse);
        font-size: 0.5rem;
        font-weight: 700;
        font-family: var(--font-mono);
        padding: 1px 4px;
        text-transform: uppercase;
    }

    .tile-container {
        position: relative;
    }

    .tile-remove-btn {
        position: absolute;
        top: -6px;
        right: -6px;
        width: 16px;
        height: 16px;
        background: var(--error);
        color: white;
        border: 1px solid var(--bg-base);
        font-size: 12px;
        line-height: 1;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        opacity: 0;
        transition: opacity 0.15s ease;
        z-index: 10;
    }

    .tile-container:hover .tile-remove-btn,
    .tile-remove-btn:focus {
        opacity: 1;
    }

    @media (hover: none) {
        .tile-remove-btn {
            opacity: 0.8;
        }
    }

    .tile-placeholder {
        width: 40px;
        height: 56px;
        border: 1px dashed var(--border);
        background: var(--bg-elevated);
    }

    .shanten-error {
        margin-top: var(--space-2);
    }

    .shanten-error-text {
        font-size: 0.75rem;
        color: var(--error);
    }

    .hand-notation {
        font-family: var(--font-mono);
        font-size: 0.75rem;
        color: var(--text-secondary);
        background: var(--bg-elevated);
        border: 1px solid var(--border);
        padding: var(--space-2) var(--space-3);
        margin-top: var(--space-3);
        margin-bottom: 0;
        word-break: break-all;
    }

    .dora-content {
        margin-top: var(--space-4);
        display: flex;
        flex-direction: column;
        gap: var(--space-3);
    }

    .dora-row {
        display: flex;
        align-items: center;
        gap: var(--space-3);
    }

    .dora-label {
        width: 60px;
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .dora-tiles {
        display: flex;
        align-items: center;
        gap: var(--space-2);
        flex-wrap: wrap;
    }

    .dora-tile-wrapper {
        position: relative;
    }

    .dora-remove-btn {
        position: absolute;
        top: -4px;
        right: -4px;
        width: 14px;
        height: 14px;
        background: var(--error);
        color: white;
        border: 1px solid var(--bg-base);
        font-size: 10px;
        line-height: 1;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0;
        opacity: 0;
        transition: opacity 0.15s ease;
        z-index: 10;
    }

    .dora-tile-wrapper:hover .dora-remove-btn,
    .dora-remove-btn:focus {
        opacity: 1;
    }

    @media (hover: none) {
        .dora-remove-btn {
            opacity: 0.8;
        }
    }

    .dora-add-btn {
        padding: var(--space-1) var(--space-2);
        background: var(--bg-elevated);
        border: 1px dashed var(--border);
        color: var(--text-secondary);
        font-size: 0.6875rem;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .dora-add-btn:hover {
        background: var(--accent-muted);
        border-color: var(--accent);
        color: var(--accent);
    }

    .aka-display {
        font-size: 0.75rem;
        color: var(--text-secondary);
        padding-top: var(--space-2);
        border-top: 1px solid var(--border);
    }

    .aka-count {
        color: var(--man-color);
        font-weight: 600;
        font-family: var(--font-mono);
    }

    /* Results Card */
    .results-card {
        min-height: 200px;
    }

    /* Footer */
    .footer {
        padding: var(--space-4) var(--space-6);
        text-align: center;
        border-top: 1px solid var(--border);
        background: var(--bg-surface);
        color: var(--text-muted);
        font-size: 0.8125rem;
    }

    .footer a {
        color: var(--accent);
        text-decoration: none;
    }

    .footer a:hover {
        text-decoration: underline;
    }

    /* Responsive */
    @media (max-width: 1024px) {
        .calculator-layout {
            grid-template-columns: 1fr;
        }

        .options-section {
            order: -1;
        }
    }

    @media (max-width: 768px) {
        .header {
            padding: var(--space-3);
        }

        .header-content {
            flex-direction: column;
            align-items: flex-start;
            gap: var(--space-2);
        }

        .tagline {
            padding-left: 0;
            border-left: none;
        }

        .logo {
            font-size: 1.125rem;
        }

        .main {
            padding: var(--space-2);
        }

        .card {
            padding: var(--space-3);
        }
    }
</style>
