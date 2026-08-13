use crate::{simplify::Simplify, symbol::Symbol};

#[test]
fn rewriting() {
    let a = Symbol::new("a", crate::dimension::Unit::Unitless);
    let b = Symbol::new("b", crate::dimension::Unit::Unitless);
    let c = Symbol::new("c", crate::dimension::Unit::Unitless);

    ((a * b) + (a * c)).simplify();
    panic!()
}
