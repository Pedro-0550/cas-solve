use crate::{
    ast::ops::{cos, cosh, log, sin, sinh},
    diff::Differentiable,
    dimension::Unit,
    simplify::Simplify,
    symbol::Symbol,
};

#[test]
fn diff() {
    let x = Symbol::new("x");
    let y = Symbol::new("y");
    let f_of_xy = (x * y
        + x
        + y * log(x, y) * (cos(x ^ 2) ^ 2) * sinh(y) / y / x
            * x
            * x
            * y
            * 1245)
        ^ (cosh(x));

    panic!(
        "f(x, y) = {}\n∂f(x, y)/∂x = {}\n∂f(x, y)/∂y = {}",
        f_of_xy.simplify(),
        f_of_xy.diff(x),
        f_of_xy.diff(y)
    )
}
