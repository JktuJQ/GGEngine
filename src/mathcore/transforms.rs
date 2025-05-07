//! `mathcore::transforms` submodule implements enums and functions which use transformation matrices to
//! perform translation, rotation, scaling, reflection operations on objects. This module
//! also defines traits that provide transforming interfaces to objects.
//!

use crate::mathcore::{
    matrices::Matrix3x3,
    vectors::{Point, Vector2},
    {Angle, Size},
};
use serde::{Deserialize, Serialize};

/// [`Transform`] struct-like enum represents 3 basic matrix transformations.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Transform {
    /// Translation moves an object along given vector.
    ///
    Translation {
        /// Vector along which object will be translated.
        ///
        vector: Vector2,
    },

    /// Rotation operation rotates the original object's coordinate system for the given angle.
    ///
    Rotation {
        /// Angle for which coordinate system will be rotated.
        ///
        angle: Angle,
    },

    /// Scaling transform changes the size of an object by expanding or contracting all vertices
    /// along axes by given scalar values.
    ///
    Scaling {
        /// Size scaling factor.
        ///
        size_scale: (Size, Size),
    },
}
impl Transform {
    /// Constructs corresponding transformation matrix by using values.
    ///
    /// # Examples
    /// ### Translation
    /// ```rust
    /// # use ggengine::mathcore::transforms::Transform;
    /// # use ggengine::mathcore::matrices::{Matrix3x1, Matrix3x3};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// let transform: Transform = Transform::Translation { vector: Vector2 { x: 2.0, y: 3.0 } };
    /// let matrix: Matrix3x3 = transform.matrix();
    /// assert_eq!(matrix.as_array(),
    ///     [[1.0, 0.0, 2.0],
    ///      [0.0, 1.0, 3.0],
    ///      [0.0, 0.0, 1.0]]
    /// );
    /// let point: Vector2 = Vector2 { x: 0.0, y: 2.0 };
    /// let transformed: Vector2 = matrix.apply_to(point);
    /// assert_eq!(transformed, Vector2 { x: 2.0, y: 5.0 });  // x' = x1 + x2
    ///                                                      // y' = y1 + y2
    /// ```
    ///
    /// ### Rotation
    /// ```rust
    /// # use ggengine::mathcore::transforms::Transform;
    /// # use ggengine::mathcore::matrices::{Matrix3x1, Matrix3x3};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// # use ggengine::mathcore::Angle;
    /// let transform: Transform = Transform::Rotation { angle: Angle::from_degrees(90.0) };
    /// let matrix: Matrix3x3 = transform.matrix().round_up_to(2);
    /// assert_eq!(matrix.as_array(),
    ///     [[0.0, -1.0, 0.0],
    ///      [1.0, 0.0, 0.0],
    ///      [0.0, 0.0, 1.0]]
    /// );
    /// let point: Vector2 = Vector2 { x: 0.0, y: 2.0 };
    /// let transformed: Vector2 = matrix.apply_to(point);
    /// assert_eq!(transformed, Vector2 { x: -2.0, y: 0.0 });  // x' = x * cos(angle) - y * sin(angle)
    ///                                                       // y' = x * sin(angle) + y * cos(angle)
    /// ```
    ///
    /// ### Scaling
    /// ```rust
    /// # use ggengine::mathcore::transforms::Transform;
    /// # use ggengine::mathcore::matrices::{Matrix3x1, Matrix3x3};
    /// # use ggengine::mathcore::vectors::Vector2;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// # use ggengine::mathcore::Size;
    /// let transform: Transform = Transform::Scaling {
    ///     size_scale: (
    ///         Size::try_from(3.0).expect("Value is in correct range."),
    ///         Size::try_from(2.0).expect("Value is in correct range.")
    ///     )
    /// };
    /// let matrix: Matrix3x3 = transform.matrix();
    /// assert_eq!(matrix.as_array(),
    ///     [[3.0, 0.0, 0.0],
    ///      [0.0, 2.0, 0.0],
    ///      [0.0, 0.0, 1.0]]
    /// );
    /// let point: Vector2 = Vector2 { x: 2.0, y: 2.0 };
    /// let transformed: Vector2 = matrix.apply_to(point);
    /// assert_eq!(transformed, Vector2 { x: 6.0, y: 4.0 });  // x' = x * width_scale
    ///                                                      // y' = y * height_scale
    /// ```
    ///
    pub fn matrix(self) -> Matrix3x3 {
        let mut matrix: Matrix3x3 = Matrix3x3::identity();
        match self {
            Self::Translation { vector } => {
                matrix[0][2] = vector.x;
                matrix[1][2] = vector.y;
            }
            Self::Rotation { angle } => {
                let (sin, cos): (f32, f32) = angle.sin_cos();
                matrix[0][0] = cos;
                matrix[0][1] = -sin;
                matrix[1][0] = sin;
                matrix[1][1] = cos;
            }
            Self::Scaling { size_scale } => {
                matrix[0][0] = size_scale.0.get();
                matrix[1][1] = size_scale.1.get();
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
    /// # use ggengine::mathcore::{Angle, Size, floats::FloatOperations, vectors::Vector2, transforms::Transform, matrices::Matrix3x3};
    /// let rotation: Transform = Transform::Rotation { angle: Angle::DEG90 };
    /// let translation: Transform = Transform::Translation { vector: Vector2 { x: 3.0, y: 2.0 } };
    ///
    /// let size_scale: Size = Size::try_from(2.0).expect("Value is in correct range.");
    /// let scale: Transform = Transform::Scaling { size_scale: (size_scale, size_scale) };
    /// assert_eq!(Transform::combine([rotation, translation, scale].into_iter()).correct_to(0).as_array(),
    /// [
    ///     [-0.0, -2.0, 6.0],
    ///     [2.0, -0.0, 4.0],
    ///     [0.0, 0.0, 1.0]
    /// ]);  // rotation -> translation -> scaling
    /// ```
    ///
    pub fn combine(transforms: impl DoubleEndedIterator<Item = Transform>) -> Matrix3x3 {
        transforms
            .rev()
            .fold(Matrix3x3::identity(), |acc, transform| {
                acc * transform.matrix()
            })
    }
}

/// [`Translatable`] trait defines properties of translatable objects (objects that can be moved
/// across plane).
///
pub trait Translatable {
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
/// [`Rotatable`] trait defines properties of rotating objects.
///
/// Rotation should be performed on counterclockwise direction (`Transform::ROTATION` matrix supplies it),
/// although on screen it would appear as clockwise (since y-axis is directed down). That suggests
/// that implementation of this trait should be using `Transform::ROTATION` matrix to be uniform relating to other objects.
///
pub trait Rotatable {
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
/// [`Scalable`] trait defines properties of scalable objects (objects that can be resized).
///
pub trait Scalable {
    /// Returns current size.
    ///
    fn size(&self) -> (Size, Size);

    /// Scales object's size by a factor of `size_scale`.
    ///
    fn scale(&mut self, size_scale: (Size, Size));
    /// Sets object's size to given values.
    ///
    fn set_size(&mut self, size: (Size, Size)) {
        let (x, y): (Size, Size) = self.size();
        self.scale((size.0 / x, size.1 / y));
    }
}
/// [`Transformable`] super-trait defines properties of transformable object.
///
/// This trait requires [`Translatable`], [`Rotatable`] and [`Scalable`] traits to be implemented.
/// This trait is automatically implemented if possible.
///
pub trait Transformable: Translatable + Rotatable + Scalable {}
impl<T: Translatable + Rotatable + Scalable> Transformable for T {}
