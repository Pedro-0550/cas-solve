/* --------------------------------- TRAITS --------------------------------- */

pub trait SetElement {}

pub trait Set {
    type Element: SetElement;

    fn contains<T>(&self, element: Self::Element) -> bool;

    fn is_subset(&self, other: impl Set) -> bool;
    fn is_superset(&self, other: impl Set) -> bool;

    fn union<S: Set>(&self, other: &S) -> Union<Self, S>;
    fn intersect<S: Set>(&self, other: &S) -> Intersection<Self, S>;
    fn difference<S: Set>(&self, other: &S) -> Difference<Self, S>;

    fn complement(&self) -> Self;
}

/* ---------------------------------- ENUMS --------------------------------- */

#[derive(Clone, Copy, PartialEq)]
pub enum Bound<T>
where
    T: Num + Clone + Copy, {
    Unbounded,
    Closed(T),
    Open(T),
}

pub struct Union<T>(Vec<T>);

/* --------------------------------- STRUCTS -------------------------------- */

pub struct Union {}

#[derive(Clone, Copy, PartialEq)]
pub struct Interval<T>
where
    T: Num + Clone + Copy, {
    lower: Bound<T>,
    upper: Bound<T>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Range {
    RealComponents { re: Interval<f64>, im: Interval<f64> },
    IntegerComponents { re: Interval<i64>, im: Interval<i64> },
}
