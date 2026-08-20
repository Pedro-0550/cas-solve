use std::{
    array, collections::HashMap, hash::Hash, ops::Mul, sync::LazyLock,
    time::Duration,
};

use ahash::AHashMap;
use itertools::Itertools;
use num::{One, Zero, complex::ComplexFloat};

use crate::{
    core::scalar::{Scalar, gcd_f64},
    dimension::Quantity,
    expr::{
        Expr, Node,
        ops::{Double, Single, Variadic, cos, sin, tan},
    },
    simplify::normal::Normalize,
    symbol::Symbol, // set::Set,
};

/* --------------------------------- MODULES -------------------------------- */

#[cfg(test)]
mod test;

pub mod normal;

/* --------------------------------- TRAITS --------------------------------- */

pub trait Simplify {
    fn simplify(&self, ctx: &mut SimplifyContext) -> Expr;
    // fn range(&self) -> Set;
}

/* -------------------------------- CONSTANTS ------------------------------- */

// const CACHE_CAPACITY: u64 = 4096;
// const CACHE: LazyLock<Cache<Expr, Expr>> = LazyLock::new(|| {
//     Cache::builder()
//         .initial_capacity(CACHE_CAPACITY as usize / 8)
//         .max_capacity(CACHE_CAPACITY)
//         .time_to_live(Duration::from_weeks(1))
//         .time_to_idle(Duration::from_weeks(1))
//         .build()
// });

/* --------------------------------- STRUCTS -------------------------------- */

enum SimplificationStep {
    GroupTerms {},
    FactorTerms {},
}

pub struct SimplifyContext {
    steps: Option<Vec<SimplificationStep>>,
    cache: HashMap<Expr, Expr>,
}

// struct Path {
//     expr: Expr,
//     cost: usize,
//     seen: HashSet<Expr>,
// }

/* ---------------------------------- IMPLS --------------------------------- */

impl SimplifyContext {
    pub fn new() -> Self {
        Self { steps: None, cache: HashMap::new() }
    }
}

impl Expr {
    pub(crate) fn simplify_inner(&self, ctx: &mut SimplifyContext) -> Self {
        if self.node().is_symbol() || self.node().is_const() {
            return self.clone();
        }

        if let Some(hit) = ctx.cache.get(self) {
            return hit.clone();
        }

        let simplified = match self.node() {
            Node::Variadic(variadic) => variadic.simplify(ctx),
            Node::Single(single) => single.simplify(ctx),
            Node::Double(double) => double.simplify(ctx),
            Node::Matrix(matrix) => todo!(),
            _ => unreachable!(),
        }
        .normalize(true);

        ctx.cache.insert(self.clone(), simplified.clone());
        simplified
    }
}

impl Simplify for Expr {
    fn simplify(&self, ctx: &mut SimplifyContext) -> Expr {
        if self.node().is_symbol() || self.node().is_const() {
            return self.clone();
        }

        let initial = self.normalize(true);
        let mut step = initial.clone();

        // if let Some(hit) = CACHE.get(&step) {
        //     return hit;
        // }

        loop {
            let simplified = step.simplify_inner(ctx);

            if simplified == step {
                break;
            }

            step = simplified;
        }

        // CACHE.insert(initial, step.clone());

        step
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

macro_rules! trig_simplify {
    ($inv:ident, $fn:ident, $expr:ident, $self:ident, $ctx:ident) => {
        match $expr.node() {
            Node::Const(qty) => Scalar::from(qty.value().$fn()).into(),
            Node::Single(op) if let Single::$inv(ref x) = *op => {
                x.simplify_inner($ctx)
            }
            _ => $self.with_arg($self.arg().simplify_inner($ctx)).into(),
        }
    };
}

impl Simplify for Single {
    fn simplify(&self, ctx: &mut SimplifyContext) -> Expr {
        match self {
            Single::Sin(x) => trig_simplify!(Asin, sin, x, self, ctx),
            Single::Cos(x) => trig_simplify!(Acos, cos, x, self, ctx),
            Single::Tan(x) => trig_simplify!(Atan, tan, x, self, ctx),
            Single::Sinh(x) => trig_simplify!(Asinh, sinh, x, self, ctx),
            Single::Cosh(x) => trig_simplify!(Acosh, cosh, x, self, ctx),
            Single::Tanh(x) => trig_simplify!(Atanh, tanh, x, self, ctx),

            Single::Transpose(expr) => todo!(),
            Single::Conj(expr) => todo!(),
            Single::Arg(expr) => todo!(),
            Single::Det(expr) => todo!(),
            Single::Norm(expr) => todo!(),
            _ => self.with_arg(self.arg().simplify_inner(ctx)).into(),
        }
    }
}

impl Simplify for Double {
    fn simplify(&self, ctx: &mut SimplifyContext) -> Expr {
        let simplified = self
            .with_args(array::from_fn(|i| self.args()[i].simplify_inner(ctx)))
            .into();

        match &simplified {
            Double::Pow { base, exp } => {
                if let Node::Double(Double::Pow {
                    base: inner_base,
                    exp: inner_exp,
                }) = &base.node()
                    && let Node::Const(exp) = exp.node()
                    && let Node::Const(inner_exp) = inner_exp.node()
                    && exp.value().is_integer()
                    && inner_exp.value().is_integer()
                {
                    (inner_base ^ (*exp * *inner_exp)).simplify_inner(ctx)
                } else if let Node::Const(qty) = exp.node()
                    && qty.value().is_zero()
                {
                    (1.0).into()
                } else if let Node::Const(qty) = exp.node()
                    && qty.value().is_one()
                {
                    base.clone()
                } else if let Node::Const(qty) = base.node()
                    && qty.value().is_one()
                {
                    (1.0).into()
                } else {
                    simplified.into()
                }
            }
            _ => simplified.into(),
        }
    }
}

impl Simplify for Variadic {
    fn simplify(&self, ctx: &mut SimplifyContext) -> Expr {
        let simplified =
            self.operands().iter().map(|expr| expr.simplify_inner(ctx));

        let mut groupings =
            AHashMap::<Expr, Scalar>::with_capacity(self.operands().len());

        for term in simplified {
            /* -------------------------------------------------------------------------- */
            // joins coefficients: 2x + x -> 3x
            if self.is_add()
                && let Node::Variadic(Variadic::Mul(terms)) = term.node()
            {
                let (coef, exprs) = extract_const(&terms);
                *groupings
                    .entry(Variadic::Mul(exprs.collect()).normalize(false))
                    .or_insert(0.0.into()) +=
                    coef.unwrap_or(1.0.into()).value();
            /* -------------------------------------------------------------------------- */
            // joins integer powers -> x^2 * x^3 -> x^5
            } else if self.is_mul()
                && let Node::Double(Double::Pow { base, exp }) = term.node()
                && let Node::Const(exp) = exp.node()
                && exp.value().is_integer()
            {
                if let Node::Variadic(Variadic::Mul(terms)) = base.node() {
                    for term in terms {
                        *groupings.entry(term.clone()).or_insert(0.0.into()) +=
                            exp.value();
                    }
                } else {
                    *groupings.entry(base.clone()).or_insert(0.0.into()) +=
                        exp.value();
                }
            /* -------------------------------------------------------------------------- */
            } else {
                *groupings.entry(term.clone()).or_insert(0.0.into()) += 1;
            }
        }

        // TODO: no need to allocate twice here, first in aggregated then in common if its add
        let mut aggregated: Vec<Expr> = match self {
            Variadic::Add(_) => groupings
                .into_iter()
                .map(|(base, coef)| {
                    if coef == 1.0.into() {
                        base
                    } else if coef == 0.0.into() {
                        0.0.into()
                    } else {
                        base * coef
                    }
                    .normalize(false)
                })
                .collect(),
            Variadic::Mul(_) => groupings
                .into_iter()
                .map(|(base, exp)| {
                    if exp == 1.0.into() {
                        base
                    } else if exp == 0.0.into() {
                        1.0.into()
                    } else {
                        base ^ exp
                    }
                    .normalize(false)
                })
                .collect(),
        };

        /* -------------------------------------------------------------------------- */
        // Partial factoring -> x * a + x * b -> x(a + b)
        // x * a + x * b + y * c + y * d -> x(a+b) + y(b+c)

        if self.is_add() && aggregated.len() >= 2 {
            // TODO: when domain is implemented allow fractional exponents to be factored as well
            //
            // I really wrote this. With my free will.
            //
            // The factor table stores the individual factors for each term in this addition and their exponents,
            // as well as the term's coefficient.
            //
            // For the expression (x * y * 3) + (4 * x^2 / y) + (y^3) + (4y^2) it would look something like:
            // [
            //  ({ x: 1, y:  1 }, 3)
            //  ({ x: 2, y: -1 }, 4)
            //  ({ y: 3        }, 1)
            //  ({ y: 2        }, 4)
            // ]
            // We want to factor this into: y * ((x * 3) + y * (4 + y)) + (4 * x^2 / y)
            // During factoring, powers over multiplication are expanded.
            // Then, we group unique factors by exponent sign, ignoring terms where its 0, producing 2 groups for each factor like so:
            // x:
            // + [
            //  ({ x: 1, y:  1 }, 3)
            //  ({ x: 2, y: -1 }, 4)
            // ]
            // - []
            //
            // y:
            // + [
            //  ({ x: 1, y:  1 }, 3)
            //  ({ y: 3        }, 1)
            //  ({ y: 2        }, 4)
            // ]
            // - [
            //  ({ x: 2, y: -1 }, 4)
            // ]
            //
            // The group with the most terms is factored first, and its terms are removed from other groups.
            // We start from the term with the highest exp, then move out.
            // First we calculate what exp each level will take by subtracting the sum of every previous exp, starting from the lowest exp:
            // y -> y^1
            // y^2 -> y^(2-1) -> y^1
            // y^3 -> y^(3 - (1 + 1)) -> y^1
            //
            // For n factors with the same exp, the GCD of the coefficient must be taken to pull it out
            //
            // After the first step, we have
            // x:
            // + []
            // - []
            //
            // y:
            // + []
            // - [
            //  ({ x: 2, y: -1 }, 4)
            // ]
            //
            // This would then be repeated until we only have 1 term left, which we already do.
            // The last term is left unfactored.

            let mut factor_table =
                Vec::<(AHashMap<Expr, f64>, f64)>::with_capacity(
                    aggregated.len(),
                );

            aggregated.sort_unstable();
            let (lone_const, terms) = extract_const(&aggregated);

            for term in terms {
                match term.node() {
                    Node::Variadic(Variadic::Mul(t)) => {
                        let mut map = AHashMap::with_capacity(t.len());
                        let (const_factor, factors) = extract_const(&t);

                        let const_factor = const_factor.unwrap_or(1.0.into());

                        for fac in factors {
                            match fac.node() {
                                Node::Double(Double::Pow { base, exp })
                                    if let Node::Const(exp) = exp.node()
                                        && exp.value().is_integer() =>
                                {
                                    *map.entry(base.clone()).or_insert(0.0) +=
                                        exp.value().re;
                                }
                                _ => {
                                    *map.entry(fac).or_insert(0.0) += 1.0;
                                }
                            }
                        }

                        if const_factor.value().is_real() {
                            factor_table.push((map, const_factor.value().re))
                        } else {
                            map.insert(const_factor.into(), 1.0);
                            factor_table.push((map, 1.0))
                        }
                    }
                    Node::Double(Double::Pow { base, exp })
                        if let Node::Const(exp) = exp.node()
                            && exp.value().is_integer() =>
                    {
                        factor_table.push((
                            AHashMap::from([(base.clone(), exp.value().re)]),
                            1.0,
                        ))
                    }
                    Node::Const(_) => unreachable!(),
                    _ => {
                        factor_table.push((AHashMap::from([(term, 1.0)]), 1.0))
                    }
                }
            }

            if !factor_table.is_empty() {
                let (first_terms, first_const) = &factor_table[0];

                let mut factored_terms = first_terms.clone();
                let mut factored_const = *first_const;

                for (terms, c) in &factor_table[1..] {
                    factored_terms.retain(|base, exp| match terms.get(base) {
                        Some(other_exp) => {
                            *exp = exp.min(*other_exp);
                            true
                        }
                        None => false,
                    });

                    factored_const = gcd_f64(factored_const, *c);
                }

                if !factored_terms.is_empty() || factored_const != 1.0 {
                    let common_factor = (factored_terms
                        .iter()
                        .fold(Expr::from(1.0), |acc, (base, exp)| {
                            acc * (base ^ *exp)
                        })
                        * factored_const)
                        .simplify_inner(ctx);

                    let mapped_terms = Variadic::Add(
                        factor_table
                            .iter()
                            .map(|(terms, c)| {
                                terms.iter().fold(
                                    Expr::from(1.0),
                                    |acc, (base, exp)| {
                                        let remaining = *exp
                                            - factored_terms
                                                .get(base)
                                                .copied()
                                                .unwrap_or(0.0);

                                        if remaining == 0.0 {
                                            acc
                                        } else if remaining == 1.0 {
                                            acc * base
                                        } else {
                                            acc * (base ^ remaining)
                                        }
                                    },
                                ) * (*c / factored_const)
                            })
                            .collect(),
                    );

                    aggregated = vec![
                        (common_factor * Expr::from(mapped_terms)),
                        lone_const.unwrap_or(0.0.into()).into(),
                    ]
                }
            }
        }

        /* -------------------------------------------------------------------------- */

        if aggregated.len() <= 1 {
            aggregated.pop().unwrap_or(0.into())
        } else {
            self.with_operands(aggregated).into()
        }
    }
}

pub fn separate_consts(
    terms: impl Iterator<Item = Expr> + Clone,
) -> (impl Iterator<Item = Quantity>, impl Iterator<Item = Expr>) {
    (
        terms.clone().into_iter().filter_map(|expr| match expr.node() {
            Node::Const(qty) => Some(*qty),
            _ => None,
        }),
        terms.into_iter().filter(|expr| !expr.node().is_const()),
    )
}

pub fn extract_const(
    terms: &Vec<Expr>,
) -> (Option<Quantity>, impl Iterator<Item = Expr>) {
    let constant = terms
        .get(0)
        .and_then(|x| x.clone().into_node().try_unwrap_const().ok());

    let exprs = terms.iter().cloned().filter(|expr| !expr.node().is_const());

    (constant, exprs)
}
