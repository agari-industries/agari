<script lang="ts">
  import { tileToUnicode } from '../agari';

  interface Props {
    tile: string;
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    selected?: boolean;
    showCount?: boolean;
    count?: number;
    red?: boolean;
    onclick?: () => void;
  }

  let {
    tile,
    size = 'md',
    disabled = false,
    selected = false,
    showCount = false,
    count = 4,
    red = false,
    onclick,
  }: Props = $props();

  // Determine tile suit for coloring
  const getSuitColor = (t: string): string => {
    if (t.endsWith('m')) return '#c41e3a'; // Man - red
    if (t.endsWith('p')) return '#1e90ff'; // Pin - blue
    if (t.endsWith('s')) return '#228b22'; // Sou - green
    if (t.endsWith('z')) {
      const num = parseInt(t[0]);
      if (num >= 5) {
        // Dragons
        if (num === 5) return '#666666'; // White
        if (num === 6) return '#228b22'; // Green
        if (num === 7) return '#c41e3a'; // Red
      }
      return '#2f2f2f'; // Winds - black
    }
    return '#2f2f2f';
  };

  // Get tile value display
  const getTileValue = (t: string): string => {
    if (t.endsWith('z')) {
      const num = parseInt(t[0]);
      const honors = ['東', '南', '西', '北', '白', '發', '中'];
      return honors[num - 1] || '';
    }
    // Red five (0) displays as 5
    if (t[0] === '0') return '5';
    return t[0];
  };

  // Check if tile is a red five (0m, 0p, 0s notation)
  const isRedFive = (t: string): boolean => {
    return t[0] === '0' && (t.endsWith('m') || t.endsWith('p') || t.endsWith('s'));
  };

  // Get suit symbol
  const getSuitSymbol = (t: string): string | null => {
    if (t.endsWith('m')) return '萬';
    if (t.endsWith('p')) return '●';
    if (t.endsWith('s')) return '竹';
    return null;
  };

  const sizeClasses = {
    sm: 'w-7 h-10 text-sm',
    md: 'w-10 h-14 text-lg',
    lg: 'w-12 h-16 text-xl',
  };

  // Determine if this tile should be rendered as red (either explicit red prop or 0-notation)
  const isRed = $derived(red || isRedFive(tile));
  const color = $derived(isRed ? '#c41e3a' : getSuitColor(tile));
  const value = $derived(getTileValue(tile));
  const suitSymbol = $derived(getSuitSymbol(tile));
  const isHonor = $derived(tile.endsWith('z'));
</script>

<button
  type="button"
  class="tile-button {sizeClasses[size]}"
  class:disabled
  class:selected
  class:clickable={!!onclick && !disabled}
  {disabled}
  onclick={onclick}
  aria-label="Tile {tile}"
>
  <svg viewBox="0 0 40 56" class="w-full h-full">
    <!-- Tile background with 3D effect -->
    <defs>
      <linearGradient id="tileGradient-{tile}" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" style="stop-color:#fff8e7" />
        <stop offset="100%" style="stop-color:#f5f0e6" />
      </linearGradient>
      <filter id="tileShadow" x="-20%" y="-20%" width="140%" height="140%">
        <feDropShadow dx="0" dy="2" stdDeviation="1" flood-color="#5c4a3a" flood-opacity="0.5" />
      </filter>
    </defs>

    <!-- Main tile body -->
    <rect
      x="2"
      y="2"
      width="36"
      height="50"
      rx="4"
      fill="url(#tileGradient-{tile})"
      stroke="#8b7355"
      stroke-width="2"
      filter="url(#tileShadow)"
    />

    <!-- Inner border for depth -->
    <rect
      x="4"
      y="4"
      width="32"
      height="46"
      rx="3"
      fill="none"
      stroke="#d4c4a8"
      stroke-width="1"
    />

    <!-- Tile content -->
    {#if isHonor}
      <!-- Honor tile - single large character -->
      <text
        x="20"
        y="34"
        text-anchor="middle"
        dominant-baseline="middle"
        fill={color}
        font-size="22"
        font-weight="bold"
        font-family="serif"
      >
        {value}
      </text>
    {:else}
      <!-- Suited tile - number and suit -->
      <text
        x="20"
        y="22"
        text-anchor="middle"
        dominant-baseline="middle"
        fill={color}
        font-size="18"
        font-weight="bold"
      >
        {value}
      </text>

      {#if suitSymbol}
        {#if tile.endsWith('p')}
          <!-- Pin - circles pattern -->
          <g fill={color}>
            {#if tile[0] === '1' || isRed}
              <circle cx="20" cy="40" r="5" />
            {:else if tile[0] === '2'}
              <circle cx="15" cy="40" r="4" />
              <circle cx="25" cy="40" r="4" />
            {:else if tile[0] === '3'}
              <circle cx="20" cy="36" r="3.5" />
              <circle cx="14" cy="44" r="3.5" />
              <circle cx="26" cy="44" r="3.5" />
            {:else}
              <text x="20" y="44" text-anchor="middle" font-size="12">{suitSymbol}</text>
            {/if}
          </g>
        {:else if tile.endsWith('m')}
          <!-- Man - 萬 character -->
          <text
            x="20"
            y="44"
            text-anchor="middle"
            dominant-baseline="middle"
            fill={color}
            font-size="12"
            font-family="serif"
          >
            {suitSymbol}
          </text>
        {:else if tile.endsWith('s')}
          <!-- Sou - bamboo lines -->
          <g stroke={color} stroke-width="2" stroke-linecap="round">
            {#if tile[0] === '1' || isRed}
              <line x1="20" y1="35" x2="20" y2="48" />
              <circle cx="20" cy="35" r="3" fill={color} />
            {:else}
              <line x1="16" y1="36" x2="16" y2="46" />
              <line x1="24" y1="36" x2="24" y2="46" />
            {/if}
          </g>
        {/if}
      {/if}
    {/if}

    <!-- Red five indicator -->
    {#if isRed}
      <circle cx="32" cy="8" r="4" fill="#c41e3a" />
    {/if}
  </svg>

  <!-- Count badge -->
  {#if showCount}
    <span class="count-badge" class:zero={count === 0}>
      {count}
    </span>
  {/if}
</button>

<style>
  .tile-button {
    position: relative;
    border: none;
    background: transparent;
    padding: 0;
    cursor: default;
    transition: transform 0.1s ease, filter 0.1s ease;
  }

  .tile-button.clickable {
    cursor: pointer;
  }

  .tile-button.clickable:hover:not(.disabled) {
    transform: translateY(-3px);
    filter: brightness(1.05);
  }

  .tile-button.clickable:active:not(.disabled) {
    transform: translateY(1px);
  }

  .tile-button.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .tile-button.selected {
    filter: drop-shadow(0 0 8px rgba(233, 69, 96, 0.7));
  }

  .tile-button.selected::after {
    content: '';
    position: absolute;
    inset: -2px;
    border: 2px solid #e94560;
    border-radius: 6px;
    pointer-events: none;
  }

  .count-badge {
    position: absolute;
    bottom: -4px;
    right: -4px;
    background: #0f3460;
    color: #eaeaea;
    font-size: 10px;
    font-weight: bold;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid #8b7355;
  }

  .count-badge.zero {
    background: #e94560;
  }
</style>
