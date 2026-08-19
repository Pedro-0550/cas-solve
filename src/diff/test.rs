use crate::{
    diff::Differentiable,
    dimension::Unit,
    expr::ops::{cos, cosh, log, sin, sinh},
    simplify::{Simplify, SimplifyContext},
    symbol::Symbol,
};

#[test]
fn diff() {
    let x = Symbol::new("x");
    let y = Symbol::new("y");
    let f_of_xy = ((x * y
        + x
        + y * log(x, y) * (cos(x ^ 2) ^ 2) * sinh(y) / y / x
            * x
            * x
            * y
            * 1245)
        ^ (cosh(x)))
    .simplify(&mut SimplifyContext::new());

    panic!(
        "f(x, y) = {}\n∂f(x, y)/∂x = {}\n∂f(x, y)/∂y = {}",
        f_of_xy,
        f_of_xy.diff(x),
        f_of_xy.diff(y)
    )

    // let expr = cos(x ^ 2) ^ 2;
    // panic!("{}", expr.diff(x));
}
