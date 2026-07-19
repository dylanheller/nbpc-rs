mod active_path;
mod decision_history;
mod decoder;
mod layer;
mod pool;
mod state;

pub use decoder::decode_map;

/// `src = 0bAABB` \
/// returns `(0bBB, 0bAA)`
#[inline(always)]
const fn get_decisions(src: u8) -> (u8, u8) {
    (src & 0b11, src >> 2)
}

/// `dst = 0bAABB` and `src = 0bXX` -> \
/// > `dst = 0bAAXX` if `oddness_bit == 0` \
/// > `dst = 0bXXBB` if `oddness_bit == 1`
#[inline(always)]
const fn set_decision(dst: &mut u8, src: u8, oddness_bit: usize) {
    *dst = *dst & (0xFF ^ (0b11 << (oddness_bit * 2))) | (src << (oddness_bit * 2));
}
