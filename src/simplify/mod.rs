use itertools::Itertools;

use crate::{
    ast::{Expr, Node, intrinsic::Intrinsic},
    dimension::{Quantity, Unit},
};

/* --------------------------------- MODULES -------------------------------- */

mod algebraic;
mod trig;

#[macro_export]
macro_rules! identity {
    ($($sym:ident),+; $from:expr => $to:expr) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym), crate::dimension::Unit::Unitless);
        )+

        Identity {
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

pub struct Identity {
    from: Expr,
    to: Expr,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Simplify for Expr {
    fn simplify(self) -> Expr {
        match self.node() {
            Node::Const(_) => self,
            Node::Symbol(_) => self,
            Node::Intrinsic(intr) => intr.simplify(),
            _ => todo!(),
        }
    }
}

impl Simplify for Intrinsic {
    fn simplify(self) -> Expr {
        match self {
            Intrinsic::Add(operands) => {
                let mut simplified = operands
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
            Intrinsic::Pow { base, exp } => self.into(),
            _ => todo!(),
        }
    }
}

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
