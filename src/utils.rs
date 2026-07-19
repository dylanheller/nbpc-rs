use crate::xoshiro128plusplus::Xoshiro128PlusPlus;

#[inline(always)]
pub const fn bit_reverse(x: usize, bits: u32) -> usize {
    x.reverse_bits() >> (usize::BITS - bits)
}

/// UB if `arr[i] >= 4`
pub fn polar_encode(arr: &mut [u8]) {
    debug_assert!(!arr.iter().any(|&b| b >= 4));

    let n = arr.len();
    assert!(n.is_power_of_two());

    let mut step = 1;

    while step < n {
        for off in (0..n).step_by(2 * step) {
            let (l_half, r_half) = arr[off..off + 2 * step].split_at_mut(step);

            for (l, &mut r) in l_half.iter_mut().zip(r_half) {
                // GF4_GAMMA_MUL[x] is equal to this
                *l ^= (((r.count_ones() as u8) << 1) | (r >> 1)) & 0b11;
            }
        }

        step *= 2;
    }
}

pub fn dibits_to_bytes(dibits: &[u8]) -> Vec<u8> {
    assert!(dibits.len().is_multiple_of(4));

    dibits
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[b78, b56, b34, b12]| [(b78 << 6) | (b56 << 4) | (b34 << 2) | b12])
        .collect()
}

pub fn fisher_yates<T: Clone>(arr: &mut [T], mut rng: Xoshiro128PlusPlus) {
    assert!(u32::try_from(arr.len()).is_ok());
    for i in (1..arr.len()).rev() {
        let j = rng.random_bounded((i + 1) as u32);
        let tmp = arr[i].clone();
        arr[i] = arr[j as usize].clone();
        arr[j as usize] = tmp;
    }
}

pub fn revert_order<T: Clone>(arr: &[T], rng: Xoshiro128PlusPlus) -> Box<[T]> {
    let n_u32 = u32::try_from(arr.len()).unwrap();

    let mut order: Vec<u32> = (0..n_u32).collect();
    fisher_yates(&mut order, rng);

    let mut out = Box::new_uninit_slice(arr.len());

    for (i, &o) in order.iter().enumerate() {
        out[o as usize].write(arr[i].clone());
    }

    unsafe { out.assume_init() }
}

#[inline(always)]
pub const fn argmax4(x: &[f64; 4]) -> u8 {
    let mut idx = 0;
    let mut max = x[0];

    if x[1] > max {
        max = x[1];
        idx = 1;
    }

    if x[2] > max {
        max = x[2];
        idx = 2;
    }

    if x[3] > max {
        idx = 3;
    }

    idx
}

#[inline]
pub fn argmax_iter(iter: impl Iterator<Item = f64>) -> Option<usize> {
    iter.enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}
