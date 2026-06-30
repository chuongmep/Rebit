//! Analytic geometry primitives: lines, planes, bounding boxes, and CSG
//! primitive shapes.
//!
//! These are "geometry-level" objects that exist independently of topology
//! (they have no explicit vertex/edge/face graph).  The topology layer
//! ([`super::topology`]) references shapes like `Plane` to define face
//! geometry.

use core_math::{Interval, Point3D, Scalar, Tolerance, Vector3D};

// ---------------------------------------------------------------------------
// Line
// ---------------------------------------------------------------------------

/// An infinite line defined by an origin point and a direction vector.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// A point on the line.
    pub origin: Point3D,
    /// Direction of the line (should be normalized for consistent results).
    pub direction: Vector3D,
}

impl Line {
    /// Create a new line; `direction` should be normalized.
    #[inline]
    pub fn new(origin: Point3D, direction: Vector3D) -> Self {
        Self { origin, direction }
    }

    /// Closest point on this line to a query point.
    pub fn closest_point(&self, point: &Point3D) -> Point3D {
        let v = Vector3D::between(&self.origin, point);
        let t = v.dot(&self.direction);
        self.origin
            + self.direction.scale(t)
    }

    /// Distance from this line to a point.
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let closest = self.closest_point(point);
        closest.distance_to(point)
    }
}

// ---------------------------------------------------------------------------
// Plane
// ---------------------------------------------------------------------------

/// An infinite plane defined by a normal and a signed distance from origin.
///
/// Equation: `normal · p + d = 0` for any point `p` on the plane.
/// The distance `d` is chosen so that `-(normal · any_point_on_plane) = d`.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    /// Unit normal vector.
    pub normal: Vector3D,
    /// Signed distance from origin along the normal.
    pub d: Scalar,
}

impl Plane {
    /// Create a plane from a unit normal and a point on the plane.
    #[inline]
    pub fn from_normal_and_point(normal: Vector3D, point: &Point3D) -> Self {
        let d = Scalar::new(-normal.dot(&Vector3D::new(point.x, point.y, point.z)).value);
        Self { normal, d }
    }

    /// Signed distance from a point to this plane.
    ///
    /// Positive values mean the point lies on the side of the plane toward
    /// which the normal points.
    #[inline]
    pub fn signed_distance(&self, point: &Point3D) -> Scalar {
        let v = Vector3D::new(point.x, point.y, point.z);
        Scalar::new(self.normal.dot(&v).value + self.d.value)
    }

    /// Project a point onto the plane.
    #[inline]
    pub fn project_point(&self, point: &Point3D) -> Point3D {
        let dist = self.signed_distance(point);
        let offset = self.normal.scale(Scalar::new(-dist.value));
        Point3D::new(
            point.x + offset.x,
            point.y + offset.y,
            point.z + offset.z,
        )
    }

    /// `true` when the point lies on the plane within tolerance.
    #[inline]
    pub fn contains_point(&self, point: &Point3D, tol: &Tolerance) -> bool {
        tol.nearly_equal(self.signed_distance(point).value, 0.0)
    }
}

// ---------------------------------------------------------------------------
// BoundingBox (axis-aligned)
// ---------------------------------------------------------------------------

/// An axis-aligned bounding box defined by `[x_min, x_max] × [y_min, y_max] × [z_min, z_max]`.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundingBox {
    /// Interval along X axis.
    pub x: Interval,
    /// Interval along Y axis.
    pub y: Interval,
    /// Interval along Z axis.
    pub z: Interval,
}

impl BoundingBox {
    /// Create a bounding box from two opposite corners (order does not matter).
    #[inline]
    pub fn from_corners(a: &Point3D, b: &Point3D) -> Self {
        Self {
            x: Interval::new(
                Scalar::new(a.x.value.min(b.x.value)),
                Scalar::new(a.x.value.max(b.x.value)),
            ),
            y: Interval::new(
                Scalar::new(a.y.value.min(b.y.value)),
                Scalar::new(a.y.value.max(b.y.value)),
            ),
            z: Interval::new(
                Scalar::new(a.z.value.min(b.z.value)),
                Scalar::new(a.z.value.max(b.z.value)),
            ),
        }
    }

    /// Create a bounding box that tightly encloses a set of points.
    ///
    /// Returns `None` when the iterator is empty.
    pub fn from_points(mut points: impl Iterator<Item = Point3D>) -> Option<Self> {
        let first = points.next()?;
        let mut bbox = Self::from_corners(&first, &first);
        for pt in points {
            bbox.expand_by_point(&pt);
        }
        Some(bbox)
    }

    /// Expand the box to include `point`.
    #[inline]
    pub fn expand_by_point(&mut self, point: &Point3D) {
        self.x.expand(point.x);
        self.y.expand(point.y);
        self.z.expand(point.z);
    }

    /// Expand the box to include another bounding box.
    #[inline]
    pub fn expand_by_box(&mut self, other: &Self) {
        self.x.expand(other.x.min);
        self.x.expand(other.x.max);
        self.y.expand(other.y.min);
        self.y.expand(other.y.max);
        self.z.expand(other.z.min);
        self.z.expand(other.z.max);
    }

    /// Corner point at (min, min, min).
    #[inline]
    pub fn min_corner(&self) -> Point3D {
        Point3D::new(self.x.min, self.y.min, self.z.min)
    }

    /// Corner point at (max, max, max).
    #[inline]
    pub fn max_corner(&self) -> Point3D {
        Point3D::new(self.x.max, self.y.max, self.z.max)
    }

    /// Center of the bounding box.
    #[inline]
    pub fn center(&self) -> Point3D {
        Point3D::new(self.x.midpoint(), self.y.midpoint(), self.z.midpoint())
    }

    /// Volume: `dx * dy * dz`.
    #[inline]
    pub fn volume(&self) -> Scalar {
        self.x.length() * self.y.length() * self.z.length()
    }

    /// `true` when `point` is inside (or on the boundary) of the box under `tol`.
    #[inline]
    pub fn contains_point(&self, point: &Point3D, tol: &Tolerance) -> bool {
        self.x.contains(point.x, tol)
            && self.y.contains(point.y, tol)
            && self.z.contains(point.z, tol)
    }

    /// `true` when two bounding boxes overlap under `tol`.
    #[inline]
    pub fn overlaps(&self, other: &Self, tol: &Tolerance) -> bool {
        self.x.overlaps(&other.x, tol)
            && self.y.overlaps(&other.y, tol)
            && self.z.overlaps(&other.z, tol)
    }
}

// ---------------------------------------------------------------------------
// CSG primitive shapes (unbaked — used to describe solids)
// ---------------------------------------------------------------------------

/// A CSG (Constructive Solid Geometry) primitive.
///
/// These represent simple 3D shapes that can later be combined with
/// boolean operations (union, subtract, intersect) in Phase B.
#[derive(Debug, Clone, PartialEq)]
pub enum CsgPrimitive {
    /// Axis-aligned box specified by two opposite corners.
    Box {
        /// Minimum corner.
        min: Point3D,
        /// Maximum corner.
        max: Point3D,
    },
    /// Right circular cylinder along the Z axis.
    Cylinder {
        /// Center of the bottom face.
        base_center: Point3D,
        /// Radius.
        radius: Scalar,
        /// Height (positive = along +Z).
        height: Scalar,
    },
    /// Sphere.
    Sphere {
        /// Center of the sphere.
        center: Point3D,
        /// Radius.
        radius: Scalar,
    },
}

impl CsgPrimitive {
    /// Compute the axis-aligned bounding box of this primitive.
    pub fn bounding_box(&self) -> BoundingBox {
        match self {
            Self::Box { min, max } => BoundingBox::from_corners(min, max),
            Self::Cylinder {
                base_center,
                radius,
                height,
            } => {
                let r = radius.value;
                let h = height.value;
                BoundingBox::from_corners(
                    &Point3D::new(
                        base_center.x.value - r,
                        base_center.y.value - r,
                        base_center.z.value,
                    ),
                    &Point3D::new(
                        base_center.x.value + r,
                        base_center.y.value + r,
                        base_center.z.value + h,
                    ),
                )
            }
            Self::Sphere { center, radius } => {
                let r = radius.value;
                BoundingBox::from_corners(
                    &Point3D::new(center.x.value - r, center.y.value - r, center.z.value - r),
                    &Point3D::new(center.x.value + r, center.y.value + r, center.z.value + r),
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::scalar;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn line_closest_point_on_axis() {
        let line = Line::new(Point3D::ORIGIN, Vector3D::X);
        let pt = Point3D::new(5.0, 3.0, 0.0);
        let closest = line.closest_point(&pt);
        assert!(closest.nearly_equal(&Point3D::new(5.0, 0.0, 0.0), &tol()));
    }

    #[test]
    fn plane_signed_distance() {
        // Plane y = 2, normal = +Y
        let plane = Plane::from_normal_and_point(Vector3D::Y, &Point3D::new(0.0, 2.0, 0.0));
        assert!(tol().nearly_equal(
            plane.signed_distance(&Point3D::new(1.0, 5.0, 3.0)).value,
            3.0,
        ));
        assert!(tol().nearly_equal(
            plane.signed_distance(&Point3D::new(1.0, -1.0, 3.0)).value,
            -3.0,
        ));
        assert!(plane.contains_point(&Point3D::new(10.0, 2.0, 42.0), &tol()));
    }

    #[test]
    fn plane_project_point() {
        let plane = Plane::from_normal_and_point(Vector3D::Z, &Point3D::ORIGIN);
        let projected = plane.project_point(&Point3D::new(1.0, 2.0, 5.0));
        assert!(projected.nearly_equal(&Point3D::new(1.0, 2.0, 0.0), &tol()));
    }

    #[test]
    fn bounding_box_from_points() {
        let pts = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(0.0, 5.0, -3.0),
        ];
        let bbox = BoundingBox::from_points(pts.into_iter()).unwrap();
        assert!(bbox.min_corner().nearly_equal(
            &Point3D::new(0.0, 0.0, -3.0),
            &tol()
        ));
        assert!(bbox.max_corner().nearly_equal(
            &Point3D::new(10.0, 5.0, 0.0),
            &tol()
        ));
    }

    #[test]
    fn bounding_box_empty_iterator() {
        let bbox: Option<BoundingBox> = BoundingBox::from_points(std::iter::empty());
        assert!(bbox.is_none());
    }

    #[test]
    fn bounding_box_contains_point() {
        let bbox = BoundingBox::from_corners(
            &Point3D::new(0.0, 0.0, 0.0),
            &Point3D::new(10.0, 10.0, 10.0),
        );
        assert!(bbox.contains_point(&Point3D::new(5.0, 5.0, 5.0), &tol()));
        assert!(!bbox.contains_point(&Point3D::new(10.1, 5.0, 5.0), &tol()));
    }

    #[test]
    fn bounding_boxes_overlap() {
        let a = BoundingBox::from_corners(
            &Point3D::new(0.0, 0.0, 0.0),
            &Point3D::new(5.0, 5.0, 5.0),
        );
        let b = BoundingBox::from_corners(
            &Point3D::new(4.0, 4.0, 4.0),
            &Point3D::new(10.0, 10.0, 10.0),
        );
        let c = BoundingBox::from_corners(
            &Point3D::new(10.0, 10.0, 10.0),
            &Point3D::new(20.0, 20.0, 20.0),
        );
        assert!(a.overlaps(&b, &tol()));
        assert!(!a.overlaps(&c, &tol()));
    }

    #[test]
    fn csg_primitive_bbox_sphere() {
        let s = CsgPrimitive::Sphere {
            center: Point3D::new(1.0, 2.0, 3.0),
            radius: scalar(2.0),
        };
        let bbox = s.bounding_box();
        assert!(bbox.min_corner().nearly_equal(
            &Point3D::new(-1.0, 0.0, 1.0),
            &tol()
        ));
        assert!(bbox.max_corner().nearly_equal(
            &Point3D::new(3.0, 4.0, 5.0),
            &tol()
        ));
    }
}