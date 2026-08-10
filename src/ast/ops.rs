use std::{
    ops::{Add, BitXor, Div, Mul, Neg, Sub},
    sync::LazyLock,
};

use crate::{
    Complex,
    ast::{Expr, ExprNode, intrinsic::Intrinsic},
    dimension::Quantity,
    symbol::Symbol,
};

/* ---------------------------------- IMPLS --------------------------------- */

impl<T> From<T> for Expr
where
    ExprNode: From<T>,
{
    fn from(value: T) -> Self {
        ExprNode::from(value).register()
    }
}

impl From<Intrinsic> for ExprNode {
    fn from(v: Intrinsic) -> Self {
        Self::Intrinsic(v)
    }
}

impl From<Quantity> for ExprNode {
    fn from(v: Quantity) -> Self {
        Self::Const(v)
    }
}

impl From<Complex> for ExprNode {
    fn from(v: Complex) -> Self {
        Self::Const(v.into())
    }
}

impl From<f64> for ExprNode {
    fn from(v: f64) -> Self {
        Self::Const(v.into())
    }
}

impl From<Symbol> for ExprNode {
    fn from(v: Symbol) -> Self {
        Self::Symbol(v)
    }
}

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
            impl_op!($t0, $ty, Add, add, |lhs, rhs| Intrinsic::Add(vec![lhs, rhs]), $config);
            impl_op!($t0, $ty, Mul, mul, |lhs, rhs| Intrinsic::Mul(vec![lhs, rhs]), $config);
            impl_op!($t0, $ty, Div, div, |lhs, rhs| Intrinsic::Div {num: lhs,denom: rhs }, $config);
            impl_op!($t0, $ty, Sub, sub, |lhs, rhs: Expr| Intrinsic::Add(vec![lhs, -rhs]), $config);
            impl_op!($t0, $ty, BitXor, bitxor, |lhs, rhs: Expr| Intrinsic::Pow {
                base: lhs,
                exp: rhs
            }, $config);

        )+
    };
}

impl_expr_ops!(Expr, [f64, Complex, Quantity, Symbol], symmetrical);
impl_expr_ops!(Expr, [Expr], normal);

impl_expr_ops!(Symbol, [f64, Complex, Quantity], symmetrical);
impl_expr_ops!(Symbol, [Symbol], normal);

impl Neg for Expr {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        Intrinsic::Neg(self).into()
    }
}

impl Neg for Symbol {
    type Output = Expr;

    fn neg(self) -> Self::Output {
        Intrinsic::Neg(self.into()).into()
    }
}

impl From<&Expr> for Expr {
    fn from(value: &Expr) -> Self {
        value.clone()
    }
}
