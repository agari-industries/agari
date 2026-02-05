import type { Translations } from './types';

export const ja: Translations = {
  // Header
  tagline: '立直麻雀 点数計算機',

  // Loading/Error states
  loadingCalculator: '計算機を読み込み中...',
  failedToLoad: '計算機の読み込みに失敗しました:',

  // Hand builder
  buildYourHand: '手牌を作成',
  clear: 'クリア',
  addMeld: '副露を追加:',
  chi: 'チー',
  pon: 'ポン',
  openKan: '明槓',
  closedKan: '暗槓',

  // Meld builder
  buildingChi: 'チーを作成中',
  buildingPon: 'ポンを作成中',
  buildingOpenKan: '明槓を作成中',
  buildingClosedKan: '暗槓を作成中',
  hintChi: '(同じ色の連続した3枚を選択)',
  hintPon: '(同じ牌を3枚選択)',
  hintKan: '(同じ牌を4枚選択)',
  cancel: 'キャンセル',
  confirmAddMeld: '追加',

  // Melds display
  calledMelds: '副露',

  // Hand display
  yourHand: '手牌',
  selectWinningTileHint: '和了牌をクリックして選択',
  winBadge: '和了',

  // Shanten
  complete: '和了形',
  tenpai: '聴牌',
  shanten: '向聴',

  // Dora section
  doraIndicators: 'ドラ表示牌',
  dora: 'ドラ',
  uraDora: '裏ドラ',
  addButton: '+ 追加',
  akadoraInHand: '手牌の赤ドラ:',

  // Results
  results: '結果',
  calculating: '計算中...',
  inferredWinningTile: '和了牌を推定:',
  han: '翻',
  fu: '符',
  pts: '点',
  all: 'オール',
  dealer: '親',
  dealerOya: '親',
  yaku: '役',
  ura: '裏',
  aka: '赤',
  fuBreakdown: '符計算',
  fuBase: '基本',
  fuMenzenRon: '門前ロン',
  fuTsumo: 'ツモ',
  fuMelds: '面子',
  fuPair: '雀頭',
  fuWait: '待ち',
  fuTotal: '合計',
  structure: '手牌構成:',
  enterCompleteHand: '完成形を入力して点数を計算',

  // Options
  options: 'オプション',
  winType: '和了方法',
  ron: 'ロン',
  tsumo: 'ツモ',
  winds: '風',
  round: '場風',
  seat: '自風',
  riichi: '立直',
  openHandNotice: '副露あり — 立直不可',
  doubleRiichi: 'ダブル立直',
  ippatsu: '一発',
  situational: '状況役',
  haitei: '海底摸月',
  houtei: '河底撈魚',
  rinshanKaihou: '嶺上開花',
  chankan: '槍槓',
  firstTurnYakuman: '第一巡役満',
  tenhou: '天和',
  chiihou: '地和',

  // Calculate button
  calculateScore: '点数計算',

  // Footer
  footerPoweredBy: 'Powered by',
  footerDescription: '— Rustで書かれた立直麻雀点数計算エンジン',

  // Tile picker
  selectTile: '牌を選択',

  // Wind names
  windEast: '東',
  windSouth: '南',
  windWest: '西',
  windNorth: '北',

  // Language
  language: '言語',
};
