/**
 * Internationalization (i18n) module for Agari WebUI
 *
 * Uses Svelte writable store for reactivity and localStorage for persistence.
 */

import { writable, derived, get } from "svelte/store";
import type { Locale, Translations } from "./types";
import { en } from "./en";
import { ja } from "./ja";

// Mapping from backend yaku names to translation keys
const yakuNameMap: Record<string, keyof Translations> = {
  Riichi: "yakuRiichi",
  Ippatsu: "yakuIppatsu",
  "Menzen Tsumo": "yakuMenzenTsumo",
  "Tanyao (All Simples)": "yakuTanyao",
  Pinfu: "yakuPinfu",
  "Iipeikou (Pure Double Sequence)": "yakuIipeikou",
  "Yakuhai: East Wind": "yakuYakuhaiEast",
  "Yakuhai: South Wind": "yakuYakuhaiSouth",
  "Yakuhai: West Wind": "yakuYakuhaiWest",
  "Yakuhai: North Wind": "yakuYakuhaiNorth",
  "Yakuhai: White Dragon (Haku)": "yakuYakuhaiWhite",
  "Yakuhai: Green Dragon (Hatsu)": "yakuYakuhaiGreen",
  "Yakuhai: Red Dragon (Chun)": "yakuYakuhaiRed",
  "Rinshan Kaihou (After Kan)": "yakuRinshanKaihou",
  "Chankan (Robbing the Kan)": "yakuChankan",
  "Haitei Raoyue (Last Tile Draw)": "yakuHaitei",
  "Houtei Raoyui (Last Tile Discard)": "yakuHoutei",
  "Double Riichi": "yakuDoubleRiichi",
  "Toitoi (All Triplets)": "yakuToitoi",
  "Sanshoku Doujun (Mixed Triple Sequence)": "yakuSanshokuDoujun",
  "Sanshoku Doukou (Triple Triplets)": "yakuSanshokuDoukou",
  "Ittsu (Pure Straight)": "yakuIttsu",
  "Chiitoitsu (Seven Pairs)": "yakuChiitoitsu",
  "Chanta (Outside Hand)": "yakuChanta",
  "San Ankou (Three Concealed Triplets)": "yakuSanAnkou",
  "San Kantsu (Three Kans)": "yakuSanKantsu",
  "Honroutou (All Terminals and Honors)": "yakuHonroutou",
  "Shousangen (Little Three Dragons)": "yakuShousangen",
  "Honitsu (Half Flush)": "yakuHonitsu",
  "Junchan (Terminals in All Groups)": "yakuJunchan",
  "Ryanpeikou (Twice Pure Double Sequence)": "yakuRyanpeikou",
  "Chinitsu (Full Flush)": "yakuChinitsu",
  "Tenhou (Heavenly Hand)": "yakuTenhou",
  "Chiihou (Earthly Hand)": "yakuChiihou",
  "Kokushi Musou (Thirteen Orphans)": "yakuKokushiMusou",
  "Suuankou (Four Concealed Triplets)": "yakuSuuankou",
  "Daisangen (Big Three Dragons)": "yakuDaisangen",
  "Shousuushii (Little Four Winds)": "yakuShousuushii",
  "Daisuushii (Big Four Winds)": "yakuDaisuushii",
  "Tsuuiisou (All Honors)": "yakuTsuuiisou",
  "Chinroutou (All Terminals)": "yakuChinroutou",
  "Ryuuiisou (All Green)": "yakuRyuuiisou",
  "Chuuren Poutou (Nine Gates)": "yakuChuurenPoutou",
  "Kokushi Juusanmen (Kokushi Musou 13-wait)": "yakuKokushi13Wait",
  "Suuankou Tanki": "yakuSuuankouTanki",
  "Junsei Chuuren Poutou": "yakuJunseiChuurenPoutou",
  "Suu Kantsu (Four Kans)": "yakuSuuKantsu",
};

// Mapping from backend score level names to translation keys
const scoreLevelMap: Record<string, keyof Translations> = {
  Mangan: "scoreLevelMangan",
  Haneman: "scoreLevelHaneman",
  Baiman: "scoreLevelBaiman",
  Sanbaiman: "scoreLevelSanbaiman",
  Yakuman: "scoreLevelYakuman",
  "Double Yakuman": "scoreLevelDoubleYakuman",
  "Counted Yakuman": "scoreLevelCountedYakuman",
};

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

/**
 * Translate a yaku name from backend to current locale
 */
export function translateYaku(backendName: string): string {
  const trans = get(t);
  const key = yakuNameMap[backendName];
  if (key) {
    return trans[key] as string;
  }
  // Fallback to backend name if no translation found
  return backendName;
}

/**
 * Translate a score level from backend to current locale
 */
export function translateScoreLevel(backendLevel: string): string {
  const trans = get(t);
  const key = scoreLevelMap[backendLevel];
  if (key) {
    return trans[key] as string;
  }
  // Fallback to backend level if no translation found
  return backendLevel;
}
