use std::time::{self, Instant};

use crate::{
    expr::ops::{cos, cosh, log, sinh},
    simplify::Simplify,
    symbol::Symbol,
};

#[test]
fn factoring() {
    let x = Symbol::new("x");
    let y = Symbol::new("y");

    let start = Instant::now();
    let simp =
        (x + x * y + (1245 * x * y * (cos(x ^ 2) ^ 2) * sinh(y) * log(x, y)))
            .simplify(&mut None);

    panic!("{}, in {} ms", simp, start.elapsed().as_millis())
}
