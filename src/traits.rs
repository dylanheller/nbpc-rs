use std::ops::{Div, Mul};

mod seal {
    pub trait SealedUnsigned {}

    impl SealedUnsigned for u8 {}
    impl SealedUnsigned for u16 {}
    impl SealedUnsigned for u32 {}
    impl SealedUnsigned for u64 {}
    impl SealedUnsigned for u128 {}
    impl SealedUnsigned for usize {}

    pub trait SealedFloat {}

    impl SealedFloat for f32 {}
    impl SealedFloat for f64 {}
}

pub trait Unsigned: Clone + Copy + seal::SealedUnsigned {
    fn lsb2_u8(self) -> u8;
    fn add_1_checked(self) -> Option<Self>;
    fn sub_1_checked(self) -> Option<Self>;
    fn signed_diff(a: Self, b: Self) -> i128;
}

macro_rules! impl_trait_unsigned {
    ($($type:ty), *) => {
        $(impl Unsigned for $type {
            #[inline(always)]
            fn lsb2_u8(self) -> u8 {
                (self & 0b11) as u8
            }

            #[inline(always)]
            fn add_1_checked(self) -> Option<Self> {
                self.checked_add(1)
            }

            #[inline(always)]
            fn sub_1_checked(self) -> Option<Self> {
                self.checked_sub(1)
            }

            #[inline(always)]
            fn signed_diff(a: Self, b: Self) -> i128 {
                let a_i128 = i128::try_from(a).unwrap();
                let b_i128 = i128::try_from(b).unwrap();
                a_i128.strict_sub(b_i128)
            }
        })*
    };
}

impl_trait_unsigned!(u8, u16, u32, u64, u128, usize);

pub trait Float:
    Clone + Copy + From<f32> + Into<f64> + Div<Output = Self> + Mul<Output = Self> + seal::SealedFloat
{
    fn max_(self, b: Self) -> Self;
    fn is_finite_(self) -> bool;
    fn is_non_negative(self) -> bool;
}

macro_rules! impl_trait_float {
    ($($type:ty), *) => {
        $(impl Float for $type {
            fn max_(self, b: Self) -> Self {
                self.max(b)
            }

            fn is_finite_(self) -> bool {
                self.is_finite()
            }

            fn is_non_negative(self) -> bool {
                self >= 0.0
            }
        })*
    };
}

impl_trait_float!(f32, f64);
