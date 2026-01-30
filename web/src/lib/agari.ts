/**
 * Agari WASM bindings loader and TypeScript types
 */

// WASM module will be loaded dynamically
let wasmModule: typeof import('./wasm/agari_wasm') | null = null;
let initPromise: Promise<void> | null = null;

/**
 * Initialize the WASM module
 * Must be called before using any scoring functions
 */
export async function initAgari(): Promise<void> {
  if (wasmModule) return;

  if (!initPromise) {
    initPromise = (async () => {
      const wasm = await import('./wasm/agari_wasm');
      await wasm.default();
      wasmModule = wasm;
    })();
  }

  await initPromise;
}

/**
 * Check if WASM module is loaded
 */
export function isLoaded(): boolean {
  return wasmModule !== null;
}

// ============================================================================
// Type definitions
// ============================================================================

export interface ScoreRequest {
  hand: string;
  winning_tile?: string;
  is_tsumo: boolean;
  is_riichi: boolean;
  is_double_riichi: boolean;
  is_ippatsu: boolean;
  round_wind: 'east' | 'south' | 'west' | 'north';
  seat_wind: 'east' | 'south' | 'west' | 'north';
  dora_indicators: string[];
  ura_dora_indicators: string[];
  is_last_tile: boolean;
  is_rinshan: boolean;
  is_chankan: boolean;
  is_tenhou: boolean;
  is_chiihou: boolean;
}

export interface ScoreResponse {
  success: boolean;
  error?: string;
  result?: ScoringOutput;
}

export interface ScoringOutput {
  yaku: YakuInfo[];
  han: number;
  fu: number;
  dora: DoraInfo;
  total_han: number;
  score_level: string;
  payment: PaymentInfo;
  is_dealer: boolean;
  is_counted_yakuman: boolean;
  fu_breakdown: FuBreakdownInfo;
  hand_structure: string;
}

export interface YakuInfo {
  name: string;
  han: number;
  is_yakuman: boolean;
}

export interface DoraInfo {
  regular: number;
  ura: number;
  aka: number;
  total: number;
}

export interface PaymentInfo {
  total: number;
  from_discarder?: number;
  from_dealer?: number;
  from_non_dealer?: number;
}

export interface FuBreakdownInfo {
  base: number;
  menzen_ron: number;
  tsumo: number;
  melds: number;
  pair: number;
  wait: number;
  raw_total: number;
  rounded: number;
}

export interface ShantenResponse {
  success: boolean;
  error?: string;
  shanten?: number;
  best_type?: string;
  description?: string;
}

export interface UkeireResponse {
  success: boolean;
  error?: string;
  shanten?: number;
  tiles?: UkeireTileInfo[];
  total_count?: number;
}

export interface UkeireTileInfo {
  tile: string;
  available: number;
}

export interface ValidationResult {
  valid: boolean;
  error?: string;
}

// ============================================================================
// API functions
// ============================================================================

/**
 * Score a mahjong hand
 */
export function scoreHand(request: ScoreRequest): ScoreResponse {
  if (!wasmModule) {
    return { success: false, error: 'WASM module not loaded. Call initAgari() first.' };
  }
  return wasmModule.score_hand(request) as ScoreResponse;
}

/**
 * Calculate shanten for a hand
 */
export function calculateShanten(hand: string): ShantenResponse {
  if (!wasmModule) {
    return { success: false, error: 'WASM module not loaded. Call initAgari() first.' };
  }
  return wasmModule.calculate_shanten_js(hand) as ShantenResponse;
}

/**
 * Calculate ukeire (tile acceptance) for a hand
 */
export function calculateUkeire(hand: string): UkeireResponse {
  if (!wasmModule) {
    return { success: false, error: 'WASM module not loaded. Call initAgari() first.' };
  }
  return wasmModule.calculate_ukeire_js(hand) as UkeireResponse;
}

/**
 * Validate a hand string
 */
export function validateHand(hand: string): ValidationResult {
  if (!wasmModule) {
    return { valid: false, error: 'WASM module not loaded. Call initAgari() first.' };
  }
  return wasmModule.validate_hand(hand) as ValidationResult;
}

// ============================================================================
// Helper functions
// ============================================================================

/**
 * Create a default score request with reasonable defaults
 */
export function createDefaultRequest(hand: string): ScoreRequest {
  return {
    hand,
    is_tsumo: false,
    is_riichi: false,
    is_double_riichi: false,
    is_ippatsu: false,
    round_wind: 'east',
    seat_wind: 'east',
    dora_indicators: [],
    ura_dora_indicators: [],
    is_last_tile: false,
    is_rinshan: false,
    is_chankan: false,
    is_tenhou: false,
    is_chiihou: false,
  };
}

/**
 * Tile notation helper - converts a tile code to display format
 */
export function tileToDisplay(tile: string): string {
  // Already in display format
  return tile;
}

/**
 * Get Unicode character for a tile
 */
export function tileToUnicode(tile: string): string {
  const match = tile.match(/^(\d)([mps])$|^(\d)z$/);
  if (!match) return tile;

  if (match[1] && match[2]) {
    // Suited tile
    const value = parseInt(match[1]);
    const suit = match[2];
    const baseCode =
      suit === 'm' ? 0x1f007 : // Man
      suit === 'p' ? 0x1f019 : // Pin
      0x1f010; // Sou
    return String.fromCodePoint(baseCode + value - 1);
  } else if (match[3]) {
    // Honor tile
    const value = parseInt(match[3]);
    if (value <= 4) {
      // Winds: East=1, South=2, West=3, North=4
      return String.fromCodePoint(0x1f000 + value - 1);
    } else {
      // Dragons: White=5, Green=6, Red=7
      return String.fromCodePoint(0x1f004 + value - 5);
    }
  }

  return tile;
}

/**
 * All tile types in standard order
 */
export const ALL_TILES = [
  // Man (Characters)
  '1m', '2m', '3m', '4m', '5m', '6m', '7m', '8m', '9m',
  // Pin (Dots)
  '1p', '2p', '3p', '4p', '5p', '6p', '7p', '8p', '9p',
  // Sou (Bamboo)
  '1s', '2s', '3s', '4s', '5s', '6s', '7s', '8s', '9s',
  // Honors: Winds (East, South, West, North) + Dragons (White, Green, Red)
  '1z', '2z', '3z', '4z', '5z', '6z', '7z',
] as const;

export type TileCode = typeof ALL_TILES[number];

/**
 * Tile display names
 */
export const TILE_NAMES: Record<string, string> = {
  '1z': 'East',
  '2z': 'South',
  '3z': 'West',
  '4z': 'North',
  '5z': 'White',
  '6z': 'Green',
  '7z': 'Red',
};

/**
 * Wind names for display
 */
export const WIND_NAMES = {
  east: '東 East',
  south: '南 South',
  west: '西 West',
  north: '北 North',
} as const;
