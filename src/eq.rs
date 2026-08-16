use crate::{dimension::Quantity, expr::Expr, symbol::Symbol};

pub struct Equation {
    lhs: Expr,
    rhs: Expr,
}

pub struct System {
    equations: Vec<Equation>,
    vars: Vec<Variable>,
}

pub enum Variable {
    Unknown(Symbol),
    Known(Symbol, Quantity),
}
