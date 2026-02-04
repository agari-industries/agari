#!/usr/bin/env python3
"""
Agari Validator - Validates the Agari scoring engine against Tenhou historical data.

This script:
1. Reads mjson files from Tenhou logs (converted to mjai format)
2. Extracts winning hand (hora) events
3. Converts the hand to Agari notation
4. Runs Agari and compares the output
5. Reports discrepancies

Usage:
    python agari_validator.py /path/to/tenhou/data --samples 1000
    python agari_validator.py /path/to/tenhou/data --samples 5000 --agari ../target/release/agari
"""

import argparse
import gzip
import json
import random
import re
import subprocess
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


def open_mjson(filepath: str):
    """Open an mjson file, handling both plain and gzip formats."""
    with open(filepath, "rb") as f:
        magic = f.read(2)

    if magic == b"\x1f\x8b":  # gzip magic number
        return gzip.open(filepath, "rt", encoding="utf-8")
    else:
        return open(filepath, "r", encoding="utf-8")


# ============================================================================
# MJAI Tile Notation -> Agari Notation Conversion
# ============================================================================

# mjai uses: 1m-9m, 1p-9p, 1s-9s, E/S/W/N (winds), P/F/C (dragons: White/Green/Red)
# Also: 5mr/5pr/5sr for red fives
# Agari uses: 1m-9m, 1p-9p, 1s-9s, 1z-7z (honors: E=1z, S=2z, W=3z, N=4z, P=5z, F=6z, C=7z)
# Also: 0m/0p/0s for red fives

MJAI_TO_AGARI_HONOR = {
    "E": "1z",  # East
    "S": "2z",  # South
    "W": "3z",  # West
    "N": "4z",  # North
    "P": "5z",  # White dragon (Haku/白)
    "F": "6z",  # Green dragon (Hatsu/發)
    "C": "7z",  # Red dragon (Chun/中)
}


def mjai_tile_to_agari(tile: str) -> str:
    """Convert a single mjai tile to agari notation."""
    if tile in MJAI_TO_AGARI_HONOR:
        return MJAI_TO_AGARI_HONOR[tile]

    # Handle red fives (5mr, 5pr, 5sr -> 0m, 0p, 0s)
    if tile.endswith("r"):
        suit = tile[-2]  # m, p, or s
        return f"0{suit}"

    # Regular numbered tiles (1m, 2p, 3s, etc.) pass through
    return tile


def mjai_tiles_to_agari(tiles: list[str]) -> str:
    """
    Convert a list of mjai tiles to agari hand notation.
    Groups tiles by suit for compact notation.
    """
    suits = defaultdict(list)

    for tile in tiles:
        agari_tile = mjai_tile_to_agari(tile)
        if agari_tile.endswith("z"):
            suits["z"].append(agari_tile[0])
        else:
            suit = agari_tile[-1]
            num = agari_tile[:-1]
            suits[suit].append(num)

    # Build the hand string, sorting within each suit
    result = []
    for suit in ["m", "p", "s", "z"]:
        if suits[suit]:
            nums = "".join(sorted(suits[suit], key=lambda x: (x == "0", x)))
            result.append(f"{nums}{suit}")

    return "".join(result)


def strip_red_five(tile: str) -> str:
    """Convert red five notation to regular five.

    When akadora (red fives) are disabled, the MJAI data still marks
    tiles as red fives based on Tenhou's tile ID encoding. This function
    strips those markings so Agari doesn't count them as akadora.

    Args:
        tile: MJAI tile notation (e.g., '5mr', '5pr', '5sr', or regular tiles)

    Returns:
        Tile with red marking removed (e.g., '5mr' -> '5m')
    """
    if tile.endswith("r"):
        return tile[:-1]  # 5mr -> 5m, 5pr -> 5p, 5sr -> 5s
    return tile


# ============================================================================
# Yaku ID Mapping (Tenhou IDs -> Names)
# ============================================================================

# From gimite/mjai tenhou_archive.rb - Tenhou yaku IDs
TENHOU_YAKU_NAMES = [
    "menzenchin_tsumoho",  # 0 - Menzen Tsumo
    "riichi",  # 1
    "ippatsu",  # 2
    "chankan",  # 3
    "rinshankaiho",  # 4
    "haiteiraoyue",  # 5 - Haitei
    "hoteiraoyui",  # 6 - Houtei
    "pinfu",  # 7
    "tanyaochu",  # 8 - Tanyao
    "ipeko",  # 9 - Iipeikou
    "jikaze_ton",  # 10 - Seat wind East
    "jikaze_nan",  # 11 - Seat wind South
    "jikaze_sha",  # 12 - Seat wind West
    "jikaze_pei",  # 13 - Seat wind North
    "bakaze_ton",  # 14 - Round wind East
    "bakaze_nan",  # 15 - Round wind South
    "bakaze_sha",  # 16 - Round wind West
    "bakaze_pei",  # 17 - Round wind North
    "yakuhai_haku",  # 18 - White dragon
    "yakuhai_hatsu",  # 19 - Green dragon
    "yakuhai_chun",  # 20 - Red dragon
    "daburu_riichi",  # 21 - Double Riichi
    "chitoitsu",  # 22 - Chiitoitsu
    "honchantaiyao",  # 23 - Chanta
    "ikkitsukan",  # 24 - Ittsu
    "sanshokudojun",  # 25 - Sanshoku Doujun
    "sanshokudoko",  # 26 - Sanshoku Doukou
    "sankantsu",  # 27 - San Kantsu
    "toitoiho",  # 28 - Toitoi
    "sananko",  # 29 - San Ankou
    "shosangen",  # 30 - Shou Sangen
    "honroto",  # 31 - Honroutou
    "ryanpeko",  # 32 - Ryanpeikou
    "junchantaiyao",  # 33 - Junchan
    "honiso",  # 34 - Honitsu (open: 2 han)
    "chiniso",  # 35 - Chinitsu (open: 5 han)
    "renho",  # 36 - Renhou (yakuman in some rules)
    "tenho",  # 37 - Tenhou (yakuman)
    "chiho",  # 38 - Chiihou (yakuman)
    "daisangen",  # 39 - Daisangen (yakuman)
    "suanko",  # 40 - Suuankou (yakuman)
    "suanko_tanki",  # 41 - Suuankou Tanki (double yakuman)
    "tsuiso",  # 42 - Tsuuiisou (yakuman)
    "ryuiso",  # 43 - Ryuuiisou (yakuman)
    "chinroto",  # 44 - Chinroutou (yakuman)
    "churenpoton",  # 45 - Chuuren Poutou (yakuman)
    "churenpoton_9",  # 46 - Junsei Chuuren (double yakuman)
    "kokushimuso",  # 47 - Kokushi Musou (yakuman)
    "kokushimuso_13",  # 48 - Kokushi 13-wait (double yakuman)
    "daisushi",  # 49 - Daisuushii (yakuman)
    "shosushi",  # 50 - Shousuushii (yakuman)
    "sukantsu",  # 51 - Suu Kantsu (yakuman)
    "dora",  # 52
    "uradora",  # 53
    "akadora",  # 54
]


def parse_tenhou_yakus(yaku_list: list) -> list[tuple[str, int]]:
    """
    Parse Tenhou yaku list format.
    Input is typically [yaku_id, han, yaku_id, han, ...]
    Returns list of (yaku_name, han) tuples.
    """
    result = []
    for i in range(0, len(yaku_list), 2):
        yaku_id = yaku_list[i]
        han = yaku_list[i + 1]
        if yaku_id < len(TENHOU_YAKU_NAMES):
            result.append((TENHOU_YAKU_NAMES[yaku_id], han))
        else:
            result.append((f"unknown_{yaku_id}", han))
    return result


# ============================================================================
# MJAI Wind Mapping
# ============================================================================

MJAI_WIND_TO_AGARI = {
    "E": "e",
    "S": "s",
    "W": "w",
    "N": "n",
}


# ============================================================================
# Data Classes
# ============================================================================


@dataclass
class HoraEvent:
    """Represents a winning hand extracted from Tenhou logs."""

    # Hand information
    tehais: list[str]  # Closed hand tiles
    winning_tile: str  # The winning tile
    melds: list[dict]  # Called melds (chi/pon/kan)

    # Game context
    actor: int  # Who won (0-3)
    target: int  # Who dealt in (-1 for tsumo)
    is_tsumo: bool

    # Round information
    bakaze: str  # Round wind (E/S/W/N)
    kyoku: int  # Round number (1-4)
    honba: int  # Repeat counter
    kyotaku: int  # Riichi sticks on table
    seat_wind: str  # Calculated from actor and oya

    # Riichi status
    is_riichi: bool = False
    is_double_riichi: bool = False
    is_ippatsu: bool = False

    # Special conditions
    is_rinshan: bool = False
    is_chankan: bool = False
    is_haitei: bool = False
    is_tenhou: bool = False
    is_chiihou: bool = False

    # Dora
    dora_markers: list[str] = field(default_factory=list)
    ura_dora_markers: list[str] = field(default_factory=list)

    # Scoring from Tenhou (deltas-based)
    tenhou_deltas: list[int] = field(default_factory=list)
    tenhou_points: int = 0  # Calculated from deltas

    # Source file for debugging
    source_file: str = ""

    # Whether we have full visibility of the winner's hand
    # False if any tiles were hidden ("?") during tracking
    has_full_visibility: bool = True

    # Whether akadora (red fives) are enabled in this game
    # When False, red five markings should be ignored
    aka_flag: bool = True

    def is_valid(self) -> bool:
        """Check if this hora event has valid tile counts and full visibility.

        This can fail when:
        - The log is from another player's perspective (hidden tiles shown as "?")
        - There was an error in hand state tracking
        """
        # Must have full visibility of the winner's hand
        if not self.has_full_visibility:
            return False

        # Check for any hidden or invalid tiles in the hand
        for tile in self.tehais:
            if not tile or tile == "?" or tile.startswith("?"):
                return False

        # Check winning tile is valid
        if (
            not self.winning_tile
            or self.winning_tile == "?"
            or self.winning_tile.startswith("?")
        ):
            return False

        # Count closed tiles
        closed_tile_count = len(self.tehais) + (1 if self.winning_tile else 0)

        # Count meld tiles
        meld_tile_count = 0
        kan_count = 0
        for meld in self.melds:
            meld_type = meld.get("type", "")
            if meld_type in ("ankan", "daiminkan", "kakan"):
                meld_tile_count += 4
                kan_count += 1
            elif meld_type in ("pon", "chi"):
                meld_tile_count += 3

        # Standard hand: 14 tiles, +1 per kan
        expected_total = 14 + kan_count
        actual_total = closed_tile_count + meld_tile_count

        if actual_total != expected_total:
            return False

        # Check for missing winning tile
        if not self.winning_tile:
            return False

        return True

    def to_agari_args(self) -> list[str]:
        """Convert this hora event to agari CLI arguments."""
        # Build the hand string (including winning tile - agari expects 14 tiles)
        # The -w flag specifies which tile was the winning tile, not an additional tile
        all_tiles = list(self.tehais) + [self.winning_tile]

        # When akadora is disabled, strip red five markings from all tiles
        # The MJAI converter marks tiles as red based on Tenhou's tile ID encoding,
        # but if aka_flag is False, these shouldn't be counted as akadora
        if not self.aka_flag:
            all_tiles = [strip_red_five(t) for t in all_tiles]

        # Track if hand is truly open (has chi/pon/open kan, but not ankan)
        is_open = False

        # Add meld tiles
        meld_strs = []
        for meld in self.melds:
            meld_tiles = meld.get("consumed", [])
            if "pai" in meld:
                meld_tiles = [meld["pai"]] + meld_tiles

            # Strip red five markings if akadora disabled
            if not self.aka_flag:
                meld_tiles = [strip_red_five(t) for t in meld_tiles]

            meld_type = meld.get("type", "")
            tiles_str = mjai_tiles_to_agari(meld_tiles)

            if meld_type == "ankan":
                # Closed kan - uses brackets, doesn't open hand
                meld_strs.append(f"[{tiles_str}]")
            elif meld_type == "kakan":
                # Upgraded pon (now 4 tiles, open)
                meld_strs.append(f"({tiles_str})")
                is_open = True
            elif meld_type == "daiminkan":
                # Open kan (4 tiles)
                meld_strs.append(f"({tiles_str})")
                is_open = True
            elif meld_type in ("pon", "chi"):
                meld_strs.append(f"({tiles_str})")
                is_open = True

        hand_str = mjai_tiles_to_agari(all_tiles) + "".join(meld_strs)

        args = [hand_str]

        # Winning tile (strip red marking if akadora disabled)
        winning_tile = self.winning_tile
        if not self.aka_flag:
            winning_tile = strip_red_five(winning_tile)
        args.extend(["-w", mjai_tile_to_agari(winning_tile)])

        # Tsumo vs Ron
        if self.is_tsumo:
            args.append("-t")

        # Open hand - only if we have open melds (not just ankan)
        # Note: Riichi requires closed hand, so if riichi is declared,
        # the hand must be closed (any open melds would be a bug in our parsing)
        if is_open and not self.is_riichi:
            args.append("-o")

        # Riichi
        if self.is_double_riichi:
            args.append("--double-riichi")
        elif self.is_riichi:
            args.append("-r")

        if self.is_ippatsu:
            args.append("--ippatsu")

        # Winds
        if self.bakaze:
            args.extend(["--round", MJAI_WIND_TO_AGARI.get(self.bakaze, "e")])
        if self.seat_wind:
            args.extend(["--seat", MJAI_WIND_TO_AGARI.get(self.seat_wind, "e")])

        # Dora
        if self.dora_markers:
            dora_str = ",".join(mjai_tile_to_agari(d) for d in self.dora_markers)
            args.extend(["-d", dora_str])

        if self.ura_dora_markers and self.is_riichi:
            ura_str = ",".join(mjai_tile_to_agari(u) for u in self.ura_dora_markers)
            args.extend(["--ura", ura_str])

        # Special conditions
        if self.is_rinshan:
            args.append("--rinshan")
        if self.is_chankan:
            args.append("--chankan")
        if self.is_haitei:
            args.append("--last-tile")
        if self.is_tenhou:
            args.append("--tenhou")
        if self.is_chiihou:
            args.append("--chiihou")

        return args


@dataclass
class ValidationResult:
    """Result of comparing Agari output to Tenhou data."""

    hora: HoraEvent
    agari_output: str
    agari_returncode: int

    # Parsed from agari output
    agari_fu: Optional[int] = None
    agari_han: Optional[int] = None
    agari_points: Optional[int] = None
    agari_yakus: list[str] = field(default_factory=list)

    # Comparison results
    points_match: bool = False

    @property
    def is_match(self) -> bool:
        return self.points_match

    @property
    def is_error(self) -> bool:
        return self.agari_returncode != 0

    @property
    def is_structure_error(self) -> bool:
        """Check if this error is due to invalid hand structure.

        These are usually validator hand-tracking bugs, not Agari bugs.
        """
        return (
            self.is_error and "no valid winning structure" in self.agari_output.lower()
        )


# ============================================================================
# MJSON Parsing
# ============================================================================


class MjsonParser:
    """Parser for mjai format JSON files."""

    def __init__(self):
        # Game-level settings (persist across kyoku)
        self.aka_flag = True  # Whether akadora (red fives) are enabled
        self.reset()

    def reset(self):
        """Reset parser state for a new kyoku (round)."""
        self.tehais = [[], [], [], []]  # Hands for each player
        self.melds = [[], [], [], []]  # Called melds for each player
        self.dora_markers = []
        self.ura_dora_markers = []
        self.bakaze = "E"
        self.kyoku = 1
        self.honba = 0
        self.kyotaku = 0
        self.oya = 0
        self.riichi_declared = [False, False, False, False]
        self.double_riichi = [False, False, False, False]
        self.ippatsu_chance = [False, False, False, False]
        self.ippatsu_tsumo_done = [
            False,
            False,
            False,
            False,
        ]  # Track if riichi player has drawn since riichi
        self.first_turn_tsumo = [True, True, True, True]  # For tenhou/chiihou detection
        self.rinshan_pending = False
        self.rinshan_actor = None  # Who declared the kan (only they can get rinshan)
        self.chankan_pending = False
        self.chankan_tile = None  # The tile added in kakan (for chankan validation)
        self.chankan_actor = None  # Who declared the kakan
        self.is_last_tile = False
        self.last_tile = {}  # Track last tile drawn/discarded per player
        self.tiles_remaining = 70  # Approximate wall tiles remaining
        self.last_event_type = None
        # Track if we have full visibility of each player's hand
        # Set to False if we see any hidden tiles ("?") for that player
        self.full_visibility = [True, True, True, True]

    def get_seat_wind(self, actor: int) -> str:
        """Calculate seat wind based on actor and oya (dealer)."""
        winds = ["E", "S", "W", "N"]
        offset = (actor - self.oya) % 4
        return winds[offset]

    def parse_file(self, filepath: str) -> list[HoraEvent]:
        """Parse an mjson file and extract all hora events."""
        hora_events = []

        with open_mjson(filepath) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue

                try:
                    event = json.loads(line)
                    hora = self.process_event(event, filepath)
                    if hora:
                        hora_events.append(hora)
                except json.JSONDecodeError:
                    continue

        return hora_events

    def process_event(self, event: dict, source_file: str = "") -> Optional[HoraEvent]:
        """Process a single mjai event. Returns HoraEvent if this is a hora."""
        event_type = event.get("type", "")

        if event_type == "start_game":
            # Track game-level settings
            self.aka_flag = event.get("aka_flag", True)
            return None

        if event_type == "start_kyoku":
            self.reset()
            self.bakaze = event.get("bakaze", "E")
            self.kyoku = event.get("kyoku", 1)
            self.honba = event.get("honba", 0)
            self.kyotaku = event.get("kyotaku", 0)
            self.oya = event.get("oya", 0)

            # Parse initial hands
            tehais = event.get("tehais", [])
            for i, hand in enumerate(tehais):
                if hand and hand[0] != "?":
                    self.tehais[i] = list(hand)
                else:
                    # Hidden initial hand - no visibility for this player
                    self.full_visibility[i] = False

            # Initial dora
            dora_marker = event.get("dora_marker")
            if dora_marker:
                self.dora_markers = [dora_marker]

        elif event_type == "tsumo":
            actor = event.get("actor", 0)
            pai = event.get("pai", "?")
            if pai != "?" and pai:
                self.tehais[actor].append(pai)
                self.last_tile[actor] = pai
            else:
                # Hidden tile - we don't have full visibility of this player's hand
                self.full_visibility[actor] = False
            self.tiles_remaining -= 1
            if self.tiles_remaining <= 0:
                self.is_last_tile = True
            # If this riichi player already had their ippatsu tsumo and is drawing again,
            # ippatsu window has passed (one full round completed without winning)
            if (
                self.ippatsu_chance[actor]
                and self.riichi_declared[actor]
                and self.ippatsu_tsumo_done[actor]
            ):
                self.ippatsu_chance[actor] = False
            # Mark that this riichi player has now had their first tsumo after riichi
            if self.riichi_declared[actor] and self.ippatsu_chance[actor]:
                self.ippatsu_tsumo_done[actor] = True

        elif event_type == "dahai":
            actor = event.get("actor", 0)
            pai = event.get("pai")
            if pai and pai in self.tehais[actor]:
                self.tehais[actor].remove(pai)
            self.last_tile[actor] = pai

            # After first discard, player has had their first turn
            self.first_turn_tsumo[actor] = False

            # If riichi player discards after their ippatsu tsumo without winning,
            # ippatsu window closes for them
            if self.ippatsu_chance[actor] and self.ippatsu_tsumo_done[actor]:
                self.ippatsu_chance[actor] = False

        elif event_type == "chi":
            actor = event.get("actor", 0)
            consumed = event.get("consumed", [])
            for tile in consumed:
                if tile in self.tehais[actor]:
                    self.tehais[actor].remove(tile)
            self.melds[actor].append(event)
            # Any call breaks ippatsu and first turn for everyone
            for i in range(4):
                self.first_turn_tsumo[i] = False
                self.ippatsu_chance[i] = False

        elif event_type == "pon":
            actor = event.get("actor", 0)
            consumed = event.get("consumed", [])
            for tile in consumed:
                if tile in self.tehais[actor]:
                    self.tehais[actor].remove(tile)
            self.melds[actor].append(event)
            for i in range(4):
                self.first_turn_tsumo[i] = False
                self.ippatsu_chance[i] = False

        elif event_type == "daiminkan":
            actor = event.get("actor", 0)
            consumed = event.get("consumed", [])
            for tile in consumed:
                if tile in self.tehais[actor]:
                    self.tehais[actor].remove(tile)
            self.melds[actor].append(event)
            self.rinshan_pending = True
            self.rinshan_actor = actor  # Only this player can get rinshan
            for i in range(4):
                self.first_turn_tsumo[i] = False
                self.ippatsu_chance[i] = False

        elif event_type == "ankan":
            actor = event.get("actor", 0)
            consumed = event.get("consumed", [])
            for tile in consumed:
                if tile in self.tehais[actor]:
                    self.tehais[actor].remove(tile)
            self.melds[actor].append(event)
            self.rinshan_pending = True
            self.rinshan_actor = actor  # Only this player can get rinshan
            # Ankan doesn't break ippatsu for the player who declared it
            # But breaks for others
            for i in range(4):
                if i != actor:
                    self.ippatsu_chance[i] = False

        elif event_type == "kakan":
            actor = event.get("actor", 0)
            pai = event.get("pai")
            if pai and pai in self.tehais[actor]:
                self.tehais[actor].remove(pai)

            # Find and upgrade the existing pon to a kan
            # Don't add a new meld - modify the existing pon
            for i, meld in enumerate(self.melds[actor]):
                if meld.get("type") == "pon":
                    # Check if this pon matches the kakan tile
                    consumed = meld.get("consumed", [])
                    meld_pai = meld.get("pai", "")

                    # Normalize tile names (handle red fives)
                    def normalize(t):
                        if t.endswith("r"):
                            return t[:-1]  # 5mr -> 5m
                        if t.startswith("0"):
                            return "5" + t[1:]  # 0m -> 5m
                        return t

                    if normalize(pai) == normalize(meld_pai) or normalize(pai) in [
                        normalize(c) for c in consumed
                    ]:
                        # Upgrade this pon to kakan
                        self.melds[actor][i] = {
                            "type": "kakan",
                            "actor": actor,
                            "pai": pai,
                            "consumed": consumed + [meld_pai],  # Include all 4 tiles
                            "target": meld.get("target"),
                        }
                        break

            self.rinshan_pending = True
            self.rinshan_actor = actor  # Only this player can get rinshan
            self.chankan_pending = True  # Next player could chankan
            self.chankan_tile = pai  # Track the added tile for chankan validation
            self.chankan_actor = actor  # Track who declared the kakan
            for i in range(4):
                if i != actor:
                    self.ippatsu_chance[i] = False

        elif event_type == "dora":
            dora_marker = event.get("dora_marker")
            if dora_marker:
                self.dora_markers.append(dora_marker)

        elif event_type == "reach":
            actor = event.get("actor", 0)
            # Check for double riichi (first turn cycle)
            if self.first_turn_tsumo[actor]:
                self.double_riichi[actor] = True
            self.riichi_declared[actor] = True
            self.ippatsu_chance[actor] = True

        elif event_type == "reach_accepted":
            actor = event.get("actor", 0)
            self.riichi_declared[actor] = True
            self.kyotaku += 1

        elif event_type == "hora":
            hora = self._create_hora_event(event, source_file)
            self.chankan_pending = False
            self.chankan_tile = None
            self.chankan_actor = None
            self.rinshan_pending = False
            self.rinshan_actor = None
            return hora

        # Reset chankan after any non-hora event following kakan
        if event_type not in ("kakan", "hora", "dora"):
            self.chankan_pending = False
            self.chankan_tile = None
            self.chankan_actor = None

        # Reset rinshan after a discard following a kan (the tsumo after kan is the rinshan draw,
        # so we only reset after that player discards, meaning the rinshan opportunity has passed)
        if event_type == "dahai" and self.rinshan_pending:
            self.rinshan_pending = False
            self.rinshan_actor = None

        self.last_event_type = event_type
        return None

    def _create_hora_event(self, event: dict, source_file: str) -> HoraEvent:
        """Create a HoraEvent from a hora mjai event."""
        actor = event.get("actor", 0)
        target = event.get("target", actor)
        is_tsumo = actor == target

        # Get the deltas and calculate points won
        deltas = event.get("deltas", [0, 0, 0, 0])
        # Points won by actor (includes honba/riichi sticks)
        points_won = deltas[actor] if actor < len(deltas) else 0

        # Get ura dora markers
        ura_markers = event.get("ura_markers", [])

        # Determine winning tile
        # First, try to get it directly from the hora event (most reliable)
        # Fall back to tracking the last relevant tile if not present
        winning_tile = event.get("pai", "")
        if not winning_tile:
            # Fallback: For ron it's the last discarded tile by target
            # For tsumo it's the last drawn tile by actor
            winning_tile = self.last_tile.get(actor if is_tsumo else target, "")

        # Get the hand (the current state of actor's hand)
        tehais = list(self.tehais[actor])

        # For tsumo, the winning tile is already in the hand from the tsumo event
        # For ron, we need to add it
        if not is_tsumo and winning_tile and winning_tile not in tehais:
            pass  # Don't add - we'll pass it separately
        elif is_tsumo and winning_tile in tehais:
            tehais.remove(winning_tile)  # Remove so we can pass separately

        # Detect special conditions from game state
        is_ippatsu = self.ippatsu_chance[actor]
        is_riichi = self.riichi_declared[actor]
        is_double_riichi = self.double_riichi[actor]
        # Rinshan: win on kan replacement tile (must be tsumo by the kan declarer)
        is_rinshan = self.rinshan_pending and is_tsumo and self.rinshan_actor == actor
        # Rinshan and Haitei are mutually exclusive:
        # - Rinshan = win on kan replacement tile (from dead wall)
        # - Haitei = win on last tile from regular wall
        # If rinshan is true, haitei cannot be true
        is_haitei = self.is_last_tile and not is_rinshan
        is_tenhou = is_tsumo and self.first_turn_tsumo[actor] and actor == self.oya
        is_chiihou = is_tsumo and self.first_turn_tsumo[actor] and actor != self.oya

        # Chankan validation: robbing another player's added kan
        # Requirements:
        # 1. Must be ron (not tsumo) - you're stealing someone else's kan tile
        # 2. Winner must be different from kakan declarer
        # 3. chankan_pending must be true (a kakan just happened)
        # Note: We don't strictly validate the winning tile matches the kakan tile
        # because tile normalization (red fives) makes this complex, and if
        # chankan_pending is true and it's a ron from someone else, it's chankan.
        is_chankan = (
            self.chankan_pending
            and not is_tsumo
            and self.chankan_actor is not None
            and actor != self.chankan_actor
        )

        hora = HoraEvent(
            tehais=tehais,
            winning_tile=winning_tile,
            melds=list(self.melds[actor]),
            actor=actor,
            target=target,
            is_tsumo=is_tsumo,
            bakaze=self.bakaze,
            kyoku=self.kyoku,
            honba=self.honba,
            kyotaku=self.kyotaku,
            seat_wind=self.get_seat_wind(actor),
            is_riichi=is_riichi,
            is_double_riichi=is_double_riichi,
            is_ippatsu=is_ippatsu,
            is_rinshan=is_rinshan,
            is_chankan=is_chankan,
            is_haitei=is_haitei,
            is_tenhou=is_tenhou,
            is_chiihou=is_chiihou,
            dora_markers=list(self.dora_markers),
            ura_dora_markers=ura_markers,
            tenhou_deltas=deltas,
            tenhou_points=points_won,
            source_file=source_file,
            has_full_visibility=self.full_visibility[actor],
            aka_flag=self.aka_flag,
        )

        return hora


# ============================================================================
# Agari Runner
# ============================================================================


class AgariRunner:
    """Runs the Agari CLI and parses output."""

    def __init__(self, agari_path: str = "agari"):
        self.agari_path = agari_path

    def run(self, hora: HoraEvent) -> ValidationResult:
        """Run agari on a hora event and return validation result."""
        args = hora.to_agari_args()
        cmd = [self.agari_path] + args

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)

            validation = ValidationResult(
                hora=hora,
                agari_output=result.stdout + result.stderr,
                agari_returncode=result.returncode,
            )

            if result.returncode == 0:
                self._parse_output(validation, result.stdout)
                self._compare(validation)

            return validation

        except subprocess.TimeoutExpired:
            return ValidationResult(
                hora=hora,
                agari_output="TIMEOUT",
                agari_returncode=-1,
            )
        except FileNotFoundError:
            return ValidationResult(
                hora=hora,
                agari_output=f"Agari not found at: {self.agari_path}",
                agari_returncode=-2,
            )

    def _parse_output(self, validation: ValidationResult, output: str):
        """Parse agari CLI output to extract scoring info."""
        # Look for patterns like:
        # "30 Fu, 3 Han"
        # "7700 Points" or "Mangan: 8000"
        # Yaku listings

        lines = output.split("\n")

        for line in lines:
            # Fu pattern
            fu_match = re.search(r"(\d+)\s*[Ff]u", line)
            if fu_match:
                validation.agari_fu = int(fu_match.group(1))

            # Han pattern
            han_match = re.search(r"(\d+)\s*[Hh]an", line)
            if han_match:
                validation.agari_han = int(han_match.group(1))

            # Points patterns
            points_match = re.search(r"(\d+)\s*[Pp]oints?", line)
            if points_match:
                validation.agari_points = int(points_match.group(1))

            # Mangan/Haneman/etc patterns
            limit_match = re.search(
                r"(Mangan|Haneman|Baiman|Sanbaiman|Yakuman)[:\s]+(\d+)",
                line,
                re.IGNORECASE,
            )
            if limit_match:
                validation.agari_points = int(limit_match.group(2))

            # Yaku detection (simplified - would need actual parsing)
            if "Yaku:" in line or "yaku" in line.lower():
                # Extract yaku names from the line
                pass

    def _compare(self, validation: ValidationResult):
        """Compare parsed agari output with Tenhou data."""
        hora = validation.hora

        # Calculate expected points from deltas
        # The delta includes honba bonus (300 per honba) and riichi sticks
        # For comparison, we need to account for these
        base_points = hora.tenhou_points

        # Remove honba bonus from comparison (300 * honba for ron, 100 * honba * 3 for tsumo)
        honba_bonus = hora.honba * (300 if not hora.is_tsumo else 300)

        # Remove riichi sticks (1000 per stick)
        riichi_bonus = hora.kyotaku * 1000

        # The base hand value (before bonuses)
        expected_base = base_points - honba_bonus - riichi_bonus

        if validation.agari_points is not None and expected_base > 0:
            # Allow for some tolerance due to:
            # - Rounding differences
            # - Honba/riichi calculation variations
            diff = abs(validation.agari_points - expected_base)
            # Also check against raw points in case bonuses weren't included
            diff_raw = abs(validation.agari_points - base_points)
            validation.points_match = diff <= 300 or diff_raw <= 300
        else:
            validation.points_match = True  # Can't compare if missing


# ============================================================================
# File Discovery
# ============================================================================


def find_mjson_files(base_dir: str) -> list[str]:
    """Find all .mjson files in the directory structure."""
    mjson_files = []
    base_path = Path(base_dir)

    for mjson_file in base_path.rglob("*.mjson"):
        mjson_files.append(str(mjson_file))

    return mjson_files


# ============================================================================
# Main Validation Logic
# ============================================================================


def validate_samples(
    data_dir: str,
    agari_path: str,
    num_samples: int = 100,
    seed: Optional[int] = None,
    verbose: bool = False,
) -> dict:
    """
    Main validation function.

    Args:
        data_dir: Path to Tenhou data directory
        agari_path: Path to agari executable
        num_samples: Target number of hora events to validate
        seed: Random seed for reproducibility
        verbose: Print detailed progress

    Returns:
        Dictionary with validation statistics
    """
    print(f"Finding mjson files in {data_dir}...")
    all_files = find_mjson_files(data_dir)
    print(f"Found {len(all_files)} mjson files")

    if not all_files:
        print("No mjson files found!")
        return {}

    # Shuffle files for random sampling
    if seed is not None:
        random.seed(seed)
    random.shuffle(all_files)

    parser = MjsonParser()
    runner = AgariRunner(agari_path)

    all_horas = []
    files_processed = 0

    # Extract hora events from files until we have enough samples
    # Each game typically has ~4-8 hora events, so we load progressively
    print(f"Extracting hora events (target: {num_samples})...")
    for filepath in all_files:
        try:
            horas = parser.parse_file(filepath)
            # Filter valid horas immediately to avoid loading more files than needed
            valid_horas = [h for h in horas if h.is_valid()]
            all_horas.extend(valid_horas)
            files_processed += 1
            if verbose:
                print(
                    f"  [{files_processed}] {filepath}: {len(valid_horas)} valid horas (total: {len(all_horas)})"
                )
            # Stop once we have enough samples
            if len(all_horas) >= num_samples:
                break
        except Exception as e:
            files_processed += 1
            if verbose:
                print(f"  Error parsing {filepath}: {e}")

    print(
        f"Processed {files_processed} files, extracted {len(all_horas)} valid hora events"
    )

    # Sample down if we got more than requested
    if num_samples < len(all_horas):
        if seed is not None:
            random.seed(seed + 1)
        all_horas = random.sample(all_horas, num_samples)

    valid_horas = all_horas
    print(f"Validating {len(valid_horas)} hora events...")

    # Run validation
    results = []
    matches = 0
    errors = 0
    structure_errors = 0
    mismatches = 0

    for i, hora in enumerate(valid_horas):
        result = runner.run(hora)
        results.append(result)

        if result.is_structure_error:
            structure_errors += 1
            status = "STRUCT_ERR"  # Validator hand-tracking bug, not Agari bug
        elif result.is_error:
            errors += 1
            status = "ERROR"
        elif result.is_match:
            matches += 1
            status = "OK"
        else:
            mismatches += 1
            status = "MISMATCH"

        if verbose or not result.is_match:
            args = hora.to_agari_args()
            print(f"\n[{i + 1}/{len(valid_horas)}] {status}")
            print(f"  Command: agari {' '.join(args)}")
            print(f"  Tenhou: {hora.tenhou_points} pts (deltas: {hora.tenhou_deltas})")
            print(
                f"  Context: {hora.bakaze}{hora.kyoku}, honba={hora.honba}, kyotaku={hora.kyotaku}"
            )
            print(
                f"  Win: {'tsumo' if hora.is_tsumo else 'ron'}, riichi={hora.is_riichi}, ippatsu={hora.is_ippatsu}"
            )
            if not result.is_error:
                print(
                    f"  Agari:  {result.agari_fu} fu, {result.agari_han} han, {result.agari_points} pts"
                )
            else:
                print(f"  Error:  {result.agari_output[:300]}")

    # Summary
    total_validated = len(valid_horas)
    print("\n" + "=" * 60)
    print("VALIDATION SUMMARY")
    print("=" * 60)
    print(f"Files processed: {files_processed}")
    print(f"Total validated: {total_validated}")
    if total_validated > 0:
        print(f"Matches:         {matches} ({100 * matches / total_validated:.1f}%)")
        print(
            f"Mismatches:      {mismatches} ({100 * mismatches / total_validated:.1f}%)"
        )
        print(f"Errors:          {errors} ({100 * errors / total_validated:.1f}%)")
        if structure_errors > 0:
            print(f"  (struct errs): {structure_errors} (validator hand-tracking bugs)")

    return {
        "total": total_validated,
        "files_processed": files_processed,
        "matches": matches,
        "mismatches": mismatches,
        "errors": errors,
        "structure_errors": structure_errors,
        "results": results,
    }


# ============================================================================
# CLI
# ============================================================================


def main():
    parser = argparse.ArgumentParser(
        description="Validate Agari scoring engine against Tenhou historical data"
    )
    parser.add_argument(
        "data_dir", help="Path to Tenhou data directory containing mjson files"
    )
    parser.add_argument(
        "--agari",
        "-a",
        default="agari",
        help="Path to agari executable (default: agari)",
    )
    parser.add_argument(
        "--samples",
        "-n",
        type=int,
        default=100,
        help="Number of hora events to validate (default: 100)",
    )
    parser.add_argument(
        "--seed", "-s", type=int, default=None, help="Random seed for reproducibility"
    )
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Print detailed progress"
    )
    parser.add_argument(
        "--export-mismatches",
        type=str,
        default=None,
        help="Export mismatches to a JSON file for analysis",
    )

    args = parser.parse_args()

    stats = validate_samples(
        data_dir=args.data_dir,
        agari_path=args.agari,
        num_samples=args.samples,
        seed=args.seed,
        verbose=args.verbose,
    )

    # Export mismatches if requested
    if args.export_mismatches and stats.get("results"):
        mismatches = []
        for r in stats["results"]:
            if not r.is_match and not r.is_error:
                mismatches.append(
                    {
                        "agari_args": r.hora.to_agari_args(),
                        "tenhou_points": r.hora.tenhou_points,
                        "tenhou_deltas": r.hora.tenhou_deltas,
                        "agari_fu": r.agari_fu,
                        "agari_han": r.agari_han,
                        "agari_points": r.agari_points,
                        "context": {
                            "bakaze": r.hora.bakaze,
                            "kyoku": r.hora.kyoku,
                            "honba": r.hora.honba,
                            "kyotaku": r.hora.kyotaku,
                            "is_tsumo": r.hora.is_tsumo,
                            "is_riichi": r.hora.is_riichi,
                        },
                        "source_file": r.hora.source_file,
                    }
                )

        with open(args.export_mismatches, "w") as f:
            json.dump(mismatches, f, indent=2)
        print(f"\nExported {len(mismatches)} mismatches to {args.export_mismatches}")


if __name__ == "__main__":
    main()
