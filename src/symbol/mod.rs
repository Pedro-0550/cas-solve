use std::{
    borrow::Cow,
    collections::HashMap,
    hash::Hash,
    ops::{Add, Mul},
    rc::Rc,
    sync::{
        LazyLock, Mutex, RwLock,
        atomic::{AtomicU16, AtomicUsize, Ordering},
    },
};

use crate::{
    Complex,
    ast::Expr,
    dimension::{Dimension, Unit},
};

/* -------------------------------- CONSTANTS ------------------------------- */

static SYMBOLS: LazyLock<RwLock<HashMap<SymbolId, SymbolInfo>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/* --------------------------------- STRUCTS -------------------------------- */

// pub struct SymbolicContext {
//     info: HashMap<SymbolId, SymbolInfo>,
//     next_id: SymbolId,
// }

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub struct SymbolId(u32);

pub struct SymbolInfo {
    name: String,
    unit: Unit,
}

#[derive(PartialEq, Clone, Debug, Copy, Hash, Eq)]
pub struct Symbol(SymbolId);

/* ---------------------------------- IMPLS --------------------------------- */

impl Symbol {
    pub fn new(name: &str, unit: Unit) -> Self {
        let mut symbols = SYMBOLS.write().unwrap();

        // if let Some((&id, _)) =
        //     symbols.iter().find(|(_, info)| info.name == name)
        // {
        //     return Symbol(id);
        // }

        let id = SymbolId(NEXT_ID.fetch_add(1, Ordering::Relaxed) as u32);

        symbols.insert(id, SymbolInfo { name: name.to_owned(), unit });

        Symbol(id)
    }

    pub fn name(&self) -> String {
        let symbols = SYMBOLS.read().unwrap();

        symbols.get(&self.0).expect("invalid SymbolId").name.clone()
    }

    pub fn unit(&self) -> Unit {
        let symbols = SYMBOLS.read().unwrap();

        symbols.get(&self.0).expect("invalid SymbolId").unit
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
