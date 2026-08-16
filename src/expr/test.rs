use crate::{
    dimension::{Unit, si::*},
    simplify::Simplify,
    symbol::Symbol,
};

#[test]
fn formatting() {
    let x = Symbol::new("x").set_unit(Hz);
    let y = Symbol::new("y").set_unit(Hz);
    panic!("{}", (((x * y) ^ 2) * y + 10e6 * V / (s * s)).simplify())
}

#[test]
fn simplifying() {
    let x = Symbol::new("x").set_unit(Hz);
    let y = Symbol::new("y").set_unit(Hz);
    let z = Symbol::new("z").set_unit(Hz);

    panic!("{}", (x + 0.0 + y * 1.0 + x * z + x * y).simplify())
}
