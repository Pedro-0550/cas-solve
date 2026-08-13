use std::{
    fmt::{Display, Pointer},
    mem::discriminant,
    ops::{Add, Mul},
};

use num::complex::Complex64;

use crate::{
    Scalar,
    arena::{Arena, Handle},
    ast::intrinsic::Intrinsic,
    dimension::{Dimension, DimensionalAnalysisError, Quantity, Unit},
    simplify::{Simplify, Transformation},
    symbol::Symbol,
};

/* -------------------------------- CONSTANTS ------------------------------- */

static NODES: Arena<Node> = Arena::new();

/* --------------------------------- MODULES -------------------------------- */

pub mod intrinsic;
pub mod ops;

#[cfg(test)]
mod test;
/* ---------------------------------- ENUMS --------------------------------- */

#[derive(PartialEq, Clone, Debug)]
pub enum Node {
    Symbol(Symbol),
    Const(Quantity),

    Intrinsic(Intrinsic),
    ElementwiseIntrinsic(Intrinsic),

    Matrix { rows: usize, cols: usize, elements: Vec<Expr> },
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
    Terms(Vec<usize>),
}

// #[derive(Clone, PartialEq, Eq)]
// pub struct Match {
//     bindings: Vec<Binding>,
//     matched: Vec<usize>,
// }

/* ---------------------------------- IMPLS --------------------------------- */

impl Expr {
    pub fn node(&self) -> Node {
        NODES.get_cloned(self.0).unwrap()
    }

    /// Returns true if two expressions are structurally equivalent, such as `(a + b) * x` and `x * (b + a)`.
    /// This method does not perform simplification before comparison.
    /// Partial eq checks for strict equivalence
    pub fn structural_eq(&self, rhs: Expr) -> bool {
        match (self.node(), rhs.node()) {
            (Node::Intrinsic(lhs), Node::Intrinsic(rhs)) => match (lhs, rhs) {
                (Intrinsic::Add(lhs_terms), Intrinsic::Add(rhs_terms))
                | (Intrinsic::Mul(lhs_terms), Intrinsic::Mul(rhs_terms)) => {
                    if lhs_terms.len() != rhs_terms.len() {
                        return false;
                    }
                    let mut remaining_rhs = rhs_terms;

                    lhs_terms.iter().all(|lhs_term| {
                        remaining_rhs
                            .iter()
                            .position(|rhs_term| {
                                lhs_term.structural_eq(*rhs_term)
                            })
                            .inspect(|i| _ = remaining_rhs.swap_remove(*i))
                            .is_some()
                    })
                }

                (Intrinsic::Acos(lhs), Intrinsic::Acos(rhs))
                | (Intrinsic::Asin(lhs), Intrinsic::Asin(rhs))
                | (Intrinsic::Inv(lhs), Intrinsic::Inv(rhs))
                | (Intrinsic::Neg(lhs), Intrinsic::Neg(rhs))
                | (Intrinsic::Sin(lhs), Intrinsic::Sin(rhs))
                | (Intrinsic::Cos(lhs), Intrinsic::Cos(rhs))
                | (Intrinsic::Norm(lhs), Intrinsic::Norm(rhs))
                | (Intrinsic::Transpose(lhs), Intrinsic::Transpose(rhs)) => {
                    lhs.structural_eq(rhs)
                }

                (
                    Intrinsic::Log { base: lhs_base, arg: lhs_arg },
                    Intrinsic::Log { base: rhs_base, arg: rhs_arg },
                ) => {
                    lhs_base.structural_eq(rhs_base)
                        && lhs_arg.structural_eq(rhs_arg)
                }

                (
                    Intrinsic::Pow { base: lhs_base, exp: lhs_exp },
                    Intrinsic::Pow { base: rhs_base, exp: rhs_exp },
                ) => {
                    lhs_base.structural_eq(rhs_base)
                        && lhs_exp.structural_eq(rhs_exp)
                }
                _ => false,
            },
            (Node::Symbol(lhs), Node::Symbol(rhs)) => lhs == rhs,
            (Node::Const(lhs), Node::Const(rhs)) => lhs == rhs,
            (Node::ElementwiseIntrinsic(_), Node::ElementwiseIntrinsic(_)) => {
                todo!()
            }
            _ => false,
        }
    }

    pub fn substitute(self, bindings: &[Binding]) -> Self {
        match self.node() {
            Node::Intrinsic(intr) => match intr {
                Intrinsic::Add(exprs) => Intrinsic::Add(
                    exprs.iter().map(|x| x.substitute(bindings)).collect(),
                )
                .into(),

                Intrinsic::Mul(exprs) => Intrinsic::Mul(
                    exprs.iter().map(|x| x.substitute(bindings)).collect(),
                )
                .into(),

                Intrinsic::Acos(expr)
                | Intrinsic::Asin(expr)
                | Intrinsic::Inv(expr)
                | Intrinsic::Neg(expr)
                | Intrinsic::Sin(expr)
                | Intrinsic::Cos(expr)
                | Intrinsic::Norm(expr)
                | Intrinsic::Transpose(expr) => expr.substitute(bindings),

                Intrinsic::Log { base, arg } => Intrinsic::Log {
                    base: base.substitute(bindings),
                    arg: arg.substitute(bindings),
                }
                .into(),
                Intrinsic::Pow { base, exp } => Intrinsic::Pow {
                    base: base.substitute(bindings),
                    exp: exp.substitute(bindings),
                }
                .into(),
            },
            Node::Symbol(sym) => {
                if let Some(binding) = bindings.iter().find(|b| b.from == sym) {
                    binding.to
                } else {
                    self
                }
            }
            _ => self,
            // TODO: elementwise
        }
    }

    /// Rewrites this expression by trying to apply a transformation. If the transformation pattern does not match, does nothing.
    pub fn rewrite(self, transformation: Transformation) -> Self {
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
            Some(Match::Terms(indices)) => {
                match self.node().as_intrinsic().unwrap() {
                    Intrinsic::Mul(terms) => Intrinsic::Mul(replace_terms(
                        transformation,
                        &bindings,
                        &terms,
                        &indices,
                    ))
                    .into(),
                    Intrinsic::Add(terms) => Intrinsic::Add(replace_terms(
                        transformation,
                        &bindings,
                        &terms,
                        &indices,
                    ))
                    .into(),
                    _ => unreachable!(),
                }
            }
            None => match self.node() {
                Node::Symbol(_) | Node::Const(_) => self,
                Node::Intrinsic(intrinsic) => match intrinsic {
                    Intrinsic::Add(exprs) => Intrinsic::Add(
                        exprs
                            .iter()
                            .map(|x| x.rewrite(transformation))
                            .collect(),
                    )
                    .into(),
                    Intrinsic::Mul(exprs) => Intrinsic::Mul(
                        exprs
                            .iter()
                            .map(|x| x.rewrite(transformation))
                            .collect(),
                    )
                    .into(),
                    Intrinsic::Neg(expr) => {
                        Intrinsic::Neg(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Sin(expr) => {
                        Intrinsic::Sin(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Cos(expr) => {
                        Intrinsic::Cos(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Asin(expr) => {
                        Intrinsic::Asin(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Acos(expr) => {
                        Intrinsic::Acos(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Pow { base, exp } => Intrinsic::Pow {
                        base: base.rewrite(transformation),
                        exp: exp.rewrite(transformation),
                    }
                    .into(),
                    Intrinsic::Log { base, arg } => Intrinsic::Log {
                        base: base.rewrite(transformation),
                        arg: arg.rewrite(transformation),
                    }
                    .into(),
                    Intrinsic::Norm(expr) => {
                        Intrinsic::Norm(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Inv(expr) => {
                        Intrinsic::Inv(expr.rewrite(transformation)).into()
                    }
                    Intrinsic::Transpose(expr) => {
                        Intrinsic::Transpose(expr.rewrite(transformation))
                            .into()
                    }
                },
                Node::ElementwiseIntrinsic(intrinsic) => todo!(),
                Node::Matrix { rows, cols, elements } => todo!(),
            },
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
        match pattern.node() {
            Node::Symbol(symb) => {
                for binding in &*bindings {
                    if binding.from == symb {
                        return binding
                            .to
                            .structural_eq(self)
                            .then_some(Match::Whole);
                    }
                }

                bindings.push(Binding { from: symb, to: self });
                Some(Match::Whole)
            }
            Node::Const(qty)
                if let Node::Const(target_qty) = self.node()
                    && qty.value() == target_qty.value() =>
            {
                Some(Match::Whole)
            }
            Node::Intrinsic(pat_intr)
                if let Node::Intrinsic(target_intr) = self.node() =>
            {
                match (pat_intr, target_intr) {
                    // This is a variation of bipartite graph matching problem and theres various algorithms to use.
                    // The thing is that assigning for example "x" to be "y + 10", means that every other occurance of
                    // "y + 10" must be x as well.
                    // Since n is very small I chose backtracking which is simple
                    //
                    //
                    // Did i mention i hate tree algorithms btw, its because thats why
                    (
                        Intrinsic::Add(pat_exprs),
                        Intrinsic::Add(target_exprs),
                    )
                    | (
                        Intrinsic::Mul(pat_exprs),
                        Intrinsic::Mul(target_exprs),
                    ) => {
                        let initial_len = bindings.len();
                        let mut tries = vec![
                            vec![false; target_exprs.len()];
                            pat_exprs.len()
                        ];
                        let mut matched = vec![false; target_exprs.len()];
                        let mut pat_i = 0;
                        // (binding length before ops, matched_i) @ step
                        let mut step = Vec::new();

                        loop {
                            let pat_expr = pat_exprs[pat_i];

                            let Some(target_i) = matched
                                .iter()
                                .enumerate()
                                .position(|(i, m)| !*m && !tries[pat_i][i])
                            else {
                                if pat_i == 0 {
                                    break;
                                }
                                let (last_start_len, last_match) =
                                    step.pop().unwrap();
                                bindings.truncate(last_start_len);

                                tries[pat_i].fill(false);

                                *matched.get_mut(last_match).unwrap() = false;
                                pat_i -= 1;
                                continue;
                            };

                            let target_expr = target_exprs[target_i];
                            *tries
                                .get_mut(pat_i)
                                .unwrap()
                                .get_mut(target_i)
                                .unwrap() = true;

                            let before_len = bindings.len();
                            if let Some(_) =
                                target_expr.match_by(pat_expr, bindings)
                            {
                                pat_i += 1;
                                *matched.get_mut(target_i).unwrap() = true;
                                step.push((before_len, target_i));
                            } else {
                                bindings.truncate(before_len);
                            }

                            if pat_i == pat_exprs.len() {
                                break;
                            }
                        }

                        let success = pat_i == pat_exprs.len();
                        if !success {
                            bindings.truncate(initial_len);
                            return None;
                        }

                        Some(Match::Terms(
                            matched
                                .iter()
                                .enumerate()
                                .filter_map(|(i, matched)| matched.then_some(i))
                                .collect(),
                        ))
                    }
                    (
                        Intrinsic::Pow { base: pat_base, exp: pat_exp },
                        Intrinsic::Pow { base: target_base, exp: target_exp },
                    ) => target_base
                        .match_by(pat_base, bindings)
                        .and(target_exp.match_by(pat_exp, bindings)),
                    (
                        Intrinsic::Acos(pat_expr),
                        Intrinsic::Acos(target_expr),
                    )
                    | (
                        Intrinsic::Asin(pat_expr),
                        Intrinsic::Asin(target_expr),
                    )
                    | (Intrinsic::Inv(pat_expr), Intrinsic::Inv(target_expr))
                    | (Intrinsic::Neg(pat_expr), Intrinsic::Neg(target_expr))
                    | (Intrinsic::Sin(pat_expr), Intrinsic::Sin(target_expr))
                    | (Intrinsic::Cos(pat_expr), Intrinsic::Cos(target_expr))
                    | (
                        Intrinsic::Norm(pat_expr),
                        Intrinsic::Norm(target_expr),
                    )
                    | (
                        Intrinsic::Transpose(pat_expr),
                        Intrinsic::Transpose(target_expr),
                    ) => target_expr.match_by(pat_expr, bindings),

                    _ => None,
                }
            }
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

    /// Returns `true` if the expr is [`Symbol`].
    ///
    /// [`Symbol`]: Expr::Symbol
    #[must_use]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Node::Symbol(..))
    }

    pub fn as_symbol(&self) -> Option<&Symbol> {
        if let Self::Symbol(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`Const`].
    ///
    /// [`Const`]: Expr::Const
    #[must_use]
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(..))
    }

    pub fn as_const(&self) -> Option<&Quantity> {
        if let Self::Const(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`Intrinsic`].
    ///
    /// [`Intrinsic`]: Expr::Intrinsic
    #[must_use]
    pub fn is_intrinsic(&self) -> bool {
        matches!(self, Self::Intrinsic(..))
    }

    pub fn as_intrinsic(&self) -> Option<&Intrinsic> {
        if let Self::Intrinsic(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`ElementwiseIntrinsic`].
    ///
    /// [`ElementwiseIntrinsic`]: Expr::ElementwiseIntrinsic
    #[must_use]
    pub fn is_elementwise_intrinsic(&self) -> bool {
        matches!(self, Self::ElementwiseIntrinsic(..))
    }

    pub fn as_elementwise_intrinsic(&self) -> Option<&Intrinsic> {
        if let Self::ElementwiseIntrinsic(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the expr is [`Matrix`].
    ///
    /// [`Matrix`]: Expr::Matrix
    #[must_use]
    pub fn is_matrix(&self) -> bool {
        matches!(self, Self::Matrix { .. })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.node() {
            Node::Const(qty) => qty.fmt(f),
            Node::Intrinsic(intr) => intr.fmt(f),
            Node::Symbol(symb) => symb.fmt(f),
            _ => todo!(),
        }
    }
}
