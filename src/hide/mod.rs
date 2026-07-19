mod cpd;

pub use cpd::{hide_cpd, unhide_cpd};

use crate::traits::Float;
#[cfg(feature = "args_validation")]
use crate::{NBPCError, Unsigned};

#[cfg(feature = "args_validation")]
macro_rules! precondition {
    ($condition:expr, $err_message:expr) => {
        if !$condition {
            return Err(NBPCError::IncorrectInput(String::from($err_message)));
        }
    };
}

#[cfg(feature = "args_validation")]
fn validate_args<U: Unsigned, F: Float>(
    cover: &[U],
    rho_m1: &[F],
    rho_p1: &[F],
    k: usize,
    l: usize,
) -> Result<(), NBPCError> {
    let n = cover.len();

    precondition!(l > 0 && l <= 64, "`l` must be in `1..=64`");
    precondition!(!cover.is_empty(), "`cover` can't be empty");
    precondition!(k < n, "payload >= 2bpp is not allowed");
    precondition!(
        rho_m1.len() == n,
        "lengths of `rho_m1` and `cover` do not match"
    );
    precondition!(
        rho_p1.len() == n,
        "lengths of `rho_p1` and `cover` do not match"
    );
    for (&m1, &p1) in rho_m1.iter().zip(rho_p1) {
        precondition!(
            m1.is_non_negative() && p1.is_non_negative(),
            "costs must be non-negative"
        );
        precondition!(m1.is_finite_() && p1.is_finite_(), "costs must be finite");
    }

    Ok(())
}

/// Costs into `0..1000` range
fn normalize_cost_map<F: Float>(rho_m1: &[F], rho_p1: &[F]) -> (Vec<f64>, Vec<f64>) {
    let mut rho_m1_max = F::from(0_f32);
    let mut rho_p1_max = F::from(0_f32);

    for (&m1, &p1) in rho_m1.iter().zip(rho_p1) {
        rho_m1_max = rho_m1_max.max_(m1);
        rho_p1_max = rho_p1_max.max_(p1);
    }

    // In case all costs are zero to avoid division by zero.
    rho_m1_max = rho_m1_max.max_(F::from(0_f32));
    rho_p1_max = rho_p1_max.max_(F::from(0_f32));

    let rho_m1: Vec<f64> = rho_m1
        .iter()
        .map(|&x| (x / rho_m1_max * F::from(1000_f32)).into())
        .collect();
    let rho_p1: Vec<f64> = rho_p1
        .iter()
        .map(|&x| (x / rho_p1_max * F::from(1000_f32)).into())
        .collect();

    (rho_m1, rho_p1)
}
