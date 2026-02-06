/**
 * Tile theme store for Agari WebUI
 *
 * Uses Svelte writable store for reactivity and localStorage for persistence.
 * Allows switching between light (default) and dark tile themes.
 */

import { writable, get } from "svelte/store";

export type TileTheme = "light" | "dark";

// Storage key for tile theme preference
const TILE_THEME_STORAGE_KEY = "agari-tile-theme";

// Available tile themes with display info
export const availableTileThemes: {
  code: TileTheme;
  name: string;
}[] = [
  { code: "light", name: "Light" },
  { code: "dark", name: "Dark" },
];

/**
 * Get the initial tile theme from localStorage or default to 'light'
 */
function getInitialTileTheme(): TileTheme {
  if (typeof window === "undefined") return "light";

  const stored = localStorage.getItem(TILE_THEME_STORAGE_KEY);
  if (stored && (stored === "light" || stored === "dark")) {
    return stored;
  }

  return "dark";
}

// Create the tile theme store
function createTileThemeStore() {
  const { subscribe, set, update } = writable<TileTheme>(getInitialTileTheme());

  return {
    subscribe,
    set: (newTheme: TileTheme) => {
      if (typeof window !== "undefined") {
        localStorage.setItem(TILE_THEME_STORAGE_KEY, newTheme);
      }
      set(newTheme);
    },
    update,
  };
}

// Export the tile theme store
export const tileTheme = createTileThemeStore();

// Helper for non-reactive access
export function getTileTheme(): TileTheme {
  return get(tileTheme);
}
