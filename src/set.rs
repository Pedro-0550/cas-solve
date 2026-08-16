/* --------------------------------- TRAITS --------------------------------- */

use std::ops::{BitAnd, BitOr, Sub};

use float_eq::float_eq;

use crate::{Scalar, scalar};

/* --------------------------------- STRUCTS -------------------------------- */

// Impl this one day: https://arxiv.org/pdf/2402.06430

#[derive(Clone)]
pub struct Set(Vec<Rect>);

#[derive(Clone)]
pub struct Rect {
    re: Interval,
    im: Interval,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Interval {
    lower: Bound,
    upper: Bound,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Bound {
    Unbounded,
    Closed(f64),
    Open(f64),
}

/* ---------------------------------- IMPLS --------------------------------- */

impl Set {
    pub const Z: Self = todo!();
    pub const Z_NZ: Self = todo!();
    pub const Z_P: Self = todo!();
    pub const Z_N: Self = todo!();
    pub const Z_NP: Self = todo!();
    pub const Z_NN: Self = todo!();

    pub const R: Self = todo!();
    pub const R_NZ: Self = todo!();
    pub const R_P: Self = todo!();
    pub const R_N: Self = todo!();
    pub const R_NP: Self = todo!();
    pub const R_NN: Self = todo!();

    pub const I: Self = todo!();
    pub const I_P: Self = todo!();
    pub const I_N: Self = todo!();

    pub const C: Self = todo!();
    pub const C_Q1: Self = todo!();
    pub const C_Q2: Self = todo!();
    pub const C_Q3: Self = todo!();
    pub const C_Q4: Self = todo!();
    pub const C_NZ: Self = todo!();

    pub fn contains(&self, x: &Scalar) -> bool {
        todo!()
    }

    pub fn is_subset(&self, other: &Set) -> bool {
        todo!()
    }

    pub fn is_superset(&self, other: &Set) -> bool {
        todo!()
    }

    pub fn difference(&self, other: &Set) -> Set {
        todo!()
    }

    pub fn union(&self, other: &Set) -> Set {
        todo!()
    }

    pub fn intersect(&self, other: &Set) -> Set {
        todo!()
    }

    pub fn rect(re: Interval, im: Interval) -> Self {
        todo!()
    }

    pub fn single(val: impl Into<Scalar>) -> Self {
        todo!()
    }

    pub fn real(interval: Interval) -> Self {
        todo!()
    }

    pub fn imag(interval: Interval) -> Self {
        todo!()
    }
}

impl Interval {
    pub const UNBOUNDED: Self = todo!();

    pub fn new(lower: Bound, upper: Bound) -> Self {
        Self { lower, upper }
    }
}

impl Bound {
    pub fn to(self, upper: Bound) -> Interval {
        todo!()
    }
}

pub fn open(val: f64) -> Bound {
    todo!()
}

pub fn closed(val: f64) -> Bound {
    todo!()
}

impl Sub<Set> for Set {
    type Output = Set;

    fn sub(self, rhs: Set) -> Self::Output {
        todo!()
    }
}

impl BitAnd<Set> for Set {
    type Output = Set;

    fn bitand(self, rhs: Set) -> Self::Output {
        todo!()
    }
}

impl BitOr<Set> for Set {
    type Output = Set;

    fn bitor(self, rhs: Set) -> Self::Output {
        todo!()
    }
}
