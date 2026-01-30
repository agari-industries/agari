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

  let wasmLoaded = $state(false);
  let wasmError = $state<string | null>(null);

  // Hand state
  let handTiles = $state<TileEntry[]>([]);
  let nextTileId = $state(0);

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
    // Subtract hand tiles
    for (const entry of handTiles) {
      if (counts[entry.tile] !== undefined) {
        counts[entry.tile]--;
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
    return counts;
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

  // Count red fives
  const akaCount = $derived(handTiles.filter(t => t.isRed).length);

  // Max tiles based on mode
  const maxTiles = $derived(mode === 'score' ? 14 : 13);

  // Can calculate score
  const canCalculate = $derived(
    wasmLoaded &&
    handTiles.length >= (mode === 'score' ? 14 : 1)
  );

  // ============================================================================
  // Functions
  // ============================================================================

  // Add tile to hand
  function addTile(tile: string, isRed: boolean = false) {
    if (handTiles.length >= maxTiles) return;
    if (tileCounts[tile] <= 0) return;

    handTiles = [...handTiles, { tile, isRed, id: nextTileId++ }];
    recalculateShanten();
  }

  // Remove tile from hand
  function removeTile(index: number) {
    handTiles = handTiles.filter((_, i) => i !== index);
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
    doraIndicators = [];
    uraDoraIndicators = [];
    scoreResult = null;
    scoreError = null;
    shantenResult = null;
  }

  // Calculate shanten in real-time
  function recalculateShanten() {
    if (!wasmLoaded || handTiles.length === 0) {
      shantenResult = null;
      return;
    }

    const result = calculateShanten(handString);
    shantenResult = result;
  }

  // Calculate score
  function calculate() {
    if (!canCalculate) return;

    isCalculating = true;
    scoreError = null;
    scoreResult = null;

    try {
      const request: ScoreRequest = {
        hand: handString,
        is_tsumo: isTsumo,
        is_riichi: isRiichi,
        is_double_riichi: isDoubleRiichi,
        is_ippatsu: isIppatsu,
        round_wind: roundWind,
        seat_wind: seatWind,
        dora_indicators: doraIndicators.map(d => d.tile),
        ura_dora_indicators: uraDoraIndicators.map(d => d.tile),
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
              onSelect={addTile}
              tileCounts={tileCounts}
              showRedFives={true}
            />
          </div>

          <!-- Hand Display -->
          <div class="card">
            <HandDisplay
              tiles={handTiles}
              onRemove={removeTile}
              maxTiles={maxTiles}
              label="Your Hand"
            />

            <!-- Shanten Display -->
            {#if shantenResult && shantenResult.success && handTiles.length > 0}
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
                        {#each ALL_TILES as tile}
                          {#if tileCounts[tile] > 0}
                            <option value={tile}>{tile}</option>
                          {/if}
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
                          {#each ALL_TILES as tile}
                            {#if tileCounts[tile] > 0}
                              <option value={tile}>{tile}</option>
                            {/if}
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
