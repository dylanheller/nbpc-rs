use crate::{DibitView, UNFROZEN};

pub fn compute_beta_expansion(m: u8, beta: f64) -> Vec<f64> {
    let n = 1_usize.strict_shl(u32::from(m));
    let mut weights = vec![0.0; n];

    for p in 0..m {
        let step = 1 << p;
        let power = beta.powi(i32::from(p));

        let (l_half, r_half) = weights[..2 * step].split_at_mut(step);

        for (l, r) in l_half.iter().zip(r_half) {
            *r = l + power;
        }
    }

    weights
}

pub fn get_frozen_mask(msg: DibitView<'_>, m: u8, beta: f64) -> Vec<u8> {
    let n = 1_usize.strict_shl(u32::from(m));
    let k = msg.len();

    assert!(k <= n, "`msg` can't be longer than the cover");

    let bexp = compute_beta_expansion(m, beta);
    let mut idx: Vec<usize> = (0..n).collect();

    let (idx_k, ..) = idx.select_nth_unstable_by(k, |&i, &j| unsafe {
        bexp.get_unchecked(i).total_cmp(bexp.get_unchecked(j))
    });
    idx_k.sort_unstable_by(|&i, &j| unsafe {
        bexp.get_unchecked(i).total_cmp(bexp.get_unchecked(j))
    });

    let mut frozen_mask = vec![UNFROZEN; n];

    for (mi, &i) in idx_k.iter().enumerate() {
        unsafe {
            *frozen_mask.get_unchecked_mut(i) = msg.get(mi);
        }
    }

    frozen_mask
}
