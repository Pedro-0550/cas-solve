#![allow(non_upper_case_globals)]

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use crate::{
    dimension::{Quantity, Unit, si::*},
    symbol::Symbol,
};

macro_rules! constants {
    (
        $(
            $(#[$meta:meta])*
            $name:ident = $value:expr
        ),* $(,)?
    ) => {
        constants!(@defs 0;
            $(
                $(#[$meta])*
                $name = $value
            ),*
        );

        pub(super) fn register() {
            constants!(@register 0;
                $(
                    $name = $value
                ),*
            );
        }
    };

    (@defs $i:expr;
        $(#[$meta:meta])*
        $name:ident = $value:expr
        $(, $($rest:tt)*)?
    ) => {
        $(#[$meta])*
        pub const $name: Symbol =
            Symbol(crate::arena::Handle::new($i));

        constants!(@defs $i + 1; $($($rest)*)?);
    };

    (@defs $i:expr;) => {};

    (@register $i:expr;
        $name:ident = $value:expr
        $(, $($rest:tt)*)?
    ) => {
        crate::symbol::SYMBOLS.insert_at(
            $i,
            crate::symbol::SymbolInfo {
                name: stringify!($name).to_owned(),
                unit: ($value).unit(),
                domain: crate::set::Set::C_NZ
            },
        );

        constants!(@register $i + 1; $($($rest)*)?);
    };

    (@register $i:expr;) => {};
}

/* -------------------------------- CONSTANTS ------------------------------- */

constants! {
    /// Archimedes's constant
    π = 3.1415926535897932384 * Unit::Unitless,

    /// Euler's number
    e = 2.7182818284590452353 * Unit::Unitless,

    /// Speed of light in a vacuum
    /// Source: CODATA 2022 Adjustment
    c = 299792458.0 * m / s,

    // TODO: add more useful constants
}
