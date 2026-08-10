use core::fmt;
use std::{
    collections::HashMap,
    ops::{BitXor, Div, Mul},
    sync::{
        LazyLock, Mutex, RwLock,
        atomic::{Atomic, AtomicU16, AtomicUsize, Ordering},
    },
};

use num::{complex::Complex64, pow::Pow};
use paste::paste;
use thiserror::Error;

use crate::{
    Complex,
    arena::{Arena, Handle},
    ast::Expr,
};

pub mod isq;
pub mod other;
pub mod si;

#[cfg(test)]
mod test;

/* -------------------------------- CONSTANTS ------------------------------- */

type Composition = Vec<(Unit, i8)>;

static COMPOSITIONS: Arena<Composition> = Arena::new();

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
/// Unit scale is always checked, so you cannot equal eV to J in an equation regardless of chosen level.
///
/// Order is preserved during compositions, and different orders of the same compositions are not eq,
/// but order is ignored during checking, and only the equivalence is taken into account.
#[derive(PartialEq, Clone, Copy, Debug)]
#[allow(non_snake_case)]
pub enum Unit {
    Base { symbol: &'static str, dimension: Dimension },
    Derived { symbol: &'static str, base: &'static [(Unit, i8)] },
    Composed(Handle<Composition>),
    Scaled { symbol: &'static str, base: &'static Unit, scale: f64 },
    Unitless,
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Quantity {
    /// Normalizes this quantity to its non-scaled form.
    /// If this quantity is given in a scaled unit such as eV, it will convert to Joule and scale its value appropriately.
    /// This will be done recursively until a non-scaled unit is reached.
    /// If this quantity is given in a composition containing scaled units, all scaled units will be reduced to non-scaled form.
    ///
    /// If it does not contain any scale, self is returned instead.
    ///
    /// For example, `10 eV` becomes `1.602176634e-18 J`.
    /// This also folds compositions of scaled but dimensionally equal units: `10 eV/J` becomes `1.602176634e-18 (unitless)`
    pub fn normalize(self) -> Quantity {
        let Quantity(mut current_val, mut current_unit) = self;

        loop {
            let normalized = match current_unit {
                Unit::Scaled { base, scale, .. } => {
                    current_val *= scale;
                    *base
                }
                Unit::Composed(id) => {
                    let mut composition = COMPOSITIONS.get_cloned(id).unwrap();

                    for (unit, exp) in composition.iter_mut() {
                        if let Unit::Scaled { base, scale, .. } = *unit {
                            *unit = *base;
                            current_val *= scale.powi(*exp as i32);
                        }
                    }

                    Unit::new_composition(composition)
                }
                _ => current_unit,
            };

            if normalized == current_unit {
                break;
            }

            current_unit = normalized;
        }

        Quantity(current_val, current_unit)
    }

    pub fn value(&self) -> Complex {
        return self.0;
    }

    pub fn unit(&self) -> Unit {
        return self.1;
    }
}

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
    /// For example, `s^-1 * m`, `m * Hz` and `km hr^-1` are equivalent at the dimensional level, but not in the representational level.
    /// In contrast, `s^-1 * m` and `m * s^-1` are equivalent in both worlds.
    pub fn repr_eq(self, rhs: Unit) -> bool {
        match (self, rhs) {
            (Unit::Composed(self_id), Unit::Composed(rhs_id)) => {
                let self_comp = COMPOSITIONS.get_cloned(self_id).unwrap();
                let rhs_comp = COMPOSITIONS.get_cloned(rhs_id).unwrap();

                self_comp.iter().all(|x| rhs_comp.contains(x))
            }
            (..) => self == rhs,
        }
    }

    /// Returns true if this unit is atomic, that is, represented by a single base unit or derived unit, and an optional multiplier.
    /// False if its composed, including exponentiation of a single unit.
    /// TODO: recursively check scaled unit's atomicity
    fn is_atomic(&self) -> bool {
        match self {
            Self::Base { .. } | Self::Derived { .. } => true,

            Self::Scaled { base, .. } => base.is_atomic(),

            Self::Unitless | Self::Composed(_) => false,
        }
    }

    fn new_composition(mut comp: Composition) -> Self {
        fold_composition(&mut comp);

        if let Some((existing_id, _)) = COMPOSITIONS.find(|_, c| **c == comp) {
            return Unit::Composed(existing_id);
        }

        let id = COMPOSITIONS.insert(comp);

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
            match (self, rhs) {
                (Unit::Composed(id), rhs) if rhs.is_atomic() => {
                    let mut lhs_comp = COMPOSITIONS.get_cloned(id).unwrap();

                    lhs_comp.push((rhs, 1));
                    lhs_comp
                }

                (_, Unit::Unitless) => return self,

                (lhs, Unit::Composed(id)) if lhs.is_atomic() => {
                    let mut rhs_comp = COMPOSITIONS.get_cloned(id).unwrap();
                    let mut new_comp = vec![(self, 1)];

                    new_comp.append(&mut rhs_comp);
                    new_comp
                }
                (Unit::Unitless, _) => return rhs,

                (Unit::Composed(lhs_id), Unit::Composed(rhs_id)) => {
                    let mut lhs_comp = COMPOSITIONS.get_cloned(lhs_id).unwrap();
                    let mut rhs_comp = COMPOSITIONS.get_cloned(rhs_id).unwrap();

                    lhs_comp.append(&mut rhs_comp);
                    lhs_comp
                }

                (lhs, rhs) if lhs.is_atomic() && rhs.is_atomic() => {
                    vec![(lhs, 1), (rhs, 1)]
                }
                _ => unreachable!(),
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
            _ if self.is_atomic() => Unit::new_composition(vec![(self, exp)]),
            Self::Composed(id) => {
                let comp = COMPOSITIONS
                    .get_cloned(id)
                    .unwrap()
                    .iter()
                    .map(|(unit, e)| (*unit, e * exp))
                    .collect();

                Unit::new_composition(comp)
            }
            _ => unreachable!(),
        }
    }
}

impl Mul<Unit> for Quantity {
    type Output = Quantity;

    fn mul(self, unit: Unit) -> Self::Output {
        Quantity(self.0, self.1 * unit)
    }
}

impl Mul<Unit> for f64 {
    type Output = Quantity;

    fn mul(self, unit: Unit) -> Self::Output {
        Quantity(Complex { re: self, im: 0.0 }, unit)
    }
}

impl Mul<Unit> for Complex64 {
    type Output = Quantity;

    fn mul(self, unit: Unit) -> Self::Output {
        Quantity(self, unit)
    }
}

impl Div<Unit> for f64 {
    type Output = Quantity;

    fn div(self, unit: Unit) -> Self::Output {
        Quantity(Complex { re: self, im: 0.0 }, unit ^ -1)
    }
}

impl Div<Unit> for Complex64 {
    type Output = Quantity;

    fn div(self, unit: Unit) -> Self::Output {
        Quantity(self, unit ^ -1)
    }
}

impl Div<Unit> for Quantity {
    type Output = Quantity;

    fn div(self, unit: Unit) -> Self::Output {
        Quantity(self.0, self.1 / unit)
    }
}
