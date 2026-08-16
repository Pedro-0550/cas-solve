use std::{
    array,
    fmt::{Display, Pointer},
    mem::discriminant,
};

use derive_more::{From, IsVariant};
use itertools::Itertools;

use crate::{
    arena::{Arena, Handle},
    dimension::Quantity,
    expr::ops::{Double, Matrix, Single, Variadic},
    normal::Normalize,
    simplify::{Simplify, Transformation},
    symbol::Symbol,
};

/* -------------------------------- CONSTANTS ------------------------------- */

static NODES: Arena<Node> = Arena::new();

/* --------------------------------- MODULES -------------------------------- */

pub mod impls;
pub mod ops;

#[cfg(test)]
mod test;
/* ---------------------------------- ENUMS --------------------------------- */

#[derive(PartialEq, Clone, Debug, From, IsVariant)]
pub enum Node {
    Symbol(Symbol),
    Const(Quantity),

    Variadic(Variadic),
    Single(Single),
    Double(Double),

    #[from(ignore)]
    Matrix(Matrix),
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Expr(Handle<Node>);

#[derive(Clone, PartialEq, Eq)]
pub struct Binding {
    from: Symbol,
    to: Expr,
}

pub enum Match {
    Whole,
    Terms(Vec<(usize, usize)>),
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Expr {
    pub fn node(&self) -> Node {
        NODES.get_cloned(self.0).unwrap()
    }

    /// Returns the total number of nodes in this expression
    pub fn size(&self) -> usize {
        match self.node() {
            Node::Symbol(_symbol) => 1,
            Node::Const(_quantity) => 1,
            Node::Variadic(variadic) => {
                variadic.operands_ref().iter().map(|x| x.size()).sum::<usize>()
                    + 1
            }
            Node::Single(single) => single.arg().size() + 1,
            Node::Double(double) => {
                double.args()[0].size() + double.args()[1].size() + 1
            }
            Node::Matrix(matrix) => {
                matrix.elements().iter().map(|x| x.size()).sum::<usize>() + 1
            }
        }
    }

    /// Returns true if two expressions's normalized forms are equal, such as `(a + b) * x` and `x * (b + a)`.
    /// This method does not perform simplification before comparison.
    pub fn structural_eq(&self, rhs: Expr) -> bool {
        self.normalize() == rhs.normalize()
    }

    pub fn substitute(self, bindings: &[Binding]) -> Self {
        match self.node() {
            Node::Variadic(op) => op
                .with_operands(
                    op.operands_ref()
                        .iter()
                        .map(|x| x.substitute(bindings))
                        .collect(),
                )
                .into(),

            Node::Single(op) => {
                op.with_arg(op.arg().substitute(bindings)).into()
            }

            Node::Double(op) => op
                .with_args(array::from_fn(|i| {
                    op.args()[i].substitute(bindings)
                }))
                .into(),

            Node::Symbol(sym) => {
                if let Some(binding) = bindings.iter().find(|b| b.from == sym) {
                    binding.to
                } else {
                    self
                }
            }

            Node::Matrix(m) => {
                Node::Matrix(m.map(|el| el.substitute(bindings))).into()
            }

            _ => self,
        }
    }

    /// Rewrites this expression by trying to apply a transformation. If the transformation pattern does not match, does nothing.
    pub fn rewrite(
        self,
        transformation: Transformation,
        recursive: bool,
    ) -> Self {
        let mut bindings = Vec::new();

        fn replace_terms(
            transformation: Transformation,
            bindings: &[Binding],
            terms: &[Expr],
            matches: &[usize],
        ) -> Vec<Expr> {
            let mut new_terms: Vec<_> = terms
                .into_iter()
                .enumerate()
                .filter_map(|(i, term)| {
                    (!matches.contains(&i)).then_some(term).cloned()
                })
                .collect();
            new_terms.push(transformation.to.substitute(bindings));
            new_terms
        }

        match self.match_by(transformation.from, &mut bindings) {
            Some(Match::Whole) => transformation.to.substitute(&bindings),
            Some(Match::Terms(matches)) => {
                let Node::Variadic(op) = self.node() else {
                    unreachable!();
                };

                op.with_operands(replace_terms(
                    transformation,
                    &bindings,
                    &op.operands_ref(),
                    matches
                        .iter()
                        .map(|(_, t_idx)| *t_idx)
                        .collect::<Vec<_>>()
                        .as_slice(),
                ))
                .into()
            }
            None if recursive => match self.node() {
                Node::Symbol(_) | Node::Const(_) => self,
                Node::Variadic(op) => op
                    .with_operands(
                        op.operands_ref()
                            .iter()
                            .map(|x| {
                                x.rewrite(transformation.clone(), recursive)
                            })
                            .collect(),
                    )
                    .into(),
                Node::Single(op) => op
                    .with_arg(
                        op.arg().rewrite(transformation.clone(), recursive),
                    )
                    .into(),
                Node::Double(op) => op
                    .with_args(array::from_fn(|i| {
                        op.args()[i].rewrite(transformation.clone(), recursive)
                    }))
                    .into(),
                Node::Matrix(_m) => todo!(),
            },
            _ => self,
        }
    }

    /// Matches a pattern expression against a subexpression of self, by binding the pattern's symbols to parts of this expression.
    /// If it does not match, `bindings` is unchanged.
    /// For commutative ops, returns a Match::Terms containing the operands that were matched.
    /// On non commutative, that value is Match::Whole, since the entire expression must match.
    /// On an insuccesful match, returns None.
    /// Examples:
    /// `(x^2 + y + x + 10).match_by(a^2 + a + 10)` would return Match::Terms([0, 1, 2]), and would bind `a` to `x`, leaving y unbound.
    pub fn match_by(
        self,
        pattern: Expr,
        bindings: &mut Vec<Binding>,
    ) -> Option<Match> {
        // TODO: impl greedy matching, matching x * 1 against y * a * b * g * 1 should match x -> y * a * b * g and 1 -> 1
        match (pattern.node(), self.node()) {
            (Node::Symbol(symb), _) => {
                for binding in &*bindings {
                    if binding.from == symb {
                        return binding
                            .to
                            .structural_eq(self)
                            .then_some(Match::Whole);
                    }
                }

                // if self.range().is_subset(&symb.domain()) {
                bindings.push(Binding { from: symb, to: self });
                Some(Match::Whole)
                // } else {
                // None
                // }
            }
            (Node::Const(pat_qty), Node::Const(target_qty)) => {
                (pat_qty.value() == target_qty.value()).then_some(Match::Whole)
            }
            // TODO: handle non commutability of the matrix
            (Node::Variadic(pat_op), Node::Variadic(target_op))
                if discriminant(&pat_op) == discriminant(&target_op) =>
            {
                // This is a variation of bipartite graph matching problem and theres various algorithms to use.
                // The thing is that assigning for example "x" to be "y + 10", means that every other occurance of
                // "y + 10" must be x as well.
                // Since n is very small I chose backtracking which is simple
                //
                //
                // Did i mention i hate tree algorithms btw, its because thats why

                fn backtrack(
                    pat: &[Expr],
                    target: &[Expr],
                    bindings: &mut Vec<Binding>,
                    matches: &mut Vec<(usize, usize)>,
                    pat_idx: usize,
                ) -> bool {
                    if pat_idx == pat.len() {
                        return true;
                    }

                    for target_idx in 0..target.len() {
                        if matches.iter().any(|&(_, i)| i == target_idx) {
                            continue;
                        }

                        let bindings_sp = bindings.len();
                        let matches_sp = matches.len();

                        if target[target_idx]
                            .match_by(pat[pat_idx], bindings)
                            .is_some()
                        {
                            matches.push((pat_idx, target_idx));

                            if backtrack(
                                pat,
                                target,
                                bindings,
                                matches,
                                pat_idx + 1,
                            ) {
                                return true;
                            }
                        }

                        bindings.truncate(bindings_sp);
                        matches.truncate(matches_sp);
                    }

                    false
                }

                let mut matches = Vec::new();

                if backtrack(
                    pat_op.operands_ref(),
                    target_op.operands_ref(),
                    bindings,
                    &mut matches,
                    0,
                ) {
                    Some(Match::Terms(matches))
                } else {
                    None
                }
            }

            (Node::Double(pat_ops), Node::Double(target_ops))
                if discriminant(&pat_ops) == discriminant(&target_ops) =>
            {
                target_ops.args()[0].match_by(pat_ops.args()[0], bindings).and(
                    target_ops.args()[1].match_by(pat_ops.args()[1], bindings),
                )
            }
            (Node::Single(pat_op), Node::Single(target_op))
                if discriminant(&pat_op) == discriminant(&target_op) =>
            {
                target_op.arg().match_by(pat_op.arg(), bindings)
            }
            (Node::Matrix(_pat), Node::Matrix(_target)) => todo!(),

            _ => None,
        }
    }
}

impl Node {
    pub fn register(self) -> Expr {
        if let Some((existing_id, _)) = NODES.find(|_, n| *n == self) {
            return Expr(existing_id);
        }

        let id = NODES.insert(self);
        Expr(id)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node() {
            Node::Const(qty) => qty.fmt(f),
            Node::Double(op) => op.fmt(f),
            Node::Single(op) => op.fmt(f),
            Node::Variadic(op) => op.fmt(f),
            Node::Symbol(symb) => symb.fmt(f),
            Node::Matrix(_m) => todo!(),
        }
    }
}

// impl PartialOrd for Expr {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for Expr {
//     fn cmp(&self, other: &Self) -> Ordering {
//         match (self.node(), other.node()) {
//             (Node::Symbol(lhs), Node::Symbol(rhs)) => {
//                 lhs.name().cmp(&rhs.name())
//             }
//             (Node::Const(lhs), Node::Const(rhs)) => {
//                 let lhs = lhs.value();
//                 let rhs = rhs.value();
//                 lhs.re
//                     .total_cmp(&rhs.re)
//                     .then_with(|| lhs.im.total_cmp(&rhs.im))
//             }
//             (Node::Intrinsic(lhs), Node::Intrinsic(rhs)) => match (lhs, rhs) {},
//         }
//     }
// }
