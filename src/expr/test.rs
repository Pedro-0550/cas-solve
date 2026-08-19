use crate::{
    dimension::{Unit, si::*},
    simplify::{Simplify, SimplifyContext},
    symbol::Symbol,
};

#[test]
fn formatting() {
    let x = Symbol::new("x").set_unit(Hz);
    let y = Symbol::new("y").set_unit(Hz);
    panic!(
        "{}",
        (((x * y) ^ 2) * y + 10e6 * V / (s * s))
            .simplify(&mut SimplifyContext::new())
    )
}

#[test]
fn simplifying() {
    let x = Symbol::new("x").set_unit(Hz);
    let y = Symbol::new("y").set_unit(Hz);
    let z = Symbol::new("z").set_unit(Hz);

    panic!("{}", (x * z + x * y).simplify(&mut SimplifyContext::new()))
}
