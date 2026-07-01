//! Parametric curve types — Phase C (Bézier cubic, spline segments).
#![allow(missing_docs)]

use core_math::{Point3D, Scalar, Vector3D, scalar};

/// A cubic Bézier curve defined by four control points P₀–P₃.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierCubic {
    pub p0: Point3D,
    pub p1: Point3D,
    pub p2: Point3D,
    pub p3: Point3D,
}

impl BezierCubic {
    /// Evaluate the curve at parameter t ∈ [0, 1].
    pub fn evaluate(&self, t: f64) -> Point3D {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let u2 = u * u;
        let t2 = t * t;
        let u3 = u2 * u;
        let t3 = t2 * t;
        Point3D::new(
            u3 * self.p0.x.value
                + 3.0 * u2 * t * self.p1.x.value
                + 3.0 * u * t2 * self.p2.x.value
                + t3 * self.p3.x.value,
            u3 * self.p0.y.value
                + 3.0 * u2 * t * self.p1.y.value
                + 3.0 * u * t2 * self.p2.y.value
                + t3 * self.p3.y.value,
            u3 * self.p0.z.value
                + 3.0 * u2 * t * self.p1.z.value
                + 3.0 * u * t2 * self.p2.z.value
                + t3 * self.p3.z.value,
        )
    }

    /// Evaluate the derivative at t.
    pub fn derivative(&self, t: f64) -> Vector3D {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        let a = 3.0 * u * u;
        let b = 6.0 * u * t;
        let c = 3.0 * t * t;
        Vector3D::new(
            a * (self.p1.x - self.p0.x).value
                + b * (self.p2.x - self.p1.x).value
                + c * (self.p3.x - self.p2.x).value,
            a * (self.p1.y - self.p0.y).value
                + b * (self.p2.y - self.p1.y).value
                + c * (self.p3.y - self.p2.y).value,
            a * (self.p1.z - self.p0.z).value
                + b * (self.p2.z - self.p1.z).value
                + c * (self.p3.z - self.p2.z).value,
        )
    }

    /// Approximate length by sampling at `samples` points.
    pub fn length(&self, samples: usize) -> Scalar {
        let n = samples.max(2);
        let mut total = 0.0_f64;
        let mut prev = self.evaluate(0.0);
        for i in 1..n {
            let t = i as f64 / (n - 1) as f64;
            let pt = self.evaluate(t);
            total += prev.distance_to(&pt).value;
            prev = pt;
        }
        scalar(total)
    }

    /// Subdivide into two cubic Bézier curves at t.
    pub fn subdivide(&self, t: f64) -> (Self, Self) {
        let t = t.clamp(0.0, 1.0);
        let q0 = lerp_point(self.p0, self.p1, t);
        let q1 = lerp_point(self.p1, self.p2, t);
        let q2 = lerp_point(self.p2, self.p3, t);
        let r0 = lerp_point(q0, q1, t);
        let r1 = lerp_point(q1, q2, t);
        let b = lerp_point(r0, r1, t);
        (
            Self {
                p0: self.p0,
                p1: q0,
                p2: r0,
                p3: b,
            },
            Self {
                p0: b,
                p1: r1,
                p2: q2,
                p3: self.p3,
            },
        )
    }
}

fn lerp_point(a: Point3D, b: Point3D, t: f64) -> Point3D {
    Point3D::new(
        a.x.value + t * (b.x.value - a.x.value),
        a.y.value + t * (b.y.value - a.y.value),
        a.z.value + t * (b.z.value - a.z.value),
    )
}

/// Tessellate a Bézier cubic into a polyline with at most `max_error` chordal deviation.
pub fn tessellate_bezier(curve: &BezierCubic, max_error: f64) -> Vec<Point3D> {
    let mut points = vec![curve.p0];
    tessellate_recursive(curve, &mut points, 0.0, 1.0, max_error);
    points.push(curve.p3);
    points
}

fn tessellate_recursive(
    curve: &BezierCubic,
    points: &mut Vec<Point3D>,
    t0: f64,
    t1: f64,
    max_error: f64,
) {
    let tm = (t0 + t1) * 0.5;
    let p0 = curve.evaluate(t0);
    let p1 = curve.evaluate(t1);
    let pm = curve.evaluate(tm);
    let chord = p0.distance_to(&p1).value;
    if chord < 1e-12 {
        return;
    }
    let dev = point_to_line_distance(&pm, &p0, &p1);
    if dev > max_error {
        tessellate_recursive(curve, points, t0, tm, max_error);
        tessellate_recursive(curve, points, tm, t1, max_error);
    } else {
        points.push(pm);
    }
}

fn point_to_line_distance(pt: &Point3D, a: &Point3D, b: &Point3D) -> f64 {
    let abv = Vector3D::new(
        b.x.value - a.x.value,
        b.y.value - a.y.value,
        b.z.value - a.z.value,
    );
    let apv = Vector3D::new(
        pt.x.value - a.x.value,
        pt.y.value - a.y.value,
        pt.z.value - a.z.value,
    );
    let cross = abv.cross(&apv);
    let ab_len = abv.length().value;
    if ab_len < 1e-15 {
        return apv.length().value;
    }
    cross.length().value / ab_len
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Tolerance;
    #[test]
    fn bezier_endpoints() {
        let c = BezierCubic {
            p0: Point3D::new(0.0, 0.0, 0.0),
            p1: Point3D::new(1.0, 0.0, 0.0),
            p2: Point3D::new(2.0, 1.0, 0.0),
            p3: Point3D::new(3.0, 0.0, 0.0),
        };
        let s = c.evaluate(0.0);
        let e = c.evaluate(1.0);
        assert!(s.nearly_equal(&c.p0, &Tolerance::default()));
        assert!(e.nearly_equal(&c.p3, &Tolerance::default()));
    }
    #[test]
    fn bezier_subdivision_identity() {
        let c = BezierCubic {
            p0: Point3D::new(0.0, 0.0, 0.0),
            p1: Point3D::new(1.0, 2.0, 0.0),
            p2: Point3D::new(2.0, 2.0, 0.0),
            p3: Point3D::new(3.0, 0.0, 0.0),
        };
        let (a, b) = c.subdivide(0.5);
        assert!(a.p3.nearly_equal(&b.p0, &Tolerance::default()));
    }
    #[test]
    fn tessellation_produces_polyline() {
        let c = BezierCubic {
            p0: Point3D::new(0.0, 0.0, 0.0),
            p1: Point3D::new(1.0, 2.0, 0.0),
            p2: Point3D::new(2.0, 2.0, 0.0),
            p3: Point3D::new(3.0, 0.0, 0.0),
        };
        let pts = tessellate_bezier(&c, 0.1);
        assert!(pts.len() >= 2);
        assert!(pts[0].nearly_equal(&c.p0, &Tolerance::default()));
        assert!(
            pts.last()
                .unwrap()
                .nearly_equal(&c.p3, &Tolerance::default())
        );
    }
}
