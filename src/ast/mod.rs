use std::{
    fmt::Display,
    ops::{Add, Mul},
};

use num::complex::Complex64;

use crate::{
    Complex,
    ast::intrinsic::Intrinsic,
    dimension::{Dimension, DimensionalAnalysisError, Quantity, Unit},
    symbol::Symbol,
};

/* --------------------------------- MODULES -------------------------------- */

mod intrinsic;
mod ops;

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Symbol(Symbol),
    Const(Quantity),

    Intrinsic(Box<Intrinsic>),
    ElementwiseIntrinsic(Box<Intrinsic>),

    Matrix { rows: usize, cols: usize, elements: Vec<Expr> },
}

impl From<Intrinsic> for Expr {
    fn from(v: Intrinsic) -> Self {
        Self::Intrinsic(Box::new(v))
    }
}

impl From<Quantity> for Expr {
    fn from(v: Quantity) -> Self {
        Self::Const(v)
    }
}

impl From<Complex> for Expr {
    fn from(v: Complex) -> Self {
        Self::Const(v.into())
    }
}

impl From<f64> for Expr {
    fn from(v: f64) -> Self {
        Self::Const(v.into())
    }
}

impl From<Symbol> for Expr {
    fn from(v: Symbol) -> Self {
        Self::Symbol(v)
    }
}

impl Expr {
    /// Returns `true` if the expr is [`Symbol`].
    ///
    /// [`Symbol`]: Expr::Symbol
    #[must_use]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(..))
    }

    pub fn as_symbol(&self) -> Option<&Symbol> {
        if let Self::Symbol(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`Const`].
    ///
    /// [`Const`]: Expr::Const
    #[must_use]
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(..))
    }

    pub fn as_const(&self) -> Option<&Quantity> {
        if let Self::Const(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`Intrinsic`].
    ///
    /// [`Intrinsic`]: Expr::Intrinsic
    #[must_use]
    pub fn is_intrinsic(&self) -> bool {
        matches!(self, Self::Intrinsic(..))
    }

    pub fn as_intrinsic(&self) -> Option<&Box<Intrinsic>> {
        if let Self::Intrinsic(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`ElementwiseIntrinsic`].
    ///
    /// [`ElementwiseIntrinsic`]: Expr::ElementwiseIntrinsic
    #[must_use]
    pub fn is_elementwise_intrinsic(&self) -> bool {
        matches!(self, Self::ElementwiseIntrinsic(..))
    }

    pub fn as_elementwise_intrinsic(&self) -> Option<&Box<Intrinsic>> {
        if let Self::ElementwiseIntrinsic(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`Matrix`].
    ///
    /// [`Matrix`]: Expr::Matrix
    #[must_use]
    pub fn is_matrix(&self) -> bool {
        matches!(self, Self::Matrix { .. })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
