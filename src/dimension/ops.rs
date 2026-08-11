use std::ops::{Add, BitXor, Div, DivAssign, Mul, MulAssign, Sub};

use num::{
    Num,
    complex::{Complex32, Complex64},
};

use crate::{
    Scalar,
    dimension::{COMPOSITIONS, Dimension, Quantity, Unit},
};

impl Dimension {
    pub const fn pow(self, exponent: i8) -> Self {
        Self {
            T: self.T * exponent,
            L: self.L * exponent,
            M: self.M * exponent,
            I: self.I * exponent,
            Θ: self.Θ * exponent,
            J: self.J * exponent,
            N: self.N * exponent,
        }
    }
}

impl const Mul for Dimension {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            T: self.T + rhs.T,
            L: self.L + rhs.L,
            M: self.M + rhs.M,
            I: self.I + rhs.I,
            N: self.N + rhs.N,
            Θ: self.Θ + rhs.Θ,
            J: self.J + rhs.J,
        }
    }
}

impl const MulAssign for Dimension {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl const DivAssign for Dimension {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl const BitXor<i8> for Dimension {
    type Output = Dimension;

    fn bitxor(self, rhs: i8) -> Self::Output {
        Self {
            T: self.T * rhs,
            L: self.L * rhs,
            M: self.M * rhs,
            I: self.I * rhs,
            N: self.N * rhs,
            Θ: self.Θ * rhs,
            J: self.J * rhs,
        }
    }
}

impl const Div for Dimension {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            T: self.T - rhs.T,
            L: self.L - rhs.L,
            M: self.M - rhs.M,
            I: self.I - rhs.I,
            N: self.N - rhs.N,
            Θ: self.Θ - rhs.Θ,
            J: self.J - rhs.J,
        }
    }
}

impl Mul for Unit {
    type Output = Unit;

    fn mul(self, rhs: Self) -> Self::Output {
        let result = {
            match (self, rhs) {
                (Unit::Composed(id), rhs) if rhs.is_atomic() => {
                    let mut lhs_comp = COMPOSITIONS.get_cloned(id).unwrap();

                    lhs_comp.push((rhs, 1));
                    lhs_comp
                }

                (_, Unit::Unitless) => return self,

                (lhs, Unit::Composed(id)) if lhs.is_atomic() => {
                    let mut rhs_comp = COMPOSITIONS.get_cloned(id).unwrap();
                    let mut new_comp = vec![(self, 1)];

                    new_comp.append(&mut rhs_comp);
                    new_comp
                }
                (Unit::Unitless, _) => return rhs,

                (Unit::Composed(lhs_id), Unit::Composed(rhs_id)) => {
                    let mut lhs_comp = COMPOSITIONS.get_cloned(lhs_id).unwrap();
                    let mut rhs_comp = COMPOSITIONS.get_cloned(rhs_id).unwrap();

                    lhs_comp.append(&mut rhs_comp);
                    lhs_comp
                }

                (lhs, rhs) if lhs.is_atomic() && rhs.is_atomic() => {
                    vec![(lhs, 1), (rhs, 1)]
                }
                _ => unreachable!(),
            }
        };

        Unit::new_composition(result)
    }
}

impl Div for Unit {
    type Output = Unit;

    fn div(self, rhs: Self) -> Self::Output {
        self * (rhs ^ -1)
    }
}

// Cursed cursed cursed cursed cursed cursed
//
//
//
// cursed
impl BitXor<i8> for Unit {
    type Output = Unit;

    fn bitxor(self, exp: i8) -> Self::Output {
        match self {
            Self::Unitless => self,
            _ if self.is_atomic() => Unit::new_composition(vec![(self, exp)]),
            Self::Composed(id) => {
                let comp = COMPOSITIONS
                    .get_cloned(id)
                    .unwrap()
                    .iter()
                    .map(|(unit, e)| (*unit, e * exp))
                    .collect();

                Unit::new_composition(comp)
            }
            _ => unreachable!(),
        }
    }
}

macro_rules! impl_qty_from_scalar {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Quantity {
                fn from(value: $t) -> Self {
                    Quantity(value.into(), Unit::Unitless)
                }
            }
        )*
    };
}

impl_qty_from_scalar!(
    u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, Complex32, Complex64,
    Scalar
);

impl From<Unit> for Quantity {
    fn from(unit: Unit) -> Self {
        Quantity(1.0.into(), unit)
    }
}

macro_rules! impl_op {
    ($t0:ty, $ty:ty, $op:ident, $method:ident, $expr:expr, normal) => {
        impl $op<$ty> for $t0 {
            type Output = Quantity;

            fn $method(self, rhs: $ty) -> Quantity {
                $expr(self.into(), rhs.into()).into()
            }
        }
    };
    ($t0:ty, $ty:ty, $op:ident, $method:ident, $expr:expr, symmetrical) => {
        impl $op<$ty> for $t0 {
            type Output = Quantity;

            fn $method(self, rhs: $ty) -> Quantity {
                $expr(self.into(), rhs.into()).into()
            }
        }

        impl $op<$t0> for $ty {
            type Output = Quantity;

            fn $method(self, rhs: $t0) -> Quantity {
                $expr(self.into(), rhs.into()).into()
            }
        }
    };
}

macro_rules! impl_qty_ops {
    (
        $t0:ty, [$($ty:ty),+ $(,)?], $config:tt
    ) => {
        $(
            impl_op!($t0, $ty, Add, add, |lhs: Quantity, rhs: Quantity| {
                assert!(lhs.unit().repr_eq(rhs.unit()), "cannot add two quantities with different units");
                Quantity(lhs.value() + rhs.value(), lhs.unit())
            }, $config);
            impl_op!($t0, $ty, Mul, mul, |lhs: Quantity, rhs: Quantity| Quantity(lhs.value() * rhs.value(), lhs.unit() * rhs.unit()), $config);
            impl_op!($t0, $ty, Div, div, |lhs: Quantity, rhs: Quantity| Quantity(lhs.value() / rhs.value(), lhs.unit() / rhs.unit()), $config);
            impl_op!($t0, $ty, Sub, sub, |lhs: Quantity, rhs: Quantity| {
                assert!(lhs.unit().repr_eq(rhs.unit()), "cannot subtract two quantities with different units");
                Quantity(lhs.value() - rhs.value(), lhs.unit())
            }, $config);
            // impl_op!($t0, $ty, BitXor, bitxor, |lhs, rhs| Quantity(lhs.value() - rhs.value(), lhs.unit()), $config);

        )+
    };
}

impl_qty_ops!(Unit, [Scalar, f64, i64, Complex64, Complex32], symmetrical);

impl_qty_ops!(
    Quantity,
    [Scalar, Unit, f64, i64, Complex64, Complex32],
    symmetrical
);
impl_qty_ops!(Quantity, [Quantity], normal);

macro_rules! impl_qty_partial_eq {
    ($($t:ty),*) => {
        $(
            impl PartialEq<$t> for Quantity {
                fn eq(&self, other: &$t) -> bool {
                    self.0 == Scalar::from(*other) && self.1 == Unit::Unitless
                }
            }

            impl PartialEq<Quantity> for $t {
                fn eq(&self, other: &Quantity) -> bool {
                    other.0 == Scalar::from(*self) && other.1 == Unit::Unitless
                }
            }

        )*
    };
}

impl_qty_partial_eq!(f64, i64, Complex64, Complex32);
