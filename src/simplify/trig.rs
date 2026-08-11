use crate::{
    ast::{
        intrinsic::{acos, asin, cos, sin},
        *,
    },
    identity,
    simplify::Identity,
    symbol::Symbol,
};

fn trig_identities() -> [Identity; 5] {
    [
        identity!(x; sin(x) ^ 2.0 + cos(x) ^ 2.0 => 1.0),
        identity!(x; sin(-x) => -sin(x)),
        identity!(x; cos(-x) => cos(x)),
        identity!(x; sin(asin(x)) => x),
        identity!(x; cos(acos(x)) => x),
    ]
}
