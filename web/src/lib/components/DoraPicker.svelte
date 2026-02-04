<script lang="ts">
  import { ALL_TILES } from '../agari';
  import Tile from './Tile.svelte';

  interface Props {
    onSelect: (tile: string) => void;
    onClose: () => void;
    disabledTiles?: Set<string>;
  }

  let {
    onSelect,
    onClose,
    disabledTiles = new Set(),
  }: Props = $props();

  const manTiles = ALL_TILES.filter((t) => t.endsWith('m'));
  const pinTiles = ALL_TILES.filter((t) => t.endsWith('p'));
  const souTiles = ALL_TILES.filter((t) => t.endsWith('s'));
  const honorTiles = ALL_TILES.filter((t) => t.endsWith('z'));

  const isDisabled = (tile: string): boolean => disabledTiles.has(tile);

  const handleClick = (tile: string) => {
    if (!isDisabled(tile)) {
      onSelect(tile);
    }
  };

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

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
      <div class="tile-row">
        {#each manTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile {tile} size="sm" disabled={isDisabled(tile)} />
          </button>
        {/each}
        <button
          type="button"
          class="tile-btn"
          class:disabled={isDisabled('0m')}
          disabled={isDisabled('0m')}
          onclick={() => handleClick('0m')}
        >
          <Tile tile="5m" size="sm" red={true} disabled={isDisabled('0m')} />
        </button>
      </div>

      <div class="tile-row">
        {#each pinTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile {tile} size="sm" disabled={isDisabled(tile)} />
          </button>
        {/each}
        <button
          type="button"
          class="tile-btn"
          class:disabled={isDisabled('0p')}
          disabled={isDisabled('0p')}
          onclick={() => handleClick('0p')}
        >
          <Tile tile="5p" size="sm" red={true} disabled={isDisabled('0p')} />
        </button>
      </div>

      <div class="tile-row">
        {#each souTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile {tile} size="sm" disabled={isDisabled(tile)} />
          </button>
        {/each}
        <button
          type="button"
          class="tile-btn"
          class:disabled={isDisabled('0s')}
          disabled={isDisabled('0s')}
          onclick={() => handleClick('0s')}
        >
          <Tile tile="5s" size="sm" red={true} disabled={isDisabled('0s')} />
        </button>
      </div>

      <div class="tile-row">
        {#each honorTiles as tile}
          <button
            type="button"
            class="tile-btn"
            class:disabled={isDisabled(tile)}
            disabled={isDisabled(tile)}
            onclick={() => handleClick(tile)}
          >
            <Tile {tile} size="sm" disabled={isDisabled(tile)} />
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
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .picker-panel {
    background: var(--bg-surface);
    border: 1px solid var(--border);
    padding: var(--space-4);
    max-width: 90vw;
    max-height: 90vh;
    overflow: auto;
  }

  .picker-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border);
  }

  .picker-header span {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  .close-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    font-size: 1rem;
    color: var(--text-secondary);
    cursor: pointer;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: var(--bg-muted);
    border-color: var(--text-muted);
    color: var(--text-primary);
  }

  .picker-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .tile-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    justify-content: center;
  }

  .tile-btn {
    background: none;
    border: 1px solid transparent;
    padding: 1px;
    cursor: pointer;
    transition: border-color 0.1s ease;
  }

  .tile-btn:hover:not(:disabled) {
    border-color: var(--accent);
  }

  .tile-btn.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @media (max-width: 480px) {
    .picker-panel {
      padding: var(--space-3);
    }
  }
</style>
