//! test_oracles — golden-model testing and geometry validation oracles.
#![forbid(unsafe_code)]
use geometry_kernel::{Point3D, Tolerance};
pub fn assert_point_nearly_equal(a: &Point3D, b: &Point3D, tol: &Tolerance) -> bool {
    a.nearly_equal(b, tol)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn points_are_nearly_equal() {
        let tol = Tolerance::default();
        assert!(assert_point_nearly_equal(
            &Point3D::new(1.0, 2.0, 3.0),
            &Point3D::new(1.000000001, 2.0, 3.0),
            &tol
        ));
    }
}
