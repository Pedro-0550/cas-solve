use num::complex::Complex64;

use crate::{Num, intrinsic::Intrinsic, symbol::Symbol, unit::Unit};

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(PartialEq, Clone)]
pub enum Expr {
    Symbol(Symbol),
    Const(Num),

    Func(Box<Function>),
    ElementwiseFunc(Box<Function>),

    Intrinsic(Box<Intrinsic>),
    ElementwiseIntrinsic(Box<Intrinsic>),

    Matrix { rows: usize, cols: usize, elements: Vec<Expr> },
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(PartialEq, Clone)]
pub struct Function {
    arguments: Vec<Symbol>,
    value: Expr,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Expr {
    pub fn simplify(&self) -> Expr {
        match self {
            Expr::Intrinsic(intrinsic) => match **intrinsic {
                Intrinsic::Add()
            },
        }
    }
}
