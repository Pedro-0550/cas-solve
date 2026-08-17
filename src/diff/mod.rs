use num::complex::Complex64;

use crate::{
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic, cos, cosh, ln, sin, sinh, sqrt},
    },
    simplify::{Simplify, Step, normal::Normalize},
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
    fn diff(&self, symbol: Symbol, steps: &mut Option<Vec<Step>>) -> Expr;
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Differentiable for Expr {
    fn diff(&self, symbol: Symbol, steps: &mut Option<Vec<Step>>) -> Expr {
        match self.simplify(steps).node() {
            Node::Const(_) => 0.into(),
            Node::Symbol(s) => if symbol == s { 1 } else { 0 }.into(),
            Node::Variadic(op) => op.diff(symbol, steps),
            Node::Single(op) => {
                op.arg().diff(symbol, steps) * op.diff(symbol, steps)
            }
            Node::Double(op) => op.diff(symbol, steps),
            _ => todo!(),
        }
        .simplify(steps)
    }
}

impl Differentiable for Single {
    fn diff(&self, symbol: Symbol, steps: &mut Option<Vec<Step>>) -> Expr {
        match self {
            Single::Sin(u) => cos(u),
            Single::Cos(u) => -sin(u),
            Single::Tan(u) => 1 / (cos(u) ^ 2),
            Single::Asin(u) => 1 / sqrt(1 - (u ^ 2)),
            Single::Acos(u) => -1 / sqrt(1 - (u ^ 2)),
            Single::Atan(u) => 1 / ((u ^ 2) + 1),
            Single::Sinh(u) => cosh(u),
            Single::Cosh(u) => sinh(u),
            Single::Tanh(u) => 1 / (cosh(u) ^ 2),
            Single::Asinh(u) => 1 / sqrt((u ^ 2) + 1),
            Single::Acosh(u) => 1 / sqrt((u ^ 2) - 1),
            Single::Atanh(u) => 1 / (1 - (u ^ 2)),
            Single::Transpose(u) => {
                Single::Transpose(u.diff(symbol, steps)).into()
            }
            Single::Conj(_u) => todo!(),
            Single::Arg(_u) => todo!(),
            Single::Det(_u) => todo!(),
            Single::Norm(_u) => todo!(),
        }
    }
}

impl Differentiable for Variadic {
    fn diff(&self, symbol: Symbol, steps: &mut Option<Vec<Step>>) -> Expr {
        match self {
            Variadic::Add(terms) => Variadic::Add(
                terms.iter().map(|expr| expr.diff(symbol, steps)).collect(),
            )
            .into(),
            Variadic::Mul(terms) => Variadic::Add(
                terms
                    .iter()
                    .enumerate()
                    .map(|(i, expr)| {
                        let mut factors = Vec::with_capacity(terms.len());
                        factors.push(expr.diff(symbol, steps));
                        factors.extend(
                            terms
                                .iter()
                                .enumerate()
                                .filter_map(|(j, x)| (i != j).then_some(*x)),
                        );
                        Variadic::Mul(factors).into()
                    })
                    .collect(),
            )
            .into(),
        }
    }
}

impl Differentiable for Double {
    fn diff(&self, symbol: Symbol, steps: &mut Option<Vec<Step>>) -> Expr {
        match self {
            Double::Pow { base, exp } => {
                (base ^ exp)
                    * (base.diff(symbol, steps) * exp / base
                        + exp.diff(symbol, steps) * ln(base))
            }
            Double::Log { base, arg } => {
                if *base == e.into() {
                    arg.diff(symbol, steps) / arg
                } else if base.diff(symbol, steps) == 0.into() {
                    arg.diff(symbol, steps) / (arg * ln(base))
                } else {
                    ((arg.diff(symbol, steps) / arg) * ln(base)
                        - (base.diff(symbol, steps) / base) * ln(arg))
                        / (ln(base) ^ 2)
                }
            }
            Self::Atan2 { a: _, b: _ } => todo!(),
        }
    }
}
