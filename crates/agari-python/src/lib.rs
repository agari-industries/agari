use pyo3::prelude::*;

use agari::context::{GameContext, WinType};
use agari::hand::{decompose_hand_with_melds, KanType, Meld};
use agari::parse::TileCounts;
use agari::shanten;
use agari::tile::Honor;
use agari::yaku::detect_yaku_with_context;

fn honor_from_index(i: u8) -> PyResult<Honor> {
    Ok(match i {
        0 => Honor::East,
        1 => Honor::South,
        2 => Honor::West,
        3 => Honor::North,
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "wind index must be 0..3 (E,S,W,N)",
            ))
        }
    })
}

fn vec_to_tilecounts(arr: Vec<u8>) -> PyResult<TileCounts> {
    let arr: [u8; 34] = arr
        .try_into()
        .map_err(|v: Vec<u8>| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("expected 34-element array, got {}", v.len()),
        ))?;
    Ok(shanten::array_to_tilecounts(&arr))
}

#[pyfunction]
fn calculate_shanten(hand: Vec<u8>, num_melds: u8) -> PyResult<i8> {
    let counts = vec_to_tilecounts(hand)?;
    let result = shanten::calculate_shanten_with_melds(&counts, num_melds);
    Ok(result.shanten)
}

#[pyfunction]
fn calculate_ukeire(hand: Vec<u8>, num_melds: u8, visible: Vec<u8>) -> PyResult<(i8, u8)> {
    let counts = vec_to_tilecounts(hand)?;
    let visible_counts = vec_to_tilecounts(visible)?;
    let result = shanten::calculate_ukeire_with_melds_and_visible(&counts, num_melds, &visible_counts);
    Ok((result.shanten, result.total_count))
}

#[pyfunction]
fn compute_riichi_features(hand: Vec<u8>, num_melds: u8, visible: Vec<u8>) -> PyResult<(f32, f32, f32)> {
    let counts = vec_to_tilecounts(hand)?;
    let visible_counts = vec_to_tilecounts(visible)?;
    let result = shanten::calculate_ukeire_with_melds_and_visible(&counts, num_melds, &visible_counts);
    let tenpai_flag = if result.shanten == 0 { 1.0 } else { 0.0 };
    let shanten_norm = result.shanten as f32 / 6.0;
    let waits_norm = result.total_count as f32 / 46.0;
    Ok((tenpai_flag, shanten_norm, waits_norm))
}

#[pyfunction]
fn batch_compute_riichi_features(
    hands: Vec<Vec<u8>>,
    num_melds: Vec<u8>,
    visible: Vec<Vec<u8>>,
) -> PyResult<Vec<(f32, f32, f32)>> {
    if hands.len() != num_melds.len() || hands.len() != visible.len() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "hands, num_melds, and visible must have the same length",
        ));
    }
    hands
        .into_iter()
        .zip(num_melds)
        .zip(visible)
        .map(|((h, m), v)| compute_riichi_features(h, m, v))
        .collect()
}

#[pyfunction]
fn valid_chi_combinations(hand: Vec<u8>, discarded_tile: u8) -> PyResult<Vec<(u8, u8)>> {
    let counts = vec_to_tilecounts(hand)?;
    let tile = shanten::index_to_tile(discarded_tile as usize);
    let combos = shanten::valid_chi_combinations(&counts, tile);
    Ok(combos
        .into_iter()
        .map(|(a, b)| (shanten::tile_to_index(a) as u8, shanten::tile_to_index(b) as u8))
        .collect())
}

#[pyfunction]
fn shanten_after_chi(
    hand: Vec<u8>,
    _discarded_tile: u8,
    combo: (u8, u8),
    num_melds: u8,
) -> PyResult<i8> {
    let counts = vec_to_tilecounts(hand)?;
    let combo_tiles = (
        shanten::index_to_tile(combo.0 as usize),
        shanten::index_to_tile(combo.1 as usize),
    );
    Ok(shanten::shanten_after_chi(&counts, combo_tiles, num_melds))
}

#[pyfunction]
fn shanten_after_kan(
    hand: Vec<u8>,
    kan_tile: u8,
    kan_type: u8,
    num_melds: u8,
) -> PyResult<i8> {
    let counts = vec_to_tilecounts(hand)?;
    let tile = shanten::index_to_tile(kan_tile as usize);
    let kt = match kan_type {
        0 => KanType::Open,
        1 => KanType::Closed,
        2 => KanType::Added,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "kan_type must be 0 (open), 1 (closed), or 2 (added)",
        )),
    };
    Ok(shanten::shanten_after_kan(&counts, tile, kt, num_melds))
}

#[pyfunction]
fn is_permanent_furiten(hand: Vec<u8>, own_discards: Vec<u8>, num_melds: u8) -> PyResult<bool> {
    let arr: [u8; 34] = hand
        .try_into()
        .map_err(|v: Vec<u8>| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("expected 34-element array, got {}", v.len()),
        ))?;
    let counts = shanten::array_to_tilecounts(&arr);
    let s = shanten::calculate_shanten_with_melds(&counts, num_melds).shanten;
    if s != 0 {
        return Ok(false);
    }
    // Find winning tiles: adding tile i makes shanten == -1
    for i in 0..34u8 {
        let mut test_arr = arr;
        if test_arr[i as usize] >= 4 {
            continue; // can't have more than 4 of a tile
        }
        test_arr[i as usize] += 1;
        let tc = shanten::array_to_tilecounts(&test_arr);
        let sh = shanten::calculate_shanten_with_melds(&tc, num_melds).shanten;
        if sh == -1 && own_discards.contains(&i) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Does this open tenpai hand have a (non-situational) yaku on at least one
/// winning tile? Uses the real agari scoring engine (decompose + detect_yaku).
///
/// closed: 34 concealed-tile counts (excludes called-meld tiles, excludes the
///   winning tile). meld_kinds[i]/meld_tiles[i] describe each called meld:
///   kind 0=chi(open run, tile=lowest), 1=pon(open triplet), 2=daiminkan(open),
///   3=ankan(closed kan), 4=kakan(added kan). bakaze/seat are wind indices
///   0..3 (E,S,W,N). Situational yaku (riichi/ippatsu/tsumo/haitei/houtei/
///   rinshan/chankan) are intentionally NOT credited: this answers "can this
///   hand legitimately declare a win by its own shape", i.e. is it NOT a dead
///   open hand.
#[pyfunction]
fn hand_has_yaku(
    closed: Vec<u8>,
    meld_kinds: Vec<u8>,
    meld_tiles: Vec<u8>,
    bakaze: u8,
    seat: u8,
) -> PyResult<bool> {
    if meld_kinds.len() != meld_tiles.len() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "meld_kinds and meld_tiles must have the same length",
        ));
    }
    let round_wind = honor_from_index(bakaze)?;
    let seat_wind = honor_from_index(seat)?;

    let mut called: Vec<Meld> = Vec::with_capacity(meld_kinds.len());
    for (k, t) in meld_kinds.iter().zip(meld_tiles.iter()) {
        let tile = shanten::index_to_tile(*t as usize);
        called.push(match k {
            0 => Meld::shuntsu_open(tile),
            1 => Meld::koutsu_open(tile),
            2 => Meld::kan(tile, KanType::Open),
            3 => Meld::kan(tile, KanType::Closed),
            4 => Meld::kan(tile, KanType::Added),
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "meld kind must be 0..4 (chi,pon,daiminkan,ankan,kakan)",
                ))
            }
        });
    }
    let num_melds = called.len() as u8;

    let mut arr: [u8; 34] = closed.try_into().map_err(|v: Vec<u8>| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "expected 34-element closed array, got {}",
            v.len()
        ))
    })?;

    for w in 0..34usize {
        if arr[w] >= 4 {
            continue;
        }
        arr[w] += 1;
        let counts = shanten::array_to_tilecounts(&arr);
        if shanten::calculate_shanten_with_melds(&counts, num_melds).shanten == -1 {
            let winning_tile = shanten::index_to_tile(w);
            let ctx = GameContext::new(WinType::Ron, round_wind, seat_wind)
                .with_winning_tile(winning_tile)
                .open();
            for structure in decompose_hand_with_melds(&counts, &called) {
                let res = detect_yaku_with_context(&structure, &counts, &ctx);
                if res.is_yakuman || res.total_han >= 1 {
                    return Ok(true);
                }
            }
        }
        arr[w] -= 1;
    }
    Ok(false)
}

#[pymodule]
fn agari_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hand_has_yaku, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_shanten, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_ukeire, m)?)?;
    m.add_function(wrap_pyfunction!(compute_riichi_features, m)?)?;
    m.add_function(wrap_pyfunction!(batch_compute_riichi_features, m)?)?;
    m.add_function(wrap_pyfunction!(valid_chi_combinations, m)?)?;
    m.add_function(wrap_pyfunction!(shanten_after_chi, m)?)?;
    m.add_function(wrap_pyfunction!(shanten_after_kan, m)?)?;
    m.add_function(wrap_pyfunction!(is_permanent_furiten, m)?)?;
    Ok(())
}
