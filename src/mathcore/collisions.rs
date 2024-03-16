//! `mathcore::collisions` submodule defines several collision systems
//! that are used to detect and resolve collisions between two shapes.
//!

use crate::mathcore::vectors::Vertex;
use crate::mathcore::{
    shapes::{Convex, Segment, Shape},
    vectors::{Point, Vector2},
    Sign,
};

/// `CollisionSystem` trait defines systems that can detect collisions between two shapes and
/// and resolve collisions between them.
///
pub trait CollisionSystem<S1, S2>
where
    S1: Shape,
    S2: Shape,
{
    /// Returns whether two shapes collide or not.
    ///
    fn is_colliding(&self, shape1: &S1, shape2: &S2) -> bool;
    /// Statically resolves collision between two shapes.
    ///
    fn resolve(&self, shape1: &mut S1, shape2: &S2);
}

/// `SATSystem` is a collision system that can detect and resolve collisions between two convex shapes
/// by using algorithm which is based on separating axis theorem.
///
/// One of main features of this system is that is returns early when there is no collision.
///
#[derive(Copy, Clone, Debug)]
pub struct SATSystem;
impl SATSystem {
    /// Implements iterative algorithm of finding axis projection boundaries.
    ///
    fn axis_projection_boundaries(axis_projection: Vector2, vertices: &[Vertex]) -> (f32, f32) {
        let (mut min, mut max): (f32, f32) = (f32::INFINITY, f32::NEG_INFINITY);
        for vertex in vertices {
            let q: f32 = axis_projection.dot_product(*vertex);
            (min, max) = (min.min(q), max.max(q));
        }
        (min, max)
    }
}
impl<S1, S2> CollisionSystem<S1, S2> for SATSystem
where
    S1: Convex,
    S2: Convex,
{
    fn is_colliding(&self, shape1: &S1, shape2: &S2) -> bool {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
            }

            for edge in s1.edges() {
                let [a, b] = edge.points;
                let axis_projection: Vector2 =
                    Vector2::from([-(b.y - a.y), b.x - a.x]).normalized();

                let (min1, max1): (f32, f32) =
                    SATSystem::axis_projection_boundaries(axis_projection, s1.vertices());
                let (min2, max2): (f32, f32) =
                    SATSystem::axis_projection_boundaries(axis_projection, s2.vertices());

                if !(max2 >= min1 && max1 >= min2) {
                    return false;
                }
            }
        }
        true
    }
    fn resolve(&self, shape1: &mut S1, shape2: &S2) {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);

        let mut overlap: f32 = f32::INFINITY;

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
            }

            for edge in s1.edges() {
                let [a, b] = edge.points;
                let axis_projection: Vector2 =
                    Vector2::from([-(b.y - a.y), b.x - a.x]).normalized();

                let (min1, max1): (f32, f32) =
                    SATSystem::axis_projection_boundaries(axis_projection, s1.vertices());
                let (min2, max2): (f32, f32) =
                    SATSystem::axis_projection_boundaries(axis_projection, s2.vertices());

                overlap = overlap.min(max1.min(max2) - min1.max(min2));

                if !(max2 >= min1 && max1 >= min2) {
                    return;
                }
            }
        }

        let d: Vector2 = (shape2.origin() - shape1.origin()).normalized();
        shape1.translate_on(-(d * overlap));
    }
}

/// `DiagonalsSystem` is a collision system that uses intersections between shapes's edges and diagonals to detect and resolve collision.
///
/// One of main features of this system is that is returns early when there is collision
/// (but collision resolving still requires full algorithm cycle).
///
/// ### Note
/// There are few flaws of this algorithm:
/// 1. One shape can be significantly smaller than other and due to high speed be placed inside other shape while not colliding with diagonals or edges.
/// 2. Shape diagonal can be intersecting with another shape diagonal which will lead to doubling displacement which is a bit ugly.
///
/// That said, using `SATSystem` algorithm for collision resolving is preferred (also early return can sometimes perform faster) and
/// collision detection of this algorithm should primarily be used when two shapes are usually colliding and you just need to handle the case when they are not.
///
#[derive(Copy, Clone, Debug)]
pub struct DiagonalsSystem;
impl<S1, S2> CollisionSystem<S1, S2> for DiagonalsSystem
where
    S1: Convex,
    S2: Convex,
{
    fn is_colliding(&self, shape1: &S1, shape2: &S2) -> bool {
        let (mut s1, mut s2): (&dyn Convex, &dyn Convex) = (shape1, shape2);

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
            }

            let center: Point = s1.origin();
            for vertex in s1.vertices() {
                let half_diagonal: Segment = Segment::from([center, *vertex]);
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
        let (mut center1, center2): (Point, Point) = (shape1.origin(), shape2.origin());
        let mut sign: Sign = Sign::Negative;

        for shape in 0..2 {
            if shape == 1 {
                (s1, s2) = (s2, s1);
                sign = -sign;
            }

            for vertex in s1.vertices() {
                let half_diagonal: Segment =
                    Segment::from([if shape == 0 { center1 } else { center2 }, *vertex]);

                let mut displacement: Vector2 = Vector2::zero();

                for edge in s2.edges() {
                    if let Some(intersection_point) = half_diagonal.intersection(edge) {
                        displacement += half_diagonal.slope()
                            - (intersection_point - (if shape == 0 { center1 } else { center2 }));
                    }
                }
                center1 += displacement * (sign as i8 as f32);
            }
        }

        shape1.translate_to(center1);
    }
}

/// Returns whether two shapes are colliding or not.
///
/// Detection is performed by using provided collision system.
///
/// # Example
/// ```rust
/// # use ggengine::mathcore::collisions::{CollisionSystem, SATSystem, DiagonalsSystem, is_colliding};
/// # use ggengine::mathcore::{shapes::Rect, vectors::Point, {Angle, Size}};
/// let rect1: Rect = Rect::from_origin(
///     Point::from([0.0, 0.0]),
///     Angle::default(),
///     Size::from_value(2.0),
///     Size::from_value(2.0),
/// );
/// let rect2: Rect = Rect::from_origin(
///     Point::from([1.0, 0.0]),
///     Angle::default(),
///     Size::from_value(2.0),
///     Size::from_value(2.0),
/// );
/// assert!(is_colliding(&SATSystem, &rect1, &rect2));
/// assert!(is_colliding(&DiagonalsSystem, &rect1, &rect2));
/// ```
///
pub fn is_colliding<S1: Shape, S2: Shape>(
    collision_system: &(impl CollisionSystem<S1, S2> + ?Sized),
    shape1: &S1,
    shape2: &S2,
) -> bool {
    collision_system.is_colliding(shape1, shape2)
}
/// Statically resolves collision between two shapes.
///
/// Resolving is performed by using provided collision system.
///
/// # Example
/// ```rust
/// # use ggengine::mathcore::collisions::{CollisionSystem, SATSystem, DiagonalsSystem, resolve};
/// # use ggengine::mathcore::{shapes::{PolygonLike, Rect}, vectors::{Vertex, Point}, {Angle, Size}};
/// let mut rect1: Rect = Rect::from_origin(
///     Point::from([0.0, 0.0]),
///     Angle::default(),
///     Size::from_value(2.0),
///     Size::from_value(2.0),
/// );
/// let rect2: Rect = Rect::from_origin(
///     Point::from([1.0, 0.0]),
///     Angle::default(),
///     Size::from_value(2.0),
///     Size::from_value(2.0),
/// );
/// resolve(&SATSystem, &mut rect1, &rect2); // or `resolve(&DiagonalsSystem, &mut rect1, &rect2);`
/// ```
///
pub fn resolve<S1: Shape, S2: Shape>(
    collision_system: &(impl CollisionSystem<S1, S2> + ?Sized),
    shape1: &mut S1,
    shape2: &S2,
) {
    collision_system.resolve(shape1, shape2);
}

#[cfg(test)]
mod tests {
    use super::CollisionSystem;
    use crate::mathcore::{
        shapes::{PolygonLike, Rect},
        vectors::{Point, Vertex},
        {Angle, Size},
    };

    #[test]
    fn sat_system() {
        use super::SATSystem;

        let mut rect1: Rect = Rect::from_origin(
            Point::from([0.0, 0.0]),
            Angle::default(),
            Size::from_value(2.0),
            Size::from_value(2.0),
        );
        let rect2: Rect = Rect::from_origin(
            Point::from([1.0, 0.0]),
            Angle::default(),
            Size::from_value(2.0),
            Size::from_value(2.0),
        );
        assert!(SATSystem.is_colliding(&rect1, &rect2));
        SATSystem.resolve(&mut rect1, &rect2);
        assert_eq!(
            rect1.vertices(),
            [
                Vertex::from([-2.0, 1.0]),
                Vertex::from([0.0, 1.0]),
                Vertex::from([0.0, -1.0]),
                Vertex::from([-2.0, -1.0])
            ],
        )
    }

    #[test]
    fn diagonals_system() {
        use super::DiagonalsSystem;

        let mut rect1: Rect = Rect::from_origin(
            Point::from([0.0, 0.0]),
            Angle::default(),
            Size::from_value(2.0),
            Size::from_value(2.0),
        );
        let rect2: Rect = Rect::from_origin(
            Point::from([1.0, 0.0]),
            Angle::default(),
            Size::from_value(2.0),
            Size::from_value(2.0),
        );
        assert!(DiagonalsSystem.is_colliding(&rect1, &rect2));
        DiagonalsSystem.resolve(&mut rect1, &rect2);
        assert_eq!(
            rect1.vertices(),
            [
                Vertex::from([-1.0, 1.0]),
                Vertex::from([1.0, 1.0]),
                Vertex::from([1.0, -1.0]),
                Vertex::from([-1.0, -1.0])
            ],
        )
    }

    #[test]
    fn helper_fns() {
        use super::{is_colliding, CollisionSystem, DiagonalsSystem, SATSystem};

        let mut collision_system: &dyn CollisionSystem<Rect, Rect>;

        let rect1: Rect = Rect::from_origin(
            Point::from([0.0, 0.0]),
            Angle::default(),
            Size::from_value(2.0),
            Size::from_value(2.0),
        );
        let rect2: Rect = Rect::from_origin(
            Point::from([1.0, 0.0]),
            Angle::default(),
            Size::from_value(2.0),
            Size::from_value(2.0),
        );

        collision_system = &SATSystem;
        assert!(is_colliding(collision_system, &rect1, &rect2));

        collision_system = &DiagonalsSystem;
        assert!(is_colliding(collision_system, &rect1, &rect2));
    }
}
