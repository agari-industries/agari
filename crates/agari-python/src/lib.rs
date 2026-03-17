use pyo3::prelude::*;

use agari::hand::KanType;
use agari::parse::TileCounts;
use agari::shanten;

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

#[pymodule]
fn agari_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
