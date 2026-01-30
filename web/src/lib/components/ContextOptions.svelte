<script lang="ts">
  import { WIND_NAMES } from '../agari';

  interface Props {
    /** Whether win is by tsumo (self-draw) */
    isTsumo: boolean;
    /** Whether riichi was declared */
    isRiichi: boolean;
    /** Whether double riichi was declared */
    isDoubleRiichi: boolean;
    /** Whether ippatsu (win within one turn of riichi) */
    isIppatsu: boolean;
    /** Round wind */
    roundWind: 'east' | 'south' | 'west' | 'north';
    /** Seat wind */
    seatWind: 'east' | 'south' | 'west' | 'north';
    /** Whether won on the last tile */
    isLastTile: boolean;
    /** Whether won on kan replacement tile */
    isRinshan: boolean;
    /** Whether ron on another player's added kan */
    isChankan: boolean;
    /** Whether tenhou */
    isTenhou: boolean;
    /** Whether chiihou */
    isChiihou: boolean;
    /** Whether hand has open melds (chi, pon, open kan) */
    hasOpenMelds?: boolean;
    /** Callback for any option change */
    onChange: () => void;
  }

  let {
    isTsumo = $bindable(),
    isRiichi = $bindable(),
    isDoubleRiichi = $bindable(),
    isIppatsu = $bindable(),
    roundWind = $bindable(),
    seatWind = $bindable(),
    isLastTile = $bindable(),
    isRinshan = $bindable(),
    isChankan = $bindable(),
    isTenhou = $bindable(),
    isChiihou = $bindable(),
    hasOpenMelds = false,
    onChange,
  }: Props = $props();

  // When hand becomes open, uncheck riichi-related options
  $effect(() => {
    if (hasOpenMelds) {
      if (isRiichi) isRiichi = false;
      if (isDoubleRiichi) isDoubleRiichi = false;
      if (isIppatsu) isIppatsu = false;
    }
  });

  const winds = ['east', 'south', 'west', 'north'] as const;
  const windSymbols = { east: '東', south: '南', west: '西', north: '北' };

  // Handle riichi toggle
  const handleRiichiChange = () => {
    if (!isRiichi) {
      isDoubleRiichi = false;
      isIppatsu = false;
    }
    onChange();
  };

  // Handle double riichi toggle
  const handleDoubleRiichiChange = () => {
    if (isDoubleRiichi) {
      isRiichi = true;
    }
    onChange();
  };

  // Is dealer (East seat)
  const isDealer = $derived(seatWind === 'east');
</script>

<div class="context-options">
  <!-- Win Type -->
  <div class="option-section">
    <h3 class="section-title">Win Type</h3>
    <div class="toggle-group">
      <button
        type="button"
        class="toggle-btn"
        class:active={!isTsumo}
        onclick={() => { isTsumo = false; onChange(); }}
      >
        <span class="toggle-icon">🀄</span>
        <span>Ron</span>
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={isTsumo}
        onclick={() => { isTsumo = true; onChange(); }}
      >
        <span class="toggle-icon">🎯</span>
        <span>Tsumo</span>
      </button>
    </div>
  </div>

  <!-- Winds -->
  <div class="option-section">
    <h3 class="section-title">Winds</h3>
    <div class="winds-grid">
      <div class="wind-selector">
        <label class="wind-label">Round</label>
        <div class="wind-buttons">
          {#each winds as wind}
            <button
              type="button"
              class="wind-btn"
              class:active={roundWind === wind}
              onclick={() => { roundWind = wind; onChange(); }}
              title={WIND_NAMES[wind]}
            >
              {windSymbols[wind]}
            </button>
          {/each}
        </div>
      </div>
      <div class="wind-selector">
        <label class="wind-label">Seat</label>
        <div class="wind-buttons">
          {#each winds as wind}
            <button
              type="button"
              class="wind-btn"
              class:active={seatWind === wind}
              onclick={() => { seatWind = wind; onChange(); }}
              title={WIND_NAMES[wind]}
            >
              {windSymbols[wind]}
            </button>
          {/each}
        </div>
      </div>
    </div>
    {#if isDealer}
      <div class="dealer-badge">👑 Dealer (Oya)</div>
    {/if}
  </div>

  <!-- Riichi Options -->
  <div class="option-section">
    <h3 class="section-title">Riichi</h3>
    {#if hasOpenMelds}
      <div class="open-hand-notice">
        🔓 Open hand — Riichi not available
      </div>
    {/if}
    <div class="checkbox-group">
      <label class="checkbox-item" class:disabled={hasOpenMelds}>
        <input
          type="checkbox"
          bind:checked={isRiichi}
          disabled={hasOpenMelds}
          onchange={handleRiichiChange}
        />
        <span class="checkbox-label">Riichi</span>
        <span class="han-indicator">+1 han</span>
      </label>
      <label class="checkbox-item" class:disabled={hasOpenMelds || !isRiichi}>
        <input
          type="checkbox"
          bind:checked={isDoubleRiichi}
          disabled={hasOpenMelds || !isRiichi}
          onchange={handleDoubleRiichiChange}
        />
        <span class="checkbox-label">Double Riichi</span>
        <span class="han-indicator">+1 han</span>
      </label>
      <label class="checkbox-item" class:disabled={hasOpenMelds || !isRiichi}>
        <input
          type="checkbox"
          bind:checked={isIppatsu}
          disabled={hasOpenMelds || !isRiichi}
          onchange={onChange}
        />
        <span class="checkbox-label">Ippatsu</span>
        <span class="han-indicator">+1 han</span>
      </label>
    </div>
  </div>

  <!-- Situational Yaku -->
  <div class="option-section">
    <h3 class="section-title">Situational</h3>
    <div class="checkbox-group">
      <label class="checkbox-item">
        <input
          type="checkbox"
          bind:checked={isLastTile}
          onchange={onChange}
        />
        <span class="checkbox-label">
          {isTsumo ? 'Haitei (Last Draw)' : 'Houtei (Last Discard)'}
        </span>
        <span class="han-indicator">+1 han</span>
      </label>
      <label class="checkbox-item" class:disabled={!isTsumo}>
        <input
          type="checkbox"
          bind:checked={isRinshan}
          disabled={!isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Rinshan Kaihou</span>
        <span class="han-indicator">+1 han</span>
      </label>
      <label class="checkbox-item" class:disabled={isTsumo}>
        <input
          type="checkbox"
          bind:checked={isChankan}
          disabled={isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Chankan</span>
        <span class="han-indicator">+1 han</span>
      </label>
    </div>
  </div>

  <!-- Yakuman Starters -->
  <div class="option-section">
    <h3 class="section-title">First Turn Yakuman</h3>
    <div class="checkbox-group">
      <label class="checkbox-item" class:disabled={!isDealer || !isTsumo}>
        <input
          type="checkbox"
          bind:checked={isTenhou}
          disabled={!isDealer || !isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Tenhou (Blessing of Heaven)</span>
        <span class="han-indicator yakuman">役満</span>
      </label>
      <label class="checkbox-item" class:disabled={isDealer || !isTsumo}>
        <input
          type="checkbox"
          bind:checked={isChiihou}
          disabled={isDealer || !isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Chiihou (Blessing of Earth)</span>
        <span class="han-indicator yakuman">役満</span>
      </label>
    </div>
  </div>
</div>

<style>
  .context-options {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .option-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .section-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
  }

  /* Toggle Buttons */
  .toggle-group {
    display: flex;
    gap: 0.5rem;
  }

  .toggle-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border: 2px solid var(--text-secondary);
    border-radius: 8px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toggle-btn:hover {
    border-color: var(--accent);
  }

  .toggle-btn.active {
    border-color: var(--accent);
    background: var(--accent);
    color: white;
  }

  .toggle-icon {
    font-size: 1.25rem;
  }

  /* Wind Buttons */
  .winds-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .wind-selector {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .wind-label {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .wind-buttons {
    display: flex;
    gap: 0.25rem;
  }

  .wind-btn {
    width: 40px;
    height: 40px;
    border-radius: 6px;
    border: 2px solid var(--text-secondary);
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .wind-btn:hover {
    border-color: var(--accent);
  }

  .wind-btn.active {
    border-color: var(--accent);
    background: var(--accent);
    color: white;
  }

  .dealer-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: var(--bg-primary);
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    width: fit-content;
  }

  /* Checkbox Items */
  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-secondary);
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.2s ease;
  }

  .checkbox-item:hover:not(.disabled) {
    background: rgba(233, 69, 96, 0.1);
  }

  .checkbox-item.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-item input[type="checkbox"] {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .checkbox-item.disabled input[type="checkbox"] {
    cursor: not-allowed;
  }

  .checkbox-label {
    flex: 1;
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .han-indicator {
    font-size: 0.75rem;
    padding: 0.125rem 0.375rem;
    background: var(--accent);
    color: white;
    border-radius: 4px;
    font-weight: 600;
  }

  .han-indicator.yakuman {
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: var(--bg-primary);
  }

  .open-hand-notice {
    font-size: 0.8rem;
    color: var(--text-secondary);
    background: rgba(255, 193, 7, 0.15);
    border: 1px solid rgba(255, 193, 7, 0.3);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.5rem;
  }

  @media (max-width: 768px) {
    .winds-grid {
      grid-template-columns: 1fr;
    }

    .toggle-btn {
      padding: 0.5rem 0.75rem;
      font-size: 0.8rem;
    }

    .wind-btn {
      width: 36px;
      height: 36px;
      font-size: 1rem;
    }
  }
</style>
