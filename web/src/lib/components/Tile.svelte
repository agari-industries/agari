<script lang="ts">
  // Import all tile SVGs statically
  import Man1 from '../assets/tiles/Man1.svg';
  import Man2 from '../assets/tiles/Man2.svg';
  import Man3 from '../assets/tiles/Man3.svg';
  import Man4 from '../assets/tiles/Man4.svg';
  import Man5 from '../assets/tiles/Man5.svg';
  import Man5Dora from '../assets/tiles/Man5-Dora.svg';
  import Man6 from '../assets/tiles/Man6.svg';
  import Man7 from '../assets/tiles/Man7.svg';
  import Man8 from '../assets/tiles/Man8.svg';
  import Man9 from '../assets/tiles/Man9.svg';

  import Pin1 from '../assets/tiles/Pin1.svg';
  import Pin2 from '../assets/tiles/Pin2.svg';
  import Pin3 from '../assets/tiles/Pin3.svg';
  import Pin4 from '../assets/tiles/Pin4.svg';
  import Pin5 from '../assets/tiles/Pin5.svg';
  import Pin5Dora from '../assets/tiles/Pin5-Dora.svg';
  import Pin6 from '../assets/tiles/Pin6.svg';
  import Pin7 from '../assets/tiles/Pin7.svg';
  import Pin8 from '../assets/tiles/Pin8.svg';
  import Pin9 from '../assets/tiles/Pin9.svg';

  import Sou1 from '../assets/tiles/Sou1.svg';
  import Sou2 from '../assets/tiles/Sou2.svg';
  import Sou3 from '../assets/tiles/Sou3.svg';
  import Sou4 from '../assets/tiles/Sou4.svg';
  import Sou5 from '../assets/tiles/Sou5.svg';
  import Sou5Dora from '../assets/tiles/Sou5-Dora.svg';
  import Sou6 from '../assets/tiles/Sou6.svg';
  import Sou7 from '../assets/tiles/Sou7.svg';
  import Sou8 from '../assets/tiles/Sou8.svg';
  import Sou9 from '../assets/tiles/Sou9.svg';

  import Ton from '../assets/tiles/Ton.svg';
  import Nan from '../assets/tiles/Nan.svg';
  import Shaa from '../assets/tiles/Shaa.svg';
  import Pei from '../assets/tiles/Pei.svg';
  import Haku from '../assets/tiles/Haku.svg';
  import Hatsu from '../assets/tiles/Hatsu.svg';
  import Chun from '../assets/tiles/Chun.svg';

  import Back from '../assets/tiles/Back.svg';

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

  // Map of tile codes to imported SVG paths
  const tileMap: Record<string, string> = {
    // Man (Characters)
    '1m': Man1,
    '2m': Man2,
    '3m': Man3,
    '4m': Man4,
    '5m': Man5,
    '6m': Man6,
    '7m': Man7,
    '8m': Man8,
    '9m': Man9,
    // Pin (Dots)
    '1p': Pin1,
    '2p': Pin2,
    '3p': Pin3,
    '4p': Pin4,
    '5p': Pin5,
    '6p': Pin6,
    '7p': Pin7,
    '8p': Pin8,
    '9p': Pin9,
    // Sou (Bamboo)
    '1s': Sou1,
    '2s': Sou2,
    '3s': Sou3,
    '4s': Sou4,
    '5s': Sou5,
    '6s': Sou6,
    '7s': Sou7,
    '8s': Sou8,
    '9s': Sou9,
    // Honors - Winds
    '1z': Ton,   // East
    '2z': Nan,   // South
    '3z': Shaa,  // West
    '4z': Pei,   // North
    // Honors - Dragons
    '5z': Haku,  // White
    '6z': Hatsu, // Green
    '7z': Chun,  // Red
    // Red fives (aka dora)
    '0m': Man5Dora,
    '0p': Pin5Dora,
    '0s': Sou5Dora,
    // Back tile
    'back': Back,
  };

  // Check if tile is a red five based on notation (0m, 0p, 0s) or red prop
  const isRedFive = (t: string): boolean => {
    return t[0] === '0' && (t.endsWith('m') || t.endsWith('p') || t.endsWith('s'));
  };

  // Get the SVG path for a tile
  const getTileSvg = (t: string, isRed: boolean): string => {
    // Handle 0-notation for red fives
    if (isRedFive(t)) {
      return tileMap[t] || tileMap['back'];
    }

    // If red prop is set and it's a 5, use the dora version
    if (isRed && t[0] === '5') {
      const suit = t[1];
      return tileMap[`0${suit}`] || tileMap[t] || tileMap['back'];
    }

    return tileMap[t] || tileMap['back'];
  };

  const sizeClasses = {
    sm: 'tile-sm',
    md: 'tile-md',
    lg: 'tile-lg',
  };

  const tileSvg = $derived(getTileSvg(tile, red));
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
  <img src={tileSvg} alt="Mahjong tile {tile}" class="tile-image" draggable="false" />

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
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .tile-image {
    width: 100%;
    height: 100%;
    object-fit: contain;
    pointer-events: none;
  }

  /* Size variants - maintaining roughly 5:7 aspect ratio */
  .tile-sm {
    width: 28px;
    height: 40px;
  }

  .tile-md {
    width: 40px;
    height: 56px;
  }

  .tile-lg {
    width: 52px;
    height: 72px;
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
