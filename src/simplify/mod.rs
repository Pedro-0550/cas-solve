use std::{array, collections::HashMap};

use itertools::Itertools;
use num::{Zero, complex::ComplexFloat};

use crate::{
    core::scalar::Scalar,
    dimension::Quantity,
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic, cos, sin, tan},
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
        if self.node().is_symbol() || self.node().is_const() {
            return *self;
        }

        let mut step = self.normalize();

        loop {
            let simplified = match step.node() {
                Node::Variadic(variadic) => variadic.simplify(),
                Node::Single(single) => single.simplify(),
                Node::Double(double) => double.simplify(),
                Node::Matrix(matrix) => todo!(),
                _ => step,
            }
            .normalize();

            if simplified == step {
                break;
            }

            println!("{} -> {} PASSED", step, simplified);

            step = simplified;
        }

        step
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

macro_rules! impl_inv_trig_simplify {
    ($inv:ident, $fn:ident, $expr:ident, $self:ident) => {
        match $expr.node() {
            Node::Const(qty) => Scalar::from(qty.value().$fn()).into(),
            Node::Single(Single::$inv(x)) => x.simplify(),
            _ => $self.with_arg($self.arg().simplify()).into(),
        }
    };
}

impl Simplify for Single {
    fn simplify(&self) -> Expr {
        match self {
            Single::Sin(x) => impl_inv_trig_simplify!(Asin, sin, x, self),
            Single::Cos(x) => impl_inv_trig_simplify!(Acos, cos, x, self),
            Single::Tan(x) => impl_inv_trig_simplify!(Atan, tan, x, self),
            Single::Sinh(x) => impl_inv_trig_simplify!(Asinh, sinh, x, self),
            Single::Cosh(x) => impl_inv_trig_simplify!(Acosh, cosh, x, self),
            Single::Tanh(x) => impl_inv_trig_simplify!(Atanh, tanh, x, self),

            Single::Transpose(expr) => todo!(),
            Single::Conj(expr) => todo!(),
            Single::Arg(expr) => todo!(),
            Single::Det(expr) => todo!(),
            Single::Norm(expr) => todo!(),
            _ => self.with_arg(self.arg().simplify()).into(),
        }
    }
}

impl Simplify for Double {
    fn simplify(&self) -> Expr {
        let simplified = self
            .with_args(array::from_fn(|i| self.args()[i].simplify()))
            .into();

        match simplified {
            Double::Pow { base, exp } => {
                if let Node::Double(Double::Pow {
                    base: inner_base,
                    exp: inner_exp,
                }) = base.node()
                    && let Node::Const(exp) = exp.node()
                    && let Node::Const(inner_exp) = inner_exp.node()
                    && exp.value().is_integer()
                    && inner_exp.value().is_integer()
                {
                    (inner_base ^ (exp * inner_exp)).simplify()
                } else if let Node::Const(qty) = exp.node()
                    && qty.value().is_zero()
                {
                    (1.0).into()
                } else {
                    simplified.into()
                }
            }
            _ => simplified.into(),
        }
    }
}

impl Simplify for Variadic {
    fn simplify(&self) -> Expr {
        let simplified = self.operands_ref().iter().map(Expr::simplify);

        let mut groupings =
            HashMap::<Expr, Scalar>::with_capacity(self.operands_ref().len());

        for term in simplified.into_iter() {
            if self.is_add()
                && let Node::Variadic(Variadic::Mul(terms)) = term.node()
                && let (consts, exprs) = separate_consts(terms)
                && let Ok(coef) = consts.exactly_one()
            {
                *groupings
                    .entry(Variadic::Mul(exprs.collect()).into())
                    .or_insert(0.0.into()) += coef.value();
            } else if self.is_mul()
                && let Node::Double(Double::Pow { base, exp }) = term.node()
                && let Node::Const(exp) = exp.node()
                && exp.value().is_integer()
            {
                *groupings.entry(base).or_insert(0.0.into()) += exp.value();
            } else {
                *groupings.entry(term).or_insert(0.0.into()) += 1;
            }
        }

        println!(
            "===================== Expr {} grouped into:",
            Expr::from(self.clone())
        );
        for (k, v) in &groupings {
            println!("{} => {}", k, v);
        }

        let mut aggregated: Vec<Expr> = match self {
            Variadic::Add(_) => groupings
                .into_iter()
                .map(
                    |(base, coef)| {
                        if coef == 1.0.into() { base } else { base * coef }
                    },
                )
                .collect(),
            Variadic::Mul(_) => groupings
                .into_iter()
                .map(
                    |(base, exp)| {
                        if exp == 1.0.into() { base } else { base ^ exp }
                    },
                )
                .collect(),
        };

        if aggregated.len() <= 1 {
            aggregated.pop().unwrap_or(0.into())
        } else {
            self.with_operands(aggregated).into()
        }
    }
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
