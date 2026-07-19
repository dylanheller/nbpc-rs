use crate::error::NBPCError;
use crate::frozen::{compute_beta_expansion, get_frozen_mask};
use crate::hide::normalize_cost_map;
use crate::map_decoder::decode_map;
use crate::probs::{find_alpha, optimal_pmfs};
use crate::traits::{Float, Unsigned};
use crate::utils::{bit_reverse, dibits_to_bytes, fisher_yates, polar_encode, revert_order};
use crate::xoshiro128plusplus::Xoshiro128PlusPlus;
use crate::{BETA, DibitView};
use std::iter;

#[cfg(feature = "args_validation")]
use crate::hide::validate_args;

pub fn hide_cpd<U: Unsigned, F: Float>(
    cover: &[U],
    rho_m1: &[F],
    rho_p1: &[F],
    msg_bytes: &[u8],
    l: usize,
) -> Result<Vec<U>, NBPCError> {
    let xst = Xoshiro128PlusPlus::default();
    let msg = DibitView(msg_bytes);

    let n = cover.len();
    let k = msg.len();

    #[cfg(feature = "args_validation")]
    validate_args(cover, rho_m1, rho_p1, k, l)?;

    if k == 0 {
        return Ok(cover.to_vec());
    }

    let (rho_m1, rho_p1) = normalize_cost_map(rho_m1, rho_p1);

    let m_prime = u8::try_from((n - 1).bit_width()).unwrap();
    let n_prime = 1 << m_prime;
    let eng = n_prime - n;

    let frozen_mask = get_frozen_mask(msg, m_prime, BETA);

    let alpha = find_alpha(&rho_m1, &rho_p1, k as f64)?;
    let mut input_pmfs = optimal_pmfs(cover, &rho_m1, &rho_p1, alpha);
    input_pmfs.extend_with_zeros(eng);
    input_pmfs.scramble(xst);

    drop(rho_m1);
    drop(rho_p1);

    let mut u_hat_bitrev_ordered = decode_map(l, &input_pmfs, &frozen_mask);
    polar_encode(&mut u_hat_bitrev_ordered);
    let y_ordered = u_hat_bitrev_ordered;
    let mut y = revert_order(&y_ordered, xst).into_vec();

    let n_corrupted = y.drain(n..).map(|a| usize::from(a != 0)).sum();
    if n_corrupted > 0 {
        return Err(NBPCError::PaddingCorruption(n_corrupted));
    }

    cover
        .iter()
        .zip(&y)
        .map(|(&c, &s)| {
            let old_symbol = i16::from(c.lsb2_u8());
            let new_symbol = i16::from(s.lsb2_u8());

            let delta = ((new_symbol + 4 - old_symbol) % 4).abs();
            match delta {
                0 => Some(c),
                1 => c.add_1_checked(),
                3 => c.sub_1_checked(),
                _ => panic!("unexpected delta {delta}"),
            }
        })
        .collect::<Option<Vec<U>>>()
        .ok_or(NBPCError::OverflowingModification)
}

pub fn unhide_cpd<U: Unsigned>(stego: &[U], k_bytes: u32) -> Vec<u8> {
    if k_bytes == 0 {
        return vec![];
    }

    let mut y: Vec<u8> = stego.iter().map(|a| a.lsb2_u8()).collect();

    let n = y.len();
    let k = usize::try_from(k_bytes).unwrap() * 4;

    assert!(k <= n, "{k} {n}");

    let m_prime = u8::try_from((n - 1).bit_width()).unwrap();
    let n_prime = 1 << m_prime;
    let eng = n_prime - n;

    y.extend(iter::repeat_n(0, eng));
    fisher_yates(&mut y, Xoshiro128PlusPlus::default());

    polar_encode(&mut y);

    let mut u_hat = vec![0; n_prime];
    for i in 0..n_prime {
        u_hat[bit_reverse(i, m_prime.into())] = y[i];
    }

    let weights = compute_beta_expansion(m_prime, BETA);

    let mut idx: Vec<usize> = (0..weights.len()).collect();
    idx.sort_unstable_by(|&i, &j| weights[i].total_cmp(&weights[j]));

    let mut msg = vec![];
    for i in idx.into_iter().take(k) {
        msg.push(u_hat[i]);
    }

    dibits_to_bytes(&msg)
}
