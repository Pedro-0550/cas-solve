use crate::{
    dimension::{Unit, si::*},
    simplify::Simplify,
    symbol::Symbol,
};

#[test]
fn formatting() {
    let x = Symbol::new("x", Hz);
    let y = Symbol::new("y", Hz);
    panic!("{}", (((x * y) ^ 2) * y + 10e6 * V / (s * s)).simplify())
}
