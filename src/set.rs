// /* --------------------------------- TRAITS --------------------------------- */
// use std::ops::{BitAnd, BitOr, Sub};

// use crate::Scalar;

// /* --------------------------------- STRUCTS -------------------------------- */
// // Impl this one day: https://arxiv.org/pdf/2402.06430

// #[derive(Clone)]
// pub struct Set(Vec<Rect>);

// #[derive(Clone)]
// pub struct Rect {
//     re: Interval,
//     im: Interval,
// }

// #[derive(Clone, Copy, PartialEq)]
// pub struct Interval {
//     lower: Bound,
//     upper: Bound,
// }

// #[derive(Clone, Copy, PartialEq)]
// pub enum Bound {
//     Unbounded,
//     Closed(f64),
//     Open(f64),
// }

// /* ---------------------------------- IMPLS --------------------------------- */
// impl Set {
//     pub const Z: Self = todo!();
//     pub const Z_NZ: Self = todo!();
//     pub const Z_P: Self = todo!();
//     pub const Z_N: Self = todo!();
//     pub const Z_NP: Self = todo!();
//     pub const Z_NN: Self = todo!();

//     pub const R: Self = todo!();
//     pub const R_NZ: Self = todo!();
//     pub const R_P: Self = todo!();
//     pub const R_N: Self = todo!();
//     pub const R_NP: Self = todo!();
//     pub const R_NN: Self = todo!();

//     pub const I: Self = todo!();
//     pub const I_P: Self = todo!();
//     pub const I_N: Self = todo!();

//     pub const C: Self = todo!();
//     pub const C_Q1: Self = todo!();
//     pub const C_Q2: Self = todo!();
//     pub const C_Q3: Self = todo!();
//     pub const C_Q4: Self = todo!();
//     pub const C_NZ: Self = todo!();

//     pub fn contains(&self, _x: &Scalar) -> bool {
//         todo!()
//     }

//     pub fn is_subset(&self, _other: &Set) -> bool {
//         todo!()
//     }

//     pub fn is_superset(&self, _other: &Set) -> bool {
//         todo!()
//     }

//     pub fn difference(&self, _other: &Set) -> Set {
//         todo!()
//     }

//     pub fn union(&self, _other: &Set) -> Set {
//         todo!()
//     }

//     pub fn intersect(&self, _other: &Set) -> Set {
//         todo!()
//     }

//     pub fn rect(_re: Interval, _im: Interval) -> Self {
//         todo!()
//     }

//     pub fn single(_val: impl Into<Scalar>) -> Self {
//         todo!()
//     }

//     pub fn real(_interval: Interval) -> Self {
//         todo!()
//     }

//     pub fn imag(_interval: Interval) -> Self {
//         todo!()
//     }
// }

// impl Interval {
//     pub const UNBOUNDED: Self = todo!();

//     pub fn new(lower: Bound, upper: Bound) -> Self {
//         Self { lower, upper }
//     }
// }

// impl Bound {
//     pub fn to(self, _upper: Bound) -> Interval {
//         todo!()
//     }
// }

// pub fn open(_val: f64) -> Bound {
//     todo!()
// }

// pub fn closed(_val: f64) -> Bound {
//     todo!()
// }

// impl Sub<Set> for Set {
//     type Output = Set;

//     fn sub(self, _rhs: Set) -> Self::Output {
//         todo!()
//     }
// }

// impl BitAnd<Set> for Set {
//     type Output = Set;

//     fn bitand(self, _rhs: Set) -> Self::Output {
//         todo!()
//     }
// }

// impl BitOr<Set> for Set {
//     type Output = Set;

//     fn bitor(self, _rhs: Set) -> Self::Output {
//         todo!()
//     }
// }
