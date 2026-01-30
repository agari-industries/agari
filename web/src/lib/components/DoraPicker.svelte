<script lang="ts">
  import { ALL_TILES } from '../agari';
  import Tile from './Tile.svelte';

  interface Props {
    /** Callback when a tile is selected */
    onSelect: (tile: string) => void;
    /** Callback to close the picker */
    onClose: () => void;
    /** Optional: tiles to disable (already selected) */
    disabledTiles?: Set<string>;
  }

  let {
    onSelect,
    onClose,
    disabledTiles = new Set(),
  }: Props = $props();

  // Group tiles by suit (regular tiles only, red fives added separately at the end)
  const manTiles = ALL_TILES.filter((t) => t.endsWith('m'));
  const pinTiles = ALL_TILES.filter((t) => t.endsWith('p'));
  const souTiles = ALL_TILES.filter((t) => t.endsWith('s'));
  const honorTiles = ALL_TILES.filter((t) => t.endsWith('z'));

  // Check if tile is disabled
  const isDisabled = (tile: string): boolean => disabledTiles.has(tile);

  // Handle tile click
  const handleClick = (tile: string) => {
    if (!isDisabled(tile)) {
      onSelect(tile);
    }
  };

  // Handle click outside to close
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  // Handle escape key
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
<div class="picker-backdrop" role="dialog" aria-modal="true" onclick={handleBackdropClick}>
  <div class="picker-panel">
    <div class="picker-header">
      <span>Select Tile</span>
      <button type="button" class="close-btn" onclick={onClose}>×</button>
    </div>

    <div class="picker-grid">
      <!-- Man (Characters) -->
      <div class="tile-row">
        {#each manTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile
              {tile}
              size="sm"
              disabled={isDisabled(tile)}
            />
          </button>
        {/each}
        <!-- Red 5m at the end -->
        <button
          type="button"
          class="tile-btn"
          class:disabled={isDisabled('0m')}
          disabled={isDisabled('0m')}
          onclick={() => handleClick('0m')}
        >
          <Tile
            tile="5m"
            size="sm"
            red={true}
            disabled={isDisabled('0m')}
          />
        </button>
      </div>

      <!-- Pin (Dots) -->
      <div class="tile-row">
        {#each pinTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile
              {tile}
              size="sm"
              disabled={isDisabled(tile)}
            />
          </button>
        {/each}
        <!-- Red 5p at the end -->
        <button
          type="button"
          class="tile-btn"
          class:disabled={isDisabled('0p')}
          disabled={isDisabled('0p')}
          onclick={() => handleClick('0p')}
        >
          <Tile
            tile="5p"
            size="sm"
            red={true}
            disabled={isDisabled('0p')}
          />
        </button>
      </div>

      <!-- Sou (Bamboo) -->
      <div class="tile-row">
        {#each souTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile
              {tile}
              size="sm"
              disabled={isDisabled(tile)}
            />
          </button>
        {/each}
        <!-- Red 5s at the end -->
        <button
          type="button"
          class="tile-btn"
          class:disabled={isDisabled('0s')}
          disabled={isDisabled('0s')}
          onclick={() => handleClick('0s')}
        >
          <Tile
            tile="5s"
            size="sm"
            red={true}
            disabled={isDisabled('0s')}
          />
        </button>
      </div>

      <!-- Honors -->
      <div class="tile-row honors">
        {#each honorTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile
              {tile}
              size="sm"
              disabled={isDisabled(tile)}
            />
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .picker-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .picker-panel {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1rem;
    max-width: 90vw;
    max-height: 90vh;
    overflow: auto;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .picker-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border);
  }

  .picker-header span {
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background-color 0.2s ease, color 0.2s ease;
  }

  .close-btn:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .picker-grid {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .tile-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    justify-content: center;
  }

  .tile-row.honors {
    margin-top: 0.25rem;
  }

  .tile-btn {
    background: none;
    border: 2px solid transparent;
    border-radius: 4px;
    padding: 2px;
    cursor: pointer;
    transition: border-color 0.15s ease, transform 0.1s ease;
  }

  .tile-btn:hover:not(:disabled) {
    border-color: var(--accent);
    transform: translateY(-2px);
  }

  .tile-btn:active:not(:disabled) {
    transform: translateY(0);
  }

  .tile-btn.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (max-width: 480px) {
    .picker-panel {
      padding: 0.75rem;
    }

    .tile-row {
      gap: 0.125rem;
    }
  }
</style>
