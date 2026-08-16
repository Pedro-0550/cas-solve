use crate::{
    ast::ops::log, set::Set, simplify::Transformation, transformation,
};

pub fn transformations() -> [Transformation; 25] {
    [
        transformation!(x; x + 0.0 => x),
        transformation!(x; x - 0.0 => x),
        transformation!(x; x - x => 0.0),
        /* -------------------------------------------------------------------------- */
        transformation!(x, a, b; x * a + x * b => x * (a + b)),
        transformation!(x, a, b; x * (a + b) => x * a + x * b),
        /* -------------------------------------------------------------------------- */
        transformation!(x; x * 0.0 => 0.0),
        transformation!(x; x * 1.0 => x),
        transformation!(x; x / 1.0 => x),
        transformation!(x; x / x => 1.0),
        /* -------------------------------------------------------------------------- */
        transformation!(x; -1 * x => -x),
        transformation!(x: Set::C_NZ; x ^ 0.0 => 1.0),
        transformation!(x; x ^ 1.0 => x),
        transformation!(x: Set::R_P, a: Set::R, b: Set::R; (x ^ a) ^ b => x^(a * b)),
        transformation!(x: Set::C, a: Set::Z, b: Set::Z; (x ^ a) ^ b => x^(a * b)),
        transformation!(x: Set::R_P, a: Set::R, b: Set::R; (x ^ a) * (x ^ b) => x^(a + b)),
        transformation!(x: Set::R_P, a: Set::R, b: Set::R; (x ^ a) / (x ^ b) => x^(a - b)),
        transformation!(x; 1.0 ^ x => 1.0),
        // 0^0 IS 0, OKAY GUYS? JUST ADMIT IT
        transformation!(x: Set::C_NZ; 0.0 ^ x => 0.0),
        /* -------------------------------------------------------------------------- */
        transformation!(b: Set::R_P - Set::single(1.0); log(b, 1.0) => 0.0),
        transformation!(b: Set::R_P - Set::single(1.0); log(b, b) => 1.0),
        transformation!(x: Set::R_P, b: Set::R_NZ - Set::single(1.0), a: Set::R; log(b, x ^ a) => a * log(b, x)),
        transformation!(b: Set::R_P - Set::single(1.0), x: Set::R_NZ; b ^ log(b, x) => x),
        transformation!(x: Set::R_P, y: Set::R_P, b: Set::R_P - Set::single(1.0); log(b, x) + log(b, y) => log(b, x * y)),
        transformation!(x: Set::R_P, y: Set::R_P, b: Set::R_P - Set::single(1.0); log(b, x) - log(b, y) => log(b, x / y)),
        /* -------------------------------------------------------------------------- */
        transformation!(x; -(-x) => x),
    ]
}
