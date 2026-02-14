/**
 * URL state serialization/deserialization for shareable links.
 *
 * Encodes the calculator state into query parameters using the agari-core
 * terminal input syntax for the hand notation, and short keys for options.
 */

// ============================================================================
// Types (mirrors App.svelte internal types)
// ============================================================================

export interface TileEntry {
  tile: string;
  isRed?: boolean;
  id: number;
}

export interface Meld {
  type: "chi" | "pon" | "kan" | "ankan";
  tiles: TileEntry[];
  id: number;
}

export type Wind = "east" | "south" | "west" | "north";

export interface AppState {
  handTiles: TileEntry[];
  melds: Meld[];
  winningTile?: string;
  doraIndicators: TileEntry[];
  uraDoraIndicators: TileEntry[];
  isTsumo: boolean;
  isRiichi: boolean;
  isDoubleRiichi: boolean;
  isIppatsu: boolean;
  roundWind: Wind;
  seatWind: Wind;
  isLastTile: boolean;
  isRinshan: boolean;
  isChankan: boolean;
  isTenhou: boolean;
  isChiihou: boolean;
}

// ============================================================================
// Wind encoding helpers
// ============================================================================

const WIND_TO_CODE: Record<Wind, string> = {
  east: "e",
  south: "s",
  west: "w",
  north: "n",
};

const CODE_TO_WIND: Record<string, Wind> = {
  e: "east",
  s: "south",
  w: "west",
  n: "north",
};

// ============================================================================
// Serialization (state → URL)
// ============================================================================

function buildHandString(tiles: TileEntry[]): string {
  if (tiles.length === 0) return "";

  const groups: Record<string, string[]> = { m: [], p: [], s: [], z: [] };
  for (const entry of tiles) {
    const suit = entry.tile[1];
    const value = entry.isRed ? "0" : entry.tile[0];
    if (groups[suit]) {
      groups[suit].push(value);
    }
  }

  let result = "";
  for (const [suit, values] of Object.entries(groups)) {
    if (values.length > 0) {
      result += values.join("") + suit;
    }
  }
  return result;
}

function buildMeldNotation(melds: Meld[]): string {
  let meldStr = "";
  for (const meld of melds) {
    const tiles = meld.tiles.map((t) => (t.isRed ? "0" : t.tile[0])).join("");
    const suit = meld.tiles[0].tile[1];
    if (meld.type === "ankan") {
      meldStr += `[${tiles}${suit}]`;
    } else {
      meldStr += `(${tiles}${suit})`;
    }
  }
  return meldStr;
}

function buildTileList(tiles: TileEntry[]): string {
  return tiles.map((t) => t.tile).join(",");
}

export function serializeToUrl(state: AppState): string {
  const params = new URLSearchParams();

  // Hand + melds
  const hand = buildHandString(state.handTiles) + buildMeldNotation(state.melds);
  if (hand) params.set("h", hand);

  // Winning tile
  if (state.winningTile) params.set("w", state.winningTile);

  // Dora
  if (state.doraIndicators.length > 0) {
    params.set("d", buildTileList(state.doraIndicators));
  }
  if (state.uraDoraIndicators.length > 0) {
    params.set("u", buildTileList(state.uraDoraIndicators));
  }

  // Winds (only if non-default)
  if (state.roundWind !== "east") params.set("rw", WIND_TO_CODE[state.roundWind]);
  if (state.seatWind !== "east") params.set("sw", WIND_TO_CODE[state.seatWind]);

  // Boolean flags (presence = true)
  if (state.isTsumo) params.set("t", "");
  if (state.isRiichi) params.set("ri", "");
  if (state.isDoubleRiichi) params.set("dri", "");
  if (state.isIppatsu) params.set("ip", "");
  if (state.isLastTile) params.set("lt", "");
  if (state.isRinshan) params.set("rs", "");
  if (state.isChankan) params.set("ck", "");
  if (state.isTenhou) params.set("th", "");
  if (state.isChiihou) params.set("ch", "");

  const qs = params.toString();
  const base = window.location.origin + window.location.pathname;
  return qs ? `${base}?${qs}` : base;
}

// ============================================================================
// Deserialization (URL → state)
// ============================================================================

export function deserializeFromUrl(): AppState | null {
  const params = new URLSearchParams(window.location.search);

  const hand = params.get("h");
  if (!hand) return null;

  let idCounter = 0;
  const nextId = () => idCounter++;

  // Parse hand notation
  const parsed = parseHandNotation(hand, nextId);

  // Parse winning tile
  const winningTile = params.get("w") || undefined;

  // Parse dora indicators
  const doraIndicators = parseTileList(params.get("d"), nextId);
  const uraDoraIndicators = parseTileList(params.get("u"), nextId);

  // Parse winds
  const roundWind = CODE_TO_WIND[params.get("rw") || ""] || "east";
  const seatWind = CODE_TO_WIND[params.get("sw") || ""] || "east";

  // Parse boolean flags
  const hasFlag = (key: string) => params.has(key);

  return {
    handTiles: parsed.tiles,
    melds: parsed.melds,
    winningTile,
    doraIndicators,
    uraDoraIndicators,
    isTsumo: hasFlag("t"),
    isRiichi: hasFlag("ri"),
    isDoubleRiichi: hasFlag("dri"),
    isIppatsu: hasFlag("ip"),
    roundWind,
    seatWind,
    isLastTile: hasFlag("lt"),
    isRinshan: hasFlag("rs"),
    isChankan: hasFlag("ck"),
    isTenhou: hasFlag("th"),
    isChiihou: hasFlag("ch"),
  };
}

// ============================================================================
// Hand notation parser
// ============================================================================

interface ParsedHand {
  tiles: TileEntry[];
  melds: Meld[];
}

/**
 * Parse agari hand notation string into TileEntry[] and Meld[].
 *
 * Only handles numeric notation (e.g. `123m456p789s11z`) since the web UI
 * only produces this subset (not letter-based honor notation like `e`/`wh`).
 *
 * Melds: `(123m)` = open, `[1111m]` = closed kan
 */
export function parseHandNotation(
  notation: string,
  nextId: () => number,
): ParsedHand {
  const tiles: TileEntry[] = [];
  const melds: Meld[] = [];
  let meldIdCounter = 0;

  // Extract melds first, then parse remaining as free tiles
  let remaining = "";
  let i = 0;

  while (i < notation.length) {
    const ch = notation[i];

    if (ch === "(" || ch === "[") {
      const closeBracket = ch === "(" ? ")" : "]";
      const closeIdx = notation.indexOf(closeBracket, i + 1);
      if (closeIdx === -1) {
        // Malformed, skip
        i++;
        continue;
      }

      const isOpen = ch === "(";
      const meldContent = notation.substring(i + 1, closeIdx);
      const meldTiles = parseTileString(meldContent, nextId);

      if (meldTiles.length > 0) {
        const meldType = detectMeldType(meldTiles, isOpen);
        melds.push({
          type: meldType,
          tiles: meldTiles,
          id: meldIdCounter++,
        });
      }

      i = closeIdx + 1;
    } else {
      remaining += ch;
      i++;
    }
  }

  // Parse free tiles from remaining string
  tiles.push(...parseTileString(remaining, nextId));

  return { tiles, melds };
}

/**
 * Parse a tile string like "123m456p789s11z" into TileEntry[].
 * Handles red fives: digit 0 → { tile: "5X", isRed: true }
 */
function parseTileString(input: string, nextId: () => number): TileEntry[] {
  const entries: TileEntry[] = [];
  const pending: { digit: string; isRed: boolean }[] = [];

  for (const ch of input) {
    if (ch >= "0" && ch <= "9") {
      const isRed = ch === "0";
      pending.push({ digit: isRed ? "5" : ch, isRed });
    } else if (ch === "m" || ch === "p" || ch === "s" || ch === "z") {
      // Flush pending digits with this suit
      for (const p of pending) {
        entries.push({
          tile: `${p.digit}${ch}`,
          isRed: p.isRed || undefined,
          id: nextId(),
        });
      }
      pending.length = 0;
    }
    // Ignore any other characters (whitespace, etc.)
  }

  return entries;
}

/**
 * Detect meld type from parsed tiles and bracket type.
 */
function detectMeldType(
  tiles: TileEntry[],
  isOpen: boolean,
): "chi" | "pon" | "kan" | "ankan" {
  if (tiles.length === 4) {
    return isOpen ? "kan" : "ankan";
  }

  // 3 tiles: check if all same (pon) or sequential (chi)
  if (tiles.length === 3) {
    const allSame = tiles.every((t) => t.tile === tiles[0].tile);
    if (allSame) return "pon";
    return "chi";
  }

  // Fallback (shouldn't happen with valid notation)
  return "pon";
}

/**
 * Parse a comma-separated tile list (e.g. "1m,5z,0p") into TileEntry[].
 * Used for dora/ura dora indicators.
 */
function parseTileList(
  input: string | null,
  nextId: () => number,
): TileEntry[] {
  if (!input) return [];

  return input.split(",").map((t) => {
    const trimmed = t.trim();
    const isRed = trimmed.startsWith("0");
    return {
      tile: trimmed,
      isRed: isRed || undefined,
      id: nextId(),
    };
  });
}

/**
 * Clear URL query parameters without adding a history entry.
 */
export function clearUrlParams(): void {
  history.replaceState({}, "", window.location.pathname);
}
