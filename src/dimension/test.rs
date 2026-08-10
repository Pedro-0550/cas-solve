use super::*;
use crate::dimension::{
    other::eV,
    si::{Hz, J, m, s},
};

/* -------------------------------- FUNCTIONS ------------------------------- */

#[test]
pub fn equivalence() {
    assert_ne!(J * m, m * J);
    assert_eq!(J * m, J * m);

    assert!(!(J * Hz).repr_eq(J / s));
    assert!(!(eV / s).repr_eq(J / s));
    assert!((J * s).repr_eq(s * J));

    assert!((J * Hz).dimensional_eq(J / s));
    assert!(!(eV * s).dimensional_eq(J / s));
}
