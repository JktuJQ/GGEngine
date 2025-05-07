//! `mathcore::vectors` submodule implements N-dimensional vectors on a plane which can be used to represent
//! force, speed, acceleration and other things.
//!

use crate::mathcore::{
    floats::{almost_equal, FloatOperations},
    Angle,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, BitXor, Mul, MulAssign, Neg, Sub, SubAssign},
};

// Macro that implement all common associated functions and methods on vectors could be replaced
// with trait and default implementation, but there are severity of architectural flaws
// (with generic consts it would be possible to implement those traits multiple times, with associated
// constants it would be impossible to implement until `generic_const_exprs` hits stable). So
// currently all of this is handled by macro.
//
/// [`impl_vector`] macro implements all common associated functions and methods on vectors.
///
macro_rules! impl_vector {
    ($struct:ident, $type:ty, $zero:literal, $one:literal) => {
        impl $struct {
            /// Initializes vector with zeroes.
            ///
            pub fn zero() -> Self {
                Self { x: $zero, y: $zero }
            }
            /// Initializes vector with ones.
            ///
            pub fn one() -> Self {
                Self { x: $one, y: $one }
            }

            /// Converts vector to array.
            ///
            pub fn as_array(self) -> [$type; 2] {
                [self.x, self.y]
            }

            /// Applies function to every vector element and returns changed vector.
            ///
            /// Allows to perform custom operations on each vector element.
            ///
            pub fn map(self, f: impl Fn($type) -> $type) -> Self {
                Self {
                    x: f(self.x),
                    y: f(self.y),
                }
            }
            /// Combines vectors by applying function on their as_array.
            ///
            /// Allows performing operations with 2 vectors.
            ///
            pub fn combine(self, other: Self, f: impl Fn($type, $type) -> $type) -> Self {
                Self {
                    x: f(self.x, other.x),
                    y: f(self.y, other.y),
                }
            }

            /// Performs dot product operation on two vectors.
            ///
            pub fn dot_product(self, other: Self) -> $type {
                self.x * other.x + self.y * other.y
            }
            /// Returns scalar that represents cross product of two-dimensional vectors.
            ///
            pub fn cross_product(self, other: Self) -> $type {
                (self.x * other.y) - (self.y * other.x)
            }

            /// Returns squared magnitude of a vector.
            ///
            pub fn sqr_magnitude(&self) -> $type {
                self.dot_product(*self)
            }
        }
    };
}

/// [`Vector2`] struct represents two-dimensional vector and two-dimensional point with `f32` coordinates on a plane.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Vector2 {
    /// X component of vector.
    ///
    pub x: f32,
    /// Y component of vector.
    ///
    pub y: f32,
}
/// Type alias for [`Vector2`].
///
pub type Point = Vector2;
/// Type alias for [`Vector2`].
///
pub type Vertex = Point;
impl_vector!(Vector2, f32, 0.0, 1.0);
impl Vector2 {
    /// Returns angle between two vectors.
    ///
    pub fn angle(self, other: Self) -> Angle {
        Angle::from_radians(
            (self.dot_product(other) / (self.magnitude() * other.magnitude())).acos(),
        )
    }

    /// Returns magnitude of vector.
    ///
    pub fn magnitude(&self) -> f32 {
        self.sqr_magnitude().sqrt()
    }

    /// Returns new vector that is normalized.
    ///
    pub fn normalized(self) -> Self {
        self * (1.0 / self.magnitude())
    }
    /// Returns new vector, which magnitude is clamped to max_magnitude.
    ///
    pub fn clamped_magnitude(self, max_magnitude: f32) -> Self {
        let magnitude: f32 = self.magnitude();
        self * (magnitude.min(max_magnitude) / magnitude)
    }

    /// Linearly interpolates between vectors a and b by t.
    ///
    /// t will be clamped between [0.0; 1.0].
    ///
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t: f32 = t.clamp(0.0, 1.0);
        self * t + other * (1.0 - t)
    }
}
impl Neg for Vector2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.map(|a| -a)
    }
}
impl Add<Vector2> for Vector2 {
    type Output = Self;

    fn add(self, rhs: Vector2) -> Self::Output {
        self.combine(rhs, |a, b| a + b)
    }
}
impl Sub<Vector2> for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Vector2) -> Self::Output {
        self.combine(rhs, |a, b| a - b)
    }
}
impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: Vector2) {
        *self = *self + rhs;
    }
}
impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, rhs: Vector2) {
        *self = *self - rhs;
    }
}
impl Mul<f32> for Vector2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|a| a * rhs)
    }
}
impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}
impl FloatOperations for Vector2 {
    /// Constructs new vector by correcting every vector component that may be wronged by float operations.
    ///
    fn correct_to(self, digits: i32) -> Self {
        self.map(|elem| elem.correct_to(digits))
    }
    /// Constructs new vector by rounding every vector component to given amount of digits after floating point.
    ///
    fn round_up_to(self, digits: i32) -> Self {
        self.map(|elem| elem.round_up_to(digits))
    }
}
impl PartialEq for Vector2 {
    fn eq(&self, other: &Self) -> bool {
        almost_equal(self.x, other.x) && almost_equal(self.y, other.y)
    }
}
impl Eq for Vector2 {}
impl BitXor for Vector2 {
    type Output = Angle;

    /// Alias for `Vector2::angle`.
    ///
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.angle(rhs)
    }
}

/// [`Vector2Int`] struct represents two-dimensional vector and two-dimensional point with `i32` coordinates on a plane.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Vector2Int {
    /// X component of vector.
    ///
    pub x: i32,
    /// Y component of vector.
    ///
    pub y: i32,
}
/// Type alias for [`Vector2Int`].
///
pub type PointInt = Vector2Int;
/// Type alias for [`Vector2Int`].
///
pub type VertexInt = PointInt;
impl_vector!(Vector2Int, i32, 0, 1);
impl Neg for Vector2Int {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.map(|a| -a)
    }
}
impl Add<Vector2Int> for Vector2Int {
    type Output = Self;

    fn add(self, rhs: Vector2Int) -> Self::Output {
        self.combine(rhs, |a, b| a + b)
    }
}
impl Sub<Vector2Int> for Vector2Int {
    type Output = Self;

    fn sub(self, rhs: Vector2Int) -> Self::Output {
        self.combine(rhs, |a, b| a - b)
    }
}
impl AddAssign<Vector2Int> for Vector2Int {
    fn add_assign(&mut self, rhs: Vector2Int) {
        *self = *self + rhs;
    }
}
impl SubAssign<Vector2Int> for Vector2Int {
    fn sub_assign(&mut self, rhs: Vector2Int) {
        *self = *self - rhs;
    }
}
impl Mul<i32> for Vector2Int {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        self.map(|a| a * rhs)
    }
}
impl MulAssign<i32> for Vector2Int {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs;
    }
}
impl PartialEq for Vector2Int {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for Vector2Int {}

impl From<Vector2Int> for Vector2 {
    fn from(vec_i: Vector2Int) -> Self {
        Self {
            x: vec_i.x as f32,
            y: vec_i.y as f32,
        }
    }
}
impl From<Vector2> for Vector2Int {
    fn from(vec_f: Vector2) -> Self {
        Self {
            x: vec_f.x as i32,
            y: vec_f.y as i32,
        }
    }
}
