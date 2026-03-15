use pyo3::prelude::*;

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

#[pymodule]
fn agari_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate_shanten, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_ukeire, m)?)?;
    m.add_function(wrap_pyfunction!(compute_riichi_features, m)?)?;
    m.add_function(wrap_pyfunction!(batch_compute_riichi_features, m)?)?;
    Ok(())
}
