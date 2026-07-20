use std::{
    fmt::{Binary, Debug, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex},
    hash::Hash,
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
    str::FromStr,
};

mod seal {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for u128 {}
    impl Sealed for usize {}

    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

pub trait Unsigned:
    Clone
    + Copy
    + Eq
    + PartialEq
    + Ord
    + PartialOrd
    + Hash
    + Default
    + Debug
    + Display
    + Binary
    + Octal
    + LowerHex
    + UpperHex
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Rem<Output = Self>
    + RemAssign
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr<Output = Self>
    + BitOrAssign
    + BitXor<Output = Self>
    + BitXorAssign
    + Shl<u8, Output = Self>
    + Shl<u16, Output = Self>
    + Shl<u32, Output = Self>
    + Shl<u64, Output = Self>
    + Shl<u128, Output = Self>
    + Shl<usize, Output = Self>
    + ShlAssign<u8>
    + ShlAssign<u16>
    + ShlAssign<u32>
    + ShlAssign<u64>
    + ShlAssign<u128>
    + ShlAssign<usize>
    + Shr<u8, Output = Self>
    + Shr<u16, Output = Self>
    + Shr<u32, Output = Self>
    + Shr<u64, Output = Self>
    + Shr<u128, Output = Self>
    + Shr<usize, Output = Self>
    + ShrAssign<u8>
    + ShrAssign<u16>
    + ShrAssign<u32>
    + ShrAssign<u64>
    + ShrAssign<u128>
    + ShrAssign<usize>
    + Sum<Self>
    + Product<Self>
    + From<u8>
    + FromStr
    + seal::Sealed
{
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
    Clone
    + Copy
    + Default
    + PartialEq
    + PartialOrd
    + Debug
    + Display
    + LowerExp
    + UpperExp
    + UpperExp
    + Neg<Output = Self>
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Mul<Self, Output = Self>
    + MulAssign<Self>
    + Div<Self, Output = Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + Rem<Self, Output = Self>
    + RemAssign<Self>
    + Sum<Self>
    + Product<Self>
    + From<f32>
    + Into<f64>
    + FromStr
    + seal::Sealed
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
