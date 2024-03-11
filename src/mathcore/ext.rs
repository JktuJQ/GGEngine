//! `mathcore::ext` hidden submodule supplies helper newtypes, enums, structs and etc. that are
//! used throughout `ggengine` crate.
//!

use crate::mathcore::floats::{equal, FloatOperations};
use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, TAU},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// [`Sign`] unit-only enum represents value's sign (value can be negative, positive or be equal to zero).
///
/// `From` implementations take sign from given value.
///
/// # Example
/// ```rust
/// # use ggengine::mathcore::Sign;
/// let mut sign: Sign = Sign::Positive;
/// sign = -sign;
/// assert_eq!(sign, Sign::Negative * Sign::Positive);
/// assert_eq!(1 * (sign as i8), -1);
/// ```
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    /// Negative sign (-1).
    ///
    Negative = -1,
    /// Zero (0).
    ///
    Zero = 0,
    /// Positive sign (+1).
    ///
    Positive = 1,
}
impl Neg for Sign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Negative => Self::Positive,
            Self::Zero => Self::Zero,
            Self::Positive => Self::Negative,
        }
    }
}
impl Mul<Self> for Sign {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Positive, Self::Positive) | (Self::Negative, Self::Negative) => Self::Positive,
            (Self::Positive, Self::Negative) | (Self::Negative, Self::Positive) => Self::Negative,
            _ => Self::Zero,
        }
    }
}
impl MulAssign<Self> for Sign {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
/// Implements `From` trait for `Sign`.
///
macro_rules! impl_sign_from {
    (i $(($t:ty, $zero:literal),)+) => {$(
        impl From<$t> for Sign {
            fn from(value: $t) -> Self {
                if value == $zero || -value == $zero {
                    Self::Zero
                }
                else if value.is_positive() {
                    Self::Positive
                }
                else {
                    Self::Negative
                }
            }
        }
    )+};

    (u $(($t:ty, $zero:literal),)+) => {$(
        impl From<$t> for Sign {
            fn from(value: $t) -> Self {
                if value == $zero {
                    Self::Zero
                }
                else {
                    Self::Positive
                }
            }
        }
    )+};

    (f $(($t:ty, $zero:literal),)+) => {$(
        impl From<$t> for Sign {
            fn from(value: $t) -> Self {
                if value == $zero || -value == $zero {
                    Self::Zero
                }
                else if value.is_sign_positive() {
                    Self::Positive
                }
                else {
                    Self::Negative
                }
            }
        }
    )+};
}
impl_sign_from!(i(i8, 0), (i16, 0), (i32, 0), (i64, 0), (i128, 0),);
impl_sign_from!(u(u8, 0), (u16, 0), (u32, 0), (u64, 0), (u128, 0),);
impl_sign_from!(f(f32, 0.0), (f64, 0.0),);

/// [`Angle`] is a newtype that restricts angle values to [0.0; TAU).
/// If given value is not finite, 0.0 will be set as angle value.
///
/// Underlying value is stored in radians, so it is the most precise mode.
///
/// # Example
/// ```rust
/// # use ggengine::mathcore::Angle;
/// # use std::f32::consts::FRAC_PI_2;
/// let angle: Angle = Angle::from_radians(-FRAC_PI_2);
/// assert_eq!(angle, Angle::from_degrees(270.0));
/// assert_eq!(angle.degrees(), 270.0);
/// assert_eq!(angle.radians(), 3.0 * FRAC_PI_2);
/// ```
///
#[derive(Copy, Clone, Debug, Default, PartialOrd)]
pub struct Angle(f32);
impl Angle {
    /// Angle that corresponds to zero.
    ///
    pub const ZERO: Angle = Angle(0.0);
    /// Angle that corresponds to 30 degree angle.
    ///
    pub const DEG30: Angle = Angle(FRAC_PI_6);
    /// Angle that corresponds to 45 degree angle.
    ///
    pub const DEG45: Angle = Angle(FRAC_PI_4);
    /// Angle that corresponds to 60 degree angle.
    ///
    pub const DEG60: Angle = Angle(FRAC_PI_3);

    /// Angle that corresponds to 90 degree angle.
    ///
    pub const DEG90: Angle = Angle(1.0 * FRAC_PI_2);
    /// Angle that corresponds to 120 degree angle.
    ///
    pub const DEG120: Angle = Angle(1.0 * FRAC_PI_2 + FRAC_PI_6);
    /// Angle that corresponds to 135 degree angle.
    ///
    pub const DEG135: Angle = Angle(1.0 * FRAC_PI_2 + FRAC_PI_4);
    /// Angle that corresponds to 150 degree angle.
    ///
    pub const DEG150: Angle = Angle(1.0 * FRAC_PI_2 + FRAC_PI_3);

    /// Angle that corresponds to 180 degree angle.
    ///
    pub const DEG180: Angle = Angle(2.0 * FRAC_PI_2);
    /// Angle that corresponds to 210 degree angle.
    ///
    pub const DEG210: Angle = Angle(2.0 * FRAC_PI_2 + FRAC_PI_6);
    /// Angle that corresponds to 225 degree angle.
    ///
    pub const DEG225: Angle = Angle(2.0 * FRAC_PI_2 + FRAC_PI_4);
    /// Angle that corresponds to 240 degree angle.
    ///
    pub const DEG240: Angle = Angle(2.0 * FRAC_PI_2 + FRAC_PI_3);

    /// Angle that corresponds to 270 degree angle.
    ///
    pub const DEG270: Angle = Angle(3.0 * FRAC_PI_2);
    /// Angle that corresponds to 300 degree angle.
    ///
    pub const DEG300: Angle = Angle(3.0 * FRAC_PI_2 + FRAC_PI_6);
    /// Angle that corresponds to 315 degree angle.
    ///
    pub const DEG315: Angle = Angle(3.0 * FRAC_PI_2 + FRAC_PI_4);
    /// Angle that corresponds to 330 degree angle.
    ///
    pub const DEG330: Angle = Angle(3.0 * FRAC_PI_2 + FRAC_PI_3);
    /// Angle that corresponds to 360 degree angle
    /// (since angles are restricted, it equals to zero angle).
    ///
    pub const DEG360: Angle = Angle(0.0);

    /// Normalizes given angle (in radians) to [0.0; 2 * PI).
    ///
    fn normalize(angle: f32) -> f32 {
        if angle.is_finite() {
            angle - ((angle / TAU).floor() * TAU)
        } else {
            0.0
        }
    }

    /// Returns angle value in radians.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// # use std::f32::consts::FRAC_PI_2;
    /// let angle: Angle = Angle::from_radians(FRAC_PI_2);
    /// assert_eq!(angle.radians(), FRAC_PI_2);
    /// ```
    ///
    pub fn radians(&self) -> f32 {
        self.0
    }
    /// Returns angle value in degrees.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// # use std::f32::consts::FRAC_PI_2;
    /// let angle: Angle = Angle::from_radians(FRAC_PI_2);
    /// assert_eq!(angle.degrees(), 90.0);
    /// ```
    ///
    pub fn degrees(&self) -> f32 {
        self.0.to_degrees()
    }

    /// Initializes zeroed angle.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// let angle: Angle = Angle::zero();
    /// assert_eq!(angle.radians(), 0.0);
    /// ```
    ///
    pub const fn zero() -> Self {
        Self::ZERO
    }
    /// Initializes angle from radians.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// # use std::f32::consts::FRAC_PI_2;
    /// let angle: Angle = Angle::from_radians(FRAC_PI_2);
    /// ```
    ///
    pub fn from_radians(radians: f32) -> Self {
        Angle(Self::normalize(radians))
    }
    /// Initializes angle from degrees.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// # use std::f32::consts::FRAC_PI_2;
    /// let angle: Angle = Angle::from_degrees(90.0);
    /// assert_eq!(angle.radians(), FRAC_PI_2);
    /// ```
    ///
    pub fn from_degrees(degrees: f32) -> Self {
        Angle::from_radians(degrees.to_radians())
    }

    /// Returns sine of angle.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let angle: Angle = Angle::from_degrees(90.0);
    /// assert_eq!(angle.sin().correct(0), 1.0);
    /// ```
    ///
    pub fn sin(&self) -> f32 {
        self.0.sin()
    }
    /// Returns cosine of angle.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// # use ggengine::mathcore::floats::FloatOperations;
    /// let angle: Angle = Angle::from_degrees(90.0);
    /// assert_eq!(angle.cos().correct(0), 0.0);
    /// ```
    ///
    pub fn cos(&self) -> f32 {
        self.0.cos()
    }
    /// Returns sine and cosine of angle packed in tuple.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Angle;
    /// let angle: Angle = Angle::from_degrees(90.0);
    /// assert_eq!(angle.sin_cos(), (angle.sin(), angle.cos()));
    /// ```
    ///
    pub fn sin_cos(&self) -> (f32, f32) {
        self.0.sin_cos()
    }
}
impl FloatOperations for Angle {
    fn correct(self, digits: i32) -> Self {
        Angle(self.0.correct(digits))
    }

    fn round_up_to(self, digits: i32) -> Self {
        Angle(self.0.round_up_to(digits))
    }
}
impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Angle::from_radians(-self.0)
    }
}
impl Add<Self> for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.0 + rhs.0)
    }
}
impl Sub<Self> for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.0 - rhs.0)
    }
}
impl Mul<f32> for Angle {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from_radians(self.0 * rhs)
    }
}
impl Div<f32> for Angle {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::from_radians(self.0 / rhs)
    }
}
impl AddAssign<Self> for Angle {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign<Self> for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl MulAssign<f32> for Angle {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}
impl DivAssign<f32> for Angle {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        equal(self.0, other.0)
    }
}
impl Eq for Angle {}

/// [`Size`] is a newtype that restricts size's value to (0.0; +inf).
/// If given value is not finite or equal to zero, 1.0 will be set as size value.
///
/// # Example
/// ```rust
/// # use ggengine::mathcore::Size;
/// assert_eq!(Size::from_value(-10.0).get(), 10.0);
/// assert_eq!(Size::from_value(0.0).get(), 1.0);
/// assert_eq!(Size::from_value(0.1).get(), 0.1);
/// ```
///
#[derive(Copy, Clone, Debug, PartialOrd)]
pub struct Size(f32);
impl Size {
    /// Normalizes given size to (0.0; +inf).
    ///
    fn normalize(size: f32) -> f32 {
        if !size.is_finite() || size == 0.0 || size == -0.0 {
            return 1.0;
        }
        size.abs()
    }

    /// Initializes [`Size`] from `f32` value
    pub fn from_value(value: f32) -> Self {
        Size(Self::normalize(value))
    }

    /// Returns size value.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::mathcore::Size;
    /// let size: Size = Size::from_value(-10.0);
    /// assert_eq!(size.get(), 10.0);
    /// ```
    ///
    pub fn get(&self) -> f32 {
        self.0
    }
}
impl FloatOperations for Size {
    fn correct(self, digits: i32) -> Self {
        Size::from_value(self.0.correct(digits))
    }

    fn round_up_to(self, digits: i32) -> Self {
        Size::from_value(self.0.round_up_to(digits))
    }
}
impl Add<Self> for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Size::from_value(self.0 + rhs.0)
    }
}
impl Sub<Self> for Size {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Size::from_value(self.0 - rhs.0)
    }
}
impl Mul<Self> for Size {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Size::from_value(self.0 * rhs.0)
    }
}
impl Div<Self> for Size {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Size::from_value(self.0 / rhs.0)
    }
}
impl AddAssign<Self> for Size {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl SubAssign<Self> for Size {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
impl MulAssign<Self> for Size {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
impl DivAssign<Self> for Size {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}
impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        equal(self.0, other.0)
    }
}
impl Eq for Size {}

/// [`Color`] struct represents RGBA model of color.
///
/// # Examples
/// ```rust
/// # use ggengine::mathcore::Color;
/// let color: Color = Color { r: 1, g: 2, b: 3, a: 4 };
/// assert_eq!(Color::RED, Color { r: 255, g: 0, b: 0, a: 255 });
/// assert_eq!(Color::GREEN, Color { r: 0, g: 255, b: 0, a: 255 });
/// assert_eq!(Color::BLUE, Color { r: 0, g: 0, b: 255, a: 255 });
/// ```
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red component of color.
    ///
    pub r: u8,

    /// Green component of color.
    ///
    pub g: u8,

    /// Blue component of color.
    ///
    pub b: u8,

    /// Alpha channel value of color.
    ///
    pub a: u8,
}

impl Color {
    /// Color that corresponds to white.
    ///
    pub const WHITE: Self = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Color that corresponds to black.
    ///
    pub const BLACK: Self = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Color that corresponds to red.
    ///
    pub const RED: Self = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Color that corresponds to green.
    ///
    pub const GREEN: Self = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Color that corresponds to blue.
    ///
    pub const BLUE: Self = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    /// Color that corresponds to yellow.
    ///
    pub const YELLOW: Self = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Color that corresponds to cyan.
    ///
    pub const CYAN: Self = Color {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Color that corresponds to magenta.
    ///
    pub const MAGENTA: Self = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };

    /// Performs hue angle conversion into exact values of red, green and blue.
    ///
    fn rgb_from_hue(hue: Angle, x: f32, c: f32, m: f32) -> (u8, u8, u8) {
        let (r, g, b): (f32, f32, f32) = if Angle::ZERO <= hue && hue < Angle::DEG60 {
            (c, x, 0.0)
        } else if Angle::DEG60 <= hue && hue < Angle::DEG60 * 2.0 {
            (x, c, 0.0)
        } else if Angle::DEG60 * 2.0 <= hue && hue < Angle::DEG60 * 3.0 {
            (0.0, c, x)
        } else if Angle::DEG60 * 3.0 <= hue && hue < Angle::DEG60 * 4.0 {
            (0.0, x, c)
        } else if Angle::DEG60 * 4.0 <= hue && hue < Angle::DEG60 * 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        (
            (255.0 * (r + m)).round() as u8,
            (255.0 * (g + m)).round() as u8,
            (255.0 * (b + m)).round() as u8,
        )
    }
    /// Performs RGB conversion to hue angle value.
    ///
    fn hue_from_rgb(min: f32, max: f32, r: f32, g: f32, b: f32) -> (f32, Angle) {
        let d = max - min;
        let h: Angle = if d != 0.0 {
            if max == r {
                Angle::DEG60 * (((g - b) / d) % 6.0)
            } else if max == g {
                Angle::DEG60 * (((b - r) / d) + 2.0)
            } else {
                Angle::DEG60 * (((r - g) / d) + 4.0)
            }
        } else {
            Angle::from_radians(0.0)
        };
        (d, h)
    }

    /// Initializes `Color` from RGBA model.
    ///
    /// Alias for manual construction of struct.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::Color;
    /// assert_eq!(Color::from_rgba(1, 2, 3, 4), Color { r: 1, g: 2, b: 3, a: 4 });
    /// ```
    ///
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    /// Initializes red, green and blue components of color performing HSV to RGB conversion.
    ///
    /// `s` and `v` should be in [0.0, 1.0] range or else they would be clamped to that range.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::{Color, Angle};
    /// assert_eq!(
    ///     Color::from_hvsa(Angle::from_degrees(0.0), 0.0, 0.0, 255),
    ///     Color::from_rgba(0, 0, 0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_hvsa(Angle::from_degrees(0.0), 0.0, 1.0, 255),
    ///     Color::from_rgba(255, 255, 255, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_hvsa(Angle::from_degrees(0.0), 1.0, 1.0, 255),
    ///     Color::from_rgba(255, 0, 0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_hvsa(Angle::from_degrees(120.0), 1.0, 1.0, 255),
    ///     Color::from_rgba(0, 255, 0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_hvsa(Angle::from_degrees(240.0), 1.0, 1.0, 255),
    ///     Color::from_rgba(0, 0, 255, 255)
    /// );
    /// ```
    ///
    pub fn from_hvsa(h: Angle, s: f32, v: f32, a: u8) -> Self {
        let (s, v): (f32, f32) = (s.clamp(0.0, 1.0), v.clamp(0.0, 1.0));

        let c: f32 = s * v;
        let x: f32 = c * (1.0 - ((h.degrees() / 60.0) % 2.0 - 1.0).abs());
        let m: f32 = v - c;

        let (r, g, b): (u8, u8, u8) = Color::rgb_from_hue(h, x, c, m);

        Self { r, g, b, a }
    }
    /// Initializes red, green and blue components of color performing HSL to RGB conversion.
    ///
    /// `s` and `l` should be in [0.0, 1.0] range or else they would be clamped to that range.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::{Color, Angle};
    /// assert_eq!(Color::from_hsla(Angle::from_degrees(0.0), 0.0, 0.0, 255), Color::from_rgba(0, 0, 0, 255));
    /// assert_eq!(Color::from_hsla(Angle::from_degrees(0.0), 0.0, 1.0, 255), Color::from_rgba(255, 255, 255, 255));
    /// assert_eq!(Color::from_hsla(Angle::from_degrees(0.0), 1.0, 0.5, 255), Color::from_rgba(255, 0, 0, 255));
    /// assert_eq!(Color::from_hsla(Angle::from_degrees(120.0), 1.0, 0.5, 255), Color::from_rgba(0, 255, 0, 255));
    /// assert_eq!(Color::from_hsla(Angle::from_degrees(240.0), 1.0, 0.5, 255), Color::from_rgba(0, 0, 255, 255));
    /// ```
    ///
    pub fn from_hsla(h: Angle, s: f32, l: f32, a: u8) -> Self {
        let (s, l): (f32, f32) = (s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));

        let c: f32 = s * (1.0 - (2.0 * l - 1.0).abs());
        let x: f32 = c * (1.0 - ((h.degrees() / 60.0) % 2.0 - 1.0).abs());
        let m: f32 = l - c / 2.0;

        let (r, g, b): (u8, u8, u8) = Color::rgb_from_hue(h, x, c, m);

        Self { r, g, b, a }
    }

    /// Returns tuple of red, green, blue and alpha values.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::Color;
    /// assert_eq!(Color::from_rgba(1, 2, 3, 4).to_rgba(), (1, 2, 3, 4));
    /// ```
    ///
    pub fn to_rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
    /// Returns tuple of hue, saturation, value and alpha channel that corresponds to this color.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::{Color, Angle};
    /// assert_eq!(
    ///     Color::from_rgba(0, 0, 0, 255).to_hsva(),
    ///     (Angle::from_degrees(0.0), 0.0, 0.0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(255, 255, 255, 255).to_hsva(),
    ///     (Angle::from_degrees(0.0), 0.0, 1.0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(255, 0, 0, 255).to_hsva(),
    ///     (Angle::from_degrees(0.0), 1.0, 1.0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(0, 255, 0, 255).to_hsva(),
    ///     (Angle::from_degrees(120.0), 1.0, 1.0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(0, 0, 255, 255).to_hsva(),
    ///     (Angle::from_degrees(240.0), 1.0, 1.0, 255)
    /// );
    /// ```
    ///
    pub fn to_hsva(self) -> (Angle, f32, f32, u8) {
        let (r, g, b): (f32, f32, f32) = (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        );

        let (min, max): (f32, f32) = (r.min(g.min(b)), r.max(g.max(b)));
        let (d, h): (f32, Angle) = Color::hue_from_rgb(min, max, r, g, b);
        let (s, v) = (if max == 0.0 { 0.0 } else { d / max }, max);

        (h, s, v, self.a)
    }
    /// Returns tuple of hue, saturation, value and alpha channel that corresponds to this color.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::mathcore::{Color, Angle};
    /// assert_eq!(
    ///     Color::from_rgba(0, 0, 0, 255).to_hsla(),
    ///     (Angle::from_degrees(0.0), 0.0, 0.0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(255, 255, 255, 255).to_hsla(),
    ///     (Angle::from_degrees(0.0), 0.0, 1.0, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(255, 0, 0, 255).to_hsla(),
    ///     (Angle::from_degrees(0.0), 1.0, 0.5, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(0, 255, 0, 255).to_hsla(),
    ///     (Angle::from_degrees(120.0), 1.0, 0.5, 255)
    /// );
    /// assert_eq!(
    ///     Color::from_rgba(0, 0, 255, 255).to_hsla(),
    ///     (Angle::from_degrees(240.0), 1.0, 0.5, 255)
    /// );
    /// ```
    ///
    pub fn to_hsla(self) -> (Angle, f32, f32, u8) {
        let (r, g, b): (f32, f32, f32) = (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        );

        let (min, max): (f32, f32) = (r.min(g.min(b)), r.max(g.max(b)));
        let (d, h): (f32, Angle) = Color::hue_from_rgb(min, max, r, g, b);
        let l: f32 = (max + min) / 2.0;
        let s: f32 = if d == 0.0 {
            0.0
        } else {
            d / (1.0 - (2.0 * l - 1.0).abs())
        };

        (h, s, l, self.a)
    }
}
