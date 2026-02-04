<script lang="ts">
  import { ALL_TILES } from '../agari';
  import Tile from './Tile.svelte';

  interface Props {
    onSelect: (tile: string, isRed?: boolean) => void;
    tileCounts?: Record<string, number>;
    showRedFives?: boolean;
    disabledTiles?: Set<string>;
  }

  let {
    onSelect,
    tileCounts = {},
    showRedFives = true,
    disabledTiles = new Set(),
  }: Props = $props();

  const manTiles = ALL_TILES.filter((t) => t.endsWith('m'));
  const pinTiles = ALL_TILES.filter((t) => t.endsWith('p'));
  const souTiles = ALL_TILES.filter((t) => t.endsWith('s'));
  const honorTiles = ALL_TILES.filter((t) => t.endsWith('z'));

  const getCount = (tile: string, isRed: boolean = false): number => {
    if (isRed) {
      const redKey = `red${tile}` as keyof typeof tileCounts;
      return tileCounts[redKey] ?? 1;
    }
    return tileCounts[tile] ?? 4;
  };

  const isDisabled = (tile: string, isRed: boolean = false): boolean => {
    if (disabledTiles.has(tile)) return true;
    return getCount(tile, isRed) <= 0;
  };

  const handleClick = (tile: string, isRed: boolean = false) => {
    if (!isDisabled(tile, isRed)) {
      onSelect(tile, isRed);
    }
  };
</script>

<div class="tile-palette">
  <!-- Man (Characters) -->
  <div class="tile-row">
    <span class="suit-label man">萬</span>
    <div class="tile-group">
      {#each manTiles as tile}
        <Tile
          {tile}
          size="md"
          disabled={isDisabled(tile)}
          showCount={true}
          count={getCount(tile)}
          onclick={() => handleClick(tile)}
        />
      {/each}
      {#if showRedFives}
        <Tile
          tile="5m"
          size="md"
          red={true}
          disabled={isDisabled('5m', true)}
          showCount={true}
          count={getCount('5m', true)}
          onclick={() => handleClick('5m', true)}
        />
      {/if}
    </div>
  </div>

  <!-- Pin (Dots) -->
  <div class="tile-row">
    <span class="suit-label pin">筒</span>
    <div class="tile-group">
      {#each pinTiles as tile}
        <Tile
          {tile}
          size="md"
          disabled={isDisabled(tile)}
          showCount={true}
          count={getCount(tile)}
          onclick={() => handleClick(tile)}
        />
      {/each}
      {#if showRedFives}
        <Tile
          tile="5p"
          size="md"
          red={true}
          disabled={isDisabled('5p', true)}
          showCount={true}
          count={getCount('5p', true)}
          onclick={() => handleClick('5p', true)}
        />
      {/if}
    </div>
  </div>

  <!-- Sou (Bamboo) -->
  <div class="tile-row">
    <span class="suit-label sou">索</span>
    <div class="tile-group">
      {#each souTiles as tile}
        <Tile
          {tile}
          size="md"
          disabled={isDisabled(tile)}
          showCount={true}
          count={getCount(tile)}
          onclick={() => handleClick(tile)}
        />
      {/each}
      {#if showRedFives}
        <Tile
          tile="5s"
          size="md"
          red={true}
          disabled={isDisabled('5s', true)}
          showCount={true}
          count={getCount('5s', true)}
          onclick={() => handleClick('5s', true)}
        />
      {/if}
    </div>
  </div>

  <!-- Honors -->
  <div class="tile-row">
    <span class="suit-label honor">字</span>
    <div class="tile-group">
      {#each honorTiles as tile}
        <Tile
          {tile}
          size="md"
          disabled={isDisabled(tile)}
          showCount={true}
          count={getCount(tile)}
          onclick={() => handleClick(tile)}
        />
      {/each}
    </div>
  </div>
</div>

<style>
  .tile-palette {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .tile-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .suit-label {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.875rem;
    font-weight: 600;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    flex-shrink: 0;
  }

  .suit-label.man {
    color: var(--man-color);
    border-color: var(--man-color);
    background: rgba(248, 113, 113, 0.1);
  }

  .suit-label.pin {
    color: var(--pin-color);
    border-color: var(--pin-color);
    background: rgba(96, 165, 250, 0.1);
  }

  .suit-label.sou {
    color: var(--sou-color);
    border-color: var(--sou-color);
    background: rgba(74, 222, 128, 0.1);
  }

  .suit-label.honor {
    color: var(--honor-color);
    border-color: var(--honor-color);
    background: rgba(161, 161, 170, 0.1);
  }

  .tile-group {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  @media (max-width: 768px) {
    .suit-label {
      width: 20px;
      height: 20px;
      font-size: 0.75rem;
    }

    .tile-row {
      gap: var(--space-2);
    }
  }
</style>
