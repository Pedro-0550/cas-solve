use crate::{ast::Expr, var::Variable};

pub struct Equation {
    lhs: Expr,
    rhs: Expr,
}

pub struct System {
    equations: Vec<Equation>,
    vars: Vec<Variable>,
}
