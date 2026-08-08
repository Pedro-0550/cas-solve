use itertools::Itertools;
use num::traits::{ConstOne, ConstZero};

use crate::{Num, expr::Expr};

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(PartialEq, Clone)]
pub enum Intrinsic {
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Neg(Expr),

    Sin(Expr),
    Cos(Expr),
    Asin(Expr),
    Acos(Expr),

    Pow { base: Expr, exp: Expr },
    Log { base: Expr, arg: Expr },
    Norm(Expr),

    Inv(Expr),
    Transpose(Expr),
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Intrinsic {
    pub fn simplify(&self) -> Expr {
        match self {
            // Intrinsic::Add(operands) => {
            //     let mut simplified =
            //         fold_consts(operands, Num::ZERO, |accum, x| accum + x);

            //     if simplified.len() <= 1 {
            //         return simplified
            //             .try_remove(0)
            //             .unwrap_or(Expr::Const(Num::ZERO));
            //     } else {
            //         return Expr::Intrinsic(Box::new(Intrinsic::Add(
            //             simplified,
            //         )));
            //     }
            // }
            Intrinsic::Mul(operands) => {
                let (mut simplified_operands, folded_const) =
                    fold_consts(operands, Num::ONE, |accum, x| accum * x);

                if folded_const == Num::ZERO {
                    return Expr::Const(Num::ZERO);
                }

                // Distributive Property
                let mut add_ops = Vec::new();

                simplified_operands.retain_mut(|expr| match expr {
                    Expr::Intrinsic(intr)
                        if let Intrinsic::Add(ops) = &mut **intr =>
                    {
                        add_ops.push(*ops);
                        false
                    }
                    _ => true,
                });

                let mut remaining_factors = simplified_operands;
                if folded_const != Num::ONE {
                    remaining_factors.push(Expr::Const(folded_const));
                }

                let mut expanded = if add_ops.is_empty() {
                    Intrinsic::Mul(remaining_factors)
                } else {
                    Intrinsic::Add(
                        add_ops
                            .into_iter()
                            .multi_cartesian_product()
                            .map(|mut exprs| {
                                exprs.extend(remaining_factors.iter().cloned());
                                Expr::Intrinsic(Box::new(Intrinsic::Mul(exprs)))
                            })
                            .collect::<Vec<_>>(),
                    )
                };

                return Expr::Intrinsic(Box::new(expanded));
            }
        }
    }
}

fn fold_consts(
    operands: &Vec<Expr>,
    init: Num,
    fold: impl FnMut(Num, &Num) -> Num,
) -> (Vec<Expr>, Num) {
    let mut simplified_operands =
        operands.iter().map(Expr::simplify).collect::<Vec<_>>();

    let folded_const =
        simplified_operands
            .iter()
            .filter_map(|expr| {
                if let Expr::Const(val) = expr { Some(val) } else { None }
            })
            .fold(init, fold);

    simplified_operands.retain(|expr| !matches!(expr, Expr::Const(_)));

    (simplified_operands, folded_const)
}
