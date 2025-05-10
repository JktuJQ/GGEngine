//! `graphicscore::primitives` hidden submodule supplies helper newtypes, enums, structs etc. that are
//! used throughout `ggengine` crate.
//!

use crate::{datacore::images::Image, graphicscore::textures::Texture, mathcore::Color};
use sdl2::{pixels::Color as SdlColor, render::BlendMode as SdlBlendMode};

/// [`BlendingType`] enum lists blending modes.
///
/// Blending is applied whenever blendable objects are combine, for example for `Image`s blending
/// is applied when performing blitting and for `Texture`s blending is applied when those are copied on each other when rendering.
///
/// Variants of this enum will provide info on how exactly blending will be applied using
/// designations that `dst` (destination color) is of type `Color` as well as `src` (source color) and
/// that all color values vary from [0; 1] where 0 corresponds to `x = 0` and 1 to `x = 255` and all out-of-bounds values are clamped
/// (those implicit conversions is made for clarity of transformations).
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlendingType {
    /// No blending is applied.
    ///
    /// ###### Pixel transformations:
    /// `dst = src`;
    ///
    None,
    /// Alpha blending
    /// (mixture of the old pixel and the new one, based on how large the `a` value of the new pixel is).
    ///
    /// ###### Pixel transformations:
    /// `dst.r = (src.r * src.a) + (dst.r * (1 - src.a))`
    ///
    /// `dst.g = (src.g * src.a) + (dst.g * (1 - src.a))`
    ///
    /// `dst.b = (src.b * src.a) + (dst.b * (1 - src.a))`
    ///
    /// `dst.a = src.a + (dst.a * (1 - src.a))`
    ///
    Alpha,
    /// Additive blending
    /// (new pixel's color is scaled by the new pixel's `a` and then is added to the old pixel).
    ///
    /// This blending makes pixels only brighter which suites it to be the choice for bright particles.
    ///
    /// ###### Pixel transformations:
    /// `dst.r = (src.r * src.a) + dst.r`
    ///
    /// `dst.g = (src.g * src.a) + dst.g`
    ///
    /// `dst.b = (src.b * src.a) + dst.b`
    ///
    /// `dst.a = dst.a`
    ///
    Additive,
    /// Multiplicative blending
    /// (combination of alpha blending and modulative blending).
    ///
    /// ###### Pixel transformations:
    /// `dst.r = (src.r * dst.r) + (dst.r * (1 - src.a))`
    ///
    /// `dst.g = (src.g * dst.g) + (dst.g * (1 - src.a))`
    ///
    /// `dst.r = (src.b * dst.b) + (dst.b * (1 - src.a))`
    ///
    /// `dst.a = dst.a`
    ///
    Multiplicative,
    /// Modulative blending
    /// (new pixel's color is scaled by the new pixel's `a` and then is added to the old pixel).
    ///
    /// This blending makes pixels only darker which suites it to be the choice for dark particles.
    ///
    /// ###### Pixel transformations:
    /// `dst.r = src.r * dst.r`
    ///
    /// `dst.g = src.g * dst.g`
    ///
    /// `dst.b = src.b * dst.b`
    ///
    /// `dst.a = dst.a`
    ///
    Modulative,
}
impl BlendingType {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Converts `sdl2` SdlBlendMode to [`BlendingType`].
    ///
    pub(crate) fn from_sdl_blend_mode(blend_mode: SdlBlendMode) -> BlendingType {
        match blend_mode {
            SdlBlendMode::None => BlendingType::None,
            SdlBlendMode::Blend => BlendingType::Alpha,
            SdlBlendMode::Add => BlendingType::Additive,
            SdlBlendMode::Mul => BlendingType::Multiplicative,
            SdlBlendMode::Mod => BlendingType::Modulative,
            _ => unreachable!("Blending should be possible"),
        }
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns `sdl2` representation of this enum.
    ///
    pub(crate) fn to_sdl_blend_mode(self) -> SdlBlendMode {
        match self {
            BlendingType::None => SdlBlendMode::None,
            BlendingType::Alpha => SdlBlendMode::Blend,
            BlendingType::Additive => SdlBlendMode::Add,
            BlendingType::Multiplicative => SdlBlendMode::Mul,
            BlendingType::Modulative => SdlBlendMode::Mod,
        }
    }
}

/// [`Blendable`] trait is implemented on structs that support blending.
///
pub trait Blendable {
    /// Sets new blending mode for blendable object.
    ///
    fn set_blend_mode(&mut self, blend_mode: BlendingType);
    /// Returns blending type that is currently applied on this object.
    ///
    fn blend_mode(&self) -> BlendingType;
}
impl Blendable for Image<'_> {
    fn set_blend_mode(&mut self, blend_mode: BlendingType) {
        self.get_sdl_surface_mut()
            .set_blend_mode(blend_mode.to_sdl_blend_mode())
            .expect("Blending should not fail")
    }
    fn blend_mode(&self) -> BlendingType {
        BlendingType::from_sdl_blend_mode(self.get_sdl_surface().blend_mode())
    }
}
impl Blendable for Texture<'_> {
    fn set_blend_mode(&mut self, blend_mode: BlendingType) {
        self.get_sdl_texture_mut()
            .set_blend_mode(blend_mode.to_sdl_blend_mode())
    }
    fn blend_mode(&self) -> BlendingType {
        BlendingType::from_sdl_blend_mode(self.get_sdl_texture().blend_mode())
    }
}

/// [`ColorModulatable`] trait is implemented on structs that support color modulation.
///
/// Color modulation is similar to blending, but it is applied only to source pixel color (`src = src * (color / 255)`).
///
pub trait ColorModulatable {
    /// Sets new color modulation for modulatable objects.
    ///
    fn set_color_modulation(&mut self, color: Color);
    /// Returns color modulation that is currently applied on this object.
    ///
    fn color_modulation(&self) -> Color;
}
impl ColorModulatable for Image<'_> {
    fn set_color_modulation(&mut self, color: Color) {
        self.get_sdl_surface_mut()
            .set_color_mod(SdlColor::from(color.to_rgba()))
    }
    fn color_modulation(&self) -> Color {
        let (r, g, b, a) = self.get_sdl_surface().color_mod().rgba();
        Color::from_rgba(r, g, b, a)
    }
}
impl ColorModulatable for Texture<'_> {
    fn set_color_modulation(&mut self, color: Color) {
        let (r, g, b, a) = color.to_rgba();

        let texture = self.get_sdl_texture_mut();
        texture.set_color_mod(r, g, b);
        texture.set_alpha_mod(a);
    }
    fn color_modulation(&self) -> Color {
        let (r, g, b) = self.get_sdl_texture().color_mod();
        let a = self.get_sdl_texture().alpha_mod();
        Color::from_rgba(r, g, b, a)
    }
}
