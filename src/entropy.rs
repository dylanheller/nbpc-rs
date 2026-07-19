//! Efficient implementation of entropy function
//! merged with the probabilities function
//! from PLS theory[^pls].
//!
//! [^pls]: T. Filler, J. Judas, and J. Fridrich, "Minimizing additive distortion in steganography using Syndrome-Trellis codes" (doi: 10.1109/tifs.2011.2134094)

use std::{
    num::NonZero,
    thread::{self, ScopedJoinHandle},
};
#[cfg(feature = "wide")]
use wide::f64x8;

const LN4_INV: f64 = 0.7213475204444817;

pub fn entropy(rho_m1: &[f64], rho_p1: &[f64], alpha: f64) -> f64 {
    #[cfg(feature = "wide")]
    return entropy_wide(rho_m1, rho_p1, alpha);
    #[cfg(not(feature = "wide"))]
    return entropy_scalar(rho_m1, rho_p1, alpha);
}

#[cfg(feature = "wide")]
pub fn entropy_wide(rho_m1: &[f64], rho_p1: &[f64], alpha: f64) -> f64 {
    let (rho_m1_blocks, rho_m1_rem) = rho_m1.as_chunks::<8>();
    let (rho_p1_blocks, rho_p1_rem) = rho_p1.as_chunks::<8>();

    let threads_res = thread::available_parallelism().map(NonZero::get);

    let total = if rho_m1.len() < 1 << 14 {
        entropy_wide_chunk(rho_m1_blocks, rho_p1_blocks, alpha)
    } else if let Ok(threads) = threads_res {
        let chunk_size = rho_m1_blocks.len().div_ceil(threads);
        let rho_m1_chunks = rho_m1_blocks.chunks(chunk_size);
        let rho_p1_chunks = rho_p1_blocks.chunks(chunk_size);

        thread::scope(|s| {
            let threads: Vec<ScopedJoinHandle<f64>> = rho_m1_chunks
                .zip(rho_p1_chunks)
                .map(|(rho_m1_blocks, rho_p1_blocks)| {
                    s.spawn(|| entropy_wide_chunk(rho_m1_blocks, rho_p1_blocks, alpha))
                })
                .collect();

            threads
                .into_iter()
                .map(|t| t.join().expect("`entropy_wide_chunk` can't panic"))
                .sum::<f64>()
        })
    } else {
        entropy_wide_chunk(rho_m1_blocks, rho_p1_blocks, alpha)
    };

    total + entropy_scalar_chunk(rho_m1_rem, rho_p1_rem, alpha)
}

#[cfg(feature = "wide")]
fn entropy_wide_chunk(rho_m1_blocks: &[[f64; 8]], rho_p1_blocks: &[[f64; 8]], alpha: f64) -> f64 {
    -rho_m1_blocks
        .iter()
        .zip(rho_p1_blocks.iter())
        .map(|(rho_m1_block, rho_p1_block)| {
            let rho_m1 = f64x8::new(*rho_m1_block);
            let rho_p1 = f64x8::new(*rho_p1_block);

            let t_m1 = -alpha * rho_m1;
            let t_p1 = -alpha * rho_p1;

            let e_m1 = t_m1.exp();
            let e_p1 = t_p1.exp();

            let z = e_m1 + 1.0 + e_p1;
            let z_ln = z.ln();

            (-z_ln + e_m1 * (t_m1 - z_ln) + e_p1 * (t_p1 - z_ln)) / z
        })
        .sum::<f64x8>()
        .reduce_add()
        * LN4_INV
}

#[allow(dead_code)]
pub fn entropy_scalar(rho_m1: &[f64], rho_p1: &[f64], alpha: f64) -> f64 {
    let threads_res = thread::available_parallelism().map(NonZero::get);

    if rho_m1.len() < 1 << 14 {
        entropy_scalar_chunk(rho_m1, rho_p1, alpha)
    } else if let Ok(threads) = threads_res {
        let chunk_size = rho_m1.len().div_ceil(threads);
        let rho_m1_chunks = rho_m1.chunks(chunk_size);
        let rho_p1_chunks = rho_p1.chunks(chunk_size);

        thread::scope(|s| {
            let threads: Vec<ScopedJoinHandle<f64>> = rho_m1_chunks
                .zip(rho_p1_chunks)
                .map(|(rho_m1_chunk, rho_p1_chunk)| {
                    s.spawn(|| entropy_scalar_chunk(rho_m1_chunk, rho_p1_chunk, alpha))
                })
                .collect();

            threads
                .into_iter()
                .map(|t| t.join().expect("`entropy_scalar_chunk` can't panic"))
                .sum::<f64>()
        })
    } else {
        entropy_scalar_chunk(rho_m1, rho_p1, alpha)
    }
}

pub fn entropy_scalar_chunk(rho_m1: &[f64], rho_p1: &[f64], alpha: f64) -> f64 {
    -rho_m1
        .iter()
        .zip(rho_p1.iter())
        .map(|(m1, p1)| {
            let t_m1 = -alpha * m1;
            let t_p1 = -alpha * p1;

            let e_m1 = t_m1.exp();
            let e_p1 = t_p1.exp();

            let z = e_m1 + 1.0 + e_p1;
            let z_ln = z.ln();

            (-z_ln + e_m1 * (t_m1 - z_ln) + e_p1 * (t_p1 - z_ln)) / z
        })
        .sum::<f64>()
        * LN4_INV
}

mod tests {
    #[test]
    #[cfg(feature = "wide")]
    fn equality() {
        use crate::entropy::{entropy_scalar, entropy_wide};

        let rho = &[34.6; 1 << 16];
        let diff = (entropy_scalar(rho, rho, 0.67) - entropy_wide(rho, rho, 0.67)).abs();

        println!("Diff between `entropy_scalar` and `entropy_wide`: {diff}");
        assert!(diff < 1e-12);
    }
}
