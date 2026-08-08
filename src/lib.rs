#![feature(vec_try_remove)]
#![feature(if_let_guard)]

use num::Complex;

mod eq;
mod expr;
mod intrinsic;
mod symbol;
mod unit;
mod var;

pub type Num = Complex<f64>;
