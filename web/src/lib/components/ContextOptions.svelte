<script lang="ts">
  import { WIND_NAMES } from '../agari';

  interface Props {
    isTsumo: boolean;
    isRiichi: boolean;
    isDoubleRiichi: boolean;
    isIppatsu: boolean;
    roundWind: 'east' | 'south' | 'west' | 'north';
    seatWind: 'east' | 'south' | 'west' | 'north';
    isLastTile: boolean;
    isRinshan: boolean;
    isChankan: boolean;
    isTenhou: boolean;
    isChiihou: boolean;
    hasOpenMelds?: boolean;
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

  $effect(() => {
    if (hasOpenMelds) {
      if (isRiichi) isRiichi = false;
      if (isDoubleRiichi) isDoubleRiichi = false;
      if (isIppatsu) isIppatsu = false;
    }
  });

  const winds = ['east', 'south', 'west', 'north'] as const;
  const windSymbols = { east: '東', south: '南', west: '西', north: '北' };

  const handleRiichiChange = () => {
    if (!isRiichi) {
      isDoubleRiichi = false;
      isIppatsu = false;
    }
    onChange();
  };

  const handleDoubleRiichiChange = () => {
    if (isDoubleRiichi) {
      isRiichi = true;
    }
    onChange();
  };

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
        Ron
      </button>
      <button
        type="button"
        class="toggle-btn"
        class:active={isTsumo}
        onclick={() => { isTsumo = true; onChange(); }}
      >
        Tsumo
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
      <div class="dealer-badge">Dealer (Oya)</div>
    {/if}
  </div>

  <!-- Riichi Options -->
  <div class="option-section">
    <h3 class="section-title">Riichi</h3>
    {#if hasOpenMelds}
      <div class="open-hand-notice">Open hand — Riichi not available</div>
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
        <span class="han-indicator">+1</span>
      </label>
      <label class="checkbox-item" class:disabled={hasOpenMelds || !isRiichi}>
        <input
          type="checkbox"
          bind:checked={isDoubleRiichi}
          disabled={hasOpenMelds || !isRiichi}
          onchange={handleDoubleRiichiChange}
        />
        <span class="checkbox-label">Double Riichi</span>
        <span class="han-indicator">+1</span>
      </label>
      <label class="checkbox-item" class:disabled={hasOpenMelds || !isRiichi}>
        <input
          type="checkbox"
          bind:checked={isIppatsu}
          disabled={hasOpenMelds || !isRiichi}
          onchange={onChange}
        />
        <span class="checkbox-label">Ippatsu</span>
        <span class="han-indicator">+1</span>
      </label>
    </div>
  </div>

  <!-- Situational Yaku -->
  <div class="option-section">
    <h3 class="section-title">Situational</h3>
    <div class="checkbox-group">
      <label class="checkbox-item">
        <input type="checkbox" bind:checked={isLastTile} onchange={onChange} />
        <span class="checkbox-label">
          {isTsumo ? 'Haitei (Last Draw)' : 'Houtei (Last Discard)'}
        </span>
        <span class="han-indicator">+1</span>
      </label>
      <label class="checkbox-item" class:disabled={!isTsumo}>
        <input
          type="checkbox"
          bind:checked={isRinshan}
          disabled={!isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Rinshan Kaihou</span>
        <span class="han-indicator">+1</span>
      </label>
      <label class="checkbox-item" class:disabled={isTsumo}>
        <input
          type="checkbox"
          bind:checked={isChankan}
          disabled={isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Chankan</span>
        <span class="han-indicator">+1</span>
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
        <span class="checkbox-label">Tenhou</span>
        <span class="han-indicator yakuman">役満</span>
      </label>
      <label class="checkbox-item" class:disabled={isDealer || !isTsumo}>
        <input
          type="checkbox"
          bind:checked={isChiihou}
          disabled={isDealer || !isTsumo}
          onchange={onChange}
        />
        <span class="checkbox-label">Chiihou</span>
        <span class="han-indicator yakuman">役満</span>
      </label>
    </div>
  </div>
</div>

<style>
  .context-options {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .option-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .section-title {
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0;
  }

  /* Toggle Buttons */
  .toggle-group {
    display: flex;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
  }

  .toggle-btn {
    flex: 1;
    padding: var(--space-2) var(--space-3);
    border: none;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .toggle-btn:hover {
    background: var(--bg-muted);
    color: var(--text-primary);
  }

  .toggle-btn.active {
    background: var(--accent);
    color: white;
  }

  /* Wind Buttons */
  .winds-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-4);
  }

  .wind-selector {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .wind-label {
    font-size: 0.6875rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .wind-buttons {
    display: flex;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
  }

  .wind-btn {
    flex: 1;
    padding: var(--space-2);
    border: none;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .wind-btn:hover {
    background: var(--bg-muted);
    color: var(--text-primary);
  }

  .wind-btn.active {
    background: var(--accent);
    color: white;
  }

  .dealer-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    background: var(--warning-muted);
    border: 1px solid var(--warning);
    color: var(--warning);
    font-size: 0.6875rem;
    font-weight: 600;
    width: fit-content;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Checkbox Items */
  .checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
  }

  .checkbox-item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .checkbox-item:hover:not(.disabled) {
    background: var(--bg-muted);
  }

  .checkbox-item.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .checkbox-item input[type="checkbox"] {
    width: 14px;
    height: 14px;
    cursor: pointer;
  }

  .checkbox-item.disabled input[type="checkbox"] {
    cursor: not-allowed;
  }

  .checkbox-label {
    flex: 1;
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .han-indicator {
    font-size: 0.625rem;
    font-family: var(--font-mono);
    font-weight: 600;
    padding: var(--space-1) var(--space-2);
    background: var(--accent-muted);
    border: 1px solid var(--accent);
    color: var(--accent);
  }

  .han-indicator.yakuman {
    background: var(--warning-muted);
    border-color: var(--warning);
    color: var(--warning);
  }

  .open-hand-notice {
    font-size: 0.75rem;
    color: var(--warning);
    background: var(--warning-muted);
    border: 1px solid var(--warning);
    padding: var(--space-2) var(--space-3);
  }

  @media (max-width: 768px) {
    .winds-grid {
      grid-template-columns: 1fr;
      gap: var(--space-3);
    }
  }
</style>
