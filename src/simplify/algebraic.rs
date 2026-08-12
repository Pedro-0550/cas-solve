use crate::{
    ast::intrinsic::log, simplify::Transformation, symbol::Symbol,
    transformation,
};

pub fn transformations() -> [Transformation; 14] {
    [
        // Addition
        transformation!(x; x + 0.0 => x),
        transformation!(x; x - 0.0 => x),
        transformation!(x; x - x => 0.0),
        transformation!(x, a, b; x * a + x * b => x * (a + b)),
        // Multiplication
        transformation!(x; x * 0.0 => 0.0),
        transformation!(x; x * 1.0 => x),
        // Division
        transformation!(x; x / 1.0 => x),
        transformation!(x; x / x => 1.0),
        // Powers
        transformation!(x; x ^ 0.0 => 1.0),
        transformation!(x; x ^ 1.0 => x),
        transformation!(x; 1.0 ^ x => 1.0),
        transformation!(b, x; b ^ log(b, x) => x),
        // Negation
        transformation!(x; -(-x) => x),
        transformation!(x; x + -x => 0.0),
    ]
}
