use std::ops::{Add, BitXor, Div, Mul, Neg, Sub};

use crate::{
    core::scalar::Scalar,
    dimension::Quantity,
    expr::{Double, Expr, Node, Shaped, Variadic, ops::Single},
    symbol::Symbol,
};

/* ---------------------------------- IMPLS --------------------------------- */

impl<T> From<T> for Expr
where
    Node: From<T>,
{
    fn from(value: T) -> Self {
        Node::from(value).register()
    }
}

// impl From<Intrinsic> for Node {
//     fn from(v: Intrinsic) -> Self {
//         Self::Intrinsic(v)
//     }
// }

// impl From<Quantity> for Node {
//     fn from(v: Quantity) -> Self {
//         Self::Const(v)
//     }
// }

impl From<Scalar> for Node {
    fn from(v: Scalar) -> Self {
        Self::Const(v.into())
    }
}

impl From<f64> for Node {
    fn from(v: f64) -> Self {
        Self::Const(v.into())
    }
}

impl From<i64> for Node {
    fn from(v: i64) -> Self {
        Self::Const(v.into())
    }
}

// impl From<Symbol> for Node {
//     fn from(v: Symbol) -> Self {
//         Self::Symbol(v)
//     }
// }

macro_rules! impl_op {
    ($t0:ty, $ty:ty, $op:ident, $method:ident, $expr:expr, normal) => {
        impl $op<$ty> for $t0 {
            type Output = Expr;

            fn $method(self, rhs: $ty) -> Expr {
                $expr(self.into(), rhs.into()).into()
            }
        }
    };
    ($t0:ty, $ty:ty, $op:ident, $method:ident, $expr:expr, symmetrical) => {
        impl $op<$ty> for $t0 {
            type Output = Expr;

            fn $method(self, rhs: $ty) -> Expr {
                $expr(self.into(), rhs.into()).into()
            }
        }

        impl $op<$t0> for $ty {
            type Output = Expr;

            fn $method(self, rhs: $t0) -> Expr {
                $expr(self.into(), rhs.into()).into()
            }
        }
    };
}

macro_rules! impl_expr_ops {
    (
        $t0:ty, [$($ty:ty),+ $(,)?], $config:tt
    ) => {
        $(
            impl_op!($t0, $ty, Add, add, |lhs: Expr, rhs: Expr| {
                assert_eq!(lhs.shape(), rhs.shape(), "Tried to add two expressions of different shapes: {lhs}, {rhs}");

                Variadic::Add(vec![lhs, rhs])
            }, $config);
            impl_op!($t0, $ty, Mul, mul, |lhs, rhs| Variadic::Mul(vec![lhs, rhs]), $config);
            impl_op!($t0, $ty, Div, div, |lhs: Expr, rhs: Expr| {
                assert!(
                    lhs.shape().cols == lhs.shape().rows || (lhs.shape() == rhs.shape() && lhs.shape().is_vec()),
                    "Matrix multiplication A * B requires A to have as many columns as B has rows.
                    A special case is when both A and B are vectors of equal shape, in which case Mul means dot product."
                );
                Variadic::Mul(vec![lhs, Double::Pow { base: rhs, exp: (-1.0).into() }.into()])
            }, $config);
            impl_op!($t0, $ty, Sub, sub, |lhs, rhs: Expr| Variadic::Add(vec![lhs, -rhs]), $config);
            impl_op!($t0, $ty, BitXor, bitxor, |lhs: Expr, rhs: Expr| {
                assert!(
                    lhs.shape().is_square() || lhs.shape().is_scalar(),
                    "Only square matrices can be raised to a power"
                );

                assert!(
                    rhs.shape().is_square() || rhs.shape().is_scalar(),
                    "Only square matrices can be an exponent"
                );

                assert!(
                    lhs.shape().is_square() ^ rhs.shape().is_square(),
                    "Cannot raise a matrix to the power of another matrix yet"
                );

                Double::Pow {
                    base: lhs,
                    exp: rhs
                }
            }, $config);

        )+
    };
}

impl_expr_ops!(&Expr, [i64, f64, Scalar, Quantity, Symbol], symmetrical);
impl_expr_ops!(&Expr, [&Expr], normal);

impl_expr_ops!(Expr, [i64, f64, Scalar, Quantity, Symbol, &Expr], symmetrical);
impl_expr_ops!(Expr, [Expr], normal);

impl_expr_ops!(Symbol, [i64, f64, Scalar, Quantity], symmetrical);
impl_expr_ops!(Symbol, [Symbol], normal);

impl Neg for Expr {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        Single::Neg(self).into()
    }
}

impl Neg for Symbol {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        Single::Neg(self.into()).into()
    }
}

impl From<&Expr> for Expr {
    fn from(value: &Expr) -> Self {
        value.clone()
    }
}
