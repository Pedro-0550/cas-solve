/* --------------------------------- TRAITS --------------------------------- */

use float_eq::float_eq;

use crate::{Scalar, scalar};

pub trait Set: Clone + Copy {
    fn contains(&self, element: Scalar) -> bool;

    fn is_subset(&self, other: &impl Set) -> bool;

    fn is_superset(&self, other: &impl Set) -> bool {
        other.is_subset(self)
    }

    fn union<S: Set>(&self, other: &S) -> Union<Self, S> {
        Union(self.clone(), other.clone())
    }

    fn intersect<S: Set>(&self, other: &S) -> Intersection<Self, S> {
        Intersection(self.clone(), other.clone())
    }

    fn difference<S: Set>(&self, other: &S) -> Difference<Self, S> {
        Difference(self.clone(), other.clone())
    }

    fn evaluate(&self) -> Vec<Range>;
}

/* ---------------------------------- ENUMS --------------------------------- */

// #[derive(Clone, Copy, PartialEq)]
// pub enum Bound<T>
// where
//     T: Num + Clone + Copy, {
//     Unbounded,
//     Closed(T),
//     Open(T),
// }
/* --------------------------------- STRUCTS -------------------------------- */

#[derive(Clone, Copy)]
pub struct Singleton(Scalar);

#[derive(Clone, Copy)]
pub struct Union<A: Set, B: Set>(A, B);

#[derive(Clone, Copy)]
pub struct Difference<A: Set, B: Set>(A, B);

#[derive(Clone, Copy)]
pub struct Intersection<A: Set, B: Set>(A, B);

#[derive(Clone, Copy, PartialEq)]
pub struct Range {
    components: Universe,
    re: Interval,
    im: Interval,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Universe {
    Integer,
    Real,
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

impl Range {
    pub fn new(components: Universe, re: Interval, im: Interval) -> Self {
        Self { components, re, im }
    }
}

impl Interval {
    pub fn new(lower: Bound, upper: Bound) -> Self {
        Self { lower, upper }
    }
}

impl Set for Singleton {
    fn contains(&self, element: Scalar) -> bool {
        float_eq!(self.0.re, element.re, abs <= scalar::EQ_ABS_TOL)
            && float_eq!(self.0.im, element.im, abs <= scalar::EQ_ABS_TOL)
    }

    fn is_subset(&self, other: &impl Set) -> bool {
        other.contains(self.0)
    }

    fn evaluate(&self) -> Vec<Range> {
        vec![Range::new(
            Universe::Real,
            Interval::new(Bound::Closed(self.0.re), Bound::Closed(self.0.re)),
            Interval::new(Bound::Closed(self.0.im), Bound::Closed(self.0.im)),
        )]
    }
}

impl<A: Set, B: Set> Set for Union<A, B> {
    fn contains(&self, element: Scalar) -> bool {
        self.0.contains(element) || self.1.contains(element)
    }

    fn is_subset(&self, other: &impl Set) -> bool {
        self.0.is_subset(other) && self.1.is_subset(other)
    }

    fn evaluate(&self) -> Vec<Range> {
        todo!()
    }
}

impl<A: Set, B: Set> Set for Intersection<A, B> {
    fn contains(&self, element: Scalar) -> bool {
        self.0.contains(element) && self.1.contains(element)
    }

    fn is_subset(&self, other: &impl Set) -> bool {
        todo!();
    }

    fn evaluate(&self) -> Vec<Range> {
        todo!()
    }
}
