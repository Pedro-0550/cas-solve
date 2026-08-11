use crate::{
    ast::intrinsic::log, identity, simplify::Identity, symbol::Symbol,
};

fn algebraic_identities() -> [Identity; 14] {
    [
        // Addition
        identity!(x; x + 0.0 => x),
        identity!(x; x - 0.0 => x),
        identity!(x; x - x => 0.0),
        identity!(x, a, b; x * a + x * b => x * (a + b)),
        // Multiplication
        identity!(x; x * 0.0 => 0.0),
        identity!(x; x * 1.0 => x),
        // Division
        identity!(x; x / 1.0 => x),
        identity!(x; x / x => 1.0),
        // Powers
        identity!(x; x ^ 0.0 => 1.0),
        identity!(x; x ^ 1.0 => x),
        identity!(x; 1.0 ^ x => 1.0),
        identity!(b, x; b ^ log(b, x) => x),
        // Negation
        identity!(x; -(-x) => x),
        identity!(x; x + -x => 0.0),
    ]
}
