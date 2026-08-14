use std::{
    array,
    collections::{HashMap, HashSet},
    mem::discriminant,
    path,
};

use itertools::Itertools;

use crate::{
    Scalar,
    ast::{
        Expr, Node,
        ops::{Double, Matrix, Variadic},
    },
    dimension::{Quantity, Unit},
    symbol::Symbol,
};

/* -------------------------------- CONSTANTS ------------------------------- */

const BEAM_WIDTH: usize = 64;
const MAX_DEPTH: usize = 8000;

/* --------------------------------- MODULES -------------------------------- */

mod algebraic;
#[cfg(test)]
mod test;
mod trig;

#[macro_export]
macro_rules! transformation {
    ($($sym:ident),+; $from:expr => $to:expr) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym), crate::dimension::Unit::Unitless);
        )+

        Transformation {
            from: crate::ast::Expr::from($from),
            to: crate::ast::Expr::from($to),
        }
    }};
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Simplify {
    fn simplify(&self) -> Expr;
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Transformation {
    pub from: Expr,
    pub to: Expr,
}
/* ---------------------------------- IMPLS --------------------------------- */

impl Simplify for Expr {
    fn simplify(&self) -> Expr {
        let mut step = *self;

        loop {
            let simplified = match step.node() {
                Node::Const(_) => step,
                Node::Symbol(_) => step,
                Node::Variadic(op) => op.simplify(),
                Node::Single(op) => op.with_arg(op.arg().simplify()).into(),
                Node::Double(op) => op.simplify(),
                Node::Matrix(m) => Node::Matrix(m.map(Expr::simplify)).into(),
            };

            let transformations: Vec<Transformation> = [
                trig::transformations().as_slice(),
                algebraic::transformations().as_slice(),
            ]
            .concat();

            // this could have been an array, but is it really worth it?
            let mut paths: Vec<(Transformation, Expr, usize)> =
                Vec::with_capacity(BEAM_WIDTH);
            let mut best = simplified;
            let mut best_size = simplified.size();

            // Seed search paths
            for t in &transformations {
                let rewritten = simplified.rewrite(*t, false);
                if rewritten.structural_eq(simplified) {
                    continue;
                }
                let size = rewritten.size();

                if size < best_size {
                    best = rewritten;
                    best_size = size;
                }

                paths.push((*t, rewritten, rewritten.size()));
            }

            for _ in 0..MAX_DEPTH {
                let mut leafs = Vec::with_capacity(transformations.len());

                for path in paths.clone().iter() {
                    for t in &transformations {
                        let rewritten = path.1.rewrite(*t, false);
                        if rewritten.structural_eq(path.1) {
                            continue;
                        }

                        let size = rewritten.size();

                        if size < best_size {
                            best = rewritten;
                            best_size = size;
                        }

                        leafs.push((*t, rewritten, rewritten.size()));
                    }
                }

                leafs.sort_by(|a, b| a.2.cmp(&b.2));

                if leafs.is_empty() {
                    break;
                }

                paths = leafs.iter().cloned().take(BEAM_WIDTH).collect();
            }

            // for transformation in transformations {
            //     simplified = simplified.rewrite(transformation)
            // }

            if best.structural_eq(step) {
                break;
            }

            step = best;
        }

        step
    }
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
}

impl Simplify for Variadic {
    fn simplify(&self) -> Expr {
        let simplified =
            self.operands_ref().iter().map(Expr::simplify).collect::<Vec<_>>();

        match self {
            Variadic::Add(_) => {
                let flattened = simplified
                    .into_iter()
                    .flat_map(|expr| match expr.node() {
                        Node::Variadic(Variadic::Add(exprs)) => exprs,
                        _ => vec![expr],
                    })
                    .collect::<Vec<_>>();

                let (mut folded, constant) =
                    fold_consts(flattened, 0.into(), |acc, x| {
                        if acc.unit() == Unit::Unitless {
                            (acc * x.unit()) + x
                        } else {
                            acc + x
                        }
                    });

                if constant.value() != 0.0.into() {
                    folded.push(constant.into());
                }

                if folded.len() <= 1 {
                    return folded.try_remove(0).unwrap_or(0.into());
                } else {
                    Variadic::Add(folded).into()
                }
            }
            Variadic::Mul(_) => {
                let flattened = simplified
                    .into_iter()
                    .flat_map(|expr| match expr.node() {
                        Node::Variadic(Variadic::Mul(exprs)) => exprs,
                        _ => vec![expr],
                    })
                    .collect::<Vec<_>>();

                let (folded, constant) =
                    fold_consts(flattened, 1.into(), |acc, x| acc * x);

                let mut groupings =
                    Vec::<(Expr, usize)>::with_capacity(folded.len());

                for term in folded.into_iter() {
                    if let Some((_, occurances)) = groupings
                        .iter_mut()
                        .find(|(t, _)| t.structural_eq(term))
                    {
                        *occurances += 1;
                    } else {
                        groupings.push((term, 1));
                    }
                }

                let mut aggregated: Vec<_> = groupings
                    .into_iter()
                    .map(|(base, exp)| {
                        if exp == 1 {
                            base
                        } else {
                            base ^ Scalar::from(exp as f32)
                        }
                    })
                    .collect();

                aggregated.insert(0, constant.into());

                if aggregated.len() <= 1 {
                    return aggregated.try_remove(0).unwrap_or(0.into());
                } else {
                    Variadic::Mul(aggregated).into()
                }
            }
        }
    }
}

/* -------------------------------- FUNCTIONS ------------------------------- */

fn fold_consts(
    mut operands: Vec<Expr>,
    init: Quantity,
    fold: impl FnMut(Quantity, Quantity) -> Quantity,
) -> (Vec<Expr>, Quantity) {
    let folded_const = operands
        .iter()
        .filter_map(|expr| {
            if let Node::Const(val) = expr.node() { Some(val) } else { None }
        })
        .fold(init, fold);

    operands.retain(|expr| !matches!(expr.node(), Node::Const(_)));

    (operands, folded_const)
}
