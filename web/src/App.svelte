<script lang="ts">
  import { onMount } from 'svelte';
  import {
    initAgari,
    isLoaded,
    scoreHand,
    calculateShanten,
    type ScoreRequest,
    type ScoringOutput,
    type ShantenResponse,
    ALL_TILES,
    ALL_TILES_WITH_RED,
  } from './lib/agari';
  import TilePalette from './lib/components/TilePalette.svelte';
  import HandDisplay from './lib/components/HandDisplay.svelte';
  import ContextOptions from './lib/components/ContextOptions.svelte';
  import ScoreResult from './lib/components/ScoreResult.svelte';
  import Tile from './lib/components/Tile.svelte';

  // ============================================================================
  // State
  // ============================================================================

  interface TileEntry {
    tile: string;
    isRed?: boolean;
    id: number;
  }

  interface Meld {
    type: 'chi' | 'pon' | 'kan' | 'ankan';
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
  let meldBuilderType = $state<'chi' | 'pon' | 'kan' | 'ankan'>('pon');
  let meldBuilderTiles = $state<TileEntry[]>([]);

  // Dora state
  let doraIndicators = $state<TileEntry[]>([]);
  let uraDoraIndicators = $state<TileEntry[]>([]);
  let nextDoraId = $state(0);
  let showDoraSection = $state(false);

  // Context options
  let isTsumo = $state(false);
  let isRiichi = $state(false);
  let isDoubleRiichi = $state(false);
  let isIppatsu = $state(false);
  let roundWind = $state<'east' | 'south' | 'west' | 'north'>('east');
  let seatWind = $state<'east' | 'south' | 'west' | 'north'>('east');
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
  let mode = $state<'score' | 'shanten'>('score');

  // ============================================================================
  // Derived state
  // ============================================================================

  // Calculate remaining tiles
  const tileCounts = $derived.by(() => {
    const counts: Record<string, number> = {};
    for (const tile of ALL_TILES) {
      counts[tile] = 4;
    }
    // Red fives: only 1 of each exists (they come from the pool of 4 regular 5s)
    // We track red fives separately - when a red 5 is used, it reduces both
    // the red 5 count (max 1) AND the regular 5 count (since it's one of the 4 fives)
    const redFiveCounts: Record<string, number> = { '5m': 1, '5p': 1, '5s': 1 };

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
    // Subtract dora indicators
    for (const entry of doraIndicators) {
      if (counts[entry.tile] !== undefined) {
        counts[entry.tile]--;
      }
    }
    // Subtract ura dora indicators
    for (const entry of uraDoraIndicators) {
      if (counts[entry.tile] !== undefined) {
        counts[entry.tile]--;
      }
    }

    // Add red five counts to the return object (keyed as "red5m", "red5p", "red5s")
    return { ...counts, red5m: redFiveCounts['5m'], red5p: redFiveCounts['5p'], red5s: redFiveCounts['5s'] };
  });

  // Build hand string
  const handString = $derived.by(() => {
    if (handTiles.length === 0) return '';

    // Group tiles by suit, maintaining order for building the string
    const groups: Record<string, string[]> = { m: [], p: [], s: [], z: [] };

    for (const entry of handTiles) {
      const suit = entry.tile[1];
      const value = entry.isRed ? '0' : entry.tile[0];
      if (groups[suit]) {
        groups[suit].push(value);
      }
    }

    // Build string
    let result = '';
    for (const [suit, values] of Object.entries(groups)) {
      if (values.length > 0) {
        result += values.join('') + suit;
      }
    }
    return result;
  });

  // Count red fives (in hand and melds)
  const akaCount = $derived.by(() => {
    const handAka = handTiles.filter(t => t.isRed).length;
    const meldAka = melds.reduce((acc, m) => acc + m.tiles.filter(t => t.isRed).length, 0);
    return handAka + meldAka;
  });

  // Count tiles in melds (for display purposes)
  const tilesInMelds = $derived(melds.reduce((acc, m) => acc + m.tiles.length, 0));

  // Count meld slots used for hand size calculation
  // Each meld (pon/chi/kan) uses 3 "slots" from the hand, because:
  // - Pon/Chi: 3 tiles called
  // - Kan: 4 tiles called, but you draw a replacement tile (+1 to hand)
  // So effectively, all melds reduce hand size by 3
  const meldSlotsUsed = $derived(melds.length * 3);

  // Max tiles in hand based on mode and melds
  const maxHandTiles = $derived(mode === 'score' ? 14 - meldSlotsUsed : 13 - meldSlotsUsed);

  // Selected winning tile (always use standard notation, e.g. "5s" not "0s")
  const winningTile = $derived.by(() => {
    if (selectedWinningTileIndex !== null && handTiles[selectedWinningTileIndex]) {
      const entry = handTiles[selectedWinningTileIndex];
      return entry.tile; // Red-ness is tracked separately, winning tile just identifies which tile
    }
    return undefined;
  });

  // Check if hand has open melds
  const hasOpenMelds = $derived(melds.some(m => m.type !== 'ankan'));

  // Total tiles (hand + melds)
  const totalTiles = $derived(handTiles.length + tilesInMelds);

  // Can calculate score
  const canCalculate = $derived(
    wasmLoaded &&
    totalTiles >= (mode === 'score' ? 14 : 1)
  );

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
    } else if (selectedWinningTileIndex !== null && index < selectedWinningTileIndex) {
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
  function startMeldBuilder(type: 'chi' | 'pon' | 'kan' | 'ankan') {
    meldBuilderType = type;
    meldBuilderTiles = [];
    showMeldBuilder = true;
  }

  function addTileToMeldBuilder(tile: string, isRed: boolean = false) {
    const maxTiles = meldBuilderType === 'kan' || meldBuilderType === 'ankan' ? 4 : 3;
    if (meldBuilderTiles.length >= maxTiles) return;
    if (tileCounts[tile] <= 0) return;

    // For chi, tiles must be sequential in the same suit
    if (meldBuilderType === 'chi' && meldBuilderTiles.length > 0) {
      const suit = meldBuilderTiles[0].tile[1];
      if (tile[1] !== suit || tile[1] === 'z') return; // Must be same suit, no honors
    }

    // For pon/kan, tiles must be the same
    if ((meldBuilderType === 'pon' || meldBuilderType === 'kan' || meldBuilderType === 'ankan') && meldBuilderTiles.length > 0) {
      const baseTile = meldBuilderTiles[0].tile;
      if (tile !== baseTile) return;
    }

    meldBuilderTiles = [...meldBuilderTiles, { tile, isRed, id: nextTileId++ }];
  }

  function removeTileFromMeldBuilder(index: number) {
    meldBuilderTiles = meldBuilderTiles.filter((_, i) => i !== index);
  }

  function confirmMeld() {
    const requiredTiles = meldBuilderType === 'kan' || meldBuilderType === 'ankan' ? 4 : 3;
    if (meldBuilderTiles.length !== requiredTiles) return;

    // For chi, sort tiles
    if (meldBuilderType === 'chi') {
      meldBuilderTiles.sort((a, b) => {
        const valA = a.isRed ? 5 : parseInt(a.tile[0]);
        const valB = b.isRed ? 5 : parseInt(b.tile[0]);
        return valA - valB;
      });
    }

    melds = [...melds, { type: meldBuilderType, tiles: [...meldBuilderTiles], id: nextMeldId++ }];
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
    if (tileCounts[tile] <= 0) return;
    doraIndicators = [...doraIndicators, { tile, id: nextDoraId++ }];
  }

  // Remove dora indicator
  function removeDoraIndicator(index: number) {
    doraIndicators = doraIndicators.filter((_, i) => i !== index);
  }

  // Add ura dora indicator
  function addUraDoraIndicator(tile: string, _isRed: boolean = false) {
    if (uraDoraIndicators.length >= 5) return;
    if (tileCounts[tile] <= 0) return;
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
  }

  // Build meld notation string for backend
  // Format: (111m) for open melds (pon/chi/open kan), [1111m] for closed kan (ankan)
  function buildMeldNotation(): string {
    let meldStr = '';
    for (const meld of melds) {
      const tiles = meld.tiles.map(t => t.isRed ? '0' : t.tile[0]).join('');
      const suit = meld.tiles[0].tile[1];
      if (meld.type === 'ankan') {
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
      if (tile[0] === '0') {
        return '5' + tile[1];
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
        dora_indicators: doraIndicators.map(d => normalizeRedFive(d.tile)),
        ura_dora_indicators: uraDoraIndicators.map(d => normalizeRedFive(d.tile)),
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
      } else {
        scoreError = response.error || 'Unknown error';
        scoreResult = null;
      }
    } catch (e) {
      scoreError = e instanceof Error ? e.message : 'Calculation failed';
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
    } catch (e) {
      wasmError = e instanceof Error ? e.message : 'Failed to load WASM module';
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
      <p class="tagline">Riichi Mahjong Calculator</p>
    </div>
  </header>

  <main class="main">
    {#if wasmError}
      <div class="error-banner">
        <span>⚠️ Failed to load calculator: {wasmError}</span>
      </div>
    {:else if !wasmLoaded}
      <div class="loading-banner">
        <div class="spinner"></div>
        <span>Loading calculator...</span>
      </div>
    {:else}
      <div class="calculator-layout">
        <!-- Left Column: Hand Builder -->
        <div class="hand-section">
          <div class="card">
            <div class="card-header">
              <h2>Build Your Hand</h2>
              <button class="btn btn-secondary" onclick={clearHand}>
                Clear
              </button>
            </div>

            <!-- Tile Palette -->
            <TilePalette
              onSelect={showMeldBuilder ? addTileToMeldBuilder : addTile}
              tileCounts={tileCounts}
              showRedFives={true}
            />

            <!-- Meld Builder Buttons -->
            <div class="meld-buttons">
              <span class="meld-label">Add Meld:</span>
              <button class="btn btn-sm" onclick={() => startMeldBuilder('chi')} disabled={showMeldBuilder}>Chi</button>
              <button class="btn btn-sm" onclick={() => startMeldBuilder('pon')} disabled={showMeldBuilder}>Pon</button>
              <button class="btn btn-sm" onclick={() => startMeldBuilder('kan')} disabled={showMeldBuilder}>Open Kan</button>
              <button class="btn btn-sm" onclick={() => startMeldBuilder('ankan')} disabled={showMeldBuilder}>Closed Kan</button>
            </div>

            <!-- Meld Builder Panel -->
            {#if showMeldBuilder}
              <div class="meld-builder">
                <div class="meld-builder-header">
                  <span>Building {meldBuilderType === 'ankan' ? 'Closed Kan' : meldBuilderType === 'kan' ? 'Open Kan' : meldBuilderType.charAt(0).toUpperCase() + meldBuilderType.slice(1)}</span>
                  <span class="meld-builder-hint">
                    {#if meldBuilderType === 'chi'}
                      (select 3 sequential tiles of the same suit)
                    {:else if meldBuilderType === 'pon'}
                      (select 3 of the same tile)
                    {:else}
                      (select 4 of the same tile)
                    {/if}
                  </span>
                </div>
                <div class="meld-builder-tiles">
                  {#each meldBuilderTiles as entry, index (entry.id)}
                    <Tile
                      tile={entry.tile}
                      red={entry.isRed}
                      size="md"
                      onclick={() => removeTileFromMeldBuilder(index)}
                    />
                  {/each}
                  {#each Array((meldBuilderType === 'kan' || meldBuilderType === 'ankan' ? 4 : 3) - meldBuilderTiles.length) as _}
                    <div class="meld-placeholder"></div>
                  {/each}
                </div>
                <div class="meld-builder-actions">
                  <button class="btn btn-sm btn-secondary" onclick={cancelMeldBuilder}>Cancel</button>
                  <button
                    class="btn btn-sm btn-primary"
                    onclick={confirmMeld}
                    disabled={meldBuilderTiles.length !== (meldBuilderType === 'kan' || meldBuilderType === 'ankan' ? 4 : 3)}
                  >
                    Add Meld
                  </button>
                </div>
              </div>
            {/if}
          </div>

          <!-- Melds Display -->
          {#if melds.length > 0}
            <div class="card">
              <div class="melds-display">
                <h3 class="melds-title">Called Melds</h3>
                <div class="melds-list">
                  {#each melds as meld, index (meld.id)}
                    <div class="meld-group">
                      <span class="meld-type-badge" class:open={meld.type !== 'ankan'}>
                        {meld.type === 'ankan' ? '🔒' : '📢'} {meld.type}
                      </span>
                      <div class="meld-tiles">
                        {#each meld.tiles as entry (entry.id)}
                          <Tile tile={entry.tile} red={entry.isRed} size="sm" />
                        {/each}
                      </div>
                      <button class="btn-remove-meld" onclick={() => removeMeld(index)}>×</button>
                    </div>
                  {/each}
                </div>
              </div>
            </div>
          {/if}

          <!-- Hand Display -->
          <div class="card">
            <div class="hand-header">
              <h3>Your Hand</h3>
              {#if handTiles.length > 0}
                <span class="winning-tile-hint">Click a tile to select it as winning tile</span>
              {/if}
            </div>
            <div class="hand-tiles-selectable">
              {#each handTiles as entry, index (entry.id)}
                <button
                  type="button"
                  class="tile-wrapper"
                  class:selected={selectedWinningTileIndex === index}
                  onclick={() => selectWinningTile(index)}
                  oncontextmenu={(e) => { e.preventDefault(); removeTile(index); }}
                >
                  <Tile tile={entry.tile} red={entry.isRed} size="md" />
                  {#if selectedWinningTileIndex === index}
                    <div class="winning-badge">WIN</div>
                  {/if}
                </button>
              {/each}
              {#each Array(Math.max(0, maxHandTiles - handTiles.length)) as _}
                <div class="tile-placeholder"></div>
              {/each}
            </div>
            {#if handTiles.length > 0}
              <p class="hand-hint">Right-click to remove a tile</p>
            {/if}
            {#if handTiles.length > 0 || melds.length > 0}
              <p class="hand-notation">{handString}{buildMeldNotation()}</p>
            {/if}

            <!-- Shanten Display -->
            {#if shantenResult && (handTiles.length > 0 || melds.length > 0)}
              {#if shantenResult.success}
                <div class="shanten-display">
                  {#if shantenResult.shanten === -1}
                    <span class="shanten-badge complete">✓ Complete</span>
                  {:else if shantenResult.shanten === 0}
                    <span class="shanten-badge tenpai">Tenpai</span>
                  {:else}
                    <span class="shanten-badge">{shantenResult.shanten}-shanten</span>
                  {/if}
                  <span class="shanten-type">({shantenResult.best_type})</span>
                </div>
              {:else if shantenResult.error}
                <div class="shanten-error">
                  <span class="shanten-error-text">Shanten: {shantenResult.error}</span>
                </div>
              {/if}
            {/if}
          </div>

          <!-- Dora Section -->
          <div class="card">
            <button
              class="dora-toggle"
              onclick={() => showDoraSection = !showDoraSection}
            >
              <span>Dora Indicators</span>
              <span class="toggle-arrow" class:open={showDoraSection}>▼</span>
            </button>

            {#if showDoraSection}
              <div class="dora-content">
                <div class="dora-row">
                  <label class="dora-label">Dora</label>
                  <div class="dora-tiles">
                    {#each doraIndicators as entry, index (entry.id)}
                      <Tile
                        tile={entry.tile}
                        size="sm"
                        onclick={() => removeDoraIndicator(index)}
                      />
                    {/each}
                    {#if doraIndicators.length < 5}
                      <select
                        class="dora-select"
                        onchange={(e) => {
                          const target = e.target as HTMLSelectElement;
                          if (target.value) {
                            addDoraIndicator(target.value);
                            target.value = '';
                          }
                        }}
                      >
                        <option value="">+ Add</option>
                        {#each ALL_TILES_WITH_RED as tile}
                          <option value={tile}>{tile}</option>
                        {/each}
                      </select>
                    {/if}
                  </div>
                </div>

                {#if isRiichi}
                  <div class="dora-row">
                    <label class="dora-label">Ura Dora</label>
                    <div class="dora-tiles">
                      {#each uraDoraIndicators as entry, index (entry.id)}
                        <Tile
                          tile={entry.tile}
                          size="sm"
                          onclick={() => removeUraDoraIndicator(index)}
                        />
                      {/each}
                      {#if uraDoraIndicators.length < 5}
                        <select
                          class="dora-select"
                          onchange={(e) => {
                            const target = e.target as HTMLSelectElement;
                            if (target.value) {
                              addUraDoraIndicator(target.value);
                              target.value = '';
                            }
                          }}
                        >
                          <option value="">+ Add</option>
                          {#each ALL_TILES_WITH_RED as tile}
                            <option value={tile}>{tile}</option>
                          {/each}
                        </select>
                      {/if}
                    </div>
                  </div>
                {/if}

                {#if akaCount > 0}
                  <div class="aka-display">
                    Aka Dora in hand: <span class="aka-count">{akaCount}</span>
                  </div>
                {/if}
              </div>
            {/if}
          </div>

          <!-- Results (moved from right column) -->
          <div class="card results-card">
            <h2 class="card-title">Results</h2>
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
            <h2 class="card-title">Options</h2>
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
              Calculating...
            {:else}
              Calculate Score
            {/if}
          </button>
        </div>
      </div>
    {/if}
  </main>

  <footer class="footer">
    <p>
      Powered by <a href="https://github.com/ryblogs/agari" target="_blank" rel="noopener">Agari</a>
      — A Riichi Mahjong scoring engine written in Rust
    </p>
  </footer>
</div>

<style>
  .app {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  /* Header */
  .header {
    padding: 1.5rem;
    text-align: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .header-content {
    max-width: 1200px;
    margin: 0 auto;
  }

  .logo {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin: 0;
    font-size: 2rem;
    font-weight: 700;
  }

  .logo-icon {
    font-size: 2.5rem;
  }

  .logo-text {
    background: linear-gradient(135deg, #e94560, #ff6b6b);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .tagline {
    margin: 0.5rem 0 0 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  /* Main */
  .main {
    flex: 1;
    padding: 1.5rem;
    max-width: 1400px;
    margin: 0 auto;
    width: 100%;
  }

  .error-banner,
  .loading-banner {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
    background: var(--bg-card);
    border-radius: 12px;
    color: var(--text-secondary);
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--bg-secondary);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Calculator Layout */
  .calculator-layout {
    display: grid;
    grid-template-columns: 1fr 380px;
    gap: 1.5rem;
  }

  .hand-section,
  .options-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  /* Cards */
  .card {
    background: var(--bg-card);
    border-radius: 12px;
    padding: 1.25rem;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .card-header h2,
  .card-title {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .card-header h2 {
    margin-bottom: 0;
  }

  /* Buttons */
  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s ease;
    border: none;
  }

  .btn-primary {
    background: var(--accent);
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .btn-secondary {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--text-secondary);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-primary);
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-calculate {
    width: 100%;
    padding: 1rem;
    font-size: 1rem;
  }

  /* Shanten Display */
  .shanten-display {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
  }

  .shanten-badge {
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
    font-weight: 600;
  }

  .shanten-badge.tenpai {
    background: var(--success);
    color: var(--bg-primary);
  }

  .shanten-badge.complete {
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: var(--bg-primary);
  }

  .shanten-type {
    color: var(--text-secondary);
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
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
  }

  .toggle-arrow {
    transition: transform 0.2s ease;
  }

  .toggle-arrow.open {
    transform: rotate(180deg);
  }

  /* Meld buttons */
  .meld-buttons {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    flex-wrap: wrap;
  }

  .meld-label {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .btn-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.8rem;
  }

  /* Meld builder */
  .meld-builder {
    margin-top: 1rem;
    padding: 1rem;
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-radius: 0.5rem;
  }

  .meld-builder-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    font-weight: 500;
  }

  .meld-builder-hint {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .meld-builder-tiles {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .meld-placeholder {
    width: 40px;
    height: 56px;
    border: 2px dashed rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  .meld-builder-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  /* Melds display */
  .melds-display {
    padding: 0.5rem 0;
  }

  .melds-title {
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 0.75rem;
    color: rgba(255, 255, 255, 0.8);
  }

  .melds-list {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .meld-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 0.5rem;
  }

  .meld-type-badge {
    font-size: 0.7rem;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    background: rgba(34, 197, 94, 0.2);
    color: #22c55e;
    text-transform: uppercase;
  }

  .meld-type-badge.open {
    background: rgba(249, 115, 22, 0.2);
    color: #f97316;
  }

  .meld-tiles {
    display: flex;
    gap: 0.125rem;
  }

  .btn-remove-meld {
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 50%;
    border: none;
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    transition: background 0.2s;
  }

  .btn-remove-meld:hover {
    background: rgba(239, 68, 68, 0.4);
  }

  /* Hand tiles selectable */
  .hand-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .hand-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
  }

  .winning-tile-hint {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
  }

  .hand-tiles-selectable {
    display: flex;
    flex-wrap: wrap;
    gap: 0.375rem;
  }

  .tile-wrapper {
    position: relative;
    cursor: pointer;
    border-radius: 4px;
    transition: transform 0.15s, box-shadow 0.15s;
    background: none;
    border: none;
    padding: 0;
  }

  .tile-wrapper:hover {
    transform: translateY(-2px);
  }

  .tile-wrapper.selected {
    box-shadow: 0 0 0 3px #22c55e, 0 4px 12px rgba(34, 197, 94, 0.4);
    transform: translateY(-4px);
  }

  .winning-badge {
    position: absolute;
    top: -8px;
    right: -8px;
    background: #22c55e;
    color: white;
    font-size: 0.6rem;
    font-weight: 700;
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .tile-placeholder {
    width: 40px;
    height: 56px;
    border: 2px dashed rgba(255, 255, 255, 0.15);
    border-radius: 4px;
  }

  .hand-hint {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
    margin-top: 0.5rem;
    margin-bottom: 0;
  }

  .shanten-error {
    margin-top: 0.5rem;
  }

  .shanten-error-text {
    font-size: 0.75rem;
    color: rgba(239, 68, 68, 0.8);
  }

  .hand-notation {
    font-family: monospace;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
    background: rgba(0, 0, 0, 0.3);
    padding: 0.375rem 0.75rem;
    border-radius: 0.25rem;
    margin-top: 0.5rem;
    margin-bottom: 0;
    word-break: break-all;
  }

  .dora-content {
    margin-top: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .dora-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .dora-label {
    width: 70px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .dora-tiles {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    flex-wrap: wrap;
  }

  .dora-select {
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--text-secondary);
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .aka-display {
    font-size: 0.8rem;
    color: var(--text-secondary);
    padding-top: 0.5rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .aka-count {
    color: #c41e3a;
    font-weight: 600;
  }

  /* Results Card */
  .results-card {
    min-height: 200px;
  }

  /* Footer */
  .footer {
    padding: 1.5rem;
    text-align: center;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
    font-size: 0.875rem;
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
      padding: 1rem;
    }

    .logo {
      font-size: 1.5rem;
    }

    .logo-icon {
      font-size: 2rem;
    }

    .main {
      padding: 1rem;
    }

    .card {
      padding: 1rem;
    }
  }
</style>
