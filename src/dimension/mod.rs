use core::fmt;
use std::{
    collections::HashMap,
    ops::{BitXor, Div, Mul},
    sync::{
        LazyLock, Mutex, RwLock,
        atomic::{Atomic, AtomicU16, AtomicUsize, Ordering},
    },
};

use num::pow::Pow;
use paste::paste;
use thiserror::Error;

use crate::{Complex, ast::Expr};

pub mod isq;
pub mod si;

/* -------------------------------- CONSTANTS ------------------------------- */

type Composition = Vec<(Unit, i8)>;

// I really dont like this, but you gotta do what you gotta do
static COMPOSITIONS: LazyLock<RwLock<HashMap<CompositionId, Composition>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(Error, Debug, Clone)]
pub enum DimensionalAnalysisError {
    #[error(
        "Tried to sum dimensionally incompatible expressions: {lhs} and {rhs}"
    )]
    IncompatibleSum { lhs: Expr, rhs: Expr },
    #[error(
        "Tried to apply a transcendental function to a dimensioned expression: {expr}"
    )]
    DimensionedTranscendental { expr: Expr },
}

/* --------------------------------- TRAITS --------------------------------- */

pub trait Dimensioned {
    fn analyze(&self) -> Result<Dimension, DimensionalAnalysisError>;
}

/* --------------------------------- STRUCTS -------------------------------- */

#[derive(PartialEq, Clone, Copy, Eq, Hash, Debug)]
pub struct CompositionId(usize);

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Quantity(Complex, Unit);

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[allow(non_snake_case)]
pub struct Dimension {
    T: i8, // time
    L: i8, // length
    M: i8, // mass
    I: i8, // electric current
    Θ: i8, // thermodynamic temperature
    J: i8, // luminous intensity
    N: i8, // amount of substance
}

/// This type is a *representation* of a unit.
///
/// You can have `m * s`, and `s * m`, which are mathematically exactly the same,
/// but are different representations of the `T^1 * L^1` dimension, and are not Eq.
///
/// This allows for using appropriate units for the context, like specifying a decay constant in s^-1 instead of Hz.
///
/// If you want to compare two different units by their dimensions, use [Unit.dimensional_eq].
/// If you want to compare them by their representations, use [Unit.repr_eq].
///
/// Checking can be done at the unit level, in which case you can't have an equation where one side is `s^-1 * m` and the other `Hz * m`, for example,
/// and in the dimension level, where that is perfectly fine because they are dimensionally the same, `T^-1 * L^1`.
///
/// Order is preserved during compositions, and different orders of the same compositions are not eq,
/// but order is ignored during checking, and only the equivalence is taken into account.
#[derive(PartialEq, Clone, Copy, Debug)]
#[allow(non_snake_case)]
pub enum Unit {
    Base { symbol: &'static str, dimension: Dimension },
    Derived { symbol: &'static str, base: &'static [(Unit, i8)] },
    Composed(CompositionId),
    Unitless,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl From<Complex> for Quantity {
    fn from(value: Complex) -> Self {
        Quantity(value, Unit::Unitless)
    }
}

impl From<f64> for Quantity {
    fn from(value: f64) -> Self {
        Quantity(Complex { re: value, im: 0.0 }, Unit::Unitless)
    }
}

impl Dimension {
    pub const fn pow(self, exponent: i8) -> Self {
        Self {
            T: self.T * exponent,
            L: self.L * exponent,
            M: self.M * exponent,
            I: self.I * exponent,
            Θ: self.Θ * exponent,
            J: self.J * exponent,
            N: self.N * exponent,
        }
    }
}

impl const Mul for Dimension {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            T: self.T + rhs.T,
            L: self.L + rhs.L,
            M: self.M + rhs.M,
            I: self.I + rhs.I,
            N: self.N + rhs.N,
            Θ: self.Θ + rhs.Θ,
            J: self.J + rhs.J,
        }
    }
}

impl const Div for Dimension {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            T: self.T - rhs.T,
            L: self.L - rhs.L,
            M: self.M - rhs.M,
            I: self.I - rhs.I,
            N: self.N - rhs.N,
            Θ: self.Θ - rhs.Θ,
            J: self.J - rhs.J,
        }
    }
}

fn fold_composition(composition: &mut Composition) {
    let mut i = 0;

    while i < composition.len() {
        let (unit_i, _) = composition[i];

        let mut j = i + 1;

        while j < composition.len() {
            let (unit_j, exp_j) = composition[j];
            if unit_j == unit_i {
                composition[i].1 += exp_j;
                composition.remove(j);
            } else {
                j += 1;
            }
        }

        if composition[i].1 == 0 {
            composition.remove(i);
        } else {
            i += 1;
        }
    }
}

impl Unit {
    /// Checks if two unit's *dimensions* are equivalent.
    /// For example, `s^-1 * m` and `m * Hz` are equivalent at the dimensional level, but not in the representational level.
    /// In contrast, `s^-1 * m` and `m * s^-1` are equivalent in both worlds.
    pub fn dimensional_eq(self, rhs: Unit) -> bool {
        if let Ok(self_dim) = self.analyze()
            && let Ok(rhs_dim) = rhs.analyze()
        {
            self_dim == rhs_dim
        } else {
            false
        }
    }

    /// Checks if two unit's *representations* are equivalent.
    /// For example, `s^-1 * m` and `m * Hz` are equivalent at the dimensional level, but not in the representational level.
    /// In contrast, `s^-1 * m` and `m * s^-1` are equivalent in both worlds.
    pub fn repr_eq(self, rhs: Unit) -> bool {
        match (self, rhs) {
            (Unit::Composed(self_id), Unit::Composed(rhs_id)) => {
                let compositions = COMPOSITIONS.read().unwrap();
                let self_comp = compositions.get(&self_id).unwrap();
                let rhs_comp = compositions.get(&rhs_id).unwrap();

                self_comp.iter().all(|x| rhs_comp.contains(x))
            }
            (..) => self == rhs,
        }
    }

    fn new_composition(mut comp: Composition) -> Self {
        fold_composition(&mut comp);

        let mut compositions = COMPOSITIONS.write().unwrap();

        if let Some((existing_id, _)) =
            compositions.iter().find(|(_, c)| **c == comp)
        {
            return Unit::Composed(*existing_id);
        }

        let id = CompositionId(NEXT_ID.fetch_add(1, Ordering::Relaxed));
        compositions.insert(id, comp);

        Unit::Composed(id)
    }
}

impl Dimensioned for Unit {
    fn analyze(&self) -> Result<Dimension, DimensionalAnalysisError> {
        todo!()
    }
}

impl Mul for Unit {
    type Output = Unit;

    fn mul(self, rhs: Self) -> Self::Output {
        let result = {
            let compositions = COMPOSITIONS.read().unwrap();
            match (self, rhs) {
                (
                    Unit::Composed(id),
                    Unit::Derived { .. } | Unit::Base { .. },
                ) => {
                    let mut lhs_comp = compositions.get(&id).unwrap().clone();

                    lhs_comp.push((rhs, 1));
                    lhs_comp
                }

                (_, Unit::Unitless) => return self,

                (
                    Unit::Derived { .. } | Unit::Base { .. },
                    Unit::Composed(id),
                ) => {
                    let mut rhs_comp = compositions.get(&id).unwrap().clone();
                    let mut new_comp = vec![(self, 1)];

                    new_comp.append(&mut rhs_comp);
                    new_comp
                }
                (Unit::Unitless, _) => return rhs,

                (Unit::Composed(lhs_id), Unit::Composed(rhs_id)) => {
                    let mut lhs_comp =
                        compositions.get(&lhs_id).unwrap().clone();
                    let mut rhs_comp =
                        compositions.get(&rhs_id).unwrap().clone();

                    lhs_comp.append(&mut rhs_comp);
                    lhs_comp
                }

                (
                    Unit::Derived { .. } | Unit::Base { .. },
                    Unit::Derived { .. } | Unit::Base { .. },
                ) => vec![(self, 1), (rhs, 1)],
            }
        };

        Unit::new_composition(result)
    }
}

impl Div for Unit {
    type Output = Unit;

    fn div(self, rhs: Self) -> Self::Output {
        self * rhs ^ (-1)
    }
}

// Cursed cursed cursed cursed cursed cursed
//
//
//
// cursed
impl BitXor<i8> for Unit {
    type Output = Unit;

    fn bitxor(self, exp: i8) -> Self::Output {
        match self {
            Self::Unitless => self,
            Self::Base { .. } | Self::Derived { .. } => {
                Unit::new_composition(vec![(self, exp)])
            }
            Self::Composed(id) => {
                let comp = {
                    let compositions = COMPOSITIONS.read().unwrap();

                    compositions
                        .get(&id)
                        .unwrap()
                        .iter()
                        .cloned()
                        .map(|(unit, e)| (unit, e * exp))
                        .collect()
                };

                Unit::new_composition(comp)
            }
        }
    }
}
