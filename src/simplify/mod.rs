use std::array;

use itertools::Itertools;

use crate::{
    dimension::Quantity,
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic},
    },
    simplify::normal::Normalize, // set::Set,
};

/* -------------------------------- CONSTANTS ------------------------------- */

const BEAM_WIDTH: usize = 16;
const MAX_DEPTH: usize = 800;

/* --------------------------------- MODULES -------------------------------- */

#[cfg(test)]
mod test;

pub mod normal;

/* --------------------------------- TRAITS --------------------------------- */

pub trait Simplify {
    fn simplify(&self) -> Expr;
    // fn range(&self) -> Set;
}

/* --------------------------------- STRUCTS -------------------------------- */

// struct Path {
//     expr: Expr,
//     cost: usize,
//     seen: HashSet<Expr>,
// }

/* ---------------------------------- IMPLS --------------------------------- */

impl Simplify for Expr {
    fn simplify(&self) -> Expr {
        match self.normalize().node() {
            Node::Symbol(symbol) => *self,
            Node::Const(quantity) => *self,
            Node::Variadic(variadic) => todo!(),
            Node::Single(single) => todo!(),
            Node::Double(double) => todo!(),
            Node::Matrix(matrix) => todo!(),
        }
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

impl Simplify for Single {
    fn simplify(&self) -> Expr {
        todo!()
    }

    // fn range(&self) -> Set {
    //     match self {
    //         Single::Neg(_expr) => todo!(),
    //         Single::Sin(_expr) => todo!(),
    //         Single::Cos(_expr) => todo!(),
    //         Single::Tan(_expr) => todo!(),
    //         Single::Asin(_expr) => todo!(),
    //         Single::Acos(_expr) => todo!(),
    //         Single::Atan(_expr) => todo!(),
    //         Single::Sinh(_expr) => todo!(),
    //         Single::Cosh(_expr) => todo!(),
    //         Single::Tanh(_expr) => todo!(),
    //         Single::Asinh(_expr) => todo!(),
    //         Single::Acosh(_expr) => todo!(),
    //         Single::Atanh(_expr) => todo!(),
    //         Single::Transpose(_expr) => todo!(),
    //         Single::Conj(_expr) => todo!(),
    //         Single::Arg(_expr) => todo!(),
    //         Single::Det(_expr) => todo!(),
    //         Single::Norm(_expr) => todo!(),
    //     }
    // }
}

impl Simplify for Double {
    fn simplify(&self) -> Expr {
        match self {
            Double::Pow { base, exp }
                if let Node::Double(Double::Pow {
                    base: inner_base,
                    exp: inner_exp,
                }) = base.node()
                    && let Node::Const(exp) = exp.node()
                    && let Node::Const(inner_exp) = inner_exp.node()
                    && exp.value().is_integer()
                    && inner_exp.value().is_integer() =>
            {
                (inner_base ^ (exp * inner_exp)).simplify()
            }
            _ => self
                .with_args(array::from_fn(|i| self.args()[i].simplify()))
                .into(),
        }
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

impl Simplify for Variadic {
    fn simplify(&self) -> Expr {
        self.with_operands(
            self.operands_ref().iter().map(Expr::simplify).collect::<Vec<_>>(),
        )
        .into()
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

pub fn separate_consts(
    terms: impl IntoIterator<Item = Expr> + Clone,
) -> (impl Iterator<Item = Quantity>, impl Iterator<Item = Expr>) {
    (
        terms.clone().into_iter().filter_map(|expr| match expr.node() {
            Node::Const(qty) => Some(qty),
            _ => None,
        }),
        terms.into_iter().filter(|expr| !expr.node().is_const()),
    )
}
