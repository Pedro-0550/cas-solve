use num::complex::Complex64;

use crate::{Num, symbol::Symbol};

pub enum Variable {
    Unknown(Symbol),
    Known(Symbol, Num),
}
