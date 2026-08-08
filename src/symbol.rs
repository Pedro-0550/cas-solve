use crate::unit::Unit;

#[derive(PartialEq, Clone)]
pub struct Symbol {
    id: String,
    dimension: Option<Unit>,
}
