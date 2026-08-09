use crate::ast::Expr;

/* --------------------------------- MODULES -------------------------------- */

mod algebraic;

#[macro_export]
macro_rules! identity {
    ($($sym:ident),+; $from:expr => $to:expr) => {{
        $(
            let $sym = crate::symbol::Symbol::new(stringify!($sym), crate::dimension::Unit::Unitless);
        )+

        Identity {
            from: crate::ast::Expr::from($from),
            to: crate::ast::Expr::from($to),
        }
    }};
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Simplify {
    fn simplify(self) -> Expr;
}

/* --------------------------------- STRUCTS -------------------------------- */

pub struct Identity {
    from: Expr,
    to: Expr,
}
