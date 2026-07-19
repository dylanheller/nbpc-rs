use crate::UNFROZEN;
use crate::map_decoder::{decision_history::DecisionHistory, pool::Pool, state::PolarState};
use crate::probs::InputPMFs;
use crate::utils::{argmax_iter, bit_reverse};
use std::mem::swap;

pub fn decode_map(l: usize, input_pmfs: &InputPMFs, frozen_mask: &[u8]) -> Box<[u8]> {
    assert!(l > 0 && l <= 64, "`l` must be > 0 and <= 64");
    let n = input_pmfs.len();
    assert!(
        n.is_power_of_two(),
        "`input_pmfs.len()` must be a power of two"
    );
    assert!(
        n == frozen_mask.len(),
        "`input_pmfs.len()` and `frozen_mask.len()` must be emual"
    );
    let m = n.ilog2() as usize;

    if l == 1 {
        return PolarState::greedy_pass(input_pmfs, frozen_mask);
    }

    let pool_probs = Pool::new(m, l);
    let pool_decisions = Pool::new(m, l);
    let mut states = vec![PolarState::new(m, input_pmfs, &pool_probs, &pool_decisions)];
    states.reserve(l);

    let mut new_states = Vec::with_capacity(l);
    let mut owned_states: Vec<Option<PolarState>> = Vec::with_capacity(l);
    let mut candidates: Vec<(f64, u8, usize)> = Vec::with_capacity(4 * l);

    let mut decision_hist = DecisionHistory::zeros(l, n);

    for (phi, &frozen_sym) in frozen_mask.iter().enumerate() {
        if frozen_sym == UNFROZEN {
            candidates.extend(
                states
                    .iter_mut()
                    .enumerate()
                    .flat_map(|(i, state)| state.decision_candidates(phi, i)),
            );
            candidates.select_nth_unstable_by(states.len(), |a, b| a.0.total_cmp(&b.0));

            owned_states.extend(states.drain(..).map(Some));

            let mut usage = [0; 64];
            for c in candidates.iter().take(l) {
                usage[c.2] += 1;
            }

            for (j, c) in candidates.iter().take(l).enumerate() {
                let mut st = if usage[c.2] == 1 {
                    owned_states[c.2].take().unwrap()
                } else {
                    usage[c.2] -= 1;
                    owned_states[c.2].as_ref().unwrap().clone()
                };

                st.make_decision(phi, c.1, c.0);
                decision_hist.set(phi, j, (u8::try_from(c.2).unwrap(), c.1));
                new_states.push(st);
            }

            swap(&mut states, &mut new_states);
            new_states.clear();
            candidates.clear();
            owned_states.clear();
        } else {
            for (j, state) in states.iter_mut().enumerate() {
                state.make_frozen_decision(phi, frozen_sym);
                decision_hist.set(phi, j, (u8::try_from(j).unwrap(), frozen_sym));
            }
        }
    }

    let best_path_idx = argmax_iter(states.iter().map(|s| s.pw)).unwrap();

    drop(states);
    drop(new_states);
    drop(owned_states);
    drop(candidates);
    drop(pool_probs);
    drop(pool_decisions);

    let u_hat = decision_hist.backtrack(best_path_idx);

    let mut u_hat_bitrev = Box::new_uninit_slice(n);
    for i in 0..n {
        u_hat_bitrev[bit_reverse(i, m as u32)].write(u_hat[i]);
    }

    unsafe { u_hat_bitrev.assume_init() }
}
