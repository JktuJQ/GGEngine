//! `mathcore::collisions` submodule defines several collision detectors
//! that are used to detect and resolve collisions between two shapes.
//!

use crate::mathcore::{
    shapes::{Convex, Segment, Shape},
    vectors::{Vector2, Vertex},
    Sign,
};

/// [`CollisionDetector`] trait defines systems that can detect collisions between two shapes and
/// resolve collisions between them.
///
/// Methods take immutable access to the detector itself, and so
/// if you your collision detection depends on some external factors or
/// if you need to implement collision counter - `Cell`/`RefCell` might help.
///
pub trait CollisionDetector<S1, S2>
where
    S1: Shape,
    S2: Shape,
{
    /// Returns whether two shapes collide or not.
    ///
    fn are_colliding(&self, shape1: &S1, shape2: &S2) -> bool;
    /// Resolves collision between two shapes.
    ///
    fn resolve(&self, shape1: &mut S1, shape2: &S2);
}

/// [`SATDetector`] is a collision detector that can detect and resolve collisions
/// between two convex shapes by using algorithm which is based on separating axis theorem.
///
/// One of main features of this system is that it returns early when there is no collision.
///
/// ### Applications
/// This algorithm is preferred for collision detection when two shapes are not usually colliding, so you
/// just need to handle the case when they are. Collision resolving of this algorithm is stable and quite fast.
///
#[derive(Copy, Clone, Debug)]
pub struct SATDetector;
impl SATDetector {
    /// Implements iterative algorithm of finding axis projection boundaries.
    ///
    fn axis_projection_boundaries(axis_projection: Vector2, vertices: &[Vertex]) -> (f32, f32) {
        let (mut min, mut max) = (f32::INFINITY, f32::NEG_INFINITY);
        for vertex in vertices {
            let q = axis_projection.dot_product(*vertex);
            (min, max) = (min.min(q), max.max(q));
        }
        (min, max)
    }
}
impl<S1, S2> CollisionDetector<S1, S2> for SATDetector
where
    S1: Convex,
    S2: Convex,
{
    fn are_colliding(&self, shape1: &S1, shape2: &S2) -> bool {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
            }

            for edge in s1.edges() {
                let axis_projection = Vector2 {
                    x: -(edge.point2.y - edge.point1.y),
                    y: edge.point2.x - edge.point1.x,
                }
                .normalized();

                let (min1, max1) =
                    SATDetector::axis_projection_boundaries(axis_projection, s1.vertices());
                let (min2, max2) =
                    SATDetector::axis_projection_boundaries(axis_projection, s2.vertices());

                if !(max2 >= min1 && max1 >= min2) {
                    return false;
                }
            }
        }
        true
    }
    fn resolve(&self, shape1: &mut S1, shape2: &S2) {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);

        let mut overlap = f32::INFINITY;

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
            }

            for edge in s1.edges() {
                let axis_projection = Vector2 {
                    x: -(edge.point2.y - edge.point1.y),
                    y: edge.point2.x - edge.point1.x,
                }
                .normalized();

                let (min1, max1) =
                    SATDetector::axis_projection_boundaries(axis_projection, s1.vertices());
                let (min2, max2) =
                    SATDetector::axis_projection_boundaries(axis_projection, s2.vertices());

                overlap = overlap.min(max1.min(max2) - min1.max(min2));

                if !(max2 >= min1 && max1 >= min2) {
                    return;
                }
            }
        }

        let d = (shape2.origin() - shape1.origin()).normalized();
        shape1.translate_on(-(d * overlap));
    }
}

/// [`DiagonalsDetector`] is a collision detector
/// that uses intersections between shapes edges and diagonals to detect and resolve collision.
///
/// One of main features of this system is that it returns early when there is collision
/// (but collision resolving still requires full algorithm cycle).
///
/// ### Applications
/// There are few flaws of this algorithm:
/// 1. One shape can be significantly smaller than other and due to high speed be placed inside other shape while not colliding with diagonals or edges.
/// 2. Shape diagonal can be intersecting with another shape diagonal which will lead to doubling displacement which is a bit ugly.
///
/// That said, using [`SATDetector`] algorithm for collision resolving is preferred
/// due to faster implementation and possible early returns and
/// collision detection of this algorithm should primarily be used when two shapes are usually colliding,
/// so you just need to handle the case when they are not.
///
#[derive(Copy, Clone, Debug)]
pub struct DiagonalsDetector;
impl<S1, S2> CollisionDetector<S1, S2> for DiagonalsDetector
where
    S1: Convex,
    S2: Convex,
{
    fn are_colliding(&self, shape1: &S1, shape2: &S2) -> bool {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
            }

            let center = s1.origin();
            for vertex in s1.vertices() {
                let half_diagonal = Segment {
                    point1: center,
                    point2: *vertex,
                };
                for edge in s2.edges() {
                    if half_diagonal.intersection(edge).is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }
    fn resolve(&self, shape1: &mut S1, shape2: &S2) {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);
        let (mut center1, center2) = (shape1.origin(), shape2.origin());
        let mut sign = Sign::Negative;

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
                sign = -sign;
            }

            for vertex in s1.vertices() {
                let half_diagonal = Segment {
                    point1: if shape == 0 { center1 } else { center2 },
                    point2: *vertex,
                };

                let mut displacement = Vector2::zero();

                for edge in s2.edges() {
                    if let Some(intersection_point) = half_diagonal.intersection(edge) {
                        displacement +=
                            half_diagonal.slope() - (intersection_point - half_diagonal.point1);
                    }
                }
                center1 += displacement * (sign as i8 as f32);
            }
        }

        shape1.translate_to(center1);
    }
}

#[cfg(test)]
mod tests {
    use super::CollisionDetector;
    use crate::mathcore::{
        shapes::{PolygonShape, Rect},
        vectors::{Point, Vertex},
        {Angle, Size},
    };

    #[test]
    fn sat_detector() {
        use super::SATDetector;

        let mut rect1 = Rect::from_origin(
            Point { x: 0.0, y: 0.0 },
            Angle::default(),
            Size::try_from(2.0).expect("Value is in correct range."),
            Size::try_from(2.0).expect("Value is in correct range."),
        );
        let rect2 = Rect::from_origin(
            Point { x: 1.0, y: 0.0 },
            Angle::default(),
            Size::try_from(2.0).expect("Value is in correct range."),
            Size::try_from(2.0).expect("Value is in correct range."),
        );
        assert!(SATDetector.are_colliding(&rect1, &rect2));
        SATDetector.resolve(&mut rect1, &rect2);
        assert_eq!(
            rect1.vertices(),
            [
                Vertex { x: -2.0, y: 1.0 },
                Vertex { x: 0.0, y: 1.0 },
                Vertex { x: 0.0, y: -1.0 },
                Vertex { x: -2.0, y: -1.0 }
            ],
        )
    }

    #[test]
    fn diagonals_detector() {
        use super::DiagonalsDetector;

        let mut rect1 = Rect::from_origin(
            Point { x: 0.0, y: 0.0 },
            Angle::default(),
            Size::try_from(2.0).expect("Value is in correct range."),
            Size::try_from(1.0).expect("Value is in correct range."),
        );
        let rect2 = Rect::from_origin(
            Point { x: 1.0, y: 0.0 },
            Angle::default(),
            Size::try_from(2.0).expect("Value is in correct range."),
            Size::try_from(2.0).expect("Value is in correct range."),
        );
        assert!(DiagonalsDetector.are_colliding(&rect1, &rect2));
        DiagonalsDetector.resolve(&mut rect1, &rect2);
        assert_eq!(
            rect1.vertices(),
            [
                Vertex { x: -2.0, y: 0.5 },
                Vertex { x: 0.0, y: 0.5 },
                Vertex { x: 0.0, y: -0.5 },
                Vertex { x: -2.0, y: -0.5 }
            ],
        )
    }
}
