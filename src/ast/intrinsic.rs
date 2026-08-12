use std::fmt::{Display, Pointer};

use num::pow::Pow;

use crate::{
    ast::{Expr, Node},
    dimension::Unit,
    symbol::constants::{self, e},
    util::to_superscript,
};

#[derive(PartialEq, Clone, Debug)]
pub enum Intrinsic {
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    // Div { num: Expr, denom: Expr },
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

impl Intrinsic {
    /// Returns `true` if the intrinsic is [`Add`].
    ///
    /// [`Add`]: Intrinsic::Add
    #[must_use]
    pub fn is_add(&self) -> bool {
        matches!(self, Self::Add(..))
    }

    pub fn as_add(&self) -> Option<&Vec<Expr>> {
        if let Self::Add(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the intrinsic is [`Mul`].
    ///
    /// [`Mul`]: Intrinsic::Mul
    #[must_use]
    pub fn is_mul(&self) -> bool {
        matches!(self, Self::Mul(..))
    }

    pub fn as_mul(&self) -> Option<&Vec<Expr>> {
        if let Self::Mul(v) = self { Some(v) } else { None }
    }

    /// Returns `true` if the intrinsic is [`Neg`].
    ///
    /// [`Neg`]: Intrinsic::Neg
    #[must_use]
    pub fn is_neg(&self) -> bool {
        matches!(self, Self::Neg(..))
    }

    pub fn as_neg(&self) -> Option<&Expr> {
        if let Self::Neg(v) = self { Some(v) } else { None }
    }
}

impl Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::Add(terms) => {
                for (i, term) in terms.iter().enumerate() {
                    if let Node::Intrinsic(Intrinsic::Neg(expr)) = term.node() {
                        if i > 0 {
                            f.write_str(" - ")?;
                        } else {
                            f.write_str("-")?;
                        }

                        expr.fmt(f)?;
                    } else {
                        if i > 0 {
                            f.write_str(" + ")?;
                        }

                        term.fmt(f)?;
                    }
                }
            }
            Intrinsic::Mul(terms) => {
                for (i, term) in terms.iter().enumerate() {
                    let parenthesize = !matches!(
                        term.node(),
                        Node::Symbol(_)
                            | Node::Const(_)
                            | Node::Intrinsic(Intrinsic::Pow { .. })
                    );

                    if parenthesize {
                        f.write_str("(")?;
                    }
                    term.fmt(f)?;
                    if parenthesize {
                        f.write_str(")")?;
                    }
                }
            }
            Intrinsic::Pow { base, exp } => {
                let parenthesize_base =
                    !matches!(base.node(), Node::Symbol(_) | Node::Const(_));
                let parenthesize_exp =
                    !matches!(exp.node(), Node::Symbol(_) | Node::Const(_));

                if parenthesize_base {
                    f.write_str("(")?;
                }

                base.fmt(f)?;

                if parenthesize_base {
                    f.write_str(")")?;
                }

                if let Node::Const(x) = exp.node()
                    && x.unit() == Unit::Unitless
                    && let value = x.value()
                    && value.im == 0.0
                    && value.re - value.re.round() < 1e-15
                {
                    f.write_str(&to_superscript(value.re as i32))?;
                } else {
                    f.write_str("^")?;

                    if parenthesize_exp {
                        f.write_str("(")?;
                    }

                    exp.fmt(f)?;

                    if parenthesize_exp {
                        f.write_str(")")?;
                    }
                }
            }
            Intrinsic::Neg(expr) => {
                let parenthesize = !matches!(
                    expr.node(),
                    Node::Symbol(_)
                        | Node::Const(_)
                        | Node::Intrinsic(
                            Intrinsic::Pow { .. } | Intrinsic::Mul(_)
                        )
                );

                f.write_str("-")?;

                if parenthesize {
                    f.write_str("(")?;
                }
                expr.fmt(f)?;
                if parenthesize {
                    f.write_str(")")?;
                }
            }
            Intrinsic::Inv(expr) => {
                let parenthesize =
                    !matches!(expr.node(), Node::Symbol(_) | Node::Const(_));

                if parenthesize {
                    f.write_str("(")?;
                }
                expr.fmt(f)?;
                if parenthesize {
                    f.write_str(")")?;
                }

                f.write_str(&to_superscript(-1))?;
            }
            Intrinsic::Log { base, arg } => {
                if *base == e.into() {
                    f.write_str("ln(")?;
                    arg.fmt(f)?;
                    f.write_str(")")?;
                } else {
                    f.write_str("log(")?;
                    base.fmt(f)?;
                    f.write_str(", ")?;
                    arg.fmt(f)?;
                    f.write_str(")")?;
                }
            }
            _ => todo!(),
        }
        Ok(())
    }
}

/* -------------------------------- FUNCTIONS ------------------------------- */

pub fn sin(x: impl Into<Expr>) -> Expr {
    Intrinsic::Sin(x.into()).into()
}

pub fn cos(x: impl Into<Expr>) -> Expr {
    Intrinsic::Cos(x.into()).into()
}

pub fn log(base: impl Into<Expr>, x: impl Into<Expr>) -> Expr {
    Intrinsic::Log { base: base.into(), arg: x.into() }.into()
}

pub fn tan(x: impl Into<Expr>) -> Expr {
    let x = x.into();
    sin(x) / cos(x)
}

pub fn ln(x: impl Into<Expr>) -> Expr {
    log(e, x)
}

pub fn exp(x: impl Into<Expr>) -> Expr {
    e ^ x.into()
}

pub fn asin(x: impl Into<Expr>) -> Expr {
    Intrinsic::Asin(x.into()).into()
}

pub fn acos(x: impl Into<Expr>) -> Expr {
    Intrinsic::Acos(x.into()).into()
}

// pub fn atan(expr: impl Into<Expr>) -> Expr {
//     ()
// }
