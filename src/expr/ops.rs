use std::{
    cmp::Ordering,
    fmt::{self, Display, Pointer, Write},
    iter::once,
    num::NonZero,
    ops::Index,
};

use derive_more::{IsVariant, TryUnwrap, Unwrap};
use itertools::Itertools;
use num::complex::ComplexFloat;

use crate::{
    core::util::to_superscript,
    dimension::{Quantity, Unit},
    expr::{Expr, Node, Shape, Shaped},
    simplify::separate_consts,
    symbol::constants::e,
};

#[derive(PartialEq, Clone, Debug, IsVariant, TryUnwrap, Unwrap)]
pub enum Variadic {
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
}

#[derive(PartialEq, Clone, Debug, Copy, IsVariant, TryUnwrap, Unwrap)]
pub enum Single {
    Sin(Expr),
    Cos(Expr),
    Tan(Expr),

    Asin(Expr),
    Acos(Expr),
    Atan(Expr),

    Sinh(Expr),
    Cosh(Expr),
    Tanh(Expr),

    Asinh(Expr),
    Acosh(Expr),
    Atanh(Expr),

    Transpose(Expr),
    Conj(Expr),
    Arg(Expr),
    Det(Expr),
    Norm(Expr),
}

#[derive(PartialEq, Clone, Debug, Copy, IsVariant)]
pub enum Double {
    Pow { base: Expr, exp: Expr },
    Log { base: Expr, arg: Expr },
    Atan2 { a: Expr, b: Expr },
}

/// Row-major matrix type
#[derive(PartialEq, Clone, Debug)]
pub struct Matrix {
    shape: Shape,
    elements: Vec<Expr>,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Matrix {
    /// Returns (rows, cols) for this matrix
    pub fn shape(&self) -> Shape {
        self.shape
    }

    pub fn rows(&self) -> NonZero<usize> {
        self.shape.rows
    }

    pub fn cols(&self) -> NonZero<usize> {
        self.shape.cols
    }

    pub fn elements(&self) -> &[Expr] {
        &self.elements
    }

    pub fn map(&self, f: impl FnMut(&Expr) -> Expr) -> Matrix {
        Matrix {
            shape: self.shape,
            elements: self.elements.iter().map(f).collect(),
        }
    }
}

impl Index<usize> for Matrix {
    type Output = [Expr];

    fn index(&self, row: usize) -> &Self::Output {
        let start = row * self.shape.cols.get();
        let end = start + self.shape.cols.get();
        &self.elements[start..end]
    }
}

impl Variadic {
    pub fn with_operands(&self, ops: Vec<Expr>) -> Self {
        match self {
            Variadic::Add(_) => Variadic::Add(ops),
            Variadic::Mul(_) => Variadic::Mul(ops),
        }
    }

    pub fn operands_ref(&self) -> &Vec<Expr> {
        match self {
            Variadic::Add(ops) => ops,
            Variadic::Mul(ops) => ops,
        }
    }

    pub fn operands(self) -> Vec<Expr> {
        match self {
            Variadic::Add(ops) => ops,
            Variadic::Mul(ops) => ops,
        }
    }
}

impl Shaped for Variadic {
    fn shape(&self) -> Shape {
        match self {
            Variadic::Add(exprs) => exprs.first().unwrap().shape(),
            Variadic::Mul(exprs) => {
                exprs.iter().fold(Shape::SCALAR, |acc, term| {
                    let b = term.shape();

                    // Special case: dot product, vec * vec, but same direction only
                    if acc == b && acc.is_vec() {
                        acc
                    } else {
                        Shape { rows: acc.rows, cols: b.cols }
                    }
                })
            }
        }
    }
}

impl Single {
    pub fn with_arg(&self, arg: Expr) -> Self {
        match self {
            Single::Sin(_) => Single::Sin(arg),
            Single::Cos(_) => Single::Cos(arg),
            Single::Tan(_) => Single::Tan(arg),
            Single::Asin(_) => Single::Asin(arg),
            Single::Acos(_) => Single::Acos(arg),
            Single::Atan(_) => Single::Atan(arg),
            Single::Sinh(_) => Single::Sinh(arg),
            Single::Cosh(_) => Single::Cosh(arg),
            Single::Tanh(_) => Single::Tanh(arg),
            Single::Asinh(_) => Single::Asinh(arg),
            Single::Acosh(_) => Single::Acosh(arg),
            Single::Atanh(_) => Single::Atanh(arg),
            Single::Transpose(_) => Single::Transpose(arg),
            Single::Conj(_) => Single::Conj(arg),
            Single::Arg(_) => Single::Arg(arg),
            Single::Det(_) => Single::Det(arg),
            Single::Norm(_) => Single::Norm(arg),
        }
    }

    pub fn arg(self) -> Expr {
        match self {
            Single::Sin(arg) => arg,
            Single::Cos(arg) => arg,
            Single::Tan(arg) => arg,
            Single::Asin(arg) => arg,
            Single::Acos(arg) => arg,
            Single::Atan(arg) => arg,
            Single::Sinh(arg) => arg,
            Single::Cosh(arg) => arg,
            Single::Tanh(arg) => arg,
            Single::Asinh(arg) => arg,
            Single::Acosh(arg) => arg,
            Single::Atanh(arg) => arg,
            Single::Transpose(arg) => arg,
            Single::Conj(arg) => arg,
            Single::Arg(arg) => arg,
            Single::Det(arg) => arg,
            Single::Norm(arg) => arg,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Single::Sin(_) => "sin",
            Single::Cos(_) => "cos",
            Single::Tan(_) => "tan",
            Single::Asin(_) => "asin",
            Single::Acos(_) => "acos",
            Single::Atan(_) => "atan",
            Single::Sinh(_) => "sinh",
            Single::Cosh(_) => "cosh",
            Single::Tanh(_) => "tanh",
            Single::Asinh(_) => "asinh",
            Single::Acosh(_) => "acosh",
            Single::Atanh(_) => "atanh",
            Single::Transpose(_) => "transpose",
            Single::Conj(_) => "conj",
            Single::Arg(_) => "arg",
            Single::Det(_) => "det",
            Single::Norm(_) => "norm",
        }
    }
}

impl Shaped for Single {
    fn shape(&self) -> Shape {
        match self {
            Self::Transpose(expr) => expr.shape().transpose(),
            Self::Det(_) | Self::Norm(_) => Shape::SCALAR,
            _ => self.arg().shape(),
        }
    }
}

impl Double {
    pub fn with_args(&self, args: [Expr; 2]) -> Self {
        match self {
            Double::Atan2 { .. } => Double::Atan2 { a: args[0], b: args[1] },
            Double::Log { .. } => Double::Log { base: args[0], arg: args[1] },
            Double::Pow { .. } => Double::Pow { base: args[0], exp: args[1] },
        }
    }

    pub fn args(self) -> [Expr; 2] {
        match self {
            Double::Atan2 { a, b } => [a, b],
            Double::Log { base, arg } => [base, arg],
            Double::Pow { base, exp } => [base, exp],
        }
    }
}

impl Shaped for Double {
    fn shape(&self) -> Shape {
        match self {
            Double::Pow { base, exp } => exp.shape(),
            Double::Log { base, arg } => arg.shape(),
            Double::Atan2 { a, b } => Shape::SCALAR,
        }
    }
}

impl Display for Single {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Single::Transpose(expr) => {
                let parenthesize = matches!(
                    expr.node(),
                    Node::Double(Double::Pow { .. }) | Node::Variadic(_)
                );

                write_enclosed(expr, f, parenthesize)
            }
            _ => {
                f.write_str(self.name())?;
                write_enclosed(self.arg(), f, true)
            }
        }
    }
}

impl Display for Double {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Double::Pow { base, exp } => {
                let parenthesize_base = !matches!(
                    base.node(),
                    Node::Symbol(_)
                        | Node::Const(_)
                        | Node::Single(_)
                        | Node::Double(
                            Double::Log { .. } | Double::Atan2 { .. }
                        )
                );
                let parenthesize_exp =
                    !matches!(exp.node(), Node::Symbol(_) | Node::Const(_));

                write_enclosed(base, f, parenthesize_base)?;

                if let Node::Const(x) = exp.node()
                    && x.unit() == Unit::Unitless
                    && let value = x.value()
                    && value.is_integer()
                {
                    f.write_str(&to_superscript(value.re as i32))?;
                } else {
                    f.write_str("^")?;

                    write_enclosed(exp, f, parenthesize_exp)?;
                }

                Ok(())
            }
            Double::Log { base, arg } => {
                if *base == e.into() {
                    f.write_str("ln")?;
                    write_enclosed(arg, f, true)
                } else {
                    f.write_str("log(")?;
                    base.fmt(f)?;
                    f.write_str(", ")?;
                    arg.fmt(f)?;
                    f.write_str(")")
                }
            }
            Double::Atan2 { a, b } => {
                f.write_str("atan2(")?;
                a.fmt(f)?;
                f.write_str(", ")?;
                b.fmt(f)?;
                f.write_str(")")
            }
        }
    }
}

impl Display for Variadic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variadic::Add(terms) => {
                for (i, term) in terms.iter().enumerate() {
                    if let Node::Variadic(Variadic::Mul(terms)) = term.node()
                        && let (mut consts, exprs) = separate_consts(terms)
                        && let Ok(coef) = consts.by_ref().exactly_one()
                        && coef.value().is_real()
                        && coef.value().re < 0.0
                    {
                        if i > 0 {
                            f.write_str(" - ")?;
                        } else {
                            f.write_str("-")?;
                        }

                        Variadic::Mul(
                            once(Expr::from(coef.value().abs() * coef.unit()))
                                .chain(exprs)
                                .collect(),
                        )
                        .fmt(f)?;
                    } else {
                        if i > 0 {
                            f.write_str(" + ")?;
                        }

                        term.fmt(f)?;
                    }
                }
            }
            Variadic::Mul(terms) => {
                fn joinable(expr: &Expr) -> bool {
                    matches!(
                        expr.node(),
                        Node::Symbol(_)
                            | Node::Const(_)
                            | Node::Double(Double::Pow { .. })
                    )
                }

                fn sort_terms(terms: &Vec<Expr>) -> Vec<Expr> {
                    let (mut mat_part, mut scalar_part): (
                        Vec<Expr>,
                        Vec<Expr>,
                    ) = terms.iter().partition(|x| x.shape().is_rect());

                    scalar_part.sort_by(|a, b| {
                        if matches!(a.node(), Node::Const(_)) {
                            return Ordering::Less;
                        }

                        match (joinable(a), joinable(b)) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => Ordering::Equal,
                        }
                    });

                    scalar_part.append(&mut mat_part);

                    scalar_part
                }

                let (denom, num): (Vec<Expr>, Vec<Expr>) =
                    terms.iter().partition(|expr| {
                        matches!(
                            expr.node(),
                            Node::Double(Double::Pow { exp, .. })
                            if matches!(
                                exp.node(),
                                Node::Const(qty)
                                if qty.value().im == 0.0
                                    && qty.value().re < 0.0
                            )
                        )
                    });

                let (denom, num) = (sort_terms(&denom), sort_terms(&num));

                for (i, term) in num.iter().enumerate() {
                    let parenthesize =
                        matches!(term.node(), Node::Variadic(Variadic::Add(_)));

                    if i > 0
                        && num
                            .get(i - 1)
                            .map(|x| !joinable(x))
                            .unwrap_or_default()
                        && joinable(term)
                    {
                        f.write_char('·')?;
                    }

                    write_enclosed(term, f, parenthesize)?;

                    if num.get(i + 1).map(|x| !joinable(x)).unwrap_or_default()
                    {
                        f.write_char('·')?;
                    }

                    // f.write_char('·')?;
                }

                if !num.is_empty() && !denom.is_empty() {
                    f.write_str(" / ")?;
                }

                let parenthesize_denom = denom.len() > 1
                    || denom
                        .first()
                        .map(|x| x.node().is_variadic())
                        .unwrap_or_default();

                if parenthesize_denom {
                    f.write_char('(')?;
                }

                for (i, mut term) in denom.clone().into_iter().enumerate() {
                    let parenthesize =
                        matches!(term.node(), Node::Variadic(Variadic::Add(_)));

                    if !joinable(&term) && i != 0 {
                        f.write_char('·')?;
                    }

                    if !num.is_empty() {
                        term = match term.node() {
                            Node::Double(Double::Pow { base, exp }) => {
                                let exp = match exp.node() {
                                    Node::Const(qty) => qty,
                                    _ => unreachable!(
                                        "Expression must be a Pow with negative const exp in order to be on the denominator"
                                    ),
                                };

                                if exp == -1 {
                                    base
                                } else {
                                    Double::Pow {
                                        base,
                                        exp: (exp.value().abs() * exp.unit())
                                            .into(),
                                    }
                                    .into()
                                }
                            }
                            _ => unreachable!(
                                "Expression must be a Pow with negative const exp in order to be on the denominator"
                            ),
                        }
                    }

                    write_enclosed(term, f, parenthesize)?;

                    if denom
                        .get(i + 1)
                        .map(|x| !joinable(x))
                        .unwrap_or_default()
                    {
                        f.write_char('·')?;
                    }
                }

                if parenthesize_denom {
                    f.write_char(')')?;
                }
            }
        }
        Ok(())
    }
}

/* -------------------------------- FUNCTIONS ------------------------------- */

fn write_enclosed(
    obj: impl Display,
    f: &mut std::fmt::Formatter<'_>,
    parenthesize: bool,
) -> fmt::Result {
    if parenthesize {
        f.write_str("(")?;
    }
    obj.fmt(f)?;
    if parenthesize {
        f.write_str(")")?;
    }

    Ok(())
}

macro_rules! impl_single_fn {
    ($fn:ident, $variant:ident, $name:literal) => {
        pub fn $fn(x: impl Into<Expr>) -> Expr {
            let expr = x.into();
            let shape = expr.shape();

            assert!(
                shape.is_square() || shape.is_scalar(),
                "Matrix-valued {} is only defined for square matrices",
                $name
            );

            Single::$variant(expr).into()
        }
    };
}

impl_single_fn!(sin, Sin, "sine");
impl_single_fn!(cos, Cos, "cosine");
impl_single_fn!(tan, Tan, "tangent");

impl_single_fn!(asin, Asin, "inverse sine");
impl_single_fn!(acos, Acos, "inverse cosine");
impl_single_fn!(atan, Atan, "inverse tangent");

impl_single_fn!(sinh, Sinh, "hyperbolic sine");
impl_single_fn!(cosh, Cosh, "hyperbolic cosine");
impl_single_fn!(tanh, Tanh, "hyperbolic tangent");

impl_single_fn!(asinh, Asinh, "inverse hyperbolic sine");
impl_single_fn!(acosh, Acosh, "inverse hyperbolic cosine");
impl_single_fn!(atanh, Atanh, "inverse hyperbolic tangent");

/* -------------------------------------------------------------------------- */

pub fn log(base: impl Into<Expr>, x: impl Into<Expr>) -> Expr {
    let base = base.into();
    let x = x.into();

    assert!(
        base.shape().is_scalar(),
        "Logarithm is only defined for scalar bases"
    );

    assert!(
        x.shape().is_square() || x.shape().is_scalar(),
        "Matrix-valued logarithm is only defined for square matrices"
    );

    Double::Log { base: base.into(), arg: x.into() }.into()
}

pub fn ln(x: impl Into<Expr>) -> Expr {
    log(e, x)
}

pub fn exp(x: impl Into<Expr>) -> Expr {
    e ^ x.into()
}

/* -------------------------------------------------------------------------- */

pub fn sqrt(x: impl Into<Expr>) -> Expr {
    x.into() ^ (1 / 2)
}

pub fn cbrt(x: impl Into<Expr>) -> Expr {
    x.into() ^ (1 / 3)
}

pub fn qtrt(x: impl Into<Expr>) -> Expr {
    x.into() ^ (1 / 4)
}
