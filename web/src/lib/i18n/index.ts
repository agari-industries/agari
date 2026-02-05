/**
 * Internationalization (i18n) module for Agari WebUI
 *
 * Uses Svelte writable store for reactivity and localStorage for persistence.
 */

import { writable, derived, get } from "svelte/store";
import type { Locale, Translations } from "./types";
import { en } from "./en";
import { ja } from "./ja";

// Storage key for locale preference
const LOCALE_STORAGE_KEY = "agari-locale";

// Available translations
const translations: Record<Locale, Translations> = { en, ja };

// Available locales with display names
export const availableLocales: {
  code: Locale;
  name: string;
  nativeName: string;
}[] = [
  { code: "en", name: "English", nativeName: "English" },
  { code: "ja", name: "Japanese", nativeName: "日本語" },
];

/**
 * Get the initial locale from localStorage or default to 'en'
 */
function getInitialLocale(): Locale {
  if (typeof window === "undefined") return "en";

  const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
  if (stored && (stored === "en" || stored === "ja")) {
    return stored;
  }

  // Optionally detect browser language
  const browserLang = navigator.language.split("-")[0];
  if (browserLang === "ja") {
    return "ja";
  }

  return "en";
}

// Create the locale store
function createLocaleStore() {
  const { subscribe, set, update } = writable<Locale>(getInitialLocale());

  return {
    subscribe,
    set: (newLocale: Locale) => {
      if (typeof window !== "undefined") {
        localStorage.setItem(LOCALE_STORAGE_KEY, newLocale);
      }
      set(newLocale);
    },
    update,
  };
}

// Export the locale store
export const locale = createLocaleStore();

// Derived store for translations
export const t = derived(locale, ($locale) => translations[$locale]);

// Helper object for non-reactive access (use stores in components instead)
export const i18n = {
  get locale(): Locale {
    return get(locale);
  },
  set locale(newLocale: Locale) {
    locale.set(newLocale);
  },
  get t(): Translations {
    return get(t);
  },
};

// Re-export types
export type { Locale, Translations };

// Helper to get translated wind names
export function getWindNames(localeCode: Locale = get(locale)) {
  const trans = translations[localeCode];
  return {
    east: trans.windEast,
    south: trans.windSouth,
    west: trans.windWest,
    north: trans.windNorth,
  } as const;
}
