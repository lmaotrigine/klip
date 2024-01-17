use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Neg, Not};

#[derive(Debug, Clone, Copy)]
pub struct Choice(u8);

impl Choice {
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl From<Choice> for bool {
    #[inline]
    fn from(value: Choice) -> Self {
        debug_assert!((value.0 == 0) | (value.0 == 1));
        value.0 != 0
    }
}

impl BitAnd for Choice {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        (self.0 & rhs.0).into()
    }
}

impl BitAndAssign for Choice {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitOr for Choice {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        (self.0 | rhs.0).into()
    }
}

impl BitOrAssign for Choice {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXor for Choice {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        (self.0 ^ rhs.0).into()
    }
}

impl BitXorAssign for Choice {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Not for Choice {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        (1 & (!self.0)).into()
    }
}

#[cfg(not(feature = "core_hint_black_box"))]
#[inline(never)]
fn black_box(input: u8) -> u8 {
    debug_assert!((input == 0) | (input == 1));
    unsafe { core::ptr::read_volatile(&input) }
}

#[cfg(feature = "core_hint_black_box")]
#[inline(never)]
fn black_box(input: u8) -> u8 {
    debug_assert!((input == 0) | (input == 1));
    core::hint::black_box(input)
}

impl From<u8> for Choice {
    #[inline]
    fn from(value: u8) -> Self {
        Self(black_box(value))
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait ConstantTimeEq {
    fn ct_eq(&self, other: &Self) -> Choice;

    #[inline]
    fn ct_ne(&self, other: &Self) -> Choice {
        !self.ct_eq(other)
    }
}

impl<T: ConstantTimeEq> ConstantTimeEq for [T] {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        let len = self.len();
        if len != other.len() {
            return Choice::from(0);
        }
        let mut x = 1;
        for (a, b) in self.iter().zip(other.iter()) {
            x &= a.ct_eq(b).to_u8();
        }
        x.into()
    }
}

impl ConstantTimeEq for Choice {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        !(*self ^ *other)
    }
}

macro_rules! impl_ints {
    ($u:ty, $i:ty, $w:expr) => {
        impl ConstantTimeEq for $u {
            #[inline]
            #[allow(trivial_numeric_casts)]
            fn ct_eq(&self, other: &Self) -> Choice {
                let x = self ^ other;
                let y = (x | x.wrapping_neg()) >> ($w - 1);
                ((y ^ (1)) as u8).into()
            }
        }
        impl ConstantTimeEq for $i {
            #[inline]
            fn ct_eq(&self, other: &Self) -> Choice {
                (*self as $u).ct_eq(&(*other as $u))
            }
        }
    };
}

impl_ints!(u8, i8, 8);
impl_ints!(u16, i16, 16);
impl_ints!(u32, i32, 32);
impl_ints!(u64, i64, 64);
impl_ints!(u128, i128, 128);
impl_ints!(usize, isize, core::mem::size_of::<usize>() * 8);

#[derive(Debug, Clone, Copy)]
pub struct OptionCt<T> {
    value: T,
    is_some: Choice,
}

impl<T> From<OptionCt<T>> for Option<T> {
    fn from(value: OptionCt<T>) -> Self {
        if value.is_some().to_u8() == 1 {
            Self::Some(value.value)
        } else {
            None
        }
    }
}

impl<T> OptionCt<T> {
    #[inline]
    pub const fn new(value: T, is_some: Choice) -> Self {
        Self { value, is_some }
    }

    #[inline]
    pub const fn is_some(&self) -> Choice {
        self.is_some
    }
}

pub trait ConditionallySelectable: Copy {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self;

    #[inline]
    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::conditional_select(self, other, choice);
    }

    #[inline]
    fn conditional_swap(a: &mut Self, b: &mut Self, choice: Choice) {
        let t = *a;
        a.conditional_assign(b, choice);
        b.conditional_assign(&t, choice);
    }
}

macro_rules! to_signed_int {
    (u8) => {
        i8
    };
    (u16) => {
        i16
    };
    (u32) => {
        i32
    };
    (u64) => {
        i64
    };
    (u128) => {
        i128
    };
    (i8) => {
        i8
    };
    (i16) => {
        i16
    };
    (i32) => {
        i32
    };
    (i64) => {
        i64
    };
    (i128) => {
        i128
    };
}

macro_rules! generate_integer_conditional_select {
    ($($t:tt)*) => {
        $(
            #[allow(trivial_numeric_casts)]
            impl ConditionallySelectable for $t {
                #[inline]
                fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
                    let mask = -(choice.to_u8() as to_signed_int!($t)) as $t;
                    a ^ (mask & (a ^ b))
                }

                #[inline]
                fn conditional_assign(&mut self, other: &Self, choice: Choice) {
                    let mask = -(choice.to_u8() as to_signed_int!($t)) as $t;
                    *self ^= mask & (*self ^ *other);
                }

                #[inline]
                fn conditional_swap(a: &mut Self, b: &mut Self, choice: Choice) {
                    let mask = -(choice.to_u8() as to_signed_int!($t)) as $t;
                    let t = mask & (*a ^ *b);
                    *a ^= t;
                    *b ^= t;
                }
            }
        )*
    };
}

generate_integer_conditional_select!(u8 i8);
generate_integer_conditional_select!(u16 i16);
generate_integer_conditional_select!(u32 i32);
generate_integer_conditional_select!(u64 i64);
generate_integer_conditional_select!(u128 i128);

impl ConditionallySelectable for Choice {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(u8::conditional_select(&a.0, &b.0, choice))
    }
}

impl<T: ConditionallySelectable, const N: usize> ConditionallySelectable for [T; N] {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut output = *a;
        output.conditional_assign(b, choice);
        output
    }

    fn conditional_assign(&mut self, other: &Self, choice: Choice) {
        for (a_i, b_i) in self.iter_mut().zip(other) {
            a_i.conditional_assign(b_i, choice);
        }
    }
}

pub trait ConditionallyNegatable {
    fn conditional_negate(&mut self, choice: Choice);
}

impl<T: ConditionallySelectable> ConditionallyNegatable for T
where
    for<'a> &'a T: Neg<Output = T>,
{
    #[inline]
    fn conditional_negate(&mut self, choice: Choice) {
        let self_neg = -(&*self);
        self.conditional_assign(&self_neg, choice);
    }
}
