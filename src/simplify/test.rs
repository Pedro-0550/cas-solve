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

    let expr =
        x + x * y + (1245 * x * y * (cos(x ^ 2) ^ 2) * sinh(y) * log(x, y));

    std::hint::black_box(expr.clone().simplify(&mut None));

    let start = Instant::now();
    let n = 100;

    for _ in 0..n {
        std::hint::black_box(expr.clone().simplify(&mut None));
    }

    let elapsed = start.elapsed();

    println!("n simplifications: {:?}", elapsed);
    panic!("per simplification: {:?}", elapsed / n);
}
