use crate::{
    ast::{
        intrinsic::{acos, asin, cos, sin},
        *,
    },
    simplify::Transformation,
    symbol::Symbol,
    transformation,
};

pub fn transformations() -> [Transformation; 5] {
    [
        transformation!(x; sin(x) ^ 2.0 + cos(x) ^ 2.0 => 1.0),
        transformation!(x; sin(-x) => -sin(x)),
        transformation!(x; cos(-x) => cos(x)),
        transformation!(x; sin(asin(x)) => x),
        transformation!(x; cos(acos(x)) => x),
    ]
}
