use crate::{identity, simplify::Identity, symbol::Symbol};

fn algebraic_identities() -> &'static [Identity] {
    &[
        // Addition
        identity!(x; x + 0.0 => x),
        identity!(x; x - 0.0 => x),
        identity!(x; x - x => 0.0),
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
        // Negation
        identity!(x; -(-x) => x),
        identity!(x; x + -x => 0.0),
    ]
}
