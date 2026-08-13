use std::{collections::HashMap, mem::discriminant};

use itertools::Itertools;

use crate::{
    ast::{Expr, Node, intrinsic::Intrinsic},
    dimension::{Quantity, Unit},
    symbol::Symbol,
};

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
    fn simplify(self) -> Expr;
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy)]
pub struct Transformation {
    pub from: Expr,
    pub to: Expr,
}
/* ---------------------------------- IMPLS --------------------------------- */

impl Simplify for Expr {
    fn simplify(self) -> Expr {
        let mut step = self;
        let mut step_i = 0;
        println!("expr {self} | step {step_i}: {self}");

        loop {
            let mut simplified = match step.node() {
                Node::Const(_) => step,
                Node::Symbol(_) => step,
                Node::Intrinsic(intr) => intr.simplify(),
                _ => todo!(),
            };

            let transformations: Vec<Transformation> = [
                trig::transformations().as_slice(),
                algebraic::transformations().as_slice(),
            ]
            .concat();

            println!("expr {self} | step {step_i}: {simplified}");
            step_i += 1;

            for transformation in transformations {
                simplified = simplified.rewrite(transformation)
            }

            if simplified.structural_eq(step) {
                break;
            }

            step = simplified;
        }

        return step;
    }
}

impl Simplify for Intrinsic {
    fn simplify(self) -> Expr {
        match self {
            Intrinsic::Add(operands) => {
                let simplified = operands
                    .into_iter()
                    .map(Expr::simplify)
                    .collect::<Vec<_>>();

                let flattened = simplified
                    .into_iter()
                    .flat_map(|expr| {
                        expr.node()
                            .as_intrinsic()
                            .and_then(|intr| intr.as_add().cloned())
                            .unwrap_or(vec![expr])
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

                if constant != Quantity::from(0.0) {
                    folded.push(constant.into());
                }

                if folded.len() <= 1 {
                    return folded.try_remove(0).unwrap_or(0.into());
                } else {
                    Intrinsic::Add(folded).into()
                }
            }
            Intrinsic::Mul(terms) => {
                let simplified =
                    terms.into_iter().map(Expr::simplify).collect::<Vec<_>>();

                let flattened = simplified
                    .into_iter()
                    .flat_map(|expr| {
                        expr.node()
                            .as_intrinsic()
                            .and_then(|intr| intr.as_mul().cloned())
                            .unwrap_or(vec![expr])
                    })
                    .collect::<Vec<_>>();

                let (mut folded, constant) =
                    fold_consts(flattened, 1.into(), |acc, x| acc * x);

                let mut grouped = folded
                    .chunk_by(|a, b| a == b)
                    .map(|group| (group[0], group.len(), false))
                    .filter(|(_, len, _)| *len > 1)
                    .collect::<Vec<_>>();

                for term in folded.iter_mut() {
                    let Some((group_term, occurrances, replaced)) =
                        grouped.iter_mut().find(|(t, ..)| t == term)
                    else {
                        continue;
                    };

                    if !*replaced {
                        *replaced = true;
                        *term = *group_term ^ *occurrances as i64;
                    }
                }

                folded.retain(|a| !grouped.iter().any(|(b, ..)| a == b));

                folded.insert(0, constant.into());

                if folded.len() <= 1 {
                    return folded.try_remove(0).unwrap_or(0.into());
                } else {
                    Intrinsic::Mul(folded).into()
                }
            }
            Intrinsic::Pow { base, exp } => {
                Intrinsic::Pow { base: base.simplify(), exp: exp.simplify() }
                    .into()
            }
            Intrinsic::Log { base, arg } => {
                Intrinsic::Log { base: base.simplify(), arg: arg.simplify() }
                    .into()
            }
            Intrinsic::Neg(expr) => Intrinsic::Neg(expr.simplify()).into(),
            Intrinsic::Sin(expr) => Intrinsic::Sin(expr.simplify()).into(),
            Intrinsic::Cos(expr) => Intrinsic::Cos(expr.simplify()).into(),
            Intrinsic::Asin(expr) => Intrinsic::Asin(expr.simplify()).into(),
            Intrinsic::Acos(expr) => Intrinsic::Acos(expr.simplify()).into(),
            Intrinsic::Norm(expr) => Intrinsic::Norm(expr.simplify()).into(),
            Intrinsic::Inv(expr) => Intrinsic::Inv(expr.simplify()).into(),
            Intrinsic::Transpose(expr) => {
                Intrinsic::Transpose(expr.simplify()).into()
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
