<script lang="ts">
  import { tileTheme } from '../stores/tileTheme';

  // Import light tile SVGs
  import Man1Light from '../assets/tiles/Man1.svg';
  import Man2Light from '../assets/tiles/Man2.svg';
  import Man3Light from '../assets/tiles/Man3.svg';
  import Man4Light from '../assets/tiles/Man4.svg';
  import Man5Light from '../assets/tiles/Man5.svg';
  import Man5DoraLight from '../assets/tiles/Man5-Dora.svg';
  import Man6Light from '../assets/tiles/Man6.svg';
  import Man7Light from '../assets/tiles/Man7.svg';
  import Man8Light from '../assets/tiles/Man8.svg';
  import Man9Light from '../assets/tiles/Man9.svg';

  import Pin1Light from '../assets/tiles/Pin1.svg';
  import Pin2Light from '../assets/tiles/Pin2.svg';
  import Pin3Light from '../assets/tiles/Pin3.svg';
  import Pin4Light from '../assets/tiles/Pin4.svg';
  import Pin5Light from '../assets/tiles/Pin5.svg';
  import Pin5DoraLight from '../assets/tiles/Pin5-Dora.svg';
  import Pin6Light from '../assets/tiles/Pin6.svg';
  import Pin7Light from '../assets/tiles/Pin7.svg';
  import Pin8Light from '../assets/tiles/Pin8.svg';
  import Pin9Light from '../assets/tiles/Pin9.svg';

  import Sou1Light from '../assets/tiles/Sou1.svg';
  import Sou2Light from '../assets/tiles/Sou2.svg';
  import Sou3Light from '../assets/tiles/Sou3.svg';
  import Sou4Light from '../assets/tiles/Sou4.svg';
  import Sou5Light from '../assets/tiles/Sou5.svg';
  import Sou5DoraLight from '../assets/tiles/Sou5-Dora.svg';
  import Sou6Light from '../assets/tiles/Sou6.svg';
  import Sou7Light from '../assets/tiles/Sou7.svg';
  import Sou8Light from '../assets/tiles/Sou8.svg';
  import Sou9Light from '../assets/tiles/Sou9.svg';

  import TonLight from '../assets/tiles/Ton.svg';
  import NanLight from '../assets/tiles/Nan.svg';
  import ShaaLight from '../assets/tiles/Shaa.svg';
  import PeiLight from '../assets/tiles/Pei.svg';
  import HakuLight from '../assets/tiles/Haku.svg';
  import HatsuLight from '../assets/tiles/Hatsu.svg';
  import ChunLight from '../assets/tiles/Chun.svg';

  import BackLight from '../assets/tiles/Back.svg';

  // Import dark tile SVGs
  import Man1Dark from '../assets/tiles_dark/Man1.svg';
  import Man2Dark from '../assets/tiles_dark/Man2.svg';
  import Man3Dark from '../assets/tiles_dark/Man3.svg';
  import Man4Dark from '../assets/tiles_dark/Man4.svg';
  import Man5Dark from '../assets/tiles_dark/Man5.svg';
  import Man5DoraDark from '../assets/tiles_dark/Man5-Dora.svg';
  import Man6Dark from '../assets/tiles_dark/Man6.svg';
  import Man7Dark from '../assets/tiles_dark/Man7.svg';
  import Man8Dark from '../assets/tiles_dark/Man8.svg';
  import Man9Dark from '../assets/tiles_dark/Man9.svg';

  import Pin1Dark from '../assets/tiles_dark/Pin1.svg';
  import Pin2Dark from '../assets/tiles_dark/Pin2.svg';
  import Pin3Dark from '../assets/tiles_dark/Pin3.svg';
  import Pin4Dark from '../assets/tiles_dark/Pin4.svg';
  import Pin5Dark from '../assets/tiles_dark/Pin5.svg';
  import Pin5DoraDark from '../assets/tiles_dark/Pin5-Dora.svg';
  import Pin6Dark from '../assets/tiles_dark/Pin6.svg';
  import Pin7Dark from '../assets/tiles_dark/Pin7.svg';
  import Pin8Dark from '../assets/tiles_dark/Pin8.svg';
  import Pin9Dark from '../assets/tiles_dark/Pin9.svg';

  import Sou1Dark from '../assets/tiles_dark/Sou1.svg';
  import Sou2Dark from '../assets/tiles_dark/Sou2.svg';
  import Sou3Dark from '../assets/tiles_dark/Sou3.svg';
  import Sou4Dark from '../assets/tiles_dark/Sou4.svg';
  import Sou5Dark from '../assets/tiles_dark/Sou5.svg';
  import Sou5DoraDark from '../assets/tiles_dark/Sou5-Dora.svg';
  import Sou6Dark from '../assets/tiles_dark/Sou6.svg';
  import Sou7Dark from '../assets/tiles_dark/Sou7.svg';
  import Sou8Dark from '../assets/tiles_dark/Sou8.svg';
  import Sou9Dark from '../assets/tiles_dark/Sou9.svg';

  import TonDark from '../assets/tiles_dark/Ton.svg';
  import NanDark from '../assets/tiles_dark/Nan.svg';
  import ShaaDark from '../assets/tiles_dark/Shaa.svg';
  import PeiDark from '../assets/tiles_dark/Pei.svg';
  import HakuDark from '../assets/tiles_dark/Haku.svg';
  import HatsuDark from '../assets/tiles_dark/Hatsu.svg';
  import ChunDark from '../assets/tiles_dark/Chun.svg';

  import BackDark from '../assets/tiles_dark/Back.svg';

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

  // Map of tile codes to imported SVG paths for light theme
  const lightTileMap: Record<string, string> = {
    // Man (Characters)
    '1m': Man1Light, '2m': Man2Light, '3m': Man3Light, '4m': Man4Light, '5m': Man5Light,
    '6m': Man6Light, '7m': Man7Light, '8m': Man8Light, '9m': Man9Light,
    // Pin (Dots)
    '1p': Pin1Light, '2p': Pin2Light, '3p': Pin3Light, '4p': Pin4Light, '5p': Pin5Light,
    '6p': Pin6Light, '7p': Pin7Light, '8p': Pin8Light, '9p': Pin9Light,
    // Sou (Bamboo)
    '1s': Sou1Light, '2s': Sou2Light, '3s': Sou3Light, '4s': Sou4Light, '5s': Sou5Light,
    '6s': Sou6Light, '7s': Sou7Light, '8s': Sou8Light, '9s': Sou9Light,
    // Honors - Winds
    '1z': TonLight, '2z': NanLight, '3z': ShaaLight, '4z': PeiLight,
    // Honors - Dragons
    '5z': HakuLight, '6z': HatsuLight, '7z': ChunLight,
    // Red fives (aka dora)
    '0m': Man5DoraLight, '0p': Pin5DoraLight, '0s': Sou5DoraLight,
    // Back tile
    'back': BackLight,
  };

  // Map of tile codes to imported SVG paths for dark theme
  const darkTileMap: Record<string, string> = {
    // Man (Characters)
    '1m': Man1Dark, '2m': Man2Dark, '3m': Man3Dark, '4m': Man4Dark, '5m': Man5Dark,
    '6m': Man6Dark, '7m': Man7Dark, '8m': Man8Dark, '9m': Man9Dark,
    // Pin (Dots)
    '1p': Pin1Dark, '2p': Pin2Dark, '3p': Pin3Dark, '4p': Pin4Dark, '5p': Pin5Dark,
    '6p': Pin6Dark, '7p': Pin7Dark, '8p': Pin8Dark, '9p': Pin9Dark,
    // Sou (Bamboo)
    '1s': Sou1Dark, '2s': Sou2Dark, '3s': Sou3Dark, '4s': Sou4Dark, '5s': Sou5Dark,
    '6s': Sou6Dark, '7s': Sou7Dark, '8s': Sou8Dark, '9s': Sou9Dark,
    // Honors - Winds
    '1z': TonDark, '2z': NanDark, '3z': ShaaDark, '4z': PeiDark,
    // Honors - Dragons
    '5z': HakuDark, '6z': HatsuDark, '7z': ChunDark,
    // Red fives (aka dora)
    '0m': Man5DoraDark, '0p': Pin5DoraDark, '0s': Sou5DoraDark,
    // Back tile
    'back': BackDark,
  };

  const isRedFive = (t: string): boolean => {
    return t[0] === '0' && (t.endsWith('m') || t.endsWith('p') || t.endsWith('s'));
  };

  const getTileSvg = (t: string, isRed: boolean, theme: 'light' | 'dark'): string => {
    const tileMap = theme === 'dark' ? darkTileMap : lightTileMap;
    const backTile = tileMap['back'];

    if (isRedFive(t)) return tileMap[t] || backTile;
    if (isRed && t[0] === '5') {
      const suit = t[1];
      return tileMap[`0${suit}`] || tileMap[t] || backTile;
    }
    return tileMap[t] || backTile;
  };

  const sizeClasses = { sm: 'tile-sm', md: 'tile-md', lg: 'tile-lg' };
  const tileSvg = $derived(getTileSvg(tile, red, $tileTheme));
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
  {#if showCount}
    <span class="count-badge" class:zero={count === 0}>{count}</span>
  {/if}
</button>

<style>
  .tile-button {
    position: relative;
    border: 1px solid var(--border);
    background: var(--bg-elevated);
    padding: 2px;
    cursor: default;
    transition: border-color 0.1s ease, background 0.1s ease;
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
  .tile-sm { width: 28px; height: 40px; }
  .tile-md { width: 40px; height: 56px; }
  .tile-lg { width: 52px; height: 72px; }

  .tile-button.clickable {
    cursor: pointer;
  }

  .tile-button.clickable:hover:not(.disabled) {
    border-color: var(--accent);
    background: var(--accent-muted);
  }

  .tile-button.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .tile-button.selected {
    border-color: var(--accent);
    background: var(--accent-muted);
  }

  .count-badge {
    position: absolute;
    bottom: -3px;
    right: -3px;
    background: var(--bg-surface);
    color: var(--text-secondary);
    font-size: 9px;
    font-weight: 600;
    font-family: var(--font-mono);
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
  }

  .count-badge.zero {
    background: var(--error);
    border-color: var(--error);
    color: white;
  }
</style>
