# nbpc-rs
Rust *O(Ln)-space* implementation of **Non-Binary Polar Codes (NBPC)** [1] for ternary steganographic embedding.


## Usage
> [!WARNING]
> NBPCs are guided by input probabilities that are, in turn, computed from the modification costs as stated in the PLS theory [2]. It makes them sensitive to the costs distribution. That's why you shouldn't use constants like `WET_COST = 1e10`. Instead, use relative values, like `WET_COST = rho.max() * 100`.

```rust
use nbpc_rs::{hide_cpd, unhide_cpd};

let cover: Vec<u8> = load_cover();
let (rho_m1, rho_p1): (Vec<f32>, Vec<f32>) = compute_costs(&cover);
let message = b"hello";

let stego = hide_cpd(&cover, &rho_m1, &rho_p1, message, 4)?;

let message_extracted = unhide_cpd(&stego, message.len() as u32);
assert_eq!(message_extracted, message);
```


## Error Handling: `NBPCError`
- `NBPCError::PaddingCorruption(n)`: `n` padding positions decoded to a non-zero symbol, meaning the requested payload couldn't be embedded without violating the padding invariant. Try a larger `l`, or lower the payload size
- `NBPCError::OverflowingModification`: a required +-1 modification will cause an overflow of `U` (e.g. `-1` for `0`, or `+1` for `255u8`)
- `NBPCError::NoInputPMFs`: can't calculate input PMFs with the given costs and payload


## Peak Memory Usage (w/o input data)
Let `N` denote `n` ceiled to the power of two.
- `17N`: input PMFs
- `N`: frozen mask
- `32L(N - 1)`: max total space for PMFs
- `L(N - 1)`: max total space for hard values
- `LN`: best paths

In total, roughly:
`N(18 + 34L) bytes`

---

- [1]: Q. Guan, K. Chen, W. Lu, W. Zhang, and N. Yu, “Non-Binary polar codes for steganography,” IEEE Transactions on Dependable and Secure Computing, vol. 23, no. 1, pp. 1029–1046, Sep. 2025, doi: 10.1109/tdsc.2025.3613759.
- [2]: T. Filler, J. Judas, and J. Fridrich, “Minimizing additive distortion in steganography using Syndrome-Trellis codes,” IEEE Transactions on Information Forensics and Security, vol. 6, no. 3, pp. 920–935, Apr. 2011, doi: 10.1109/tifs.2011.2134094.
