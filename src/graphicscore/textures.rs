//! `graphicscore::textures` submodule implements structs that are related to texturing.
//!
//! `Texture`s are the essential part of whole `graphicscore`. They come from the `TextureCreator`s
//! to be bound by the render target's lifetime. Those can be tricky to manage, but `ggengine` does its
//! best to provide convenient interface.
//!
//! Since most of `graphicscore` is hierarchical, almost all structs are obtained from their parents.
//! To further understand relations between [`Texture`] and other structs, it is encouraged to read docs.
//!
//! #### `Texture` vs `Image`
//! `Image` is used for software rendering (it is bound to CPU and RAM). It is well suited for manual pixels tweaking.
//!
//! `Texture` is used for hardware rendering (it is bound to GPU and VRAM). It is very fast for the actual rendering,
//! so it should be considered as primary option.
//!
//! One advantage to `Image` is that you can directly write data to it, so if you want to have procedural generated
//! `Texture`s, you would want to use `Image` and then convert it to the actual `Texture` when rendering.
//!

use crate::datacore::images::{Image, PixelFormat};
use sdl2::{
    image::LoadTexture,
    render::{
        Texture as RenderTexture, TextureAccess as RenderTextureAccess,
        TextureCreator as RenderTextureCreator, TextureQuery as RenderTextureQuery,
    },
    surface::SurfaceContext,
    video::WindowContext,
};
use std::{
    fmt,
    io::{Error, ErrorKind},
    path::Path,
};

/// [`AccessType`] enum lists variants how texture can be accessed by the renderer.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AccessType {
    /// Static access modifier
    /// (texture changed rarely or does not change at all).
    ///
    Static,
    /// Streaming access modifier
    /// (texture changed frequently, so you're able to manually write data to it).
    ///
    Streaming,
    /// Target access modifier
    /// (texture is being targeted for rendering and post-processing).
    ///
    Targeted,
}
impl AccessType {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Converts `sdl2` RenderTextureAccess to [`AccessType`].
    ///
    fn from_sdl_texture_access(texture_access: RenderTextureAccess) -> AccessType {
        match texture_access {
            RenderTextureAccess::Static => AccessType::Static,
            RenderTextureAccess::Streaming => AccessType::Streaming,
            RenderTextureAccess::Target => AccessType::Targeted,
        }
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns `sdl2` representation of this enum.
    ///
    fn to_sdl_texture_access(self) -> RenderTextureAccess {
        match self {
            AccessType::Static => RenderTextureAccess::Static,
            AccessType::Streaming => RenderTextureAccess::Streaming,
            AccessType::Targeted => RenderTextureAccess::Target,
        }
    }
}

/// [`InnerTextureCreator`] enum gathers the only two possible exact type constraints for `RenderTextureCreator` to
/// encapsulate those away. That allows to get rid of the generic argument and get the lifetime specifier even
/// for `WindowContext` variant which does not need it.
///
enum InnerTextureCreator<'a> {
    /// Variant that handles `RenderTextureCreator` for the `SurfaceContext`.
    ///
    ForImage(RenderTextureCreator<SurfaceContext<'a>>),
    /// Variant that handles `RenderTextureCreator` for the `SurfaceContext`.
    ///
    ForWindow(RenderTextureCreator<WindowContext>),
}
impl<'a> InnerTextureCreator<'a> {
    /// Returns the best pixel format for `InnerTextureCreator` or `None`, if the format is not recognised by `ggengine`.
    ///
    /// Even if the format is not recognised, it is still usable by `ggengine`.
    ///
    fn default_pixel_format(&self) -> Option<PixelFormat> {
        PixelFormat::from_sdl_pixel_format_enum(match self {
            InnerTextureCreator::ForImage(texture_creator) => {
                texture_creator.default_pixel_format()
            }
            InnerTextureCreator::ForWindow(texture_creator) => {
                texture_creator.default_pixel_format()
            }
        })
    }

    /// Creates new texture with given size, format and access type.
    ///
    /// If given format is `None`, `InnerTextureCreator` will use the best pixel format for [`Texture`].
    ///
    fn create_texture(
        &self,
        width: u32,
        height: u32,
        format: Option<PixelFormat>,
        access_type: AccessType,
    ) -> Texture {
        Texture {
            texture: match self {
                InnerTextureCreator::ForImage(texture_creator) => texture_creator
                    .create_texture(
                        format.map(|pixel_format| pixel_format.to_sdl_pixel_format_enum()),
                        access_type.to_sdl_texture_access(),
                        width,
                        height,
                    )
                    .expect("Texture creation should not fail"),
                InnerTextureCreator::ForWindow(texture_creator) => texture_creator
                    .create_texture(
                        format.map(|pixel_format| pixel_format.to_sdl_pixel_format_enum()),
                        access_type.to_sdl_texture_access(),
                        width,
                        height,
                    )
                    .expect("Texture creation should not fail"),
            },
        }
    }
    /// Creates [`Texture`] from the [`Image`].
    ///
    fn create_texture_from_image(&self, image: &Image) -> Texture {
        Texture {
            texture: match self {
                InnerTextureCreator::ForImage(texture_creator) => texture_creator
                    .create_texture_from_surface(image.get_sdl_surface())
                    .expect("Texture creation should not fail"),
                InnerTextureCreator::ForWindow(texture_creator) => texture_creator
                    .create_texture_from_surface(image.get_sdl_surface())
                    .expect("Texture creation should not fail"),
            },
        }
    }
    /// Creates [`Texture`] from bytes of supported format ('.png', '.jpg', but not raw buffer).
    ///
    fn create_texture_from_bytes(&self, bytes: Box<[u8]>) -> Result<Texture, Error> {
        match self {
            InnerTextureCreator::ForImage(texture_creator) => {
                texture_creator.load_texture_bytes(&bytes)
            }
            InnerTextureCreator::ForWindow(texture_creator) => {
                texture_creator.load_texture_bytes(&bytes)
            }
        }
        .map(|texture| Texture { texture })
        .map_err(|message| Error::new(ErrorKind::InvalidData, message))
    }
    /// Creates [`Texture`] from the file.
    ///
    fn create_texture_from_file(&self, filename: impl AsRef<Path>) -> Result<Texture, Error> {
        match self {
            InnerTextureCreator::ForImage(texture_creator) => {
                texture_creator.load_texture(filename)
            }
            InnerTextureCreator::ForWindow(texture_creator) => {
                texture_creator.load_texture(filename)
            }
        }
        .map(|texture| Texture { texture })
        .map_err(|message| Error::new(ErrorKind::InvalidInput, message))
    }
}
impl<'a> fmt::Debug for InnerTextureCreator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InnerTextureCreator::ForImage(_) => write!(f, "`TextureCreator` for `Image`s"),
            InnerTextureCreator::ForWindow(_) => write!(f, "`TextureCreator` for `Window`s"),
        }
    }
}
/// [`TextureCreator`] struct handles creations of [`Texture`]s that cannot outlive their creator.
/// This struct is the only way to obtain [`Texture`] instance.
///
/// You cannot manually instantiate [`TextureCreator`], you have to get it from the other structs.
/// It is encouraged to read docs to find out how.
///
#[derive(Debug)]
pub struct TextureCreator<'a> {
    /// Underlying `sdl` texture creator.
    ///
    texture_creator: InnerTextureCreator<'a>,
}
impl<'a> TextureCreator<'a> {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Initializes [`TextureCreator`] for image from `sdl2` texture creator.
    ///
    pub(crate) fn from_sdl_texture_creator_image(
        texture_creator: RenderTextureCreator<SurfaceContext<'a>>,
    ) -> TextureCreator<'a> {
        TextureCreator {
            texture_creator: InnerTextureCreator::ForImage(texture_creator),
        }
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Initializes [`TextureCreator`] for window from `sdl2` texture creator.
    ///
    pub(crate) fn from_sdl_texture_creator_window(
        texture_creator: RenderTextureCreator<WindowContext>,
    ) -> TextureCreator<'a> {
        TextureCreator {
            texture_creator: InnerTextureCreator::ForWindow(texture_creator),
        }
    }

    /// Returns the best pixel format for [`TextureCreator`] or `None`, if the format is not recognised by `ggengine`.
    ///
    /// Even if the format is not recognised, it is still usable by `ggengine`.
    ///
    pub fn default_pixel_format(&self) -> Option<PixelFormat> {
        self.texture_creator.default_pixel_format()
    }

    /// Creates new texture with given size, format and access type.
    ///
    /// If given format is `None`, [`TextureCreator`] will use the best pixel format for [`Texture`].
    ///
    pub fn create_texture(
        &self,
        width: u32,
        height: u32,
        format: Option<PixelFormat>,
        access_type: AccessType,
    ) -> Texture {
        self.texture_creator
            .create_texture(width, height, format, access_type)
    }
    /// Creates [`Texture`] from the [`Image`].
    ///
    pub fn create_texture_from_image(&self, image: &Image) -> Texture {
        self.texture_creator.create_texture_from_image(image)
    }
    /// Creates [`Texture`] from bytes of supported format ('.png', '.jpg', but not raw buffer).
    ///
    pub fn create_texture_from_bytes(&self, bytes: Box<[u8]>) -> Result<Texture, Error> {
        self.texture_creator.create_texture_from_bytes(bytes)
    }
    /// Creates [`Texture`] from the file.
    ///
    pub fn create_texture_from_file(&self, filename: impl AsRef<Path>) -> Result<Texture, Error> {
        self.texture_creator.create_texture_from_file(filename)
    }
}

/// [`Texture`] struct is a hardware image that is used in rendering.
///
/// [`Texture`] is the most vital struct for all the `graphicscore` and it is widely used throughout game engine.
///
/// # Example
/// ```rust, no_run
/// # use ggengine::graphicscore::textures::{TextureCreator, Texture, AccessType};
/// # use ggengine::datacore::images::PixelFormat;
/// let texture_creator: TextureCreator = todo!("obtain the texture creator");
/// let texture: Texture = texture_creator.create_texture(
///     300, 300,
///     Some(PixelFormat::RGBA8888),
///     AccessType::Static,
/// );
/// ```
///
pub struct Texture<'a> {
    /// Underlying `sdl` texture.
    ///
    texture: RenderTexture<'a>,
}
impl<'a> Texture<'a> {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns reference to underlying `RenderTexture`.
    ///
    pub(crate) fn get_sdl_texture(&self) -> &RenderTexture<'a> {
        &self.texture
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns mutable reference to underlying `RenderTexture`.
    ///
    pub(crate) fn get_sdl_texture_mut(&mut self) -> &mut RenderTexture<'a> {
        &mut self.texture
    }

    /// Accesses inner data of the [`Texture`].
    ///
    /// Texture's access type must be `AccessType::Streaming`, otherwise it is a no-op and this function
    /// just returns `None`.
    ///
    /// This function might cause performance issues, so it should not be used frequently.
    ///
    /// # Note
    /// This function should not be used to get the actual data of [`Texture`], because due to optimisation
    /// inner data does not necessarily contain the old texture data.
    ///
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::graphicscore::textures::{TextureCreator, Texture};
    /// # use std::path::Path;
    /// let texture_creator: TextureCreator = todo!("obtain the texture creator");
    /// let mut texture: Texture = texture_creator.create_texture_from_file(Path::new("i.png"))
    ///     .expect("Filename should be correct");
    /// texture.access_data_mut(|data| {
    ///     for pixel in data {
    ///         *pixel = 255;
    ///     }
    /// });
    /// ```
    ///
    pub fn access_data_mut<R>(&mut self, f: impl FnOnce(&mut [u8]) -> R) -> Option<R> {
        if self.access_type() == AccessType::Streaming {
            Some(
                self.texture
                    .with_lock(None, |data, _| f(data))
                    .expect("Texture should have `AccessType::Streaming` in this branch"),
            )
        } else {
            None
        }
    }

    /// Returns width of texture in pixels.
    ///
    pub fn width(&self) -> u32 {
        self.texture.query().width
    }
    /// Returns height of texture in pixels.
    ///
    pub fn height(&self) -> u32 {
        self.texture.query().height
    }
    /// Returns size of texture in pixels (width and height).
    ///
    pub fn size(&self) -> (u32, u32) {
        let query: RenderTextureQuery = self.texture.query();
        (query.width, query.height)
    }

    /// Returns pixel format that is used by [`Texture`].
    ///
    /// If `None` is returned, then the format is not recognised (but can still be used).
    ///
    pub fn pixel_format(&self) -> Option<PixelFormat> {
        PixelFormat::from_sdl_pixel_format_enum(self.texture.query().format)
    }

    /// Returns access type of this texture.
    ///
    pub fn access_type(&self) -> AccessType {
        AccessType::from_sdl_texture_access(self.texture.query().access)
    }
}
impl<'a> fmt::Debug for Texture<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Texture").finish()
    }
}
