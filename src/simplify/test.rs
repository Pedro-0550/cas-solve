use std::time::{self, Instant};

use crate::{
    expr::ops::{cos, cosh, log, sinh},
    simplify::{Simplify, SimplifyContext},
    symbol::Symbol,
};

#[test]
fn factoring() {
    let x = Symbol::new("x");
    let y = Symbol::new("y");

    let expr =
        x + x * y + (1245 * x * y * (cos(x ^ 2) ^ 2) * sinh(y) * log(x, y));

    std::hint::black_box(expr.clone().simplify(&mut SimplifyContext::new()));

    let start = Instant::now();
    let n = 100;

    for _ in 0..n {
        std::hint::black_box(
            expr.clone().simplify(&mut SimplifyContext::new()),
        );
    }

    let elapsed = start.elapsed();

    println!("n simplifications: {:?}", elapsed);
    println!("{}", expr.simplify(&mut SimplifyContext::new()));
    panic!("per simplification: {:?}", elapsed / n);
}
