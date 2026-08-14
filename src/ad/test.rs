use crate::{
    ad::Differentiable, ast::ops::log, dimension::Unit, simplify::Simplify,
    symbol::Symbol,
};

#[test]
fn diff() {
    let x = Symbol::new("x", Unit::Unitless);
    let y = Symbol::new("y", Unit::Unitless);

    panic!("{}", (x * y + x + y * log(x, y)).diff(x))
}
