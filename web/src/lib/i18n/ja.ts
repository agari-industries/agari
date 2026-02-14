import type { Translations } from "./types";

export const ja: Translations = {
  // Header
  tagline: "立直麻雀 点数計算機",

  // Loading/Error states
  loadingCalculator: "計算機を読み込み中...",
  failedToLoad: "計算機の読み込みに失敗しました:",

  // Hand builder
  buildYourHand: "手牌を作成",
  clear: "クリア",
  addMeld: "副露を追加:",
  chi: "チー",
  pon: "ポン",
  openKan: "明槓",
  closedKan: "暗槓",

  // Meld builder
  buildingChi: "チーを作成中",
  buildingPon: "ポンを作成中",
  buildingOpenKan: "明槓を作成中",
  buildingClosedKan: "暗槓を作成中",
  hintChi: "(同じ色の連続した3枚を選択)",
  hintPon: "(同じ牌を3枚選択)",
  hintKan: "(同じ牌を4枚選択)",
  cancel: "キャンセル",
  confirmAddMeld: "追加",

  // Melds display
  calledMelds: "副露",

  // Hand display
  yourHand: "手牌",
  selectWinningTileHint: "和了牌をクリックして選択",
  winBadge: "和了",

  // Shanten
  complete: "和了形",
  tenpai: "聴牌",
  shanten: "向聴",

  // Dora section
  doraIndicators: "ドラ表示牌",
  dora: "ドラ",
  uraDora: "裏ドラ",
  addButton: "+ 追加",
  akadoraInHand: "手牌の赤ドラ:",

  // Results
  results: "結果",
  calculating: "計算中...",
  inferredWinningTile: "和了牌を推定:",
  han: "翻",
  fu: "符",
  pts: "点",
  all: "オール",
  dealer: "親",
  dealerOya: "親",
  yaku: "役",
  ura: "裏",
  aka: "赤",
  fuBreakdown: "符計算",
  fuBase: "基本",
  fuMenzenRon: "門前ロン",
  fuTsumo: "ツモ",
  fuMelds: "面子",
  fuPair: "雀頭",
  fuWait: "待ち",
  fuTotal: "合計",
  structure: "手牌構成:",
  enterCompleteHand: "完成形を入力して点数を計算",

  // Options
  options: "オプション",
  winType: "和了方法",
  ron: "ロン",
  tsumo: "ツモ",
  winds: "風",
  round: "場風",
  seat: "自風",
  riichi: "立直",
  openHandNotice: "副露あり — 立直不可",
  doubleRiichi: "ダブル立直",
  ippatsu: "一発",
  situational: "状況役",
  haitei: "海底摸月",
  houtei: "河底撈魚",
  rinshanKaihou: "嶺上開花",
  chankan: "槍槓",
  firstTurnYakuman: "第一巡役満",
  tenhou: "天和",
  chiihou: "地和",

  // Calculate button
  calculateScore: "点数計算",

  // Footer
  footerPoweredBy: "Powered by",
  footerDescription: "— Rustで書かれた立直麻雀点数計算エンジン",

  // Tile picker
  selectTile: "牌を選択",

  // Wind names
  windEast: "東",
  windSouth: "南",
  windWest: "西",
  windNorth: "北",

  // Language
  language: "言語",

  // Share
  share: "共有",
  copiedToClipboard: "リンクをコピーしました！",

  // Tile Theme
  tileTheme: "牌",
  tileThemeLight: "白",
  tileThemeDark: "黒",

  // Score levels
  scoreLevelMangan: "満貫",
  scoreLevelHaneman: "跳満",
  scoreLevelBaiman: "倍満",
  scoreLevelSanbaiman: "三倍満",
  scoreLevelYakuman: "役満",
  scoreLevelDoubleYakuman: "ダブル役満",
  scoreLevelCountedYakuman: "数え役満",

  // Yaku names
  yakuRiichi: "立直",
  yakuIppatsu: "一発",
  yakuMenzenTsumo: "門前清自摸和",
  yakuTanyao: "断幺九",
  yakuPinfu: "平和",
  yakuIipeikou: "一盃口",
  yakuYakuhaiEast: "役牌: 東",
  yakuYakuhaiSouth: "役牌: 南",
  yakuYakuhaiWest: "役牌: 西",
  yakuYakuhaiNorth: "役牌: 北",
  yakuYakuhaiWhite: "役牌: 白",
  yakuYakuhaiGreen: "役牌: 發",
  yakuYakuhaiRed: "役牌: 中",
  yakuRinshanKaihou: "嶺上開花",
  yakuChankan: "槍槓",
  yakuHaitei: "海底摸月",
  yakuHoutei: "河底撈魚",
  yakuDoubleRiichi: "ダブル立直",
  yakuToitoi: "対々和",
  yakuSanshokuDoujun: "三色同順",
  yakuSanshokuDoukou: "三色同刻",
  yakuIttsu: "一気通貫",
  yakuChiitoitsu: "七対子",
  yakuChanta: "混全帯幺九",
  yakuSanAnkou: "三暗刻",
  yakuSanKantsu: "三槓子",
  yakuHonroutou: "混老頭",
  yakuShousangen: "小三元",
  yakuHonitsu: "混一色",
  yakuJunchan: "純全帯幺九",
  yakuRyanpeikou: "二盃口",
  yakuChinitsu: "清一色",
  yakuTenhou: "天和",
  yakuChiihou: "地和",
  yakuKokushiMusou: "国士無双",
  yakuSuuankou: "四暗刻",
  yakuDaisangen: "大三元",
  yakuShousuushii: "小四喜",
  yakuDaisuushii: "大四喜",
  yakuTsuuiisou: "字一色",
  yakuChinroutou: "清老頭",
  yakuRyuuiisou: "緑一色",
  yakuChuurenPoutou: "九蓮宝燈",
  yakuKokushi13Wait: "国士無双十三面待ち",
  yakuSuuankouTanki: "四暗刻単騎",
  yakuJunseiChuurenPoutou: "純正九蓮宝燈",
  yakuSuuKantsu: "四槓子",
};
