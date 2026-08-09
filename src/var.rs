use crate::{dimension::Quantity, symbol::Symbol};

pub enum Variable {
    Unknown(Symbol),
    Known(Symbol, Quantity),
}
