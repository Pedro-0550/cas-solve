use crate::{
    ast::{
        ops::{acos, asin, atan, cos, sin, tan},
        *,
    },
    simplify::Transformation,
    symbol::Symbol,
    transformation,
};

pub fn transformations() -> [Transformation; 6] {
    [
        transformation!(x; (sin(x) ^ 2) + (cos(x) ^ 2) => 1),
        transformation!(x; sin(-x) => -sin(x)),
        transformation!(x; cos(-x) => cos(x)),
        transformation!(x; sin(asin(x)) => x),
        transformation!(x; cos(acos(x)) => x),
        transformation!(x; tan(atan(x)) => x),
    ]
}
