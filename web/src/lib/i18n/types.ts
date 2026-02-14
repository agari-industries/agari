/**
 * Type definitions for i18n translations
 * All translation keys are defined here for type safety
 */

export type Locale = "en" | "ja";

export interface Translations {
  // Header
  tagline: string;

  // Loading/Error states
  loadingCalculator: string;
  failedToLoad: string;

  // Hand builder
  buildYourHand: string;
  clear: string;
  addMeld: string;
  chi: string;
  pon: string;
  openKan: string;
  closedKan: string;

  // Meld builder
  buildingChi: string;
  buildingPon: string;
  buildingOpenKan: string;
  buildingClosedKan: string;
  hintChi: string;
  hintPon: string;
  hintKan: string;
  cancel: string;
  confirmAddMeld: string;

  // Melds display
  calledMelds: string;

  // Hand display
  yourHand: string;
  selectWinningTileHint: string;
  winBadge: string;

  // Shanten
  complete: string;
  tenpai: string;
  shanten: string;

  // Dora section
  doraIndicators: string;
  dora: string;
  uraDora: string;
  addButton: string;
  akadoraInHand: string;

  // Results
  results: string;
  calculating: string;
  inferredWinningTile: string;
  han: string;
  fu: string;
  pts: string;
  all: string;
  dealer: string;
  dealerOya: string;
  yaku: string;
  ura: string;
  aka: string;
  fuBreakdown: string;
  fuBase: string;
  fuMenzenRon: string;
  fuTsumo: string;
  fuMelds: string;
  fuPair: string;
  fuWait: string;
  fuTotal: string;
  structure: string;
  enterCompleteHand: string;

  // Options
  options: string;
  winType: string;
  ron: string;
  tsumo: string;
  winds: string;
  round: string;
  seat: string;
  riichi: string;
  openHandNotice: string;
  doubleRiichi: string;
  ippatsu: string;
  situational: string;
  haitei: string;
  houtei: string;
  rinshanKaihou: string;
  chankan: string;
  firstTurnYakuman: string;
  tenhou: string;
  chiihou: string;

  // Calculate button
  calculateScore: string;

  // Footer
  footerPoweredBy: string;
  footerDescription: string;

  // Tile picker
  selectTile: string;

  // Wind names (with kanji for Japanese)
  windEast: string;
  windSouth: string;
  windWest: string;
  windNorth: string;

  // Language
  language: string;

  // Share
  share: string;
  copiedToClipboard: string;

  // Tile Theme
  tileTheme: string;
  tileThemeLight: string;
  tileThemeDark: string;

  // Score levels
  scoreLevelMangan: string;
  scoreLevelHaneman: string;
  scoreLevelBaiman: string;
  scoreLevelSanbaiman: string;
  scoreLevelYakuman: string;
  scoreLevelDoubleYakuman: string;
  scoreLevelCountedYakuman: string;

  // Yaku names
  yakuRiichi: string;
  yakuIppatsu: string;
  yakuMenzenTsumo: string;
  yakuTanyao: string;
  yakuPinfu: string;
  yakuIipeikou: string;
  yakuYakuhaiEast: string;
  yakuYakuhaiSouth: string;
  yakuYakuhaiWest: string;
  yakuYakuhaiNorth: string;
  yakuYakuhaiWhite: string;
  yakuYakuhaiGreen: string;
  yakuYakuhaiRed: string;
  yakuRinshanKaihou: string;
  yakuChankan: string;
  yakuHaitei: string;
  yakuHoutei: string;
  yakuDoubleRiichi: string;
  yakuToitoi: string;
  yakuSanshokuDoujun: string;
  yakuSanshokuDoukou: string;
  yakuIttsu: string;
  yakuChiitoitsu: string;
  yakuChanta: string;
  yakuSanAnkou: string;
  yakuSanKantsu: string;
  yakuHonroutou: string;
  yakuShousangen: string;
  yakuHonitsu: string;
  yakuJunchan: string;
  yakuRyanpeikou: string;
  yakuChinitsu: string;
  yakuTenhou: string;
  yakuChiihou: string;
  yakuKokushiMusou: string;
  yakuSuuankou: string;
  yakuDaisangen: string;
  yakuShousuushii: string;
  yakuDaisuushii: string;
  yakuTsuuiisou: string;
  yakuChinroutou: string;
  yakuRyuuiisou: string;
  yakuChuurenPoutou: string;
  yakuKokushi13Wait: string;
  yakuSuuankouTanki: string;
  yakuJunseiChuurenPoutou: string;
  yakuSuuKantsu: string;
}
