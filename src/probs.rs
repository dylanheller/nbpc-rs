use crate::xoshiro128plusplus::Xoshiro128PlusPlus;
use crate::{entropy::entropy, error::NBPCError, traits::Unsigned, utils::fisher_yates};
use std::iter;

pub struct InputPMFs {
    zv: Vec<u8>,
    pv_m1: Vec<f64>,
    pv_0: Vec<f64>,
}

impl InputPMFs {
    pub fn from(iter: impl Iterator<Item = (u8, [f64; 3])>, capacity: usize) -> Self {
        let mut zv = Vec::with_capacity(capacity);
        let mut pv_m1 = Vec::with_capacity(capacity);
        let mut pv_0 = Vec::with_capacity(capacity);

        for (cel_lsb2, pel) in iter {
            let z = (cel_lsb2 + 6) & 0x3;

            let [p_m1, p_0, _] = pel;

            zv.push(z);
            pv_m1.push(p_m1);
            pv_0.push(p_0);
        }

        Self { zv, pv_m1, pv_0 }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.zv.len()
    }

    pub fn extend_with_zeros(&mut self, eng: usize) {
        self.zv.extend(iter::repeat_n(2, eng));
        self.pv_m1.extend(iter::repeat_n(0.0, eng));
        self.pv_0.extend(iter::repeat_n(1.0, eng));
    }

    pub fn scramble(&mut self, xst: Xoshiro128PlusPlus) {
        fisher_yates(&mut self.zv, xst);
        fisher_yates(&mut self.pv_m1, xst);
        fisher_yates(&mut self.pv_0, xst);
    }

    pub fn iter_pairs(&self) -> impl Iterator<Item = [[f64; 4]; 2]> {
        let (zv_chunks, _) = self.zv.as_chunks::<2>();
        let (pv_m1_chunks, _) = self.pv_m1.as_chunks::<2>();
        let (pv_0_chunks, _) = self.pv_0.as_chunks::<2>();

        zv_chunks
            .iter()
            .zip(pv_m1_chunks.iter().zip(pv_0_chunks))
            .map(|(z, (p_m1, p_0))| {
                let mut prob0 = [0.0; 4];
                let mut prob1 = [0.0; 4];

                prob0[(usize::from(z[0]) + 1) & 3] = p_m1[0];
                prob0[(usize::from(z[0]) + 2) & 3] = p_0[0];
                prob0[(usize::from(z[0]) + 3) & 3] = (1.0_f64 - p_m1[0] - p_0[0]).clamp(0.0, 1.0);

                prob1[(usize::from(z[1]) + 1) & 3] = p_m1[1];
                prob1[(usize::from(z[1]) + 2) & 3] = p_0[1];
                prob1[(usize::from(z[1]) + 3) & 3] = (1.0_f64 - p_m1[1] - p_0[1]).clamp(0.0, 1.0);

                [prob0, prob1]
            })
    }
}

pub fn find_alpha(rho_m1: &[f64], rho_p1: &[f64], target_entropy: f64) -> Result<f64, NBPCError> {
    let n = rho_m1.len();
    assert_eq!(rho_p1.len(), n);

    let f = |alpha| entropy(rho_m1, rho_p1, alpha) - target_entropy;

    let mut a = 0.0;
    let mut b = 128.0;

    if f(a).is_sign_negative() || f(1e50).is_sign_positive() {
        return Err(NBPCError::NoInputPMFs);
    }

    let mut fa = f(a);
    let mut fb = f(b);
    while fb > 0.0 {
        b *= 2.0;
        fb = f(b);
    }

    let mut c = a;
    let mut e = b - a;
    let mut d = e;
    let mut fc = fa;

    for _ in 0..100 {
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }

        let tol = (2.0 * f64::EPSILON).mul_add(b.abs(), 1e-4 /* <- t */);
        let m = 0.5 * (c - b);

        if m.abs() <= tol || fb == 0.0 {
            return Ok(b);
        }

        if e.abs() < tol || fa.abs() <= fb.abs() {
            e = m;
            d = e;
        } else {
            let mut p;
            let mut q;
            let mut s = fb / fa;

            if a == c {
                p = 2.0 * m * s;
                q = 1.0 - s;
            } else {
                q = fa / fc;
                let r = fb / fc;
                p = s * (2.0 * m * q * (q - r) - (b - a) * (r - 1.0));
                q = (q - 1.0) * (r - 1.0) * (s - 1.0);
            }

            if 0.0 < p {
                q = -q;
            } else {
                p = -p;
            }

            s = e;
            e = d;

            if 2.0 * p < 3.0 * m * q - (tol * q).abs() && p < (0.5 * s * q).abs() {
                d = p / q;
            } else {
                e = m;
                d = e;
            }
        }

        a = b;
        fa = fb;

        if tol < d.abs() {
            b += d;
        } else if 0.0 < m {
            b += tol;
        } else {
            b -= tol;
        }

        fb = f(b);

        if (0.0 < fb && 0.0 < fc) || (fb <= 0.0 && fc <= 0.0) {
            c = a;
            fc = fa;
            e = b - a;
            d = e;
        }
    }

    Err(NBPCError::NoInputPMFs)
}

pub fn optimal_mod_probabilities(
    rho_m1: &[f64],
    rho_p1: &[f64],
    alpha: f64,
) -> impl Iterator<Item = [f64; 3]> {
    let n = rho_m1.len();
    assert_eq!(rho_p1.len(), n);

    let rho_iter = rho_m1.iter().zip(rho_p1);

    rho_iter.map(move |(rho_m1_el, rho_p1_el)| {
        let t_m1 = -alpha * rho_m1_el;
        let t_p1 = -alpha * rho_p1_el;

        let e_m1 = t_m1.exp();
        let e_0 = 1.0;
        let e_p1 = t_p1.exp();

        let z_inv = 1.0 / (e_m1 + e_0 + e_p1);

        let p_m1 = e_m1 * z_inv;
        let p_0 = e_0 * z_inv;
        let p_p1 = e_p1 * z_inv;

        [p_m1, p_0, p_p1]
    })
}

pub fn optimal_pmfs<U: Unsigned>(
    cover: &[U],
    rho_m1: &[f64],
    rho_p1: &[f64],
    alpha: f64,
) -> InputPMFs {
    assert_eq!(rho_m1.len(), rho_p1.len());

    #[cfg(feature = "wide")]
    let prob_iter = optimal_pmfs_wide(cover, rho_m1, rho_p1, alpha);
    #[cfg(not(feature = "wide"))]
    let prob_iter = optimal_pmfs_scalar(cover, rho_m1, rho_p1, alpha);

    InputPMFs::from(prob_iter, rho_m1.len())
}

#[cfg(feature = "wide")]
fn optimal_pmfs_wide<U: Unsigned>(
    cover: &[U],
    rho_m1: &[f64],
    rho_p1: &[f64],
    alpha: f64,
) -> impl Iterator<Item = (u8, [f64; 3])> {
    let (cover_chunks, cover_remainder) = cover.as_chunks::<8>();
    let (rho_m1_chunks, rho_m1_remainder) = rho_m1.as_chunks::<8>();
    let (rho_p1_chunks, rho_p1_remainder) = rho_p1.as_chunks::<8>();

    let rho_iter = rho_m1_chunks.iter().zip(rho_p1_chunks).zip(cover_chunks);

    rho_iter
        .flat_map(move |((rho_m1_chunk, rho_p1_chunk), sym)| {
            use wide::f64x8;

            let rho_m1 = f64x8::new(*rho_m1_chunk);
            let rho_p1 = f64x8::new(*rho_p1_chunk);

            let t_m1 = -alpha * rho_m1;
            let t_p1 = -alpha * rho_p1;

            let e_m1 = t_m1.exp();
            let e_0 = 1.0;
            let e_p1 = t_p1.exp();

            let z_inv = 1.0 / (e_m1 + e_0 + e_p1);

            let p_m1_arr = (e_m1 * z_inv).to_array();
            let p_0_arr = (e_0 * z_inv).to_array();
            let p_p1_arr = (e_p1 * z_inv).to_array();

            [
                (sym[0].lsb2_u8(), [p_m1_arr[0], p_0_arr[0], p_p1_arr[0]]),
                (sym[1].lsb2_u8(), [p_m1_arr[1], p_0_arr[1], p_p1_arr[1]]),
                (sym[2].lsb2_u8(), [p_m1_arr[2], p_0_arr[2], p_p1_arr[2]]),
                (sym[3].lsb2_u8(), [p_m1_arr[3], p_0_arr[3], p_p1_arr[3]]),
                (sym[4].lsb2_u8(), [p_m1_arr[4], p_0_arr[4], p_p1_arr[4]]),
                (sym[5].lsb2_u8(), [p_m1_arr[5], p_0_arr[5], p_p1_arr[5]]),
                (sym[6].lsb2_u8(), [p_m1_arr[6], p_0_arr[6], p_p1_arr[6]]),
                (sym[7].lsb2_u8(), [p_m1_arr[7], p_0_arr[7], p_p1_arr[7]]),
            ]
        })
        .chain(optimal_pmfs_scalar(
            cover_remainder,
            rho_m1_remainder,
            rho_p1_remainder,
            alpha,
        ))
}

fn optimal_pmfs_scalar<U: Unsigned>(
    cover: &[U],
    rho_m1: &[f64],
    rho_p1: &[f64],
    alpha: f64,
) -> impl Iterator<Item = (u8, [f64; 3])> {
    let rho_iter = rho_m1.iter().zip(rho_p1).zip(cover);

    rho_iter.map(move |((rho_m1_el, rho_p1_el), sym)| {
        let t_m1 = -alpha * rho_m1_el;
        let t_p1 = -alpha * rho_p1_el;

        let e_m1 = t_m1.exp();
        let e_0 = 1.0;
        let e_p1 = t_p1.exp();

        let z_inv = 1.0 / (e_m1 + e_0 + e_p1);

        let p_m1 = e_m1 * z_inv;
        let p_0 = e_0 * z_inv;
        let p_p1 = e_p1 * z_inv;

        (sym.lsb2_u8(), [p_m1, p_0, p_p1])
    })
}
