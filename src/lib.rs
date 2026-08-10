#![feature(vec_try_remove)]
#![feature(if_let_guard)]
#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(generic_atomic)]

mod arena;
mod ast;
mod dimension;
mod eq;
mod expr;
mod intrinsic;
mod simplify;
mod symbol;
mod var;

pub type Complex = num::Complex<f64>;
