use crate::map_decoder::{
    active_path::{ActivePathDecisions, ActivePathProbs},
    get_decisions,
    pool::Pool,
    set_decision,
};
use crate::probs::InputPMFs;
use crate::utils::{argmax4, bit_reverse};
use crate::{GF4_ADD_GAMMA_MUL, UNFROZEN};

/// State of one MAP-SC Decoder.
#[derive(Clone)]
pub struct PolarState<'a> {
    m: usize,
    pub pw: f64,
    probs: ActivePathProbs<'a>,
    input_pmfs: &'a InputPMFs,
    decisions: ActivePathDecisions<'a>,
}

impl<'a> PolarState<'a> {
    pub fn new(
        m: usize,
        input_pmfs: &'a InputPMFs,
        pool_probs: &'a Pool<[f64; 4]>,
        pool_decisions: &'a Pool<u8>,
    ) -> Self {
        let probs = ActivePathProbs::new(m, pool_probs);
        let decisions = ActivePathDecisions::new(m, pool_decisions);

        Self {
            m,
            pw: 0.0,
            probs,
            input_pmfs,
            decisions,
        }
    }

    /// MAP-SC Decoder.
    ///
    /// You can find a simple and explained implementation
    /// in C version of this library.
    pub fn greedy_pass(input_pmfs: &'a InputPMFs, frozen_mask: &[u8]) -> Box<[u8]> {
        let n = input_pmfs.len();
        assert!(n.is_power_of_two());
        assert_eq!(n, frozen_mask.len());
        let m = n.ilog2() as usize;

        let probs = ActivePathProbs::owned(m);
        let decisions = ActivePathDecisions::owned(m);

        let mut state = Self {
            m,
            pw: 0.0,
            probs,
            input_pmfs,
            decisions,
        };

        let mut u_hat_bitrev = Box::new_uninit_slice(n);
        let mut skipped_depth = 0;

        for (phi_0, &frozen_sym) in frozen_mask.iter().enumerate() {
            if frozen_sym == UNFROZEN {
                state.backward_deduce(phi_0, skipped_depth);
                skipped_depth = 0;
                let decision = argmax4(&state.probs[0][0]);
                u_hat_bitrev[bit_reverse(phi_0, m as u32)].write(decision);

                if phi_0 & 1 == 1 && phi_0 < n - 1 {
                    set_decision(&mut state.decisions[0][0], decision, 1);
                    state.forward_propagate(phi_0);
                } else {
                    set_decision(&mut state.decisions[0][0], decision, 0);
                }
            } else {
                skipped_depth = Self::deduction_depth(phi_0, m).max(skipped_depth);
                u_hat_bitrev[bit_reverse(phi_0, m as u32)].write(frozen_sym);

                if phi_0 & 1 == 1 && phi_0 < n - 1 {
                    set_decision(&mut state.decisions[0][0], frozen_sym, 1);
                    state.forward_propagate(phi_0);
                } else {
                    set_decision(&mut state.decisions[0][0], frozen_sym, 0);
                }
            }
        }

        unsafe { u_hat_bitrev.assume_init() }
    }

    pub fn decision_candidates(&mut self, phi_0: usize, id: usize) -> [(f64, u8, usize); 4] {
        self.backward_deduce(phi_0, 0);

        [
            (self.pw - self.probs[0][0][0].ln(), 0, id),
            (self.pw - self.probs[0][0][1].ln(), 1, id),
            (self.pw - self.probs[0][0][2].ln(), 2, id),
            (self.pw - self.probs[0][0][3].ln(), 3, id),
        ]
    }

    pub fn make_decision(&mut self, phi_0: usize, decision: u8, new_pw: f64) {
        self.pw = new_pw;

        if phi_0 & 1 == 1 && phi_0 < (1 << self.m) - 1 {
            set_decision(&mut self.decisions[0][0], decision, 1);
            self.forward_propagate(phi_0);
        } else {
            set_decision(&mut self.decisions[0][0], decision, 0);
        }
    }

    pub fn make_frozen_decision(&mut self, phi_0: usize, decision: u8) {
        self.backward_deduce(phi_0, 0);

        self.pw -= self.probs[0][0][decision as usize]
            .max(f64::MIN_POSITIVE)
            .ln();

        if phi_0 & 1 == 1 && phi_0 < (1 << self.m) - 1 {
            set_decision(&mut self.decisions[0][0], decision, 1);
            self.forward_propagate(phi_0);
        } else {
            set_decision(&mut self.decisions[0][0], decision, 0);
        }
    }

    #[inline(always)]
    const fn deduce_even(pim: &[f64; 4], pip: &[f64; 4]) -> [f64; 4] {
        [
            pim[0].mul_add(
                pip[0],
                pim[2].mul_add(pip[1], pim[3].mul_add(pip[2], pim[1] * pip[3])),
            ),
            pim[1].mul_add(
                pip[0],
                pim[3].mul_add(pip[1], pim[2].mul_add(pip[2], pim[0] * pip[3])),
            ),
            pim[2].mul_add(
                pip[0],
                pim[0].mul_add(pip[1], pim[1].mul_add(pip[2], pim[3] * pip[3])),
            ),
            pim[3].mul_add(
                pip[0],
                pim[1].mul_add(pip[1], pim[0].mul_add(pip[2], pim[2] * pip[3])),
            ),
        ]
    }

    #[inline(always)]
    const fn deduce_odd(pim: &[f64; 4], pip: &[f64; 4], uim: usize) -> [f64; 4] {
        let p0 = pim[GF4_ADD_GAMMA_MUL[uim][0] as usize] * pip[0];
        let p1 = pim[GF4_ADD_GAMMA_MUL[uim][1] as usize] * pip[1];
        let p2 = pim[GF4_ADD_GAMMA_MUL[uim][2] as usize] * pip[2];
        let p3 = pim[GF4_ADD_GAMMA_MUL[uim][3] as usize] * pip[3];
        let sum = p0 + p1 + p2 + p3;
        // Avoiding `inf` in `sum.recip()`.
        if sum < 5.562684646268003e-309 {
            [0.25; 4]
        } else {
            let sum_recip = sum.recip();
            [
                p0 * sum_recip,
                p1 * sum_recip,
                p2 * sum_recip,
                p3 * sum_recip,
            ]
        }
    }

    #[inline(always)]
    fn deduction_depth(phi_0: usize, m: usize) -> usize {
        (phi_0.trailing_zeros() as usize).min(m - 1)
    }

    fn backward_deduce(&mut self, phi_0: usize, skipped_depth: usize) {
        let depth = Self::deduction_depth(phi_0, self.m).max(skipped_depth);

        for lambda in (0..=depth).rev() {
            let phi = phi_0 >> lambda;
            let (probs, probs_next) = self.probs.borrow_layers_pair_discard(lambda);

            if let Some(probs_next) = probs_next {
                let (probs_next_pairs, _) = probs_next.as_chunks::<2>();

                for ((prob, [pim, pip]), &uix) in probs
                    .iter_mut()
                    .zip(probs_next_pairs)
                    .zip(&self.decisions[lambda])
                {
                    if phi.is_multiple_of(2) {
                        *prob = Self::deduce_even(pim, pip);
                    } else {
                        let (uim, _) = get_decisions(uix);
                        *prob = Self::deduce_odd(pim, pip, uim as usize);
                    }
                }
            } else {
                let probs_next_pairs = self.input_pmfs.iter_pairs();

                for ((prob, [pim, pip]), &uix) in probs
                    .iter_mut()
                    .zip(probs_next_pairs)
                    .zip(&self.decisions[lambda])
                {
                    if phi & 1 == 0 {
                        *prob = Self::deduce_even(&pim, &pip);
                    } else {
                        let (uim, _) = get_decisions(uix);
                        *prob = Self::deduce_odd(&pim, &pip, uim as usize);
                    }
                }
            }
        }
    }

    fn forward_propagate(&mut self, phi_0: usize) {
        for lambda in 0..phi_0.trailing_ones() as usize {
            let phi = phi_0 >> lambda;

            let (decisions, decisions_next) = self.decisions.borrow_layers_pair(lambda);
            let (decisions_next_pairs, _) = decisions_next.as_chunks_mut::<2>();

            for (&uix, uox_pair) in decisions.iter().zip(decisions_next_pairs) {
                let (uim, uip) = get_decisions(uix);

                let uom = GF4_ADD_GAMMA_MUL[uim as usize][uip as usize];
                let uop = uip;

                let oddness_bit = (phi / 2) & 1;
                set_decision(&mut uox_pair[0], uom, oddness_bit);
                set_decision(&mut uox_pair[1], uop, oddness_bit);
            }
        }
    }
}
