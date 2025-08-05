//! `mathcore::transforms` submodule implements enums and functions which use transformation matrices to
//! perform translation, rotation, scaling, reflection operations on objects. This module
//! also defines traits that provide transforming interfaces to objects.
//!

use crate::mathcore::{
    matrices::Matrix3x3,
    vectors::{Point, Vector2},
    Angle,
};
use serde::{Deserialize, Serialize};

/// [`Transformation`] enum represents 3 basic affine transformations.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Transformation {
    /// Translation moves an object along given vector.
    ///
    Translation(Vector2),

    /// Rotation operation rotates the original object's coordinate system for the given angle.
    ///
    Rotation(Angle),

    /// Scaling transform changes the size of an object by expanding or contracting all vertices
    /// along axes by given scalar values.
    ///
    Scaling(Vector2),
}
impl Transformation {
    /// Constructs corresponding transformation matrix by using values.
    ///
    /// # Examples
    /// ### Translation
    /// ```rust
    /// # use ggengine::mathcore::transforms::Transformation;
    /// # use ggengine::mathcore::matrices::{Matrix3x1, Matrix3x3};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// let transformation: Transformation = Transformation::Translation(Vector2 { x: 2.0, y: 3.0 });
    /// let matrix: Matrix3x3 = transformation.matrix();
    /// assert_eq!(matrix.as_array(),
    ///     [[1.0, 0.0, 2.0],
    ///      [0.0, 1.0, 3.0],
    ///      [0.0, 0.0, 1.0]]
    /// );
    /// let point: Vector2 = Vector2 { x: 0.0, y: 2.0 };
    /// let transformed: Vector2 = matrix.apply_to(point);
    /// assert_eq!(transformed, Vector2 { x: 2.0, y: 5.0 });  // x' = x1 + x2
    ///                                                       // y' = y1 + y2
    /// ```
    ///
    /// ### Rotation
    /// ```rust
    /// # use ggengine::mathcore::transforms::Transformation;
    /// # use ggengine::mathcore::matrices::{Matrix3x1, Matrix3x3};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// # use ggengine::mathcore::Angle;
    /// let transformation: Transformation = Transformation::Rotation(Angle::from_degrees(90.0));
    /// let matrix: Matrix3x3 = transformation.matrix().round_up_to(2);
    /// assert_eq!(matrix.as_array(),
    ///     [[0.0, -1.0, 0.0],
    ///      [1.0, 0.0, 0.0],
    ///      [0.0, 0.0, 1.0]]
    /// );
    /// let point: Vector2 = Vector2 { x: 0.0, y: 2.0 };
    /// let transformed: Vector2 = matrix.apply_to(point);
    /// assert_eq!(transformed, Vector2 { x: -2.0, y: 0.0 });  // x' = x * cos(angle) - y * sin(angle)
    ///                                                        // y' = x * sin(angle) + y * cos(angle)
    /// ```
    ///
    /// ### Scaling
    /// ```rust
    /// # use ggengine::mathcore::transforms::Transformation;
    /// # use ggengine::mathcore::matrices::{Matrix3x1, Matrix3x3};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let transformation: Transformation = Transformation::Scaling(Vector2 { x: 3.0, y: 2.0 });
    /// let matrix: Matrix3x3 = transformation.matrix();
    /// assert_eq!(matrix.as_array(),
    ///     [[3.0, 0.0, 0.0],
    ///      [0.0, 2.0, 0.0],
    ///      [0.0, 0.0, 1.0]]
    /// );
    /// let point: Vector2 = Vector2 { x: 2.0, y: 2.0 };
    /// let transformed: Vector2 = matrix.apply_to(point);
    /// assert_eq!(transformed, Vector2 { x: 6.0, y: 4.0 });  // x' = x * width_scale
    ///                                                       // y' = y * height_scale
    /// ```
    ///
    pub fn matrix(self) -> Matrix3x3 {
        let mut matrix = Matrix3x3::identity();
        match self {
            Self::Translation(vector) => {
                matrix[0][2] = vector.x;
                matrix[1][2] = vector.y;
            }
            Self::Rotation(angle) => {
                let (sin, cos) = angle.sin_cos();
                matrix[0][0] = cos;
                matrix[0][1] = -sin;
                matrix[1][0] = sin;
                matrix[1][1] = cos;
            }
            Self::Scaling(scale) => {
                matrix[0][0] = scale.x;
                matrix[1][1] = scale.y;
            }
        };
        matrix
    }

    /// Combines given transforms by using dot product.
    ///
    /// This function automatically reverses the order, so if you need to combine transforms `A -> B -> C`
    /// just pass them in that order (matrices multiplication will be performed in order `C * B * A`).
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::{Angle, floats::FloatOperations, vectors::Vector2, transforms::Transformation, matrices::Matrix3x3};
    /// let rotation: Transformation = Transformation::Rotation(Angle::DEG90);
    /// let translation: Transformation = Transformation::Translation(Vector2 { x: 3.0, y: 2.0 });
    /// let scale: Transformation = Transformation::Scaling(Vector2 { x: 2.0, y: 2.0 });
    /// assert_eq!(Transformation::combine([rotation, translation, scale]).correct_to(0).as_array(),
    /// [
    ///     [-0.0, -2.0, 6.0],
    ///     [2.0, -0.0, 4.0],
    ///     [0.0, 0.0, 1.0]
    /// ]);  // rotation -> translation -> scaling
    /// ```
    ///
    pub fn combine(
        transforms: impl IntoIterator<
            Item = Transformation,
            IntoIter = impl DoubleEndedIterator<Item = Transformation>,
        >,
    ) -> Matrix3x3 {
        transforms
            .into_iter()
            .rev()
            .fold(Matrix3x3::identity(), |acc, transform| {
                acc * transform.matrix()
            })
    }
}

/// [`Translate`] trait defines properties of translatable objects
/// (objects that can be moved across plane).
///
pub trait Translate {
    /// Returns origin point (position of object).
    ///
    fn origin(&self) -> Point;

    /// Translates object by a given vector.
    ///
    fn translate_on(&mut self, vector: Vector2);
    /// Translates object to a given point in place.
    ///
    fn translate_to(&mut self, point: Point) {
        self.translate_on(point - self.origin());
    }
}
/// [`Rotate`] trait defines properties of rotating objects.
///
/// Rotation should be performed on counterclockwise direction (`Transform::ROTATION` matrix supplies it),
/// although on screen it would appear as clockwise (since y-axis is directed down). That suggests
/// that implementation of this trait should be using `Transform::ROTATION` matrix to be uniform relating to other objects.
///
pub trait Rotate {
    /// Returns current angle.
    ///
    fn angle(&self) -> Angle;

    /// Rotates object by a given angle counting from current rotation.
    ///
    fn rotate_on(&mut self, angle: Angle);
    /// Rotates object by a given angle counting from zero rotation (from zero).
    ///
    fn rotate_to(&mut self, angle: Angle) {
        self.rotate_on(angle - self.angle());
    }
}
/// [`Scale`] trait defines properties of scalable objects (objects that can be resized).
///
pub trait Scale {
    /// Scales object's size by a factor of `scale`.
    ///
    /// Although scaling could be negative (that would be a reflection),
    /// most of the structs in `ggengine` do not support that,
    /// because that would interfere with other invariants.
    /// Those structs should explicitly mention that in the docs.
    ///
    fn scale(&mut self, scale: Vector2);
}
/// [`Transform`] trait defines objects that support every `ggengine` affine transformation.
///
pub trait Transform: Translate + Rotate + Scale {
    /// Performs all transformations in order.
    ///
    fn transform(&mut self, transformations: &[Transformation]) {
        for transformation in transformations {
            match transformation {
                Transformation::Translation(vector) => self.translate_on(*vector),
                Transformation::Rotation(angle) => self.rotate_on(*angle),
                Transformation::Scaling(scale) => self.scale(*scale),
            }
        }
    }
}
