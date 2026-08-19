use std::{array, collections::HashMap, ops::Mul};

use itertools::Itertools;
use maplit::hashmap;
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
    fn simplify(&self, steps: &mut Option<Vec<Step>>) -> Expr;
    // fn range(&self) -> Set;
}

/* --------------------------------- STRUCTS -------------------------------- */

enum SimplificationStep {
    GroupTerms {},
    FactorTerms {},
}

pub struct Step {
    from: Expr,
    to: Expr,
}

// struct Path {
//     expr: Expr,
//     cost: usize,
//     seen: HashSet<Expr>,
// }

/* ---------------------------------- IMPLS --------------------------------- */

impl Simplify for Expr {
    fn simplify(&self, steps: &mut Option<Vec<Step>>) -> Expr {
        if self.node().is_symbol() || self.node().is_const() {
            return self.clone();
        }

        let mut step = self.normalize();

        loop {
            let simplified = match step.node() {
                Node::Variadic(variadic) => variadic.simplify(steps),
                Node::Single(single) => single.simplify(steps),
                Node::Double(double) => double.simplify(steps),
                Node::Matrix(matrix) => todo!(),
                _ => step.clone(),
            }
            .normalize();
            println!("{}", simplified);

            if simplified == step {
                break;
            }

            step = simplified;
        }

        step
    }

    // fn range(&self) -> Set {
    //     todo!()
    // }
}

macro_rules! trig_simplify {
    ($inv:ident, $fn:ident, $expr:ident, $self:ident, $steps:ident) => {
        match $expr.node() {
            Node::Const(qty) => Scalar::from(qty.value().$fn()).into(),
            Node::Single(op) if let Single::$inv(ref x) = *op => {
                x.simplify($steps)
            }
            _ => $self.with_arg($self.arg().simplify($steps)).into(),
        }
    };
}

impl Simplify for Single {
    fn simplify(&self, steps: &mut Option<Vec<Step>>) -> Expr {
        match self {
            Single::Sin(x) => trig_simplify!(Asin, sin, x, self, steps),
            Single::Cos(x) => trig_simplify!(Acos, cos, x, self, steps),
            Single::Tan(x) => trig_simplify!(Atan, tan, x, self, steps),
            Single::Sinh(x) => trig_simplify!(Asinh, sinh, x, self, steps),
            Single::Cosh(x) => trig_simplify!(Acosh, cosh, x, self, steps),
            Single::Tanh(x) => trig_simplify!(Atanh, tanh, x, self, steps),

            Single::Transpose(expr) => todo!(),
            Single::Conj(expr) => todo!(),
            Single::Arg(expr) => todo!(),
            Single::Det(expr) => todo!(),
            Single::Norm(expr) => todo!(),
            _ => self.with_arg(self.arg().simplify(steps)).into(),
        }
    }
}

impl Simplify for Double {
    fn simplify(&self, steps: &mut Option<Vec<Step>>) -> Expr {
        let simplified = self
            .with_args(array::from_fn(|i| self.args()[i].simplify(steps)))
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
                    (inner_base ^ (*exp * *inner_exp)).simplify(steps)
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
    fn simplify(&self, steps: &mut Option<Vec<Step>>) -> Expr {
        let simplified =
            self.operands().iter().map(|expr| expr.simplify(steps));

        let mut groupings =
            HashMap::<Expr, Scalar>::with_capacity(self.operands().len());

        for term in simplified.into_iter() {
            /* -------------------------------------------------------------------------- */
            // joins coefficients: 2x + x -> 3x
            if self.is_add()
                && let Node::Variadic(Variadic::Mul(terms)) = term.node()
                && let (consts, exprs) = separate_consts(terms.iter().cloned())
                && let Ok(coef) = consts.exactly_one()
            {
                *groupings
                    .entry(Variadic::Mul(exprs.collect()).into())
                    .or_insert(0.0.into()) += coef.value();
            /* -------------------------------------------------------------------------- */
            // joins integer powers -> x^2 * x^3 -> x^5
            } else if self.is_mul()
                && let Node::Double(Double::Pow { base, exp }) = &term.node()
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
                    .normalize()
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
                    .normalize()
                })
                .collect(),
        };

        /* -------------------------------------------------------------------------- */
        // Grouping common fators -> x * a + x * b -> x(a + b)

        if self.is_add() && aggregated.len() >= 2 {
            // Each term becomes a Vec containing the exponents each factor has in that expression
            // Constants are handled separately since its easy to just get the GCD,
            // and lone constants are not factores
            // For example, 6x^2 + 3xy + 3 would yield:
            // 6x^2 -> x^2 * y^0 | 6
            // xy -> x^1 * y^1  | 3
            // with 3 as a lone constant
            // The algorithm finds the minimum exp of each factor:
            // x -> min(2, 1) -> 1
            // y -> min(0, 1) -> 0
            // And the GCD of the constants:
            // GCD(3, 6) -> 3
            // And pulls all of that out, by subtracting the factored exponents, dividing each constant by the GCD,
            // and adding the lone constant back in:
            // x^1 * y^0 * 3 * ((x^1 * y^0 * 6/3) + (x^0 * y^1 * 3/3)) + 3
            // which simplifies to:
            // 3x(2x + y) + 3
            // TODO: when domain is implemented allow fractional exponents to be factored as well

            let (consts, terms) = separate_consts(aggregated.iter().cloned());
            let Ok(lone_const) =
                consts.at_most_one().map(|x| x.unwrap_or(0.0.into()))
            else {
                panic!()
            };

            let mut factor_table =
                Vec::<(HashMap<Expr, f64>, f64)>::with_capacity(
                    aggregated.len(),
                );

            for term in terms {
                match term.node() {
                    Node::Variadic(Variadic::Mul(t)) => {
                        let mut map = HashMap::with_capacity(t.len());
                        let (const_factors, factors) =
                            separate_consts(t.iter().cloned());

                        let Ok(const_factor) = const_factors
                            .at_most_one()
                            .map(|x| x.unwrap_or(1.0.into()))
                        else {
                            panic!()
                        };

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
                            hashmap!(base.clone() => exp.value().re),
                            1.0,
                        ))
                    }
                    Node::Const(_) => unreachable!(),
                    _ => factor_table.push((hashmap!(term => 1.0), 1.0)),
                }
            }

            println!("{:#?}", factor_table);

            // TODO: elegant solution to get rid of this clone
            let (factored_terms, factored_const) = factor_table
                .iter()
                .cloned()
                .reduce(|(mut a_terms, a_c), (b_terms, b_c)| {
                    a_terms.retain(|base, exp| {
                        if let Some(b_exp) = b_terms.get(base) {
                            *exp = exp.min(*b_exp);
                            true
                        } else {
                            false
                        }
                    });

                    (a_terms, gcd_f64(a_c, b_c))
                })
                .unwrap();

            if !factored_terms.is_empty() || factored_const != 1.0 {
                let common_factor = (factored_terms
                    .iter()
                    .fold(Expr::from(1.0), |acc, (base, exp)| {
                        acc * (base ^ *exp)
                    })
                    * factored_const)
                    .simplify(steps);

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
                    (common_factor * Expr::from(mapped_terms)).normalize(),
                    lone_const.into(),
                ]
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
