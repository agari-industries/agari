import type { Translations } from "./types";

export const en: Translations = {
  // Header
  tagline: "Riichi Mahjong Calculator",

  // Loading/Error states
  loadingCalculator: "Loading calculator...",
  failedToLoad: "Failed to load calculator:",

  // Hand builder
  buildYourHand: "Build Your Hand",
  clear: "Clear",
  addMeld: "Add Meld:",
  chi: "Chi",
  pon: "Pon",
  openKan: "Open Kan",
  closedKan: "Closed Kan",

  // Meld builder
  buildingChi: "Building Chi",
  buildingPon: "Building Pon",
  buildingOpenKan: "Building Open Kan",
  buildingClosedKan: "Building Closed Kan",
  hintChi: "(select 3 sequential tiles of the same suit)",
  hintPon: "(select 3 of the same tile)",
  hintKan: "(select 4 of the same tile)",
  cancel: "Cancel",
  confirmAddMeld: "Add Meld",

  // Melds display
  calledMelds: "Called Melds",

  // Hand display
  yourHand: "Your Hand",
  selectWinningTileHint: "Click a tile to select it as winning tile",
  winBadge: "WIN",

  // Shanten
  complete: "Complete",
  tenpai: "Tenpai",
  shanten: "-shanten",

  // Dora section
  doraIndicators: "Dora Indicators",
  dora: "Dora",
  uraDora: "Ura Dora",
  addButton: "+ Add",
  akadoraInHand: "Aka Dora in hand:",

  // Results
  results: "Results",
  calculating: "Calculating...",
  inferredWinningTile: "Winning tile inferred as",
  han: "han",
  fu: "fu",
  pts: "pts",
  all: "all",
  dealer: "Dealer",
  dealerOya: "Dealer (Oya)",
  yaku: "Yaku",
  ura: "Ura",
  aka: "Aka",
  fuBreakdown: "Fu Breakdown",
  fuBase: "Base",
  fuMenzenRon: "Menzen Ron",
  fuTsumo: "Tsumo",
  fuMelds: "Melds",
  fuPair: "Pair",
  fuWait: "Wait",
  fuTotal: "Total",
  structure: "Structure:",
  enterCompleteHand: "Enter a complete hand to calculate score",

  // Options
  options: "Options",
  winType: "Win Type",
  ron: "Ron",
  tsumo: "Tsumo",
  winds: "Winds",
  round: "Round",
  seat: "Seat",
  riichi: "Riichi",
  openHandNotice: "Open hand — Riichi not available",
  doubleRiichi: "Double Riichi",
  ippatsu: "Ippatsu",
  situational: "Situational",
  haitei: "Haitei (Last Draw)",
  houtei: "Houtei (Last Discard)",
  rinshanKaihou: "Rinshan Kaihou",
  chankan: "Chankan",
  firstTurnYakuman: "First Turn Yakuman",
  tenhou: "Tenhou",
  chiihou: "Chiihou",

  // Calculate button
  calculateScore: "Calculate Score",

  // Footer
  footerPoweredBy: "Powered by",
  footerDescription: "— A Riichi Mahjong scoring engine written in Rust",

  // Tile picker
  selectTile: "Select Tile",

  // Wind names
  windEast: "東 East",
  windSouth: "南 South",
  windWest: "西 West",
  windNorth: "北 North",

  // Language
  language: "Language",

  // Tile Theme
  tileTheme: "Tiles",
  tileThemeLight: "Light",
  tileThemeDark: "Dark",

  // Score levels
  scoreLevelMangan: "Mangan",
  scoreLevelHaneman: "Haneman",
  scoreLevelBaiman: "Baiman",
  scoreLevelSanbaiman: "Sanbaiman",
  scoreLevelYakuman: "Yakuman",
  scoreLevelDoubleYakuman: "Double Yakuman",
  scoreLevelCountedYakuman: "Counted Yakuman",

  // Yaku names
  yakuRiichi: "Riichi",
  yakuIppatsu: "Ippatsu",
  yakuMenzenTsumo: "Menzen Tsumo",
  yakuTanyao: "Tanyao (All Simples)",
  yakuPinfu: "Pinfu",
  yakuIipeikou: "Iipeikou (Pure Double Sequence)",
  yakuYakuhaiEast: "Yakuhai: East Wind",
  yakuYakuhaiSouth: "Yakuhai: South Wind",
  yakuYakuhaiWest: "Yakuhai: West Wind",
  yakuYakuhaiNorth: "Yakuhai: North Wind",
  yakuYakuhaiWhite: "Yakuhai: White Dragon (Haku)",
  yakuYakuhaiGreen: "Yakuhai: Green Dragon (Hatsu)",
  yakuYakuhaiRed: "Yakuhai: Red Dragon (Chun)",
  yakuRinshanKaihou: "Rinshan Kaihou (After Kan)",
  yakuChankan: "Chankan (Robbing the Kan)",
  yakuHaitei: "Haitei Raoyue (Last Tile Draw)",
  yakuHoutei: "Houtei Raoyui (Last Tile Discard)",
  yakuDoubleRiichi: "Double Riichi",
  yakuToitoi: "Toitoi (All Triplets)",
  yakuSanshokuDoujun: "Sanshoku Doujun (Mixed Triple Sequence)",
  yakuSanshokuDoukou: "Sanshoku Doukou (Triple Triplets)",
  yakuIttsu: "Ittsu (Pure Straight)",
  yakuChiitoitsu: "Chiitoitsu (Seven Pairs)",
  yakuChanta: "Chanta (Outside Hand)",
  yakuSanAnkou: "San Ankou (Three Concealed Triplets)",
  yakuSanKantsu: "San Kantsu (Three Kans)",
  yakuHonroutou: "Honroutou (All Terminals and Honors)",
  yakuShousangen: "Shousangen (Little Three Dragons)",
  yakuHonitsu: "Honitsu (Half Flush)",
  yakuJunchan: "Junchan (Terminals in All Groups)",
  yakuRyanpeikou: "Ryanpeikou (Twice Pure Double Sequence)",
  yakuChinitsu: "Chinitsu (Full Flush)",
  yakuTenhou: "Tenhou (Heavenly Hand)",
  yakuChiihou: "Chiihou (Earthly Hand)",
  yakuKokushiMusou: "Kokushi Musou (Thirteen Orphans)",
  yakuSuuankou: "Suuankou (Four Concealed Triplets)",
  yakuDaisangen: "Daisangen (Big Three Dragons)",
  yakuShousuushii: "Shousuushii (Little Four Winds)",
  yakuDaisuushii: "Daisuushii (Big Four Winds)",
  yakuTsuuiisou: "Tsuuiisou (All Honors)",
  yakuChinroutou: "Chinroutou (All Terminals)",
  yakuRyuuiisou: "Ryuuiisou (All Green)",
  yakuChuurenPoutou: "Chuuren Poutou (Nine Gates)",
  yakuKokushi13Wait: "Kokushi Juusanmen (Kokushi Musou 13-wait)",
  yakuSuuankouTanki: "Suuankou Tanki",
  yakuJunseiChuurenPoutou: "Junsei Chuuren Poutou",
  yakuSuuKantsu: "Suu Kantsu (Four Kans)",
};
