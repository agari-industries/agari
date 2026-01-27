#!/usr/bin/env python3
"""
MJSON Data Explorer - Understand the structure of Tenhou mjson files.

This script helps explore the data format before running full validation.
It shows example events and statistics about the data.

Usage:
    python explore_mjson.py /path/to/tenhou/data
    python explore_mjson.py /path/to/single/file.mjson --single
"""

import argparse
import gzip
import json
import random
from collections import Counter, defaultdict
from pathlib import Path


def open_mjson(filepath: str):
    """Open an mjson file, handling both plain and gzip formats."""
    # Try gzip first (check magic bytes)
    with open(filepath, "rb") as f:
        magic = f.read(2)

    if magic == b"\x1f\x8b":  # gzip magic number
        return gzip.open(filepath, "rt", encoding="utf-8")
    else:
        return open(filepath, "r", encoding="utf-8")


def explore_single_file(filepath: str, max_events: int = 50):
    """Explore a single mjson file and show its structure."""
    print(f"\n{'=' * 60}")
    print(f"Exploring: {filepath}")
    print("=" * 60)

    event_types = Counter()
    hora_events = []
    sample_events = defaultdict(list)

    with open_mjson(filepath) as f:
        for i, line in enumerate(f):
            line = line.strip()
            if not line:
                continue

            try:
                event = json.loads(line)
                event_type = event.get("type", "unknown")
                event_types[event_type] += 1

                # Collect sample of each event type
                if len(sample_events[event_type]) < 2:
                    sample_events[event_type].append(event)

                # Collect all hora events
                if event_type == "hora":
                    hora_events.append(event)

            except json.JSONDecodeError as e:
                print(f"  Line {i}: JSON decode error - {e}")

    print(f"\nTotal lines: {sum(event_types.values())}")
    print("\nEvent type distribution:")
    for event_type, count in event_types.most_common():
        print(f"  {event_type}: {count}")

    print(f"\n{'=' * 60}")
    print("Sample events by type:")
    print("=" * 60)

    for event_type in [
        "start_kyoku",
        "tsumo",
        "dahai",
        "chi",
        "pon",
        "ankan",
        "hora",
        "ryukyoku",
    ]:
        if event_type in sample_events:
            print(f"\n--- {event_type} ---")
            for event in sample_events[event_type][:1]:  # Show just first sample
                print(json.dumps(event, indent=2, ensure_ascii=False))

    # Detailed hora analysis
    if hora_events:
        print(f"\n{'=' * 60}")
        print(f"HORA (Winning Hand) Analysis - {len(hora_events)} total")
        print("=" * 60)

        for i, hora in enumerate(hora_events[:3]):  # Show first 3
            print(f"\n--- Hora #{i + 1} ---")
            print(json.dumps(hora, indent=2, ensure_ascii=False))

            # Extract key fields
            print("\nKey fields:")
            print(f"  actor (winner): {hora.get('actor')}")
            print(f"  target (dealt in): {hora.get('target')}")
            print(f"  pai (winning tile): {hora.get('pai')}")
            print(f"  yakus: {hora.get('yakus', hora.get('yaku', 'N/A'))}")
            print(f"  fu: {hora.get('fu', 'N/A')}")
            print(f"  ten/points: {hora.get('ten', hora.get('points', 'N/A'))}")
            print(
                f"  ura_markers: {hora.get('ura_markers', hora.get('uradora_markers', 'N/A'))}"
            )
            print(f"  deltas: {hora.get('deltas', 'N/A')}")


def explore_directory(data_dir: str, num_files: int = 5):
    """Explore multiple mjson files to understand the data structure."""
    print(f"\n{'=' * 60}")
    print(f"Exploring directory: {data_dir}")
    print("=" * 60)

    base_path = Path(data_dir)
    mjson_files = list(base_path.rglob("*.mjson"))

    print(f"\nFound {len(mjson_files)} .mjson files")

    if not mjson_files:
        print("No mjson files found!")
        return

    # Show directory structure
    years = set()
    for f in mjson_files:
        parts = f.parts
        for p in parts:
            if p.isdigit() and len(p) == 4:
                years.add(p)

    if years:
        print(f"Years present: {sorted(years)}")

    # Sample file names
    print("\nSample file names:")
    for f in mjson_files[:5]:
        print(f"  {f}")

    # Pick random files to explore
    sample_files = random.sample(mjson_files, min(num_files, len(mjson_files)))

    for filepath in sample_files:
        explore_single_file(str(filepath), max_events=20)


def analyze_yakus(data_dir: str, num_files: int = 100):
    """Analyze yaku distribution across files."""
    print(f"\n{'=' * 60}")
    print("Yaku Distribution Analysis")
    print("=" * 60)

    base_path = Path(data_dir)
    mjson_files = list(base_path.rglob("*.mjson"))

    if len(mjson_files) > num_files:
        mjson_files = random.sample(mjson_files, num_files)

    yaku_counter = Counter()
    total_horas = 0

    for filepath in mjson_files:
        try:
            with open_mjson(str(filepath)) as f:
                for line in f:
                    try:
                        event = json.loads(line.strip())
                        if event.get("type") == "hora":
                            total_horas += 1
                            yakus = event.get("yakus", event.get("yaku", []))
                            if isinstance(yakus, list):
                                # If it's [id, han, id, han, ...] format
                                if yakus and isinstance(yakus[0], int):
                                    for i in range(0, len(yakus), 2):
                                        yaku_counter[yakus[i]] += 1
                                # If it's list of names
                                elif yakus and isinstance(yakus[0], str):
                                    for y in yakus:
                                        yaku_counter[y] += 1
                    except json.JSONDecodeError:
                        continue
        except Exception as e:
            print(f"Error reading {filepath}: {e}")

    print(f"\nAnalyzed {len(mjson_files)} files, found {total_horas} hora events")
    print("\nYaku frequency (top 30):")
    for yaku, count in yaku_counter.most_common(30):
        pct = 100 * count / total_horas if total_horas > 0 else 0
        print(f"  {yaku}: {count} ({pct:.1f}%)")


def main():
    parser = argparse.ArgumentParser(description="Explore Tenhou mjson data structure")
    parser.add_argument("path", help="Path to mjson file or directory")
    parser.add_argument(
        "--single", "-s", action="store_true", help="Treat path as a single file"
    )
    parser.add_argument(
        "--files",
        "-f",
        type=int,
        default=3,
        help="Number of files to explore (default: 3)",
    )
    parser.add_argument(
        "--yakus", "-y", action="store_true", help="Analyze yaku distribution"
    )
    parser.add_argument(
        "--yaku-files",
        type=int,
        default=100,
        help="Number of files for yaku analysis (default: 100)",
    )

    args = parser.parse_args()

    if args.single:
        explore_single_file(args.path)
    else:
        explore_directory(args.path, num_files=args.files)

        if args.yakus:
            analyze_yakus(args.path, num_files=args.yaku_files)


if __name__ == "__main__":
    main()
