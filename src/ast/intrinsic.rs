use crate::ast::Expr;

#[derive(PartialEq, Clone, Debug)]
pub enum Intrinsic {
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Div { num: Expr, denom: Expr },
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
    sin(x) / cos(x)
}

pub fn ln(x: impl Into<Expr>) -> Expr {
    log()
}

pub fn asin(x: impl Into<Expr>) -> Expr {
    Expr::Intrinsic(Intrinsic::Asin(x.into()))
}

pub fn acos(x: impl Into<Expr>) -> Expr {
    Expr::Intrinsic(Intrinsic::Acos(x.into()))
}

pub fn atan(expr: impl Into<Expr>) -> Expr {
    ()
}
