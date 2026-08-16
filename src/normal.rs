use std::{collections::HashMap, iter::once, mem::discriminant};

use num::complex::ComplexFloat;

use crate::{
    Scalar,
    dimension::Quantity,
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic},
    },
    symbol::Symbol,
};

/* --------------------------------- TRAITS --------------------------------- */

/// Adds suport for conversion into a standard form, without altering the mathematical value of an expression.
/// Variadic expressions have a standard order in which theyre stored:
/// * Symbols, sorted by name then by internal id;
/// * Functions, sorted by name and arguments;
/// * Consts, sorted as purely reals first, then purely imaginary, whcih are subsorted by value, then complex, sorted by norm;
/// * Other variadics, which are sorted by their operands;
/// * and Matrices, which are sorted by (TODO)
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

        let (consts, exprs) = (
            flattened.clone().filter_map(|expr| match expr.node() {
                Node::Const(qty) => Some(qty),
                _ => None,
            }),
            flattened.filter(|expr| !expr.node().is_const()),
        );

        let mut groupings =
            HashMap::<Expr, usize>::with_capacity(self.operands_ref().len());

        for term in exprs.into_iter() {
            *groupings.entry(term).or_insert(0) += 1;
        }

        let (mut result, positive) = match self {
            Variadic::Add(_) => {
                let folded_const = consts.fold(0.into(), |acc: Quantity, x| {
                    (acc.value() + x.value()) * x.unit()
                });

                let mut aggregated: Vec<_> = groupings
                    .into_iter()
                    .map(|(base, mul)| {
                        if mul == 1 {
                            base
                        } else {
                            base * Scalar::from(mul as f32)
                        }
                    })
                    .collect();

                if folded_const.value() != 0.0.into() {
                    aggregated.push(folded_const.into());
                }

                (aggregated, true)
            }
            Variadic::Mul(_) => {
                let folded_const =
                    consts.fold(1.into(), |acc: Quantity, x| acc * x);

                let aggregated = groupings.into_iter().map(|(base, exp)| {
                    if exp == 1 {
                        base
                    } else {
                        base ^ Scalar::from(exp as f32)
                    }
                });

                if folded_const.value() == 0.0.into() {
                    return 0.0.into();
                }

                let mut n_negative = 0;
                let mut unsigned = aggregated
                    .chain(once(folded_const.into()))
                    .map(|x| match x.node() {
                        Node::Single(Single::Neg(expr)) => {
                            n_negative += 1;
                            expr
                        }
                        _ => x,
                    })
                    .collect::<Vec<_>>();

                if folded_const.value().abs() == 1.0 {
                    unsigned.pop();
                }

                (unsigned, n_negative % 2 == 0)
            }
        };

        result.sort_unstable();

        let result = if result.len() <= 1 {
            result.pop().unwrap_or(0.into())
        } else {
            self.with_operands(result).into()
        };

        if positive { result } else { Single::Neg(result).into() }
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
            Single::Neg(_) => 0,
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
    use crate::{dimension::Unit, normal::Normalize, symbol::Symbol};

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
