#![feature(vec_try_remove)]
#![feature(if_let_guard)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(generic_atomic)]

use std::ops::Mul;

use num::Complex;

mod ad;
mod arena;
mod ast;
mod dimension;
mod eq;
mod macros;
mod scalar;
mod simplify;
mod symbol;
mod util;
mod var;

pub use scalar::*;
