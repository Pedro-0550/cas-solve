use std::array;

use itertools::Itertools;

use crate::{
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic},
    },
    normal::Normalize,
    // set::Set,
};

/* -------------------------------- CONSTANTS ------------------------------- */

const BEAM_WIDTH: usize = 16;
const MAX_DEPTH: usize = 800;

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
            from: crate::expr::Expr::from($from),
            to: crate::expr::Expr::from($to),
        }
    }};
    ($($sym:ident: $set:expr),+; $from:expr => $to:expr) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym)).set_domain($set);
        )+

        Transformation {
            from: crate::expr::Expr::from($from),
            to: crate::expr::Expr::from($to),
        }
    }};
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Simplify {
    fn simplify(&self) -> Expr;
    // fn range(&self) -> Set;
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Transformation {
    pub from: Expr,
    pub to: Expr,
}

// struct Path {
//     expr: Expr,
//     cost: usize,
//     seen: HashSet<Expr>,
// }

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

        for i in 0..MAX_DEPTH {
            let mut leafs = Vec::with_capacity(transformations.len());

            for path in paths.clone().iter() {
                for t in &transformations {
                    println!(
                        "Rewriting {} at depth {} with transformation {} -> {}",
                        path.1, i, t.from, t.to
                    );

                    let rewritten = path.1.rewrite(t.clone(), true).normalize();
                    if rewritten == path.1 {
                        println!("Didn't do anything, continuing",);

                        continue;
                    }

                    let size = rewritten.size();

                    if size < best_size {
                        best = rewritten;
                        best_size = size;
                    }

                    println!(
                        "Transformation {} -> {} ------------ PASSED",
                        t.from, t.to
                    );

                    leafs.push((t.clone(), rewritten, rewritten.size()));
                }
            }

            println!("At depth {}, best size = {}", i, best_size);

            if leafs.is_empty() {
                println!(
                    "Leafs empty at depth {}, best size = {}",
                    i, best_size
                );
                break;
            }

            if best_size == 1 {
                break;
            }

            leafs.sort_by(|a, b| a.2.cmp(&b.2));
            paths.clear();
            paths.extend(leafs.iter().cloned().take(BEAM_WIDTH));
        }

        best
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

impl Simplify for Single {
    fn simplify(&self) -> Expr {
        todo!()
    }

    // fn range(&self) -> Set {
    //     match self {
    //         Single::Neg(_expr) => todo!(),
    //         Single::Sin(_expr) => todo!(),
    //         Single::Cos(_expr) => todo!(),
    //         Single::Tan(_expr) => todo!(),
    //         Single::Asin(_expr) => todo!(),
    //         Single::Acos(_expr) => todo!(),
    //         Single::Atan(_expr) => todo!(),
    //         Single::Sinh(_expr) => todo!(),
    //         Single::Cosh(_expr) => todo!(),
    //         Single::Tanh(_expr) => todo!(),
    //         Single::Asinh(_expr) => todo!(),
    //         Single::Acosh(_expr) => todo!(),
    //         Single::Atanh(_expr) => todo!(),
    //         Single::Transpose(_expr) => todo!(),
    //         Single::Conj(_expr) => todo!(),
    //         Single::Arg(_expr) => todo!(),
    //         Single::Det(_expr) => todo!(),
    //         Single::Norm(_expr) => todo!(),
    //     }
    // }
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

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

impl Simplify for Variadic {
    fn simplify(&self) -> Expr {
        self.with_operands(
            self.operands_ref().iter().map(Expr::simplify).collect::<Vec<_>>(),
        )
        .into()
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}
