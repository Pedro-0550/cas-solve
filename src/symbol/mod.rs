use std::{
    fmt::Display,
    hash::Hash,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    arena::{Arena, Handle},
    dimension::Unit,
    // set::Set,
};

/* --------------------------------- MODULES -------------------------------- */

pub mod constants;

/* -------------------------------- CONSTANTS ------------------------------- */

static SYMBOLS: Arena<SymbolInfo> = Arena::new();
static CONSTANTS_REGISTERED: AtomicBool = AtomicBool::new(false);

/* --------------------------------- STRUCTS -------------------------------- */

// pub struct SymbolicContext {
//     info: HashMap<SymbolId, SymbolInfo>,
//     next_id: SymbolId,
// }

#[derive(Clone)]
pub struct SymbolInfo {
    name: String,
    unit: Unit,
    // domain: Set,
}

#[derive(PartialEq, Clone, Debug, Copy, Hash, Eq)]
pub struct Symbol(pub(crate) Handle<SymbolInfo>);

/* ---------------------------------- IMPLS --------------------------------- */

impl Symbol {
    pub fn new(name: &str) -> Self {
        if !CONSTANTS_REGISTERED.load(Ordering::SeqCst) {
            constants::register();
            CONSTANTS_REGISTERED.store(true, Ordering::SeqCst);
        }

        if let Some((id, _)) = SYMBOLS.find(|_k, v| &*v.name == name) {
            return Symbol(id);
        }

        let handle = SYMBOLS.insert(SymbolInfo {
            name: name.to_owned(),
            unit: Unit::Unitless,
            // domain: Set::C,
        });

        Symbol(handle)
    }

    pub fn set_unit(self, unit: Unit) -> Self {
        SYMBOLS.modify(self.0, |i| i.unit = unit);
        self
    }

    // pub fn set_domain(self, domain: Set) -> Self {
    //     SYMBOLS.modify(self.0, |i| i.domain = domain);
    //     self
    // }

    pub fn name(&self) -> String {
        SYMBOLS.get_cloned(self.0).expect("invalid symbol handle").name
    }

    pub fn unit(&self) -> Unit {
        SYMBOLS.get_cloned(self.0).expect("invalid symbol handle").unit
    }

    // pub fn domain(&self) -> Set {
    //     SYMBOLS.get_cloned(self.0).expect("invalid symbol handle").domain
    // }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}
