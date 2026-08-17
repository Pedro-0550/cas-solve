use std::{collections::HashMap, iter::once, mem::discriminant};

use itertools::Itertools;
use num::complex::ComplexFloat;

use super::separate_consts;
use crate::{
    core::scalar::Scalar,
    dimension::Quantity,
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic},
    },
    symbol::Symbol,
};

/* --------------------------------- TRAITS --------------------------------- */

/// Adds support for conversion into a standard form, without touching symbols or simplifying algebraic constructions
/// To be exact, normalize will only:
///  * Flatten nested variadics;
///  * Fold negation, such that `(-a) * (-b) -> a * b` and `(-a) * b -> -(a * b)`
///  * Fold constants into a single term;
///  * And sort terms in a standard, deterministic order
pub trait Normalize {
    fn normalize(&self) -> Expr;

    /// Returns the rank of this expression, not considering its children.
    /// In this context, rank defines the sorting ordedr during normalization.
    fn rank(&self) -> usize;
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Normalize for Variadic {
    fn normalize(&self) -> Expr {
        let normalized = self.operands_ref().iter().map(Expr::normalize);

        let flattened = normalized.flat_map(|expr| match expr.node() {
            Node::Variadic(op) if discriminant(&op) == discriminant(&self) => {
                op.operands()
            }
            _ => vec![expr],
        });

        let (consts, exprs) = separate_consts(flattened);

        // let mut groupings =
        //     HashMap::<Expr, Scalar>::with_capacity(self.operands_ref().len());

        // for term in exprs.into_iter() {
        //     if self.is_add()
        //         && let Node::Variadic(Variadic::Mul(terms)) = term.node()
        //         && let (consts, exprs) = separate_consts(terms)
        //         && let Ok(coef) = consts.exactly_one()
        //     {
        //         *groupings
        //             .entry(Variadic::Mul(exprs.collect()).into())
        //             .or_insert(0.0.into()) += coef.value();
        //     } else if self.is_mul()
        //         && let Node::Double(Double::Pow { base, exp }) = term.node()
        //         && let Node::Const(qty) = exp.node()
        //         && qty.value().is_integer()
        //     {
        //         *groupings.entry(base).or_insert(0.0.into()) += qty.value();
        //     } else {
        //         *groupings.entry(term).or_insert(0.0.into()) += 1;
        //     }
        // }

        let mut result = match self {
            Variadic::Add(_) => {
                let mut exprs = exprs.collect_vec();

                let folded_const = consts.fold(0.into(), |acc: Quantity, x| {
                    (acc.value() + x.value()) * x.unit()
                });

                // let mut aggregated: Vec<_> = groupings
                //     .into_iter()
                //     .map(
                //         |(base, mul)| {
                //             if mul == 1.0.into() { base } else { base * mul }
                //         },
                //     )
                //     .collect();

                if folded_const.value() != 0.0.into() || exprs.len() == 0 {
                    exprs.push(folded_const.into());
                }

                exprs
            }
            Variadic::Mul(_) => {
                let folded_const =
                    consts.fold(1.into(), |acc: Quantity, x| acc * x);

                // let aggregated = groupings.into_iter().map(|(base, exp)| {
                //     if exp == 1.0.into() { base } else { base ^ exp }
                // });

                if folded_const.value() == 0.0.into() {
                    return 0.0.into();
                }

                let mut exprs = exprs.collect_vec();

                if folded_const.value() != 1.0.into() || exprs.len() == 0 {
                    exprs.push(folded_const.into());
                }

                exprs
            }
        };

        result.sort_unstable();

        if result.len() <= 1 {
            result.pop().unwrap_or(0.into())
        } else {
            self.with_operands(result).into()
        }
    }

    fn rank(&self) -> usize {
        match self {
            Variadic::Mul(_) => 0,
            Variadic::Add(_) => 1,
        }
    }
}

impl Normalize for Single {
    fn normalize(&self) -> Expr {
        self.with_arg(self.arg().normalize()).into()
    }

    fn rank(&self) -> usize {
        match self {
            // Why does this start at one? We had a 0 variant but i removed it, and writing this comment definetly took
            // less time than shifting all the numbers.
            Single::Sin(_) => 1,
            Single::Cos(_) => 2,
            Single::Tan(_) => 3,
            Single::Asin(_) => 4,
            Single::Acos(_) => 5,
            Single::Atan(_) => 6,
            Single::Sinh(_) => 7,
            Single::Cosh(_) => 8,
            Single::Tanh(_) => 9,
            Single::Asinh(_) => 10,
            Single::Acosh(_) => 11,
            Single::Atanh(_) => 12,
            Single::Transpose(_) => 13,
            Single::Conj(_) => 14,
            Single::Arg(_) => 15,
            Single::Det(_) => 16,
            Single::Norm(_) => 17,
        }
    }
}

impl Normalize for Double {
    fn normalize(&self) -> Expr {
        self.with_args([self.args()[0].normalize(), self.args()[1].normalize()])
            .into()
    }

    fn rank(&self) -> usize {
        match self {
            Double::Pow { .. } => 0,
            Double::Log { .. } => 1,
            Double::Atan2 { .. } => 2,
        }
    }
}

impl Normalize for Expr {
    fn normalize(&self) -> Self {
        match self.node() {
            Node::Symbol(_) => *self,
            Node::Const(_) => *self,
            Node::Variadic(variadic) => variadic.normalize(),
            Node::Single(single) => single.normalize(),
            Node::Double(double) => double.normalize(),
            Node::Matrix(_matrix) => todo!(),
        }
    }

    fn rank(&self) -> usize {
        match self.node() {
            Node::Symbol(_) => 0,
            Node::Const(_) => 1,
            Node::Single(_) => 2,
            Node::Double(_) => 3,
            Node::Variadic(_) => 4,
            Node::Matrix(_) => 5,
        }
    }
}

impl Ord for Expr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank()).then_with(|| {
            match (self.node(), other.node()) {
                (Node::Symbol(lhs), Node::Symbol(rhs)) => lhs.cmp(&rhs),
                (Node::Const(lhs), Node::Const(rhs)) => {
                    let lhs = lhs.value();
                    let rhs = rhs.value();
                    lhs.norm()
                        .total_cmp(&rhs.norm())
                        .then_with(|| lhs.arg().total_cmp(&rhs.arg()))
                }
                (Node::Single(lhs), Node::Single(rhs)) => lhs
                    .rank()
                    .cmp(&rhs.rank())
                    .then_with(|| lhs.arg().cmp(&rhs.arg())),
                (Node::Double(lhs), Node::Double(rhs)) => lhs
                    .rank()
                    .cmp(&rhs.rank())
                    .then_with(|| lhs.args()[0].cmp(&rhs.args()[0]))
                    .then_with(|| lhs.args()[1].cmp(&rhs.args()[1])),
                (Node::Variadic(lhs), Node::Variadic(rhs)) => {
                    lhs.rank().cmp(&rhs.rank()).then_with(|| {
                        lhs.operands_ref().iter().cmp(rhs.operands_ref().iter())
                    })
                }
                (Node::Matrix(_lhs), Node::Matrix(_rhs)) => todo!(),
                _ => unreachable!(
                    "Only two nodes of the same variant can be Ordering::Equal",
                ),
            }
        })
    }
}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(&other.name()).then_with(|| self.0.0.cmp(&other.0.0))
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use crate::{dimension::Unit, simplify::normal::Normalize, symbol::Symbol};

    #[test]
    fn normalization() {
        let a = Symbol::new("a");
        let b = Symbol::new("b");
        let c = Symbol::new("c");

        panic!(
            "{}, {}",
            (a * b * -c + 0).normalize(),
            (-(1 * a * b * c)).normalize()
        );
    }
}
