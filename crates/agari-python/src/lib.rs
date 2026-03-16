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

#[pyfunction]
fn valid_chi_combinations(hand: Vec<u8>, discarded_tile: u8) -> PyResult<Vec<(u8, u8)>> {
    if hand.len() != 34 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("expected 34-element array, got {}", hand.len()),
        ));
    }
    let d = discarded_tile as usize;
    if d > 26 {
        return Ok(vec![]);
    }
    let val = d % 9;
    let mut combos = Vec::new();
    // Discarded tile is the HIGH end of sequence: (d-2, d-1, d)
    if val >= 2 && hand[d - 2] > 0 && hand[d - 1] > 0 {
        combos.push(((d - 2) as u8, (d - 1) as u8));
    }
    // Discarded tile is the MIDDLE of sequence: (d-1, d, d+1)
    if val >= 1 && val <= 7 && hand[d - 1] > 0 && hand[d + 1] > 0 {
        combos.push(((d - 1) as u8, (d + 1) as u8));
    }
    // Discarded tile is the LOW end of sequence: (d, d+1, d+2)
    if val <= 6 && hand[d + 1] > 0 && hand[d + 2] > 0 {
        combos.push(((d + 1) as u8, (d + 2) as u8));
    }
    Ok(combos)
}

#[pyfunction]
fn shanten_after_chi(
    hand: Vec<u8>,
    _discarded_tile: u8,
    combo: (u8, u8),
    num_melds: u8,
) -> PyResult<i8> {
    let mut arr: [u8; 34] = hand
        .try_into()
        .map_err(|v: Vec<u8>| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("expected 34-element array, got {}", v.len()),
        ))?;
    arr[combo.0 as usize] = arr[combo.0 as usize].checked_sub(1).ok_or_else(||
        PyErr::new::<pyo3::exceptions::PyValueError, _>("combo tile not in hand")
    )?;
    arr[combo.1 as usize] = arr[combo.1 as usize].checked_sub(1).ok_or_else(||
        PyErr::new::<pyo3::exceptions::PyValueError, _>("combo tile not in hand")
    )?;
    let new_melds = num_melds + 1;
    let mut best = i8::MAX;
    for i in 0..34 {
        if arr[i] > 0 {
            arr[i] -= 1;
            let counts = shanten::array_to_tilecounts(&arr);
            let s = shanten::calculate_shanten_with_melds(&counts, new_melds).shanten;
            if s < best {
                best = s;
            }
            arr[i] += 1;
        }
    }
    Ok(best)
}

#[pyfunction]
fn shanten_after_kan(
    hand: Vec<u8>,
    kan_tile: u8,
    kan_type: u8,
    num_melds: u8,
) -> PyResult<i8> {
    let mut arr: [u8; 34] = hand
        .try_into()
        .map_err(|v: Vec<u8>| PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("expected 34-element array, got {}", v.len()),
        ))?;
    let kt = kan_tile as usize;
    let (remove, melds) = match kan_type {
        0 => (3u8, num_melds + 1), // open (daiminkan)
        1 => (4u8, num_melds + 1), // closed (ankan)
        2 => (1u8, num_melds),     // added (kakan)
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "kan_type must be 0 (open), 1 (closed), or 2 (added)",
        )),
    };
    arr[kt] = arr[kt].checked_sub(remove).ok_or_else(||
        PyErr::new::<pyo3::exceptions::PyValueError, _>("not enough tiles in hand for kan")
    )?;
    let counts = shanten::array_to_tilecounts(&arr);
    Ok(shanten::calculate_shanten_with_melds(&counts, melds).shanten)
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
