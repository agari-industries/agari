<script lang="ts">
  import Tile from './Tile.svelte';

  interface TileEntry {
    tile: string;
    isRed?: boolean;
    id: number;
  }

  interface Props {
    /** Tiles currently in hand */
    tiles: TileEntry[];
    /** Callback when a tile is removed */
    onRemove: (index: number) => void;
    /** Maximum tiles allowed */
    maxTiles?: number;
    /** Label for the hand */
    label?: string;
  }

  let {
    tiles,
    onRemove,
    maxTiles = 14,
    label = 'Hand',
  }: Props = $props();

  const tileCount = $derived(tiles.length);
  const isEmpty = $derived(tiles.length === 0);
</script>

<div class="hand-display">
  <div class="hand-header">
    <span class="hand-label">{label}</span>
    <span class="tile-counter" class:full={tileCount >= maxTiles}>
      {tileCount} / {maxTiles}
    </span>
  </div>

  <div class="hand-area" class:empty={isEmpty}>
    {#if isEmpty}
      <div class="empty-message">
        Click tiles above to build your hand
      </div>
    {:else}
      <div class="tile-list">
        {#each tiles as entry, index (entry.id)}
          <div class="tile-wrapper">
            <Tile
              tile={entry.tile}
              size="md"
              red={entry.isRed}
              onclick={() => onRemove(index)}
            />
            <button
              type="button"
              class="remove-btn"
              onclick={() => onRemove(index)}
              aria-label="Remove tile"
            >
              ×
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  {#if tileCount > 0}
    <div class="hand-notation">
      <code>
        {#each tiles as entry}
          {entry.isRed ? '0' + entry.tile[1] : entry.tile}
        {/each}
      </code>
    </div>
  {/if}
</div>

<style>
  .hand-display {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .hand-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .hand-label {
    font-weight: 600;
    color: var(--text-primary);
  }

  .tile-counter {
    font-size: 0.875rem;
    color: var(--text-secondary);
    padding: 0.25rem 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
  }

  .tile-counter.full {
    background: var(--success);
    color: var(--bg-primary);
  }

  .hand-area {
    min-height: 80px;
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
    border: 2px dashed transparent;
    transition: border-color 0.2s ease;
  }

  .hand-area.empty {
    border-color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-message {
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .tile-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .tile-wrapper {
    position: relative;
  }

  .tile-wrapper:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--accent);
    color: white;
    border: none;
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .remove-btn:hover {
    background: var(--accent-hover);
  }

  .hand-notation {
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
  }

  .hand-notation code {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.875rem;
    color: var(--text-secondary);
    letter-spacing: 0.05em;
  }

  @media (max-width: 768px) {
    .hand-area {
      padding: 0.75rem;
    }

    .tile-list {
      gap: 0.125rem;
    }
  }
</style>
