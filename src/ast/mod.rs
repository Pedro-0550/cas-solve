use std::{
    fmt::Display,
    ops::{Add, Mul},
};

use num::complex::Complex64;

use crate::{
    Complex,
    arena::{Arena, Handle},
    ast::intrinsic::Intrinsic,
    dimension::{Dimension, DimensionalAnalysisError, Quantity, Unit},
    symbol::Symbol,
};

/* -------------------------------- CONSTANTS ------------------------------- */

const NODES: Arena<ExprNode> = Arena::new();

/* --------------------------------- MODULES -------------------------------- */

pub mod intrinsic;
pub mod ops;

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(PartialEq, Clone, Debug)]
pub enum ExprNode {
    Symbol(Symbol),
    Const(Quantity),

    Intrinsic(Intrinsic),
    ElementwiseIntrinsic(Intrinsic),

    Matrix { rows: usize, cols: usize, elements: Vec<Expr> },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Expr(Handle<ExprNode>);

impl Expr {
    pub fn node(&self) -> ExprNode {
        NODES.get_cloned(self.0).unwrap()
    }
}

impl ExprNode {
    pub fn register(self) -> Expr {
        if let Some((existing_id, _)) = NODES.find(|_, n| *n == self) {
            return Expr(existing_id);
        }

        let id = NODES.insert(self);
        Expr(id)
    }

    /// Returns `true` if the expr is [`Symbol`].
    ///
    /// [`Symbol`]: Expr::Symbol
    #[must_use]
    pub fn is_symbol(&self) -> bool {
        matches!(self, ExprNode::Symbol(..))
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

    pub fn as_intrinsic(&self) -> Option<&Intrinsic> {
        if let Self::Intrinsic(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`ElementwiseIntrinsic`].
    ///
    /// [`ElementwiseIntrinsic`]: Expr::ElementwiseIntrinsic
    #[must_use]
    pub fn is_elementwise_intrinsic(&self) -> bool {
        matches!(self, Self::ElementwiseIntrinsic(..))
    }

    pub fn as_elementwise_intrinsic(&self) -> Option<&Intrinsic> {
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
