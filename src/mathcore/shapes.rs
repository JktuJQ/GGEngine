//! `mathcore::shapes` submodule defines several traits and implements structs that are representing various shapes and geometrical primitives.
//!

use crate::mathcore::{
    floats::{almost_equal, FloatOperations},
    matrices::Matrix3x3,
    transforms::{Rotatable, Scalable, Transform, Transformable, Translatable},
    vectors::{Point, Vector2, Vertex},
    {Angle, Sign, Size},
};
use serde::{Deserialize, Serialize};

/// [`Segment`] struct represents two-dimensional line segment.
///
/// This struct is not an implementor of [`Shape`] traits because most of associated functions make
/// no sense for line segment (e.g. `perimeter` and `area` from [`Shape`], `scale` and `set_size` from [`Scalable`]).
/// Transform traits that are implemented ([`Translatable`] and [`Rotatable`]) supply comments on
/// what is considered origin and angle of a line segment.
///
/// `Segment.point1` is considered as base, so that the slope is defined as
/// `self.point2 - self.point1`.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// First point of segment.
    ///
    pub point1: Point,
    /// Second point of segment.
    ///
    pub point2: Point,
}
impl Segment {
    /// Returns length of a segment.
    ///
    pub fn length(&self) -> f32 {
        self.slope().magnitude()
    }

    /// Returns slope of a segment.
    ///
    pub fn slope(&self) -> Vector2 {
        self.point2 - self.point1
    }
    /// Returns `k` coefficient of a line that contains this segment.
    ///
    /// `k` stands for a gradient or a tangent of inclination angle of a line or a derivative from its equation ->
    /// `y = kx + b`, `k = tg(a) = dy/dx`.
    /// For vertical lines `k` equals `+inf`/`-inf` depending on direction of a segment.
    ///
    pub fn k(&self) -> f32 {
        let slope: Vector2 = self.slope().correct_to(0);
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
        self.point2.y - self.k() * self.point2.x
    }

    /// Returns point at which two segments intersect.
    /// If lines are collinear (either parallel or coincident), `None` is returned.
    ///
    pub fn intersection(self, other: Segment) -> Option<Point> {
        let (s1, s2): (Vector2, Vector2) = (self.slope(), other.slope());
        let tails: Vector2 = self.point1 - other.point1;

        let d: f32 = s1.cross_product(s2);
        if almost_equal(d, 0.0) {
            if self.point1 == other.point1 || self.point1 == other.point2 {
                return Some(self.point1);
            } else if self.point2 == other.point1 || self.point2 == other.point2 {
                return Some(self.point2);
            }
            return None;
        }

        let s: f32 = s1.cross_product(tails);
        let t: f32 = s2.cross_product(tails);

        let (s_sign, t_sign, d_sign): (Sign, Sign, Sign) =
            (Sign::from(s), Sign::from(t), Sign::from(d));
        if s_sign == d_sign
            && t_sign == d_sign
            && match d_sign {
                Sign::Positive => s <= d && t <= d,
                Sign::Negative => s >= d && t >= d,
                Sign::Zero => unreachable!("Zero case is already checked out"),
            }
        {
            let t: f32 = t / d;
            Some(self.point1 + s1 * t)
        } else {
            None
        }
    }
}
impl FloatOperations for Segment {
    fn correct_to(self, digits: i32) -> Self {
        Segment {
            point1: self.point1.correct_to(digits),
            point2: self.point2.correct_to(digits),
        }
    }

    fn round_up_to(self, digits: i32) -> Self {
        Segment {
            point1: self.point1.round_up_to(digits),
            point2: self.point2.round_up_to(digits),
        }
    }
}
impl Translatable for Segment {
    /// For a line segment, origin is a midpoint.
    ///
    fn origin(&self) -> Point {
        (self.point1 + self.point2) * 0.5
    }

    fn translate_on(&mut self, vector: Vector2) {
        self.point1 += vector;
        self.point2 += vector;
    }
}
impl Rotatable for Segment {
    /// For a line segment, angle is inclination angle of a line that contains line segment.
    ///
    fn angle(&self) -> Angle {
        Angle::from_radians(self.k().atan())
    }

    fn rotate_on(&mut self, angle: Angle) {
        let origin: Vector2 = self.origin();
        let transform_matrix: Matrix3x3 = Transform::combine(
            [
                Transform::Translation { vector: -origin },
                Transform::Rotation { angle },
                Transform::Translation { vector: origin },
            ]
            .into_iter(),
        );
        self.point1 = transform_matrix.apply_to(self.point1);
        self.point2 = transform_matrix.apply_to(self.point2);
    }
}

/// [`Shape`] trait defines two-dimensional shape on a plane which can be transformed.
///
pub trait Shape: Transformable {
    /// Returns perimeter of a shape.
    ///
    fn perimeter(&self) -> f32;
    /// Returns total surface area of a shape.
    ///
    fn area(&self) -> f32;

    /// Returns whether shape contains point or not. Point that lies on the edge or shape's border is considered lying inside shape.
    ///
    fn contains_point(&self, point: Point) -> bool;
}
/// [`PolygonLike`] trait defines shapes that can be represented by a list of vertices.
///
pub trait PolygonLike: Shape {
    /// Returns shared slice with polygon's vertices.
    ///
    fn vertices(&self) -> &[Vertex];

    /// Returns `Vec` with polygon's edges.
    ///
    /// Capacity and length of `self.edges()` is guaranteed to be equal to `self.vertices().len()`.
    ///
    fn edges(&self) -> Vec<Segment> {
        let vertices: &[Vertex] = self.vertices();
        let n: usize = vertices.len();

        let mut edges: Vec<Segment> = Vec::with_capacity(n);
        for i in 0..n {
            edges.push(Segment {
                point1: vertices[i],
                point2: vertices[(i + 1) % n],
            });
        }
        edges
    }
}
/// Implements `Shape::contains_point` method for struct that implements `PolygonLike` trait.
///
macro_rules! impl_contains_point_for_polygonlike {
    () => {
        /// Returns whether polygon contains point or not. Polygon contains point even in cases where point lies on its edge.
        ///
        fn contains_point(&self, point: Point) -> bool {
            let between: fn(f32, f32, f32) -> bool =
                |p: f32, a: f32, b: f32| p >= a.min(b) && p <= a.max(b);
            let mut inside: bool = false;

            for edge in self.edges() {
                let Segment {
                    point1: a,
                    point2: b,
                } = edge;
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
                    let c: f32 = (a - point).cross_product(b - point);
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
}
/// [`Convex`] mark trait defines polygons which are convex (every internal angle is strictly less than 180 degrees).
///
pub trait Convex: PolygonLike {}

/// [`Rect`] struct represents transformable two-dimensional rectangle on a surface.
///
/// # Examples
/// ### Initialization
/// ```rust
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonLike};
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::{Angle, Size};
/// let mut rect: Rect = Rect::from_origin(
///     Point::zero(),
///     Angle::zero(),
///     Size::try_from(3.0).expect("Value is in correct range."), Size::try_from(2.0).expect("Value is in correct range.")
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
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonLike};
/// # use ggengine::mathcore::transforms::Translatable;
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::{Angle, Size};
/// let mut rect: Rect = Rect::from_origin(
///     Point::zero(),
///     Angle::zero(),
///     Size::try_from(3.0).expect("Value is in correct range."), Size::try_from(2.0).expect("Value is in correct range.")
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
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonLike};
/// # use ggengine::mathcore::transforms::Rotatable;
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::{Angle, Size};
/// # use ggengine::mathcore::floats::FloatOperations;
/// let mut rect: Rect = Rect::from_origin(
///     Point { x: 1.5, y: 1.0 },
///     Angle::zero(),
///     Size::try_from(3.0).expect("Value is in correct range."), Size::try_from(2.0).expect("Value is in correct range.")
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
/// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonLike};
/// # use ggengine::mathcore::transforms::Scalable;
/// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
/// # use ggengine::mathcore::{Angle, Size};
/// let mut rect: Rect = Rect::from_origin(
///     Point { x: 1.5, y: 1.0 },
///     Angle::zero(),
///     Size::try_from(3.0).expect("Value is in correct range."),
///     Size::try_from(2.0).expect("Value is in correct range.")
/// );
/// rect.scale((
///     Size::try_from(2.0).expect("Value is in correct range."),
///     Size::try_from(2.0).expect("Value is in correct range.")
/// ));
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

    /// Origin of a rectangle (center point).
    ///
    origin: Point,
    /// Angle at which rectangle is currently rotated.
    ///
    angle: Angle,
    /// Tuple of rectangle's width and height.
    ///
    size: (Size, Size),
}
impl Rect {
    /// Returns width of a rectangle.
    ///
    pub fn width(&self) -> f32 {
        self.size.0.get()
    }
    /// Returns height of a rectangle.
    ///
    pub fn height(&self) -> f32 {
        self.size.1.get()
    }

    /// Constructs rectangle with given origin, angle and size.
    ///
    pub fn from_origin(origin: Point, angle: Angle, width: Size, height: Size) -> Self {
        let size: (Size, Size) = (width, height);

        let model: [Vertex; 4] = [
            Vertex { x: -0.5, y: 0.5 },
            Vertex { x: 0.5, y: 0.5 },
            Vertex { x: 0.5, y: -0.5 },
            Vertex { x: -0.5, y: -0.5 },
        ];
        let transform_matrix: Matrix3x3 = Transform::combine(
            [
                Transform::Scaling { size_scale: size },
                Transform::Rotation { angle },
                Transform::Translation { vector: origin },
            ]
            .into_iter(),
        );
        let vertices: [Vertex; 4] = model.map(|vertex| transform_matrix.apply_to(vertex));

        Rect {
            vertices,

            origin,
            angle,
            size,
        }
    }

    /// Returns array of two corner points of axis-aligned bounding box that contains rectangle.
    ///
    /// First point is `(min_x, min_y)` and the second one is `(max_x, max_y)`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::shapes::{Rect, Shape, PolygonLike};
    /// # use ggengine::mathcore::transforms::{Rotatable};
    /// # use ggengine::mathcore::vectors::{Vector2, Vertex, Point};
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// # use ggengine::mathcore::{Angle, Size};
    /// let rect: Rect = Rect::from_origin(
    ///     Point::zero(),
    ///     Angle::from_degrees(45.0),
    ///     Size::try_from(2.0).expect("Value is in correct range."),
    ///     Size::try_from(2.0).expect("Value is in correct range.")
    /// );
    /// assert_eq!(
    ///     rect.aabb().round_up_to(1),
    ///     [Point { x: -1.4, y: -1.4 }, Point { x: 1.4, y: 1.4 }] // sqrt(2)
    /// );
    /// ```
    ///
    pub fn aabb(self) -> [Point; 2] {
        let (mut min_x, mut max_x, mut min_y, mut max_y): (f32, f32, f32, f32) = (
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
        );
        for vertex in self.vertices {
            min_x = min_x.min(vertex.x);
            max_x = max_x.max(vertex.x);
            min_y = min_y.min(vertex.y);
            max_y = max_y.max(vertex.y);
        }
        [Point { x: min_x, y: min_y }, Point { x: max_x, y: max_y }]
    }
}
impl Shape for Rect {
    fn perimeter(&self) -> f32 {
        2.0 * (self.width() + self.height())
    }

    fn area(&self) -> f32 {
        self.width() * self.height()
    }

    impl_contains_point_for_polygonlike!();
}
impl PolygonLike for Rect {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}
impl Convex for Rect {}
impl Translatable for Rect {
    fn origin(&self) -> Point {
        self.origin
    }

    fn translate_on(&mut self, vector: Vector2) {
        self.origin += vector;

        self.vertices
            .iter_mut()
            .for_each(|vertex| *vertex += vector);
    }
}
impl Rotatable for Rect {
    fn angle(&self) -> Angle {
        self.angle
    }

    fn rotate_on(&mut self, angle: Angle) {
        self.angle = angle;

        let transform_matrix: Matrix3x3 = Transform::combine(
            [
                Transform::Translation {
                    vector: -self.origin,
                },
                Transform::Rotation { angle },
                Transform::Translation {
                    vector: self.origin,
                },
            ]
            .into_iter(),
        );
        self.vertices
            .iter_mut()
            .for_each(|vertex| *vertex = transform_matrix.apply_to(*vertex));
    }
}
impl Scalable for Rect {
    fn size(&self) -> (Size, Size) {
        self.size
    }

    fn scale(&mut self, size_scale: (Size, Size)) {
        self.size.0 *= size_scale.0;
        self.size.1 *= size_scale.1;

        let transform_matrix: Matrix3x3 = Transform::combine(
            [
                Transform::Translation {
                    vector: -self.origin,
                },
                Transform::Scaling { size_scale },
                Transform::Translation {
                    vector: self.origin,
                },
            ]
            .into_iter(),
        );
        self.vertices
            .iter_mut()
            .for_each(|vertex| *vertex = transform_matrix.apply_to(*vertex));
    }
}

#[cfg(test)]
mod tests {
    use crate::mathcore::{
        shapes::Segment,
        transforms::{Rotatable, Translatable},
        vectors::{Point, Vector2},
        Angle,
    };

    #[test]
    fn line_segment2d() {
        use super::Segment;

        let mut line1: Segment = Segment {
            point1: Point { x: 0.0, y: 0.0 },
            point2: Point { x: 4.0, y: 4.0 },
        };
        assert_eq!(line1.length(), 4.0 * 2.0_f32.sqrt());

        let mut line2: Segment = Segment {
            point1: Point { x: 0.0, y: 6.0 },
            point2: Point { x: 3.0, y: 0.0 },
        };

        assert_eq!((line1.k(), line1.b()), (1.0, 0.0));
        assert_eq!((line2.k(), line2.b()), (-2.0, 6.0));

        assert_eq!(line1.intersection(line2).unwrap(), Point { x: 2.0, y: 2.0 });

        line1.translate_on(Vector2 { x: -2.0, y: -2.0 });
        line1.rotate_on(Angle::from_degrees(45.0));
        line2 = Segment {
            point1: Point { x: 0.0, y: 0.0 },
            point2: Point { x: 0.0, y: 4.0 },
        };

        assert_eq!(line1.k(), f32::INFINITY);
        assert!(line1.intersection(line2).is_none());

        line1 = Segment {
            point1: Point { x: -1.0, y: 2.0 },
            point2: Point { x: 1.0, y: 2.0 },
        };
        line2 = Segment {
            point1: Point { x: 1.0, y: 2.0 },
            point2: Point { x: 2.0, y: 2.0 },
        };
        assert!(line1.intersection(line2).is_some());
    }

    #[test]
    fn rect2d() {
        use super::{PolygonLike, Rect};
        use crate::mathcore::{transforms::Scalable, Size};

        let mut rect1: Rect = Rect::from_origin(
            Point { x: 1.0, y: 1.0 },
            Angle::zero(),
            Size::try_from(3.0).expect("Value is in correct range."),
            Size::try_from(2.0).expect("Value is in correct range."),
        );

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
                Segment {
                    point1: Point { x: -0.5, y: 2.0 },
                    point2: Point { x: 2.5, y: 2.0 }
                },
                Segment {
                    point1: Point { x: 2.5, y: 2.0 },
                    point2: Point { x: 2.5, y: 0.0 }
                },
                Segment {
                    point1: Point { x: 2.5, y: 0.0 },
                    point2: Point { x: -0.5, y: 0.0 }
                },
                Segment {
                    point1: Point { x: -0.5, y: 0.0 },
                    point2: Point { x: -0.5, y: 2.0 }
                },
            ]
        );

        let mut rect2: Rect = Rect::from_origin(
            Point { x: 3.0, y: 3.0 },
            Angle::zero(),
            Size::try_from(3.0).expect("Value is in correct range."),
            Size::try_from(2.0).expect("Value is in correct range."),
        );

        // translation
        rect1.translate_on(Vector2 { x: 1.0, y: 1.0 });
        rect2.translate_to(Point { x: 2.0, y: 2.0 });
        assert_eq!(rect1.vertices(), rect2.vertices());

        // rotation
        rect1.rotate_on(Angle::from_degrees(270.0));
        rect2.rotate_to(Angle::from_degrees(-90.0));
        assert_eq!(rect1.vertices(), rect2.vertices());

        // scaling
        rect1.scale((
            Size::try_from(3.0).expect("Value is in correct range."),
            Size::try_from(3.0).expect("Value is in correct range."),
        ));
        rect2.set_size((
            Size::try_from(9.0).expect("Value is in correct range."),
            Size::try_from(6.0).expect("Value is in correct range."),
        ));
        assert_eq!(rect1.vertices(), rect2.vertices());
    }
}
