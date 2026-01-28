# Agari Validator

A Python-based validation framework for testing the Agari Riichi Mahjong scoring engine against real historical game data from Tenhou.

## Purpose

The validator ensures Agari's scoring calculations match real-world Mahjong platform results by:

1. Parsing MJAI-format game logs from Tenhou
2. Extracting winning hand (hora) events with full game context
3. Converting hands to Agari's notation format
4. Running Agari and comparing scores against Tenhou's results
5. Reporting discrepancies for investigation

This process has been used to validate Agari against hundreds of thousands of historical games, identifying and fixing numerous edge cases in fu calculation, yaku detection, and special conditions like chankan and rinshan.

## Dataset

The validation dataset is obtained from Kaggle, containing Tenhou game logs converted to MJAI format:

**Source:** https://www.kaggle.com/datasets/shokanekolouis/tenhou-to-mjai

### Downloading the Dataset

```bash
#!/bin/bash
curl -L -o ~/Downloads/tenhou-to-mjai.zip \
  https://www.kaggle.com/api/v1/datasets/download/shokanekolouis/tenhou-to-mjai
```

> **Note:** You'll need Kaggle API credentials configured. See [Kaggle API documentation](https://www.kaggle.com/docs/api) for setup instructions.

After downloading, extract the archive:

```bash
unzip ~/Downloads/tenhou-to-mjai.zip -d ~/Downloads/tenhou-to-mjai
```

## Installation

The validator uses [uv](https://github.com/astral-sh/uv) for dependency management:

```bash
cd agari_validator
uv sync
```

Or with pip:

```bash
pip install -r pyproject.toml
```

## Usage

### Basic Validation

Run validation on a sample of games:

```bash
python agari_validator.py /path/to/tenhou-to-mjai --samples 1000
```

### Full Validation

Validate all files in the dataset:

```bash
python agari_validator.py /path/to/tenhou-to-mjai --all
```

### Export Mismatches

Save mismatches to JSON for analysis:

```bash
python agari_validator.py /path/to/tenhou-to-mjai --samples 1000 --export-mismatches mismatches.json
```

### Specify Agari Binary

If your Agari binary isn't in PATH:

```bash
python agari_validator.py /path/to/data --agari /path/to/agari
```

### Command Line Options

```
usage: agari_validator.py [-h] [--samples N] [--all] [--agari PATH]
                          [--export-mismatches FILE] [--verbose]
                          data_path

positional arguments:
  data_path             Path to Tenhou MJAI data directory

options:
  -h, --help            Show help message
  --samples N           Number of files to sample (default: 100)
  --all                 Process all files (ignores --samples)
  --agari PATH          Path to agari binary (default: searches PATH)
  --export-mismatches FILE
                        Export mismatches to JSON file
  --verbose, -v         Show detailed output for each hand
```

## Understanding Results

### Match Types

- **MATCH**: Agari's score matches Tenhou exactly
- **MISMATCH**: Scores differ (potential bug or ruleset difference)
- **ERROR**: Agari returned an error (invalid hand structure)
- **STRUCT_ERR**: Hand tracking issue in validator (not an Agari bug)

### Investigating Mismatches

When a mismatch is reported, the validator shows:

```
MISMATCH: Expected 7700, got 8000
  File: 2024/2024010100gm-00a9-0000-12345678.mjson
  Command: agari 123m456p789s11122z -w 2z --riichi -d 5m
```

Run the command manually to see full details:

```bash
agari 123m456p789s11122z -w 2z --riichi -d 5m
```

### Known Ruleset Differences

Some mismatches are intentional due to ruleset variations between Agari and Tenhou. See [RULESET_DIFFERENCES.md](RULESET_DIFFERENCES.md) for documented differences:

- Junsei Chuuren Poutou (9-wait): Agari awards double yakuman, Tenhou awards single
- Kokushi 13-wait: Agari awards double yakuman, Tenhou awards single

## Exploring the Data

Use `explore_mjson.py` to understand the MJAI data format:

```bash
# Explore a single file
python explore_mjson.py /path/to/file.mjson --single

# Explore multiple files in a directory
python explore_mjson.py /path/to/tenhou-to-mjai --files 5

# Analyze yaku distribution
python explore_mjson.py /path/to/tenhou-to-mjai --yakus
```

## File Structure

```
agari_validator/
├── agari_validator.py    # Main validation script
├── explore_mjson.py      # Data exploration utility
├── main.py               # Legacy test script
├── RULESET_DIFFERENCES.md # Documented rule variations
├── mismatches.json       # Sample mismatch export
└── pyproject.toml        # Python dependencies
```

## MJAI Format

The validator expects game logs in MJAI format (JSON lines). Each line is an event:

```json
{"type": "start_kyoku", "bakaze": "E", "kyoku": 1, "honba": 0, ...}
{"type": "tsumo", "actor": 0, "pai": "5m"}
{"type": "dahai", "actor": 0, "pai": "1m"}
{"type": "hora", "actor": 0, "target": 1, "pai": "7p", "yakus": [1, 1, 7, 1], ...}
```

The validator tracks game state through these events to reconstruct hands at the moment of winning.

## Contributing

When you find a mismatch:

1. Check [RULESET_DIFFERENCES.md](RULESET_DIFFERENCES.md) first
2. Run the Agari command manually to inspect the full output
3. If it's a bug, create an issue with the command and expected vs actual results
4. Add a test case in the Agari test suite when fixing