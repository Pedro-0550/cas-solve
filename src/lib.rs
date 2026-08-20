#![feature(vec_try_remove)]
#![feature(if_let_guard)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(generic_atomic)]
#![feature(iter_map_windows)]
#![feature(const_try)]
#![feature(duration_constructors)]

use num::Complex;

pub mod core {
    pub mod arena;
    pub mod macros;
    pub mod scalar;
    pub mod util;
}

pub mod diff;
pub mod dimension;
pub mod eq;
pub mod expr;
pub mod simplify;
pub mod symbol;
