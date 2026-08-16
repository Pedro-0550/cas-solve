use std::{
    array,
    collections::{HashMap, HashSet},
    f64::consts::{FRAC_PI_2, PI},
    mem::discriminant,
    path,
    rc::Rc,
};

use itertools::Itertools;
use num::Num;

use crate::{
    Scalar,
    ast::{
        Expr, Node,
        ops::{Double, Matrix, Single, Variadic},
    },
    dimension::{Quantity, Unit},
    normal::Normalize,
    set::{Bound, Interval, Set, closed, open},
    symbol::Symbol,
};

/* -------------------------------- CONSTANTS ------------------------------- */

const BEAM_WIDTH: usize = 128;
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
            let $sym = crate::symbol::Symbol::new(stringify!($sym));
        )+

        Transformation {
            from: crate::ast::Expr::from($from),
            to: crate::ast::Expr::from($to),
        }
    }};
    ($($sym:ident: $set:expr),+; $from:expr => $to:expr) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym)).set_domain($set);
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
    fn range(&self) -> Set;
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Transformation {
    pub from: Expr,
    pub to: Expr,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Simplify for Expr {
    fn simplify(&self) -> Expr {
        let normalized_self = self.normalize();

        let transformations: Vec<Transformation> = [
            trig::transformations().as_slice(),
            algebraic::transformations().as_slice(),
        ]
        .concat();

        // this could have been an array, but is it really worth it?
        let mut paths: Vec<(Transformation, Expr, usize)> =
            Vec::with_capacity(BEAM_WIDTH);
        let mut best = normalized_self;
        let mut best_size = normalized_self.size();

        // Seed search paths
        for t in &transformations {
            let rewritten =
                normalized_self.rewrite(t.clone(), true).normalize();
            if rewritten == normalized_self {
                continue;
            }
            let size = rewritten.size();

            if size < best_size {
                best = rewritten;
                best_size = size;
            }

            paths.push((t.clone(), rewritten, rewritten.size()));
        }

        for _ in 0..MAX_DEPTH {
            let mut leafs = Vec::with_capacity(transformations.len());

            for path in paths.clone().iter() {
                for t in &transformations {
                    let rewritten =
                        path.1.rewrite(t.clone(), false).normalize();
                    if rewritten == path.1 {
                        continue;
                    }

                    let size = rewritten.size();

                    if size < best_size {
                        best = rewritten;
                        best_size = size;
                    }

                    leafs.push((t.clone(), rewritten, rewritten.size()));
                }
            }

            if leafs.is_empty() {
                break;
            }

            leafs.sort_by(|a, b| a.2.cmp(&b.2));
            paths.clear();
            paths.extend(leafs.iter().cloned().take(BEAM_WIDTH));
        }

        best
    }

    fn range(&self) -> Set {
        todo!()
    }
}

impl Simplify for Single {
    fn simplify(&self) -> Expr {
        todo!()
    }

    fn range(&self) -> Set {
        match self {
            Single::Neg(expr) => todo!(),
            Single::Sin(expr) => todo!(),
            Single::Cos(expr) => todo!(),
            Single::Tan(expr) => todo!(),
            Single::Asin(expr) => todo!(),
            Single::Acos(expr) => todo!(),
            Single::Atan(expr) => todo!(),
            Single::Sinh(expr) => todo!(),
            Single::Cosh(expr) => todo!(),
            Single::Tanh(expr) => todo!(),
            Single::Asinh(expr) => todo!(),
            Single::Acosh(expr) => todo!(),
            Single::Atanh(expr) => todo!(),
            Single::Transpose(expr) => todo!(),
            Single::Conj(expr) => todo!(),
            Single::Arg(expr) => todo!(),
            Single::Det(expr) => todo!(),
            Single::Norm(expr) => todo!(),
        }
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

    fn range(&self) -> Set {
        todo!()
    }
}

impl Simplify for Variadic {
    fn simplify(&self) -> Expr {
        self.with_operands(
            self.operands_ref().iter().map(Expr::simplify).collect::<Vec<_>>(),
        )
        .into()
    }

    fn range(&self) -> Set {
        todo!()
    }
}
