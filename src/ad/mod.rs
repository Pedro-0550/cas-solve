use num::{complex::Complex64, pow::Pow};

use crate::{
    ast::{
        Expr, Node,
        intrinsic::{Intrinsic, ln},
    },
    simplify::Simplify,
    symbol::{Symbol, constants::e},
};

/* --------------------------------- MODULES -------------------------------- */

#[cfg(test)]
mod test;

/* --------------------------------- STRUCTS -------------------------------- */

/// Todo: Explain
pub struct Dual {
    pub z: Complex64,
    pub grad: Vec<Complex64>,
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Differentiable {
    fn diff(&self, symbol: Symbol) -> Expr;
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Differentiable for Expr {
    fn diff(&self, symbol: Symbol) -> Expr {
        match self.node() {
            Node::Const(_) => 0.into(),
            Node::Symbol(s) => if symbol == s { 1 } else { 0 }.into(),
            Node::Intrinsic(intr) => intr.diff(symbol),
            _ => todo!(),
        }
    }
}

impl Differentiable for Intrinsic {
    fn diff(&self, symbol: Symbol) -> Expr {
        match self {
            Intrinsic::Add(terms) => Intrinsic::Add(
                terms.iter().map(|expr| expr.diff(symbol)).collect(),
            )
            .into(),
            Intrinsic::Mul(terms) => Intrinsic::Add(
                terms
                    .iter()
                    .map(|expr| {
                        let mut factors = Vec::with_capacity(terms.len());
                        factors.push(expr.diff(symbol));
                        factors.extend(terms.iter().filter(|x| *x != expr));
                        Intrinsic::Mul(factors).into()
                    })
                    .collect(),
            )
            .into(),
            Intrinsic::Inv(expr) => -expr.diff(symbol) / (expr ^ 2),
            Intrinsic::Neg(expr) => -expr.diff(symbol),
            Intrinsic::Pow { base, exp } => {
                (base ^ exp)
                    * (base.diff(symbol) * exp / base
                        + exp.diff(symbol) * ln(base))
            }
            Intrinsic::Log { base, arg } => {
                if *base == e.into() {
                    arg.diff(symbol) / arg
                } else {
                    (ln(arg) / ln(base)).diff(symbol)
                }
            }
            _ => todo!(),
        }
        // .simplify()
    }
}
