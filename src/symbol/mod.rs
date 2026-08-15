use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    ops::{Add, Mul},
    rc::Rc,
    sync::{
        LazyLock, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering},
    },
};

use phf::PhfHash;

use crate::{
    Complex,
    arena::{Arena, Handle},
    ast::Expr,
    dimension::{Dimension, Unit},
    simplify::Range,
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
    range: Range,
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

        if let Some((id, _)) = SYMBOLS.find(|k, v| &*v.name == name) {
            return Symbol(id);
        }

        let handle = SYMBOLS.insert(SymbolInfo {
            name: name.to_owned(),
            unit: Unit::Unitless,
            range: Range::UNBOUNDED,
        });

        Symbol(handle)
    }

    pub fn set_unit(self, unit: Unit) -> Self {
        SYMBOLS.modify(self.0, |i| i.unit = unit);
        self
    }

    pub fn set_range(self, range: Range) -> Self {
        SYMBOLS.modify(self.0, |i| i.range = range);
        self
    }

    pub fn name(&self) -> String {
        SYMBOLS.get_cloned(self.0).expect("invalid symbol handle").name
    }

    pub fn unit(&self) -> Unit {
        SYMBOLS.get_cloned(self.0).expect("invalid symbol handle").unit
    }

    pub fn range(&self) -> Range {
        SYMBOLS.get_cloned(self.0).expect("invalid symbol handle").range
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

impl PhfHash for Symbol {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.0.phf_hash(state);
    }
}

// impl SymbolicContext {
//     pub fn new() -> Self {
//         Self { info: HashMap::new(), next_id: SymbolId(0) }
//     }
//     pub fn symbol(&mut self, name: &str, dimension: Dimension) -> Symbol {
//         if let Some((&id, _)) = self.info.iter().find(|(k, v)| &*v.name == name)
//         {
//             return Symbol(id);
//         }

//         let id = self.next_id;
//         self.next_id.0 += 1;

//         let name: Rc<str> = Rc::from(name);

//         self.info.insert(id, SymbolInfo { name, dimension });

//         Symbol(id)
//     }

//     pub fn name(&self, symbol: &Symbol) -> Option<&str> {
//         self.info.get(&symbol.0).map(|x| &*x.name)
//     }

//     pub fn dimension(&self, symbol: &Symbol) -> Option<&str> {
//         self.info.get(&symbol.0).map(|x| &*x.name)
//     }
// }

// macro_rules! impl_symbol_op {
//     ($trait:ident, $method:ident, $rhs:ty, commutative) => {
//         impl std::ops::$trait<$rhs> for Symbol {
//             type Output = Expr;

//             fn $method(self, rhs: $rhs) -> Self::Output {
//                 Expr::from(self).$method(Expr::from(rhs))
//             }
//         }

//         impl std::ops::$trait<Symbol> for $rhs {
//             type Output = Expr;

//             fn $method(self, rhs: Symbol) -> Self::Output {
//                 Expr::from(self).$method(Expr::from(rhs))
//             }
//         }
//     };

//     ($trait:ident, $method:ident, $rhs:ty) => {
//         impl std::ops::$trait<$rhs> for Symbol {
//             type Output = Expr;

//             fn $method(self, rhs: $rhs) -> Self::Output {
//                 Expr::from(self).$method(Expr::from(rhs))
//             }
//         }
//     };
// }

// impl_symbol_op!(Add, add, Symbol);
// impl_symbol_op!(Add, add, Complex, commutative);
// impl_symbol_op!(Add, add, f64, commutative);

// impl_symbol_op!(Mul, mul, Symbol);
// impl_symbol_op!(Mul, mul, Complex, commutative);
// impl_symbol_op!(Mul, mul, f64, commutative);
