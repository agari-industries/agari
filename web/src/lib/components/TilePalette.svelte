<script lang="ts">
  import { ALL_TILES } from '../agari';
  import Tile from './Tile.svelte';

  interface Props {
    /** Callback when a tile is selected */
    onSelect: (tile: string, isRed?: boolean) => void;
    /** Map of tile code to count remaining (4 - tiles in hand) */
    tileCounts?: Record<string, number>;
    /** Whether to show red five buttons */
    showRedFives?: boolean;
    /** Disabled tiles */
    disabledTiles?: Set<string>;
  }

  let {
    onSelect,
    tileCounts = {},
    showRedFives = true,
    disabledTiles = new Set(),
  }: Props = $props();

  // Group tiles by suit
  const manTiles = ALL_TILES.filter((t) => t.endsWith('m'));
  const pinTiles = ALL_TILES.filter((t) => t.endsWith('p'));
  const souTiles = ALL_TILES.filter((t) => t.endsWith('s'));
  const honorTiles = ALL_TILES.filter((t) => t.endsWith('z'));

  // Get count for a tile (default 4 if not tracked)
  const getCount = (tile: string, isRed: boolean = false): number => {
    // For red fives, use the separate red5 counts (max 1 each)
    if (isRed) {
      const redKey = `red${tile}` as keyof typeof tileCounts;
      return tileCounts[redKey] ?? 1;
    }
    return tileCounts[tile] ?? 4;
  };

  // Check if tile is disabled
  const isDisabled = (tile: string): boolean => {
    return disabledTiles.has(tile) || getCount(tile) <= 0;
  };

  // Handle tile click
  const handleClick = (tile: string, isRed: boolean = false) => {
    if (!isDisabled(tile)) {
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
          disabled={getCount('5m', true) <= 0}
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
          disabled={getCount('5p', true) <= 0}
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
          disabled={getCount('5s', true) <= 0}
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
    <div class="tile-group honors">
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
    gap: 0.75rem;
  }

  .tile-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .suit-label {
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
    font-weight: bold;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .suit-label.man {
    background: rgba(196, 30, 58, 0.2);
    color: #c41e3a;
  }

  .suit-label.pin {
    background: rgba(30, 144, 255, 0.2);
    color: #1e90ff;
  }

  .suit-label.sou {
    background: rgba(34, 139, 34, 0.2);
    color: #228b22;
  }

  .suit-label.honor {
    background: rgba(47, 47, 47, 0.2);
    color: #a0a0a0;
  }

  .tile-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }

  .tile-group.honors {
    gap: 0.5rem;
  }

  @media (max-width: 768px) {
    .suit-label {
      width: 1.5rem;
      height: 1.5rem;
      font-size: 1rem;
    }

    .tile-row {
      gap: 0.5rem;
    }

    .tile-group {
      gap: 0.125rem;
    }
  }
</style>
