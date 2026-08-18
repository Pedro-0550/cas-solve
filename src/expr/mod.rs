use std::{
    array,
    fmt::{Debug, Display, Pointer},
    mem::discriminant,
    num::NonZero,
};

use derive_more::{From, IsVariant, TryUnwrap, Unwrap};
use itertools::Itertools;

use crate::{
    core::arena::{Arena, ArenaRef, Handle},
    dimension::Quantity,
    expr::ops::{Double, Matrix, Single, Variadic},
    simplify::{Simplify, normal::Normalize},
    symbol::Symbol,
};

/* -------------------------------- CONSTANTS ------------------------------- */

static NODES: Arena<Node> = Arena::new();

/* --------------------------------- MODULES -------------------------------- */

pub mod impls;
pub mod ops;

#[cfg(test)]
mod test;

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(
    PartialEq, Clone, Debug, From, IsVariant, Unwrap, TryUnwrap, Hash, Eq,
)]
pub enum Node {
    Symbol(Symbol),
    Const(Quantity),
    Variadic(Variadic),
    Single(Single),
    Double(Double),
    Matrix(Matrix),
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Expr(Handle<Node>);

#[derive(Clone, PartialEq, Eq)]
pub struct Binding {
    from: Symbol,
    to: Expr,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Shape {
    rows: NonZero<usize>,
    cols: NonZero<usize>,
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Shaped {
    fn shape(&self) -> Shape;
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Shape {
    // SAFETY:
    // As of August 2026, 1 is not equal to 0.
    // If this changes in the future, use checked version instead.
    pub const SCALAR: Self = unsafe {
        Shape {
            cols: NonZero::<usize>::new_unchecked(1),
            rows: NonZero::<usize>::new_unchecked(1),
        }
    };

    pub fn transpose(self) -> Self {
        Self { rows: self.cols, cols: self.rows }
    }

    pub fn square(size: usize) -> Self {
        Self { rows: size.try_into().unwrap(), cols: size.try_into().unwrap() }
    }

    pub fn rect(rows: usize, cols: usize) -> Self {
        Self { rows: rows.try_into().unwrap(), cols: cols.try_into().unwrap() }
    }

    pub fn is_scalar(&self) -> bool {
        self.rows.get() == 1 && self.cols.get() == 1
    }

    pub fn is_row(&self) -> bool {
        self.rows.get() > 1 && self.cols.get() == 1
    }

    pub fn is_col(&self) -> bool {
        self.rows.get() == 1 && self.cols.get() > 1
    }

    pub fn is_vec(&self) -> bool {
        (self.rows.get() > 1) ^ (self.cols.get() > 1)
    }

    pub fn is_rect(&self) -> bool {
        self.rows.get() > 1 && self.cols.get() > 1
    }

    pub fn is_square(&self) -> bool {
        self.rows.get() > 1 && self.rows == self.rows
    }
}

impl Expr {
    pub fn node(&self) -> ArenaRef<'_, Node> {
        NODES.get(self.0).unwrap()
    }

    /// Returns the total number of nodes in this expression
    pub fn size(&self) -> usize {
        match *self.node() {
            Node::Symbol(_symbol) => 1,
            Node::Const(_quantity) => 1,
            Node::Variadic(ref variadic) => {
                variadic.operands_ref().iter().map(|x| x.size()).sum::<usize>()
                    + 1
            }
            Node::Single(single) => single.arg().size() + 1,
            Node::Double(double) => {
                double.args()[0].size() + double.args()[1].size() + 1
            }
            Node::Matrix(ref matrix) => {
                matrix.elements().iter().map(|x| x.size()).sum::<usize>() + 1
            }
        }
    }

    pub fn substitute(self, bindings: &[Binding]) -> Self {
        match *self.node() {
            Node::Variadic(ref op) => op
                .with_operands(
                    op.operands_ref()
                        .iter()
                        .map(|x| x.substitute(bindings))
                        .collect(),
                )
                .into(),

            Node::Single(op) => {
                op.with_arg(op.arg().substitute(bindings)).into()
            }

            Node::Double(op) => op
                .with_args(array::from_fn(|i| {
                    op.args()[i].substitute(bindings)
                }))
                .into(),

            Node::Symbol(sym) => {
                if let Some(binding) = bindings.iter().find(|b| b.from == sym) {
                    binding.to
                } else {
                    self
                }
            }

            Node::Matrix(ref m) => {
                Node::Matrix(m.map(|el| el.substitute(bindings))).into()
            }

            _ => self,
        }
    }
}

impl Shaped for Expr {
    fn shape(&self) -> Shape {
        match *self.node() {
            Node::Symbol(symbol) => symbol.shape(),
            Node::Const(_) => Shape::SCALAR,
            Node::Variadic(ref variadic) => variadic.shape(),
            Node::Single(single) => single.shape(),
            Node::Double(double) => double.shape(),
            Node::Matrix(ref matrix) => matrix.shape(),
        }
    }
}

impl Node {
    pub fn register(self) -> Expr {
        if let Some(existing) = NODES.handle_of(&self) {
            return Expr(existing);
        }

        let id = NODES.insert(self);
        Expr(id)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self.node() {
            Node::Const(qty) => <Quantity as Display>::fmt(&qty, f),
            Node::Double(op) => <Double as Display>::fmt(&op, f),
            Node::Single(op) => <Single as Display>::fmt(&op, f),
            Node::Variadic(ref op) => <Variadic as Display>::fmt(&op, f),
            Node::Symbol(symb) => <Symbol as Display>::fmt(&symb, f),
            Node::Matrix(ref _m) => todo!(),
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self.node() {
            Node::Const(qty) => <Quantity as Display>::fmt(&qty, f),
            Node::Double(op) => write!(f, "{:?}", op),
            Node::Single(op) => write!(f, "{:?}", op),
            Node::Variadic(ref op) => write!(f, "{:?}", op),
            Node::Symbol(symb) => <Symbol as Display>::fmt(&symb, f),
            Node::Matrix(ref _m) => todo!(),
        }
    }
}

// impl PartialOrd for Expr {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for Expr {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match (self.node(), other.node()) {
//             (Node::Symbol(lhs), Node::Symbol(rhs)) => {
//                 lhs.name().cmp(&rhs.name())
//             }
//             (Node::Const(lhs), Node::Const(rhs)) => {
//                 let lhs = lhs.value();
//                 let rhs = rhs.value();
//                 lhs.re
//                     .total_cmp(&rhs.re)
//                     .then_with(|| lhs.im.total_cmp(&rhs.im))
//             }
//             (Node::Intrinsic(lhs), Node::Intrinsic(rhs)) => match (lhs, rhs) {},
//         }
//     }
// }
