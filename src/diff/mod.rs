use num::complex::Complex64;

use crate::{
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic, cos, cosh, ln, sin, sinh, sqrt},
    },
    simplify::{Simplify, SimplifyContext, normal::Normalize},
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
    fn diff(&self, s: Symbol) -> Expr {
        let mut ctx = SimplifyContext::new();
        match self.normalize(true).into_node() {
            Node::Const(_) => 0.into(),
            Node::Symbol(sym) => if sym == s { 1 } else { 0 }.into(),
            Node::Variadic(op) => op.diff(s),
            Node::Single(op) => op.arg().diff(s) * op.diff(s),
            Node::Double(op) => op.diff(s),
            _ => todo!(),
        }
        .simplify_inner(&mut ctx)
    }
}

impl Differentiable for Single {
    fn diff(&self, s: Symbol) -> Expr {
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
            Single::Transpose(u) => Single::Transpose(u.diff(s)).into(),
            Single::Conj(_u) => todo!(),
            Single::Arg(_u) => todo!(),
            Single::Det(_u) => todo!(),
            Single::Norm(_u) => todo!(),
        }
    }
}

impl Differentiable for Variadic {
    fn diff(&self, s: Symbol) -> Expr {
        match self {
            Variadic::Add(terms) => {
                Variadic::Add(terms.iter().map(|expr| expr.diff(s)).collect())
                    .into()
            }
            Variadic::Mul(terms) => Variadic::Add(
                terms
                    .iter()
                    .enumerate()
                    .map(|(i, expr)| {
                        let mut factors = Vec::with_capacity(terms.len());
                        factors.push(expr.diff(s));
                        factors.extend(terms.iter().enumerate().filter_map(
                            |(j, x)| (i != j).then_some(x.clone()),
                        ));
                        Variadic::Mul(factors).into()
                    })
                    .collect(),
            )
            .into(),
        }
    }
}

impl Differentiable for Double {
    fn diff(&self, s: Symbol) -> Expr {
        match self {
            Double::Pow { base, exp } => {
                (base ^ exp)
                    * (base.diff(s) * exp / base + exp.diff(s) * ln(base))
            }
            Double::Log { base, arg } => {
                if *base == e.into() {
                    arg.diff(s) / arg
                } else if base.diff(s) == 0.into() {
                    arg.diff(s) / (arg * ln(base))
                } else {
                    ((arg.diff(s) / arg) * ln(base)
                        - (base.diff(s) / base) * ln(arg))
                        / (ln(base) ^ 2)
                }
            }
            Self::Atan2 { a: _, b: _ } => todo!(),
        }
    }
}
