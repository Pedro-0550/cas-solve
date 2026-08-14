use crate::{
    ast::ops::log, simplify::Transformation, symbol::Symbol, transformation,
};

pub fn transformations() -> [Transformation; 18] {
    [
        transformation!(x; x + 0.0 => x),
        transformation!(x; x - 0.0 => x),
        transformation!(x; x - x => 0.0),
        transformation!(x, a, b; x * a + x * b => x * (a + b)),
        transformation!(x, a, b; x * (a + b) => x * a + x * b),
        transformation!(x; x * 0.0 => 0.0),
        transformation!(x; x * 1.0 => x),
        transformation!(x; x / 1.0 => x),
        transformation!(x; x / x => 1.0),
        transformation!(x; -1 * x => -x),
        transformation!(x; x ^ 0.0 => 1.0),
        transformation!(x; x ^ 1.0 => x),
        transformation!(x; 1.0 ^ x => 1.0),
        // 0^0 IS 0 OKAY GUYS? JUST ADMIT IT
        transformation!(x; 0.0 ^ x => 0.0),
        transformation!(b, x; b ^ log(b, x) => x),
        transformation!(x; -(-x) => x),
        transformation!(x, y; (-x) * (-y) => x*y),
        transformation!(x; x + -x => 0.0),
    ]
}
