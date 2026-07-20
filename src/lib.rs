#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    clippy::perf,
    clippy::style,
    clippy::complexity,
    clippy::suspicious,
    //---
    clippy::manual_assert,
    clippy::semicolon_if_nothing_returned
    // clippy::manual_bit_width, // nightly only
)]
#![allow(clippy::len_without_is_empty)]

mod entropy;
mod error;
mod frozen;
mod hide;
mod map_decoder;
mod probs;
mod traits;
mod utils;
mod xoshiro128plusplus;

pub use error::NBPCError;
pub use hide::{hide_cpd, unhide_cpd};
pub use traits::{Float, Unsigned};

use crate::probs::{find_alpha, optimal_mod_probabilities};
use crate::xoshiro128plusplus::Xoshiro128PlusPlus;
use std::hint::black_box;
use std::time::{SystemTime, UNIX_EPOCH};

const BETA: f64 = 1.22;
const UNFROZEN: u8 = u8::MAX;
const GF4_ADD_GAMMA_MUL: [[u8; 4]; 4] = [[0, 2, 3, 1], [1, 3, 2, 0], [2, 0, 1, 3], [3, 1, 0, 2]];

#[derive(Clone, Copy)]
struct DibitView<'a>(&'a [u8]);

impl DibitView<'_> {
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.0.len() * 4
    }

    #[inline(always)]
    pub const fn get(&self, idx: usize) -> u8 {
        (self.0[idx / 4] >> ((3 - idx % 4) * 2)) & 3
    }
}

/// Some "random" data without depending on `rand`.
fn stfuep2() -> u32 {
    std::hint::black_box(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
            .wrapping_pow(2),
    )
}

pub fn dry_run(n: usize, l: usize, rel_payload: f64) {
    let mut rng = Xoshiro128PlusPlus::from_seed([stfuep2(), stfuep2(), stfuep2(), stfuep2()]);

    let cover: Vec<u8> = (0..n).map(|_| rng.random_bounded(256) as u8).collect();
    let mut rho_m1 = vec![0.0; n];
    let mut rho_p1 = vec![0.0; n];
    for i in 0..n {
        rho_m1[i] = f64::from(1 + rng.random_bounded(100));
        rho_p1[i] = rho_m1[i];

        if cover[i] == 0 {
            rho_m1[i] = 1e6;
        } else if cover[i] == 255 {
            rho_p1[i] = 1e6;
        }
    }
    let message: Vec<u8> = (0..(n as f64 * rel_payload / 8.0).round() as usize)
        .map(|_| rng.random_bounded(256) as u8)
        .collect();

    drop(black_box(
        hide_cpd(&cover, &rho_m1, &rho_p1, &message, l).unwrap(),
    ));
}

pub fn coding_loss<U: Unsigned>(
    cover: &[U],
    stego: &[U],
    rho_m1: &[f64],
    rho_p1: &[f64],
    k_bytes: usize,
) -> f64 {
    let alpha = find_alpha(rho_m1, rho_p1, (k_bytes * 4) as f64).unwrap();
    let d_actual: f64 = stego
        .iter()
        .zip(cover.iter())
        .zip(rho_m1.iter().zip(rho_p1))
        .map(|((&new, &old), (&rho_m1_el, &rho_p1_el))| {
            let delta = U::signed_diff(new, old);
            match delta {
                0 => 0.0,
                1 => rho_p1_el,
                -1 => rho_m1_el,
                _ => panic!("unexpected delta {delta}"),
            }
        })
        .sum();

    let d_theoretical: f64 = (optimal_mod_probabilities(rho_m1, rho_p1, alpha))
        .zip(rho_m1.iter().zip(rho_p1))
        .map(|(p, (&rho_m1_el, &rho_p1_el))| p[2].mul_add(rho_p1_el, p[0] * rho_m1_el))
        .sum();

    (d_actual - d_theoretical) / d_theoretical * 100.0
}

pub fn coding_loss_(cover: &[u8], rho_m1: &[f64], rho_p1: &[f64], message: &[u8], l: usize) -> f64 {
    let stego = hide_cpd(cover, rho_m1, rho_p1, message, l).unwrap();
    coding_loss(cover, &stego, rho_m1, rho_p1, message.len())
}

/// Function for experiments.
/// Calculates coding loss for random data with given `n`, `l` and `rel_payload`.
///
/// - `n`: cover length
/// - `l (1..=64)`: list size
/// - `rel_payload`: payload size in bits per cover symbol
pub fn coding_loss_simple(n: usize, l: usize, rel_payload: f64) -> f64 {
    let mut rng = Xoshiro128PlusPlus::from_seed([stfuep2(), stfuep2(), stfuep2(), stfuep2()]);

    let cover: Vec<u8> = (0..n).map(|_| rng.random_bounded(256) as u8).collect();
    let mut rho_m1 = vec![0.0; n];
    let mut rho_p1 = vec![0.0; n];
    for i in 0..n {
        rho_m1[i] = f64::from(1 + rng.random_bounded(100));
        rho_p1[i] = rho_m1[i];

        if cover[i] == 0 {
            rho_m1[i] = 1e6;
        } else if cover[i] == 255 {
            rho_p1[i] = 1e6;
        }
    }
    let message: Vec<u8> = (0..(n as f64 * rel_payload / 8.0).round() as usize)
        .map(|_| rng.random_bounded(256) as u8)
        .collect();

    let stego = hide_cpd(&cover, &rho_m1, &rho_p1, &message, l).unwrap();

    coding_loss(&cover, &stego, &rho_m1, &rho_p1, message.len())
}

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(name = "hide_cpd")]
fn hide_cpd_py(
    cover: Vec<u8>,
    rho_m1: Vec<f64>,
    rho_p1: Vec<f64>,
    message: Vec<u8>,
    l: usize,
) -> PyResult<Vec<u8>> {
    let result = hide_cpd(&cover, &rho_m1, &rho_p1, &message, l).unwrap();
    Ok(result)
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(name = "unhide_cpd")]
fn unhide_cpd_py(stego: Vec<u8>, k_bytes: u32) -> PyResult<Vec<u8>> {
    let result = unhide_cpd(&stego, k_bytes);
    Ok(result)
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(name = "coding_loss")]
fn coding_loss_py(
    cover: Vec<u8>,
    stego: Vec<u8>,
    rho_m1: Vec<f64>,
    rho_p1: Vec<f64>,
    k_bytes: usize,
) -> PyResult<f64> {
    let result = coding_loss(&cover, &stego, &rho_m1, &rho_p1, k_bytes);
    Ok(result)
}

#[cfg(feature = "pyo3")]
#[pymodule]
fn nbpc_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hide_cpd_py, m)?)?;
    m.add_function(wrap_pyfunction!(unhide_cpd_py, m)?)?;
    m.add_function(wrap_pyfunction!(coding_loss_py, m)?)?;
    Ok(())
}
