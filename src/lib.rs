#![feature(vec_try_remove)]
#![feature(if_let_guard)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(generic_atomic)]
#![feature(iter_map_windows)]
use std::ops::Mul;

use num::Complex;

mod arena;
mod ast;
mod diff;
mod dimension;
mod eq;
mod macros;
mod normal;
mod scalar;
mod set;
mod simplify;
mod symbol;
mod util;
mod var;

pub use scalar::*;
