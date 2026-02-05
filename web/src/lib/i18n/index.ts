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
// These must match exactly what agari-wasm/src/lib.rs yaku_name() returns
export const yakuNameMap: Record<string, keyof Translations> = {
  Riichi: "yakuRiichi",
  Ippatsu: "yakuIppatsu",
  "Menzen Tsumo": "yakuMenzenTsumo",
  Tanyao: "yakuTanyao",
  Pinfu: "yakuPinfu",
  Iipeikou: "yakuIipeikou",
  "Yakuhai (East)": "yakuYakuhaiEast",
  "Yakuhai (South)": "yakuYakuhaiSouth",
  "Yakuhai (West)": "yakuYakuhaiWest",
  "Yakuhai (North)": "yakuYakuhaiNorth",
  "Yakuhai (Haku)": "yakuYakuhaiWhite",
  "Yakuhai (Hatsu)": "yakuYakuhaiGreen",
  "Yakuhai (Chun)": "yakuYakuhaiRed",
  "Rinshan Kaihou": "yakuRinshanKaihou",
  Chankan: "yakuChankan",
  "Haitei Raoyue": "yakuHaitei",
  "Houtei Raoyui": "yakuHoutei",
  "Double Riichi": "yakuDoubleRiichi",
  Toitoi: "yakuToitoi",
  "Sanshoku Doujun": "yakuSanshokuDoujun",
  "Sanshoku Doukou": "yakuSanshokuDoukou",
  Ittsu: "yakuIttsu",
  Chiitoitsu: "yakuChiitoitsu",
  Chanta: "yakuChanta",
  "San Ankou": "yakuSanAnkou",
  "San Kantsu": "yakuSanKantsu",
  Honroutou: "yakuHonroutou",
  Shousangen: "yakuShousangen",
  Honitsu: "yakuHonitsu",
  Junchan: "yakuJunchan",
  Ryanpeikou: "yakuRyanpeikou",
  Chinitsu: "yakuChinitsu",
  Tenhou: "yakuTenhou",
  Chiihou: "yakuChiihou",
  "Kokushi Musou": "yakuKokushiMusou",
  Suuankou: "yakuSuuankou",
  Daisangen: "yakuDaisangen",
  Shousuushii: "yakuShousuushii",
  Daisuushii: "yakuDaisuushii",
  Tsuuiisou: "yakuTsuuiisou",
  Chinroutou: "yakuChinroutou",
  Ryuuiisou: "yakuRyuuiisou",
  "Chuuren Poutou": "yakuChuurenPoutou",
  "Kokushi 13-Wait": "yakuKokushi13Wait",
  "Suuankou Tanki": "yakuSuuankouTanki",
  "Junsei Chuuren Poutou": "yakuJunseiChuurenPoutou",
};

// Mapping from backend score level names to translation keys
export const scoreLevelMap: Record<string, keyof Translations> = {
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
 * @param backendName - The yaku name from the backend
 * @param trans - The translations object (pass $t for reactivity in Svelte components)
 */
export function translateYaku(
  backendName: string,
  trans: Translations,
): string {
  const key = yakuNameMap[backendName];
  if (key) {
    return trans[key] as string;
  }
  // Fallback to backend name if no translation found
  return backendName;
}

/**
 * Translate a score level from backend to current locale
 * @param backendLevel - The score level from the backend
 * @param trans - The translations object (pass $t for reactivity in Svelte components)
 */
export function translateScoreLevel(
  backendLevel: string,
  trans: Translations,
): string {
  const key = scoreLevelMap[backendLevel];
  if (key) {
    return trans[key] as string;
  }
  // Fallback to backend level if no translation found
  return backendLevel;
}
