use std::{
    array,
    collections::{HashMap, HashSet},
    mem::discriminant,
    path,
    rc::Rc,
};

use itertools::Itertools;
use num::Num;
use phf::Map;

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

const BEAM_WIDTH: usize = 128;
const MAX_DEPTH: usize = 8000;

/* --------------------------------- MODULES -------------------------------- */

mod algebraic;
#[cfg(test)]
mod test;
mod trig;

#[macro_export]
macro_rules! count {
    ($($x:tt)*) => {
        <[()]>::len(&[$($x)*])
    };
}
#[macro_export]
macro_rules! transformation {
    ($($sym:ident),+; $from:expr => $to:expr) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym), crate::dimension::Unit::Unitless);
        )+

        Transformation {
            from: crate::ast::Expr::from($from),
            to: crate::ast::Expr::from($to),
            conditions: std::rc::Rc::new(std::collections::HashMap::new())
        }
    }};
    ($($sym:ident),+; $from:expr => $to:expr, if $(|$cond_sym:ident| $cond:expr),+) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym), crate::dimension::Unit::Unitless);
        )+

        let mut conds = std::collections::HashMap::<Symbol, fn(crate::ast::Expr) -> bool>::new();

        $(
            conds.insert($cond_sym, |$cond_sym: crate::ast::Expr| $cond);
        )+

        Transformation {
            from: crate::ast::Expr::from($from),
            to: crate::ast::Expr::from($to),
            conditions: std::rc::Rc::new(conds)
        }
    }};
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Simplify {
    fn simplify(&self) -> Expr;
    fn range(&self) -> Range;
}

// pub struct MatrixSet {

// }

/* --------------------------------- STRUCTS -------------------------------- */

pub type PatternDomain = fn(Symbol) -> Range;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Transformation {
    pub from: Expr,
    pub to: Expr,

    /// For each symbol in From, this returns a range that bound expressions must be contained in or equal to.
    pub domain: PatternDomain,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Range {
    pub const UNBOUNDED: Self = todo!();
    pub const REAL: Self = todo!();
    pub const IMAG: Self = todo!();
    pub const NON_ZERO: Self = todo!();
}

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
                let rewritten = simplified.rewrite(t.clone(), false);
                if rewritten.structural_eq(simplified) {
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
                        let rewritten = path.1.rewrite(t.clone(), false);
                        if rewritten.structural_eq(path.1) {
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
    fn range(&self) -> Range {}
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
        self.with_operands(
            self.operands_ref().iter().map(Expr::simplify).collect::<Vec<_>>(),
        )
        .into()
    }
}
