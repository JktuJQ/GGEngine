//! `mathcore::shapes` submodule defines several traits and implements structs that are representing various shapes and geometrical primitives.
//!

use crate::mathcore::{
    floats::{almost_equal, FloatOperations},
    transforms::{Rotate, Scale, Transform, Transformation, Translate},
    vectors::{Point, Vector2, Vertex},
    {Angle, Sign},
};
use serde::{Deserialize, Serialize};

/// [`Shape`] trait defines two-dimensional shape on a plane which can be transformed.
///
pub trait Shape: Transform {
    /// Returns perimeter of a shape.
    ///
    fn perimeter(&self) -> f32;
    /// Returns total surface area of a shape.
    ///
    fn area(&self) -> f32;

    /// Returns whether shape contains point or not. Point that lies on the edge or shape's border is considered lying inside shape.
    ///
    fn contains_point(&self, point: Point) -> bool;
    /// Returns axis-aligned bounding box of this shape.
    ///
    fn aabb(&self) -> (Point, Point);
}
/// [`PolygonShape`] trait defines shapes that can be represented by an array of vertices (polygons).
///
/// This trait should have `N` as an associated constant and all methods to return arrays of `N` size,
/// but due to current Rust limitations it is not possible.
/// Even though it could still be part of the trait, it would make it dyn incompatible.
/// Moving `N` to the trait definition would allow multiple `PolygonShape<N>` implementations
/// on the same struct, which is also undesirable.
///
pub trait PolygonShape: Shape {
    /// Returns array of polygon's vertices in clockwise order.
    ///
    fn vertices(&self) -> &[Vertex];

    /// Returns `Vec` with polygon's edges.
    ///
    /// Capacity and length of `self.edges()` is guaranteed to be equal to `self.vertices().len()`.
    ///
    fn edges(&self) -> Vec<LineSegment> {
        let vertices = self.vertices();
        let n = vertices.len();

        let mut edges = Vec::with_capacity(n);
        for i in 0..n {
            edges.push(LineSegment {
                vertices: [vertices[i], vertices[(i + 1) % n]],
            });
        }
        edges
    }
}
/// Implements `Shape::contains_point` method for struct that implements [`PolygonShape`] trait.
///
macro_rules! impl_polygonshape {
    (contains_point) => {
        /// Returns whether polygon contains point or not.
        /// Polygon contains point even in cases where point lies on its edge.
        ///
        fn contains_point(&self, point: Point) -> bool {
            let between = |p: f32, a: f32, b: f32| p >= a.min(b) && p <= a.max(b);
            let mut inside = false;

            for LineSegment { vertices: [a, b] } in self.edges() {
                if point == a || point == b {
                    return true;
                }

                if almost_equal(point.y, a.y)
                    && almost_equal(point.y, b.y)
                    && between(point.x, a.x, b.x)
                {
                    return true;
                }

                if between(point.y, a.y, b.y) {
                    if almost_equal(point.y, a.y) && b.y >= a.y
                        || almost_equal(point.y, b.y) && a.y >= b.y
                    {
                        continue;
                    }
                    let c = (a - point).cross_product(b - point);
                    if almost_equal(c, 0.0) {
                        return true;
                    }
                    if (a.y < b.y) == (c > 0.0) {
                        inside = !inside;
                    }
                }
            }
            inside
        }
    };
    (aabb) => {
        /// Returns array of two corner points of axis-aligned bounding box that contains this shape.
        ///
        /// First point is `(min_x, min_y)` and the second one is `(max_x, max_y)`.
        ///
        fn aabb(&self) -> (Point, Point) {
            let (mut min_x, mut max_x, mut min_y, mut max_y) = (
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
            );
            for vertex in self.vertices() {
                min_x = min_x.min(vertex.x);
                max_x = max_x.max(vertex.x);
                min_y = min_y.min(vertex.y);
                max_y = max_y.max(vertex.y);
            }
            (Point { x: min_x, y: min_y }, Point { x: max_x, y: max_y })
        }
    };
}
/// [`Convex`] marker trait defines polygons which are convex
/// (every internal angle is strictly less than 180 degrees).
///
pub trait Convex: PolygonShape {}

/// [`LineSegment`] struct represents two-dimensional line segment.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineSegment {
    /// Vertices of a segment.
    ///
    pub vertices: [Vertex; 2],
}
impl LineSegment {
    /// Returns length of a segment.
    ///
    pub fn length(&self) -> f32 {
        self.slope().magnitude()
    }

    /// Returns slope of a segment.
    ///
    pub fn slope(&self) -> Vector2 {
        self.vertices[1] - self.vertices[0]
    }
    /// Returns `k` coefficient of a line that contains this segment.
    ///
    /// `k` stands for a gradient or a tangent of inclination angle of a line or a derivative from its equation ->
    /// `y = kx + b`, `k = tg(a) = dy/dx`.
    /// For vertical lines `k` equals `+inf`/`-inf` depending on direction of a segment.
    ///
    pub fn k(&self) -> f32 {
        let slope = self.slope().correct_to(0);
        if almost_equal(slope.y, 0.0) {
            0.0
        } else if almost_equal(slope.x, 0.0) {
            if slope.y > 0.0 {
                f32::INFINITY
            } else {
                f32::NEG_INFINITY
            }
        } else {
            slope.y / slope.x
        }
    }
    /// Returns `b` coefficient of a line that contains this segment.
    ///
    /// `b` stands for height or y-intercept -> `y = kx + b`, `b = y - kx`.
    ///
    pub fn b(&self) -> f32 {
        self.vertices[1].y - self.k() * self.vertices[1].x
    }

    /// Returns point at which two segments intersect.
    /// If lines are collinear (either parallel or coincident), `None` is returned.
    ///
    pub fn intersection(self, other: LineSegment) -> Option<Point> {
        let (s1, s2) = (self.slope(), other.slope());
        let tails = self.vertices[0] - other.vertices[0];

        let d = s1.cross_product(s2);
        if almost_equal(d, 0.0) {
            if self.vertices[0] == other.vertices[0] || self.vertices[0] == other.vertices[1] {
                return Some(self.vertices[0]);
            } else if self.vertices[1] == other.vertices[0] || self.vertices[1] == other.vertices[1]
            {
                return Some(self.vertices[1]);
            }
            return None;
        }

        let s = s1.cross_product(tails);
        let t = s2.cross_product(tails);

        let (s_sign, t_sign, d_sign) = (Sign::from(s), Sign::from(t), Sign::from(d));
        if s_sign == d_sign
            && t_sign == d_sign
            && match d_sign {
                Sign::Positive => s <= d && t <= d,
                Sign::Negative => s >= d && t >= d,
                Sign::Zero => unreachable!("Zero case is already checked out"),
            }
        {
            let t = t / d;
            Some(self.vertices[0] + s1 * t)
        } else {
            None
        }
    }
}
impl FloatOperations for LineSegment {
    fn correct_to(self, digits: i32) -> Self {
        LineSegment {
            vertices: self.vertices.correct_to(digits),
        }
    }

    fn round_up_to(self, digits: i32) -> Self {
        LineSegment {
            vertices: self.vertices.round_up_to(digits),
        }
    }
}
impl Translate for LineSegment {
    /// For a line segment, origin is a midpoint.
    ///
    fn origin(&self) -> Point {
        self.vertices.iter().sum::<Vector2>() * 0.5
    }

    fn translate_on(&mut self, vector: Vector2) {
        self.vertices[0] += vector;
        self.vertices[1] += vector;
    }
}
impl Rotate for LineSegment {
    /// For a line segment, angle is inclination angle of a line that contains line segment.
    ///
    fn angle(&self) -> Angle {
        Angle::from_radians(self.k().atan())
    }

    fn rotate_on(&mut self, angle: Angle) {
        let origin = self.origin();
        let transform_matrix = Transformation::combine([
            Transformation::Translation(-origin),
            Transformation::Rotation(angle),
            Transformation::Translation(origin),
        ]);
        self.vertices[0] = transform_matrix.apply_to(self.vertices[0]);
        self.vertices[1] = transform_matrix.apply_to(self.vertices[1]);
    }
}
impl Scale for LineSegment {
    fn scale(&mut self, scale: Vector2) {
        let origin = self.origin();
        let transform_matrix = Transformation::combine([
            Transformation::Translation(-origin),
            Transformation::Scaling(scale),
            Transformation::Translation(origin),
        ]);
        self.vertices[0] = transform_matrix.apply_to(self.vertices[0]);
        self.vertices[1] = transform_matrix.apply_to(self.vertices[1]);
    }
}
impl Transform for LineSegment {}
impl Shape for LineSegment {
    /// [`LineSegment`] is considered a 2-gon (polygon with two sides).
    /// Due to that, the perimeter is two times the length of a segment.
    ///
    fn perimeter(&self) -> f32 {
        2.0 * self.length()
    }
    fn area(&self) -> f32 {
        0.0
    }

    fn contains_point(&self, point: Point) -> bool {
        if !almost_equal(self.slope().cross_product(point - self.vertices[0]), 0.0) {
            return false;
        }

        let aabb = self.aabb();
        (aabb.0.x <= point.x && point.x <= aabb.1.x) && (aabb.0.y <= point.y && point.y <= aabb.1.y)
    }

    impl_polygonshape!(aabb);
}
impl PolygonShape for LineSegment {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}
impl Convex for LineSegment {}

/// [`Rect`] struct represents transformable two-dimensional rectangle on a surface.
///
/// # Examples
/// ### Initialization
/// ```rust
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonShape};
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::Angle;
/// let mut rect: Rect = Rect::new(
///     Point::zero(),
///     Angle::zero(),
///     3.0,
///     2.0
/// );
/// assert_eq!(
///     rect.vertices(),
///     [
///         Vertex { x: -1.5, y: 1.0 },
///         Vertex { x: 1.5, y: 1.0 },
///         Vertex { x: 1.5, y: -1.0 },
///         Vertex { x: -1.5, y: -1.0 },
///     ]
/// );
/// assert!(rect.contains_point(Point { x: 1.2, y: 0.7 }));
/// assert_eq!(rect.width(), 3.0);
/// assert_eq!(rect.height(), 2.0);
/// ```
///
/// ### Translation
/// ```rust
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonShape};
/// # use ggengine::mathcore::transforms::Translate;
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::Angle;
/// let mut rect: Rect = Rect::new(
///     Point::zero(),
///     Angle::zero(),
///     3.0,
///     2.0
/// );
/// rect.translate_on(Vector2 { x: 1.5, y: 1.0 });
/// assert_eq!(
///     rect.vertices(),
///     [
///         Vertex { x: 0.0, y: 2.0 },
///         Vertex { x: 3.0, y: 2.0 },
///         Vertex { x: 3.0, y: 0.0 },
///         Vertex { x: 0.0, y: 0.0 },
///     ]
/// );
/// ```
///
/// ### Rotation
/// ```rust
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonShape};
/// # use ggengine::mathcore::transforms::Rotate;
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::floats::FloatOperations;
/// # use ggengine::mathcore::Angle;
/// let mut rect: Rect = Rect::new(
///     Point { x: 1.5, y: 1.0 },
///     Angle::zero(),
///     3.0,
///     2.0
/// );
/// rect.rotate_on(Angle::from_degrees(90.0));
/// assert_eq!(
///     <[Vertex; 4]>::try_from(rect.vertices()).expect("Rectangle has 4 vertices").correct_to(2),
///     [
///         Vertex { x: 0.5, y: -0.5 },
///         Vertex { x: 0.5, y: 2.5 },
///         Vertex { x: 2.5, y: 2.5 },
///         Vertex { x: 2.5, y: -0.5 },
///     ]
/// );
/// ```
///
/// ### Scaling
/// ```rust
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonShape};
/// # use ggengine::mathcore::transforms::Scale;
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::Angle;
/// let mut rect: Rect = Rect::new(
///     Point { x: 1.5, y: 1.0 },
///     Angle::zero(),
///     3.0,
///     2.0
/// );
/// rect.scale(Vector2 { x: 2.0, y: 2.0 });
/// assert_eq!(
///     rect.vertices(),
///     [
///         Vertex { x: -1.5, y: 3.0 },
///         Vertex { x: 4.5, y: 3.0 },
///         Vertex { x: 4.5, y: -1.0 },
///         Vertex { x: -1.5, y: -1.0 },
///     ]
/// );
/// ```
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Rect {
    /// Array of rectangle's vertices.
    ///
    vertices: [Vertex; 4],
}
impl Rect {
    /// Returns width of a rectangle.
    ///
    pub fn width(&self) -> f32 {
        (self.top_right() - self.top_left()).magnitude()
    }
    /// Returns height of a rectangle.
    ///
    pub fn height(&self) -> f32 {
        (self.top_left() - self.bottom_left()).magnitude()
    }

    /// Returns top left vertex of the rectangle, if it was axis-aligned.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::shapes::Rect;
    /// # use ggengine::mathcore::vectors::Vertex;
    /// assert_eq!(Rect::default().top_left(), Vertex { x: -0.5, y: 0.5 });
    /// ```
    ///
    pub fn top_left(&self) -> Vertex {
        self.vertices[0]
    }
    /// Returns top right vertex of the rectangle, if it was axis-aligned.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::shapes::Rect;
    /// # use ggengine::mathcore::vectors::Vertex;
    /// assert_eq!(Rect::default().top_right(), Vertex { x: 0.5, y: 0.5 });
    /// ```
    ///
    pub fn top_right(&self) -> Vertex {
        self.vertices[1]
    }
    /// Returns bottom right vertex of the rectangle, if it was axis-aligned.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::shapes::Rect;
    /// # use ggengine::mathcore::vectors::Vertex;
    /// assert_eq!(Rect::default().bottom_right(), Vertex { x: 0.5, y: -0.5 });
    /// ```
    ///
    pub fn bottom_right(&self) -> Vertex {
        self.vertices[2]
    }
    /// Returns bottom left vertex of the rectangle, if it was axis-aligned.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::shapes::Rect;
    /// # use ggengine::mathcore::vectors::Vertex;
    /// assert_eq!(Rect::default().bottom_left(), Vertex { x: -0.5, y: -0.5 });
    /// ```
    ///
    pub fn bottom_left(&self) -> Vertex {
        self.vertices[3]
    }

    /// Constructs rectangle with given origin, angle and size.
    ///
    /// Absolute values of width and height will be used.
    ///
    pub fn new(origin: Point, angle: Angle, width: f32, height: f32) -> Self {
        let mut rect = Rect::default();
        rect.transform(&[
            Transformation::Scaling(Vector2 {
                x: width.abs(),
                y: height.abs(),
            }),
            Transformation::Rotation(angle),
            Transformation::Translation(origin),
        ]);
        rect
    }
}
impl Default for Rect {
    /// Returns axis-aligned square with size 1 centered at zero.
    ///
    fn default() -> Self {
        Rect {
            vertices: [
                Vertex { x: -0.5, y: 0.5 },
                Vertex { x: 0.5, y: 0.5 },
                Vertex { x: 0.5, y: -0.5 },
                Vertex { x: -0.5, y: -0.5 },
            ],
        }
    }
}
impl FloatOperations for Rect {
    fn correct_to(self, digits: i32) -> Self {
        Rect {
            vertices: self.vertices.correct_to(digits),
        }
    }

    fn round_up_to(self, digits: i32) -> Self {
        Rect {
            vertices: self.vertices.round_up_to(digits),
        }
    }
}
impl Translate for Rect {
    fn origin(&self) -> Point {
        self.vertices.iter().sum::<Vector2>() * 0.25
    }

    fn translate_on(&mut self, vector: Vector2) {
        self.vertices
            .iter_mut()
            .for_each(|vertex| *vertex += vector);
    }
}
impl Rotate for Rect {
    fn angle(&self) -> Angle {
        LineSegment {
            vertices: [self.top_left(), self.top_right()],
        }
        .angle()
    }

    fn rotate_on(&mut self, angle: Angle) {
        let origin = self.origin();
        let transform_matrix = Transformation::combine([
            Transformation::Translation(-origin),
            Transformation::Rotation(angle),
            Transformation::Translation(origin),
        ]);
        self.vertices
            .iter_mut()
            .for_each(|vertex| *vertex = transform_matrix.apply_to(*vertex));
    }
}
impl Scale for Rect {
    /// Scaling of a rect does not support reflecting by passing negative scaling.
    /// An absolute value of a vector would be used.
    ///
    fn scale(&mut self, scale: Vector2) {
        let origin = self.origin();
        let transform_matrix = Transformation::combine([
            Transformation::Translation(-origin),
            Transformation::Scaling(Vector2 {
                x: scale.x.abs(),
                y: scale.y.abs(),
            }),
            Transformation::Translation(origin),
        ]);
        self.vertices
            .iter_mut()
            .for_each(|vertex| *vertex = transform_matrix.apply_to(*vertex));
    }
}
impl Transform for Rect {}
impl Shape for Rect {
    fn perimeter(&self) -> f32 {
        2.0 * (self.width() + self.height())
    }

    fn area(&self) -> f32 {
        self.width() * self.height()
    }

    impl_polygonshape!(contains_point);
    impl_polygonshape!(aabb);
}
impl PolygonShape for Rect {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}
impl Convex for Rect {}

#[cfg(test)]
mod tests {
    use crate::mathcore::{
        shapes::LineSegment,
        transforms::{Rotate, Translate},
        vectors::{Point, Vector2},
        Angle,
    };

    #[test]
    fn line_segment2d() {
        use super::LineSegment;

        let mut line1 = LineSegment {
            vertices: [Point { x: 0.0, y: 0.0 }, Point { x: 4.0, y: 4.0 }],
        };
        assert_eq!(line1.length(), 4.0 * 2.0_f32.sqrt());

        let mut line2 = LineSegment {
            vertices: [Point { x: 0.0, y: 6.0 }, Point { x: 3.0, y: 0.0 }],
        };

        assert_eq!((line1.k(), line1.b()), (1.0, 0.0));
        assert_eq!((line2.k(), line2.b()), (-2.0, 6.0));

        assert_eq!(line1.intersection(line2).unwrap(), Point { x: 2.0, y: 2.0 });

        line1.translate_on(Vector2 { x: -2.0, y: -2.0 });
        line1.rotate_on(Angle::from_degrees(45.0));
        line2 = LineSegment {
            vertices: [Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 4.0 }],
        };

        assert_eq!(line1.k(), f32::INFINITY);
        assert!(line1.intersection(line2).is_none());

        line1 = LineSegment {
            vertices: [Point { x: -1.0, y: 2.0 }, Point { x: 1.0, y: 2.0 }],
        };
        line2 = LineSegment {
            vertices: [Point { x: 1.0, y: 2.0 }, Point { x: 2.0, y: 2.0 }],
        };
        assert!(line1.intersection(line2).is_some());
    }

    #[test]
    fn rect2d() {
        use super::{PolygonShape, Rect};
        use crate::mathcore::transforms::Scale;

        let mut rect1 = Rect::new(Point { x: 1.0, y: 1.0 }, Angle::zero(), 3.0, 2.0);

        assert_eq!(
            rect1.vertices(),
            [
                Point { x: -0.5, y: 2.0 },
                Point { x: 2.5, y: 2.0 },
                Point { x: 2.5, y: 0.0 },
                Point { x: -0.5, y: 0.0 }
            ]
        );
        assert_eq!(
            rect1.edges(),
            [
                LineSegment {
                    vertices: [Point { x: -0.5, y: 2.0 }, Point { x: 2.5, y: 2.0 }]
                },
                LineSegment {
                    vertices: [Point { x: 2.5, y: 2.0 }, Point { x: 2.5, y: 0.0 }]
                },
                LineSegment {
                    vertices: [Point { x: 2.5, y: 0.0 }, Point { x: -0.5, y: 0.0 }]
                },
                LineSegment {
                    vertices: [Point { x: -0.5, y: 0.0 }, Point { x: -0.5, y: 2.0 }]
                },
            ]
        );

        let mut rect2 = Rect::new(Point { x: 3.0, y: 3.0 }, Angle::zero(), 3.0, 2.0);

        // translation
        rect1.translate_on(Vector2 { x: 1.0, y: 1.0 });
        rect2.translate_to(Point { x: 2.0, y: 2.0 });
        assert_eq!(rect1.vertices(), rect2.vertices());

        // rotation
        rect1.rotate_on(Angle::from_degrees(270.0));
        rect2.rotate_to(Angle::from_degrees(-90.0));
        assert_eq!(rect1.vertices(), rect2.vertices());

        // scaling
        rect1.scale(Vector2 { x: 3.0, y: 3.0 });
        rect2.scale(Vector2 { x: 3.0, y: 3.0 });
        assert_eq!(rect1.vertices(), rect2.vertices());
    }
}
