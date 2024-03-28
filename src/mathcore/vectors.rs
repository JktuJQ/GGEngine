//! `mathcore::vectors` submodule implements N-dimensional vectors on a plane which can be used to represent
//! force, speed, acceleration and other things.
//!

use crate::mathcore::{
    floats::{equal, FloatOperations},
    Angle,
};
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, BitXor, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

// Macros that implement all common associated functions and methods on vectors could be replaced
// by traits and their default implementations, but there are severity of architectural flaws
// (with generic consts it would be possible to implement those traits multiple times, with associated
// constants it would be impossible to implement until `generic_const_exprs` hits stable). So
// currently all of this is handled by macros.
//
/// [`impl_vector`] macro implements all common associated functions and methods on vectors.
///
/// This macro depends on manual implementation of `elements` and `set` functions and
/// `From<[$type; $size]>` trait implementation. All vector elements must be integral values.
///
macro_rules! impl_vector {
    ($struct:ident, $size:expr, $type:ty, $zero:literal, $one:literal) => {
        impl $struct {
            /// Initializes vector with zeroes.
            ///
            pub fn zero() -> Self {
                Self::from([$zero; $size])
            }
            /// Initializes vector with ones.
            ///
            pub fn one() -> Self {
                Self::from([$one; $size])
            }

            /// Applies function to every vector element and returns changed vector.
            ///
            /// Allows to perform custom operations on each vector element.
            ///
            pub fn map(self, f: impl Fn($type) -> $type) -> Self {
                let mut elements: [$type; $size] = self.elements();
                elements.iter_mut().for_each(|elem| *elem = f(*elem));
                Self::from(elements)
            }
            /// Combines vectors by applying function on their elements.
            ///
            /// Allows performing operations with 2 vectors.
            ///
            pub fn combine(self, other: Self, f: impl Fn($type, $type) -> $type) -> Self {
                let (e1, e2): ([$type; $size], [$type; $size]) =
                    (self.elements(), other.elements());
                let mut elements: [$type; $size] = [$zero; $size];
                for i in 0..$size {
                    elements[i] = f(e1[i], e2[i]);
                }
                Self::from(elements)
            }

            /// Returns squared magnitude of a vector (vector length).
            ///
            pub fn sqr_magnitude(&self) -> $type {
                self.elements().iter().fold($zero, |acc, n| acc + *n * *n)
            }

            /// Returns vector that is made from the largest components of two vectors.
            ///
            pub fn max(self, other: Self) -> Self {
                self.combine(other, |a, b| a.max(b))
            }
            /// Returns vector that is made from the smallest components of two vectors.
            ///
            pub fn min(self, other: Self) -> Self {
                self.combine(other, |a, b| a.min(b))
            }

            /// Multiplies two vectors component-wise.
            ///
            pub fn scale(self, other: Self) -> Self {
                self.combine(other, |a, b| a * b)
            }

            /// Performs dot product operation on two vectors.
            ///
            pub fn dot_product(self, other: Self) -> $type {
                self.elements()
                    .iter()
                    .zip(other.elements().iter())
                    .fold($zero, |acc, (a, b)| acc + *a * *b)
            }
        }
    };
}

/// [`impl_vectorf`] macro implements all common associated functions and methods on vectors with
/// float values.
///
/// This macro depends on [`impl_vector`] macro.
///
macro_rules! impl_vectorf {
    ($struct:ident, $istruct:ident, $size:expr) => {
        impl $struct {
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
                self / self.magnitude()
            }
            /// Returns new vector, which magnitude is clamped to max_magnitude.
            ///
            pub fn clamped_magnitude(self, max_magnitude: f32) -> Self {
                let magnitude: f32 = self.magnitude();
                self * magnitude.min(max_magnitude) / magnitude
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
        impl FloatOperations for $struct {
            /// Constructs new vector by rounding every vector component to given amount of digits after floating point.
            ///
            fn round_up_to(self, digits: i32) -> Self {
                self.map(|elem| elem.round_up_to(digits))
            }
            /// Constructs new vector by correcting every vector component that may be wronged by float operations.
            ///
            fn correct(self, digits: i32) -> Self {
                self.map(|elem| elem.correct(digits))
            }
        }
        impl PartialEq for $struct {
            fn eq(&self, other: &Self) -> bool {
                self.elements()
                    .iter()
                    .zip(other.elements().iter())
                    .all(|(&a, &b)| equal(a, b))
            }
        }
        impl Eq for $struct {}
        impl From<$istruct> for $struct {
            fn from(value: $istruct) -> Self {
                $struct::from(value.elements().map(|elem| elem as f32))
            }
        }
    };
}
/// [`impl_vectori`] macro implements all common associated functions and methods on vectors with
/// integer values.
///
/// This macro depends on [`impl_vector`] macro.
///
macro_rules! impl_vectori {
    ($struct:ident, $fstruct:ident) => {
        impl PartialEq for $struct {
            fn eq(&self, other: &Self) -> bool {
                self.elements()
                    .iter()
                    .zip(other.elements().iter())
                    .all(|(&a, &b)| a == b)
            }
        }
        impl Eq for $struct {}
        impl From<$fstruct> for $struct {
            fn from(value: $fstruct) -> Self {
                $struct::from(value.elements().map(|elem| elem as i32))
            }
        }
    };
}

// All the following macros depend on [`impl_vector`] macro.
/// [`impl_vector_vector_operations`] macro implements vector-vector operations for vector.
///
macro_rules! impl_vector_vector_operations {
    ($struct_name:ident, $rhs:ty, ($(($trait:ident, $method:ident, $op:tt),)+)) => {$(
        impl $trait<$rhs> for $struct_name {
            type Output = Self;

            fn $method(self, rhs: $rhs) -> Self::Output {
                self.combine(rhs, |a, b| a $op b)
            }
        }
    )+}
}
/// [`impl_vector_rhs_operations`] macro implements vector-rhs operations for vector.
///
macro_rules! impl_vector_rhs_operations {
    ($struct_name:ident, $rhs:ty, ($(($trait:ident, $method:ident, $op:tt),)+)) => {$(
        impl $trait<$rhs> for $struct_name {
            type Output = Self;

            fn $method(self, rhs: $rhs) -> Self::Output {
                self.map(|a| a $op rhs)
            }
        }
    )+}
}
/// [`impl_vector_assignoperations`] macro implements `...Assign` trait for vector.
///
// This could've been integrated in `impl_vector_vector_operations` and
// `impl_vector_rhs_operations`
// macros but until `concat_idents` macro is in stable it is not possible.
macro_rules! impl_vector_assignoperations {
    ($struct_name:ident, $rhs:ty, ($(($trait:ident, $method:ident, $op:tt),)+)) => {$(
        impl $trait<$rhs> for $struct_name {
            fn $method(&mut self, rhs: $rhs) {
                *self = *self $op rhs;
            }
        }
    )+}
}
/// [`impl_vector_operations`] macro implements all operation traits for vector.
///
macro_rules! impl_vector_operations {
    ($struct_name:ident, $t:ty) => {
        impl Neg for $struct_name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                self.map(|a| -a)
            }
        }
        impl Mul for $struct_name {
            type Output = $t;

            /// Performs dot product operation on two vectors.
            ///
            fn mul(self, other: Self) -> Self::Output {
                self.dot_product(other)
            }
        }
        impl_vector_vector_operations!($struct_name, Self, ((Add, add, +),
                                                            (Sub, sub, -),));
        impl_vector_assignoperations!($struct_name, Self, ((AddAssign, add_assign, +),
                                                           (SubAssign, sub_assign, -),));
        impl_vector_rhs_operations!($struct_name, $t, ((Add, add, +),
                                                       (Sub, sub, -),
                                                       (Mul, mul, *),
                                                       (Div, div, /),));
        impl_vector_assignoperations!($struct_name, $t, ((AddAssign, add_assign, +),
                                                         (SubAssign, sub_assign, -),
                                                         (MulAssign, mul_assign, *),
                                                         (DivAssign, div_assign, /),));
    }
}

/// [`Vector2`] struct represents two-dimensional vector and two-dimensional point with `f32` coordinates on a plane.
///
#[derive(Copy, Clone, Debug)]
pub struct Vector2 {
    /// X component of vector.
    ///
    pub x: f32,

    /// Y component of vector.
    ///
    pub y: f32,
}
impl Vector2 {
    /// Returns elements of vector.
    ///
    pub fn elements(&self) -> [f32; 2] {
        [self.x, self.y]
    }
    /// Sets from values to elements of vector.
    ///
    pub fn set(&mut self, elements: [f32; 2]) {
        self.x = elements[0];
        self.y = elements[1];
    }

    /// Returns scalar that represents cross product of two-dimensional vectors.
    ///
    pub fn cross_product(self, other: Self) -> f32 {
        (self.x * other.y) - (self.y * other.x)
    }
}
impl_vector!(Vector2, 2, f32, 0.0, 1.0);
impl_vectorf!(Vector2, Vector2Int, 2);
impl_vector_operations!(Vector2, f32);
impl BitXor for Vector2 {
    type Output = f32;

    /// Returns scalar that represents cross product of two-dimensional vectors.
    ///
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.cross_product(rhs)
    }
}
impl From<[f32; 2]> for Vector2 {
    fn from(arr: [f32; 2]) -> Self {
        Vector2 {
            x: arr[0],
            y: arr[1],
        }
    }
}

/// Type alias for [`Vector2`].
///
pub type Point = Vector2;
/// Type alias for [`Vector2`].
///
pub type Vertex = Point;

/// [`Vector2Int`] struct represents two-dimensional vector and two-dimensional point with `i32` coordinates on a plane.
///
#[derive(Copy, Clone, Debug)]
pub struct Vector2Int {
    /// X component of vector.
    ///
    pub x: i32,

    /// Y component of vector.
    ///
    pub y: i32,
}
impl Vector2Int {
    /// Returns elements of vector.
    ///
    pub fn elements(&self) -> [i32; 2] {
        [self.x, self.y]
    }
    /// Sets from values to elements of vector.
    ///
    pub fn set(&mut self, elements: [i32; 2]) {
        self.x = elements[0];
        self.y = elements[1];
    }

    /// Returns scalar that represents cross product of two-dimensional vectors.
    ///
    pub fn cross_product(self, other: Self) -> i32 {
        (self.x * other.y) - (self.y * other.x)
    }
}
impl_vector!(Vector2Int, 2, i32, 0, 1);
impl_vectori!(Vector2Int, Vector2);
impl_vector_operations!(Vector2Int, i32);
impl BitXor for Vector2Int {
    type Output = i32;

    /// Returns scalar that represents cross product of two-dimensional vectors.
    ///
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.cross_product(rhs)
    }
}
impl From<[i32; 2]> for Vector2Int {
    fn from(arr: [i32; 2]) -> Self {
        Vector2Int {
            x: arr[0],
            y: arr[1],
        }
    }
}

/// Type alias for [`Vector2Int`].
///
pub type PointInt = Vector2Int;
/// Type alias for [`Vector2Int`].
///
pub type VertexInt = PointInt;

#[cfg(test)]
mod tests {
    use crate::mathcore::floats::FloatOperations;

    #[test]
    fn vector() {
        use super::{Vector2, Vector2Int};

        // Vector2
        assert_eq!(Vector2::zero(), Vector2 { x: 0.0, y: 0.0 });
        assert_eq!(Vector2::one(), Vector2 { x: 1.0, y: 1.0 });

        let vec1f: Vector2 = Vector2::from([3.0, 4.0]);
        assert_eq!(vec1f.elements(), [3.0, 4.0]);
        assert_eq!(vec1f.sqr_magnitude(), 25.0);

        let vec2f: Vector2 = vec1f.scale(Vector2::from([4.0, 3.0]));
        assert_eq!(vec2f.elements(), [12.0, 12.0]);

        let mut vec3f: Vector2 = Vector2::from([5.0, 2.0]).max(Vector2::from([3.0, 6.0]));
        assert_eq!(vec3f, Vector2::from([5.0, 6.0]));

        let mut vec4f: Vector2 = Vector2::from([5.0, 2.0]).min(Vector2::from([3.0, 6.0]));
        assert_eq!(vec4f, Vector2::from([3.0, 2.0]));

        assert_eq!(vec1f.map(|n| n * 2.0), Vector2::from([6.0, 8.0]));
        assert_eq!(
            vec1f.combine(vec2f, |n1, n2| n1 * n2),
            Vector2::from([36.0, 48.0])
        );

        assert_eq!(vec1f + vec2f, Vector2::from([15.0, 16.0]));
        assert_eq!(vec1f - vec2f, Vector2::from([-9.0, -8.0]));

        assert_eq!(vec1f + 2.0, Vector2::from([5.0, 6.0]));
        assert_eq!(vec1f - 3.0, Vector2::from([0.0, 1.0]));
        assert_eq!(vec2f * 2.0, Vector2::from([24.0, 24.0]));
        assert_eq!(vec2f / 3.0, Vector2::from([4.0, 4.0]));

        vec3f += vec1f;
        assert_eq!(vec3f.elements(), [8.0, 10.0]);
        vec4f -= vec2f;
        assert_eq!(vec4f.elements(), [-9.0, -10.0]);

        vec3f += 2.0;
        assert_eq!(vec3f.elements(), [10.0, 12.0]);
        vec3f -= 3.0;
        assert_eq!(vec3f.elements(), [7.0, 9.0]);
        vec4f *= 2.0;
        assert_eq!(vec4f.elements(), [-18.0, -20.0]);
        vec4f /= 4.0;
        assert_eq!(vec4f.elements(), [-4.5, -5.0]);

        assert_eq!(-vec4f, Vector2::from([4.5, 5.0]));

        assert_eq!(
            Vector2::from([3.0, 4.0]).dot_product(Vector2::from([5.0, 3.0])),
            27.0
        );

        // Vector2Int
        assert_eq!(Vector2Int::zero(), Vector2Int { x: 0, y: 0 });
        assert_eq!(Vector2Int::one(), Vector2Int { x: 1, y: 1 });

        let vec1i: Vector2Int = Vector2Int::from([3, 4]);
        assert_eq!(vec1i.elements(), [3, 4]);
        assert_eq!(vec1i.sqr_magnitude(), 25);

        let vec2i: Vector2Int = vec1i.scale(Vector2Int::from([4, 3]));
        assert_eq!(vec2i.elements(), [12, 12]);

        let mut vec3i: Vector2Int = Vector2Int::from([5, 2]).max(Vector2Int::from([3, 6]));
        assert_eq!(vec3i, Vector2Int::from([5, 6]));

        let mut vec4i: Vector2Int = Vector2Int::from([5, 2]).min(Vector2Int::from([3, 6]));
        assert_eq!(vec4i, Vector2Int::from([3, 2]));

        assert_eq!(vec1i.map(|n| n * 2), Vector2Int::from([6, 8]));
        assert_eq!(
            vec1i.combine(vec2i, |n1, n2| n1 * n2),
            Vector2Int::from([36, 48])
        );

        assert_eq!(vec1i + vec2i, Vector2Int::from([15, 16]));
        assert_eq!(vec1i - vec2i, Vector2Int::from([-9, -8]));

        assert_eq!(vec1i + 2, Vector2Int::from([5, 6]));
        assert_eq!(vec1i - 3, Vector2Int::from([0, 1]));
        assert_eq!(vec2i * 2, Vector2Int::from([24, 24]));
        assert_eq!(vec2i / 3, Vector2Int::from([4, 4]));

        vec3i += vec1i;
        assert_eq!(vec3i.elements(), [8, 10]);
        vec4i -= vec2i;
        assert_eq!(vec4i.elements(), [-9, -10]);

        vec3i += 2;
        assert_eq!(vec3i.elements(), [10, 12]);
        vec3i -= 3;
        assert_eq!(vec3i.elements(), [7, 9]);
        vec4i *= 2;
        assert_eq!(vec4i.elements(), [-18, -20]);
        vec4i /= 4;
        assert_eq!(vec4i.elements(), [-4, -5]);

        assert_eq!(-vec4i, Vector2Int::from([4, 5]));

        assert_eq!(
            Vector2Int::from([3, 4]).dot_product(Vector2Int::from([5, 3])),
            27
        );
    }

    #[test]
    fn vectorf() {
        use super::Vector2;

        let vec1: Vector2 = Vector2::from([3.0, 4.0]);
        assert_eq!(vec1.magnitude(), 5.0);

        let vec2: Vector2 = vec1;
        assert_eq!(vec2.normalized().elements(), [0.6, 0.8]);

        assert_eq!(vec2.clamped_magnitude(0.5).magnitude(), 0.5);

        let vec3: Vector2 = Vector2::from([3.0_f32.sqrt(), 3.3]).round_up_to(0);
        assert_eq!(vec3.elements(), [2.0, 3.0]);

        let vec4: Vector2 = Vector2::from([0.00001, 2.0]).correct(0);
        assert_eq!(vec4.elements(), [0.0, 2.0]);

        assert_eq!(
            Vector2::from([1.0, -2.0])
                .angle(Vector2::from([-2.0, 1.0]))
                .degrees()
                .round(),
            143.0
        );

        let vec5: Vector2 = Vector2::from([0.0, 2.0]).lerp(Vector2::from([2.0, 0.0]), 0.5);
        assert_eq!(vec5.sqr_magnitude(), 2.0);
    }

    #[test]
    fn vectori() {}

    #[test]
    fn vector2() {
        use super::{Vector2, Vector2Int};

        let vec1: Vector2 = Vector2::from([-3.0, 2.0]);
        let vec2: Vector2 = Vector2::from([1.0, 2.0]);
        assert_eq!(vec1.cross_product(vec2), -8.0);

        assert_eq!(
            Vector2::from(Vector2Int::from([2, 2])),
            Vector2::from([2.0, 2.0])
        );
    }

    #[test]
    fn vector2int() {
        use super::{Vector2, Vector2Int};

        let vec1: Vector2Int = Vector2Int::from([-3, 2]);
        let vec2: Vector2Int = Vector2Int::from([1, 2]);
        assert_eq!(vec1.cross_product(vec2), -8);

        assert_eq!(
            Vector2Int::from(Vector2::from([2.0, 2.0])),
            Vector2Int::from([2, 2])
        );
    }
}
