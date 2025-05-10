//! `mathcore::floats` submodule implements several consts, functions and traits that help in
//! work with `f32` type.
//!
//! [`almost_equal`] function and [`EPSILON`] const are dealing with floating point equality.
//!
//! [`FloatOperations`] trait and [`CLOSE_TO_ZERO`], [`CLOSE_TO_ONE`] consts are dealing with
//! distortions that may be caused by float operations.
//!

/// Constant that is used in floating point equality.
///
/// It represents amount of difference that is allowed for two `f32` values to still be considered
/// equal.
///
pub const EPSILON: f32 = 0.00001;
/// This function implements floating point equality for `ggengine` crate.
///
/// It is used for implementing `PartialEq` on types that are based on float.
///
/// # Example
/// ```rust
/// # use ggengine::mathcore::floats::almost_equal;
/// assert!(almost_equal(0.15 + 0.15, 0.1 + 0.2));
/// ```
///
pub fn almost_equal(a: f32, b: f32) -> bool {
    if a == b {
        return true;
    }

    let diff = (a - b).abs();
    let norm = (a.abs() + b.abs()).min(f32::MAX);
    diff < (norm * EPSILON).max(f32::MIN)
}

/// Constant that is used in floating point correction.
///
/// It defines the threshold for number to be considered small enough to then be floored.
///
pub const CLOSE_TO_ZERO: f32 = 0.0001;
/// Constant that is used in floating point correction.
///
/// It defines the threshold for number to be considered big enough to then be ceiled.
///
pub const CLOSE_TO_ONE: f32 = 0.9999;
/// [`FloatOperations`] trait defines `correct` and `round_up_to` associated functions that work
/// with floating point values.
///
pub trait FloatOperations {
    /// Corrects distortions that may be caused by float operations.
    ///
    /// For example, this function fixes such things as -0.0 into 0.0,
    /// 0.0001 (anything that is less than `CLOSE_TO_ZERO`) into 0.0 and
    /// 0.9999 (anything that is greater than `CLOSE_TO_ONE`) into 1.0.
    ///
    fn correct_to(self, digits: i32) -> Self;

    /// Rounds to given amount of digits after floating point.
    ///
    /// Passing negative number shifts floating point to the left.
    ///
    fn round_up_to(self, digits: i32) -> Self;
}
impl FloatOperations for f32 {
    /// Corrects distortions that may be caused by float operations.
    ///
    /// For example, this function fixes such things as -0.0 into 0.0,
    /// 0.0001 (anything that is less than `CLOSE_TO_ZERO`) into 0.0 and
    /// 0.9999 (anything that is greater than `CLOSE_TO_ONE`) into 1.0.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// assert_eq!(-0.0_f32.correct_to(0), 0.0);
    /// assert_eq!(0.00009_f32.correct_to(0), 0.0);
    /// assert_eq!(0.99999_f32.correct_to(0), 1.0);
    ///
    /// assert_eq!(0.200009_f32.correct_to(1), 0.2);
    /// assert_eq!(2.00009_f32.correct_to(0) / 10.0, 0.2);
    ///
    /// assert_eq!(-0.199999_f32.correct_to(1), -0.2);
    /// assert_eq!(-1.99999_f32.correct_to(0) / 10.0, -0.2);
    ///
    /// assert_eq!(300009.0_f32.correct_to(-5), 300000.0);
    /// ```
    ///
    fn correct_to(self, digits: i32) -> Self {
        let mul = 10_f32.powi(digits);

        let n = self * mul;

        if n == -0.0 {
            return 0.0;
        }

        let fract = n.abs().fract();
        if !(CLOSE_TO_ZERO..=CLOSE_TO_ONE).contains(&fract) {
            return n.round() / mul;
        }

        n / mul
    }

    /// Rounds to given amount of digits after floating point.
    ///
    /// Passing negative number shifts floating point to the left.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// assert_eq!(12.345_f32.round_up_to(2), 12.35);
    /// assert_eq!(12.345_f32.round_up_to(-1), 10.0);
    /// assert_eq!(12.345_f32.round_up_to(10), 12.345);
    /// ```
    ///
    fn round_up_to(self, digits: i32) -> Self {
        let mul = 10_f32.powi(digits);
        (self * mul).round() / mul
    }
}
impl<T: FloatOperations, const N: usize> FloatOperations for [T; N] {
    fn correct_to(self, digits: i32) -> Self {
        self.map(|elem| elem.correct_to(digits))
    }

    fn round_up_to(self, digits: i32) -> Self {
        self.map(|elem| elem.round_up_to(digits))
    }
}
