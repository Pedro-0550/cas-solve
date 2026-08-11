use num::{complex::Complex64, pow::Pow};

use crate::{
    ast::{
        Expr, Node,
        intrinsic::{Intrinsic, ln},
    },
    symbol::Symbol,
};

/// Todo: Explain
pub struct Dual {
    pub z: Complex64,
    pub grad: Vec<Complex64>,
}

pub trait Differentiable {
    fn diff(&self, symbol: Symbol) -> Expr;
}

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
            Intrinsic::Div { num, denom } => {
                (num.diff(symbol) * denom - num * denom.diff(symbol))
                    / (denom ^ 2)
            }
            Intrinsic::Neg(expr) => -expr.diff(symbol),
            Intrinsic::Pow { base, exp } => {
                (base ^ exp)
                    * (base.diff(symbol) * exp / base
                        + exp.diff(symbol) * ln(base))
            }
            _ => todo!(),
        }
    }
}
