use crate::{simplify::Simplify, symbol::Symbol};

#[test]
fn rewriting() {
    let a = Symbol::new("a");
    let b = Symbol::new("b");
    let c = Symbol::new("c");

    panic!("{}", ((a * b) + (a * c)).simplify())
}
