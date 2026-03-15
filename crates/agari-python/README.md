# agari-python

Python bindings for [Agari](../../README.md)'s shanten and ukeire calculations, built with [PyO3](https://pyo3.rs).

## Install

Requires Rust and [maturin](https://www.maturin.rs/).

```bash
uv venv .venv && source .venv/bin/activate
uv tool install maturin
maturin develop -m crates/agari-python/Cargo.toml
```

## Usage

```python
import agari_core

# 34-element tile count array
# Indices: 0-8 = 1m-9m, 9-17 = 1p-9p, 18-26 = 1s-9s, 27-33 = East..Red
hand = [0] * 34
hand[0] = 1   # 1m
hand[1] = 1   # 2m
hand[2] = 1   # 3m
hand[3] = 1   # 4m
hand[4] = 1   # 5m
hand[5] = 1   # 6m
hand[6] = 1   # 7m
hand[7] = 1   # 8m
hand[8] = 1   # 9m
hand[9] = 1   # 1p
hand[18] = 1  # 1s
hand[19] = 1  # 2s
hand[20] = 1  # 3s

# Shanten (-1 = complete, 0 = tenpai)
shanten = agari_core.calculate_shanten(hand, num_melds=0)

# Ukeire (tile acceptance) with visible tiles
visible = [0] * 34
shanten, waits = agari_core.calculate_ukeire(hand, num_melds=0, visible=visible)

# Normalized features for ML (tenpai_flag, shanten/6, waits/46)
features = agari_core.compute_riichi_features(hand, num_melds=0, visible=visible)

# Batch version for preprocessing
results = agari_core.batch_compute_riichi_features(
    hands=[hand, hand],
    num_melds=[0, 0],
    visible=[visible, visible],
)
```
