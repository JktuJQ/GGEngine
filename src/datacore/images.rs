//! `datacore::images` submodule supplies instruments that help in work with image data.
//!
//! This submodule provides structs and enums which represent color and image data,
//! [`PixelFormat`] lists possible pixel formats and [`Image`] encapsulates work with images.
//!
//! To further understand relations between those structs, traits, enums and constants, it is encouraged to read docs for submodule items.
//!

use crate::datacore::assets::{FromFile, ToFile};
use bitflags::bitflags;
use sdl2::{
    image::{
        init as image_init, InitFlag as ImageInitFlag, LoadSurface as ImageLoadSurface,
        SaveSurface as ImageSaveSurface, Sdl2ImageContext as ImageContext,
    },
    pixels::PixelFormatEnum as ImagePixelFormatEnum,
    rect::Rect as Sdl2Rect,
    surface::Surface as ImageSurface,
};
use std::{
    fmt,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// [`PixelFormat`] enum lists all possible formats of color encoding.
///
/// Only RGB-based formats are supported, some with alpha channel and some without it.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    /// RGB332 color format.
    ///
    RGB332 = 336_660_481,
    /// RGB444 color format.
    ///
    RGB444 = 353_504_258,
    /// RGB555 color format.
    ///
    RGB555 = 353_570_562,
    /// BGR555 color format.
    ///
    BGR555 = 357_764_866,
    /// RGB565 color format.
    ///
    RGB565 = 353_701_890,
    /// BGR555 color format.
    ///
    BGR565 = 357_896_194,

    /// ARGB4444 color format.
    ///
    ARGB4444 = 355_602_434,
    /// RGBA4444 color format.
    ///
    RGBA4444 = 356_651_010,
    /// ABGR4444 color format.
    ///
    ABGR4444 = 359_796_738,
    /// BGRA4444 color format.
    ///
    BGRA4444 = 360_845_314,
    /// ARGB1555 color format.
    ///
    ARGB1555 = 355_667_970,
    /// RGBA5551 color format.
    ///
    RGBA5551 = 356_782_082,
    /// ABGR1555 color format.
    ///
    ABGR1555 = 359_862_274,
    /// BGRA5551 color format.
    ///
    BGRA5551 = 360_976_386,

    /// RGB24 color format.
    ///
    RGB24 = 386_930_691,
    /// BGR24 color format.
    ///
    BGR24 = 390_076_419,

    /// RGB888 color format.
    ///
    RGB888 = 370_546_692,
    /// BGR888 color format.
    ///
    BGR888 = 374_740_996,
    /// RGBX8888 color format.
    ///
    RGBX8888 = 371_595_268,
    /// BGRX8888 color format.
    ///
    BGRX8888 = 375_789_572,
    /// ARGB8888 color format.
    ///
    ARGB8888 = 372_645_892,
    /// RGBA8888 color format.
    ///
    RGBA8888 = 373_694_468,
    /// ABGR8888 color format.
    ///
    ABGR8888 = 376_840_196,
    /// BGRA8888 color format.
    ///
    BGRA8888 = 377_888_772,

    /// ARGB2101010 color format.
    ///
    ARGB2101010 = 372_711_428,
}
#[cfg(target_endian = "little")]
impl PixelFormat {
    /// 32 bit RGBA native format.
    ///
    pub const RGBA32: Self = Self::ABGR8888;
    /// 32 bit ARGB native format.
    ///
    pub const ARGB32: Self = Self::BGRA8888;
    /// 32 bit BGRA native format.
    ///
    pub const BGRA32: Self = Self::ARGB8888;
    /// 32 bit ABGR native format.
    ///
    pub const ABGR32: Self = Self::RGBA8888;
}
#[cfg(target_endian = "big")]
impl PixelFormat {
    /// 32 bit RGBA native format.
    ///
    pub const RGBA32: Self = Self::RGBA8888;
    /// 32 bit ARGB native format.
    ///
    pub const ARGB32: Self = Self::ARGB8888;
    /// 32 bit BGRA native format.
    ///
    pub const BGRA32: Self = Self::BGRA8888;
    /// 32 bit ABGR native format.
    ///
    pub const ABGR32: Self = Self::ABGR8888;
}
impl PixelFormat {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Converts `sdl2` ImagePixelFormatEnum to [`PixelFormat`].
    ///
    /// If format is not supported by this library, `None` is returned.
    ///
    pub(crate) fn from_sdl_pixel_format_enum(pixel_format: ImagePixelFormatEnum) -> Option<Self> {
        Some(match pixel_format {
            ImagePixelFormatEnum::RGB332 => Self::RGB332,
            ImagePixelFormatEnum::RGB444 => Self::RGB444,
            ImagePixelFormatEnum::RGB555 => Self::RGB555,
            ImagePixelFormatEnum::BGR555 => Self::BGR555,
            ImagePixelFormatEnum::RGB565 => Self::RGB565,
            ImagePixelFormatEnum::BGR565 => Self::BGR565,

            ImagePixelFormatEnum::ARGB4444 => Self::ARGB4444,
            ImagePixelFormatEnum::RGBA4444 => Self::RGBA4444,
            ImagePixelFormatEnum::ABGR4444 => Self::ABGR4444,
            ImagePixelFormatEnum::BGRA4444 => Self::BGRA4444,
            ImagePixelFormatEnum::ARGB1555 => Self::ARGB1555,
            ImagePixelFormatEnum::RGBA5551 => Self::RGBA5551,
            ImagePixelFormatEnum::ABGR1555 => Self::ABGR1555,
            ImagePixelFormatEnum::BGRA5551 => Self::BGRA5551,

            ImagePixelFormatEnum::RGB24 => Self::RGB24,
            ImagePixelFormatEnum::BGR24 => Self::BGR24,

            ImagePixelFormatEnum::RGB888 => Self::RGB888,
            ImagePixelFormatEnum::BGR888 => Self::BGR888,
            ImagePixelFormatEnum::RGBX8888 => Self::RGBX8888,
            ImagePixelFormatEnum::BGRX8888 => Self::BGRX8888,
            ImagePixelFormatEnum::ARGB8888 => Self::ARGB8888,
            ImagePixelFormatEnum::RGBA8888 => Self::RGBA8888,
            ImagePixelFormatEnum::ABGR8888 => Self::ABGR8888,
            ImagePixelFormatEnum::BGRA8888 => Self::BGRA8888,
            ImagePixelFormatEnum::ARGB2101010 => Self::ARGB2101010,

            _ => return None,
        })
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns `sdl2` representation of this enum.
    ///
    pub(crate) fn to_sdl_pixel_format_enum(self) -> ImagePixelFormatEnum {
        match self {
            Self::RGB332 => ImagePixelFormatEnum::RGB332,
            Self::RGB444 => ImagePixelFormatEnum::RGB444,
            Self::RGB555 => ImagePixelFormatEnum::RGB555,
            Self::BGR555 => ImagePixelFormatEnum::BGR555,
            Self::RGB565 => ImagePixelFormatEnum::RGB565,
            Self::BGR565 => ImagePixelFormatEnum::BGR565,

            Self::ARGB4444 => ImagePixelFormatEnum::ARGB4444,
            Self::RGBA4444 => ImagePixelFormatEnum::RGBA4444,
            Self::ABGR4444 => ImagePixelFormatEnum::ABGR4444,
            Self::BGRA4444 => ImagePixelFormatEnum::BGRA4444,
            Self::ARGB1555 => ImagePixelFormatEnum::ARGB1555,
            Self::RGBA5551 => ImagePixelFormatEnum::RGBA5551,
            Self::ABGR1555 => ImagePixelFormatEnum::ABGR1555,
            Self::BGRA5551 => ImagePixelFormatEnum::BGRA5551,

            Self::RGB24 => ImagePixelFormatEnum::RGB24,
            Self::BGR24 => ImagePixelFormatEnum::BGR24,

            Self::RGB888 => ImagePixelFormatEnum::RGB888,
            Self::BGR888 => ImagePixelFormatEnum::BGR888,
            Self::RGBX8888 => ImagePixelFormatEnum::RGBX8888,
            Self::BGRX8888 => ImagePixelFormatEnum::BGRX8888,
            Self::ARGB8888 => ImagePixelFormatEnum::ARGB8888,
            Self::RGBA8888 => ImagePixelFormatEnum::RGBA8888,
            Self::ABGR8888 => ImagePixelFormatEnum::ABGR8888,
            Self::BGRA8888 => ImagePixelFormatEnum::BGRA8888,
            Self::ARGB2101010 => ImagePixelFormatEnum::ARGB2101010,
        }
    }

    /// Returns how much bytes are required for one pixel in chosen format.
    ///
    pub fn pixel_byte_size(&self) -> usize {
        match self {
            Self::RGB332 => 1,

            Self::RGB444
            | Self::RGB555
            | Self::BGR555
            | Self::ARGB4444
            | Self::RGBA4444
            | Self::ABGR4444
            | Self::BGRA4444
            | Self::ARGB1555
            | Self::RGBA5551
            | Self::ABGR1555
            | Self::BGRA5551
            | Self::RGB565
            | Self::BGR565 => 2,

            Self::RGB24 | Self::BGR24 => 3,

            Self::RGB888
            | Self::BGR888
            | Self::RGBX8888
            | Self::BGRX8888
            | Self::ARGB8888
            | Self::RGBA8888
            | Self::ABGR8888
            | Self::BGRA8888
            | Self::ARGB2101010 => 4,
        }
    }

    /// Returns whether pixel format supports alpha channel or not.
    ///
    /// Only formats with letter A support alpha channels.
    ///
    pub fn supports_alpha(&self) -> bool {
        matches!(
            self,
            Self::ARGB4444
                | Self::ARGB1555
                | Self::ARGB8888
                | Self::ARGB2101010
                | Self::ABGR4444
                | Self::ABGR1555
                | Self::ABGR8888
                | Self::BGRA4444
                | Self::BGRA5551
                | Self::BGRA8888
                | Self::RGBA4444
                | Self::RGBA5551
                | Self::RGBA8888
        )
    }
}

/// [`ImageArea`] struct represents part of image that is bounded by two points: upper left and bottom right.
///
/// Note: the most left upper point of 100x100 image would be (0, 0) point and the most right lower one is (100, 100).
///
/// # Example
/// ```rust
/// # use ggengine::datacore::images::ImageArea;
/// let area: ImageArea = ImageArea::from(((100, 100), (200, 200)));
/// assert_eq!(area.width(), 100);
/// assert_eq!(area.height(), 100);
/// ```
///
#[derive(Copy, Clone, Debug, Default)]
pub struct ImageArea {
    /// Tuple with the lowest x and y coordinates.
    ///
    left_upper: (u32, u32),
    /// Tuple with the highest x and y coordinates.
    ///
    right_lower: (u32, u32),
}
impl ImageArea {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Constructs `sdl2` rect that matches image area.
    ///
    pub(crate) fn to_sdl_rect(self) -> Sdl2Rect {
        Sdl2Rect::new(
            i32::try_from(self.left_upper.0).expect("Area width should not exceed `i32::MAX`."),
            i32::try_from(self.left_upper.0).expect("Area height should not exceed `i32::MAX`."),
            self.width(),
            self.height(),
        )
    }

    /// Returns left upper point of bounded part.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::datacore::images::ImageArea;
    /// let area: ImageArea = ImageArea::from(((200, 100), (100, 200)));
    /// assert_eq!(area.left_upper(), (100, 100));
    /// ```
    ///
    pub fn left_upper(&self) -> (u32, u32) {
        self.left_upper
    }
    /// Returns right lower point of bounded part.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::datacore::images::ImageArea;
    /// let area: ImageArea = ImageArea::from(((100, 200), (200, 100)));
    /// assert_eq!(area.right_lower(), (200, 200));
    /// ```
    ///
    pub fn right_lower(&self) -> (u32, u32) {
        self.right_lower
    }

    /// Returns width of bounded part.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::datacore::images::ImageArea;
    /// let area: ImageArea = ImageArea::from(((100, 100), (200, 200)));
    /// assert_eq!(area.width(), 100);
    /// ```
    ///
    pub fn width(&self) -> u32 {
        self.right_lower.0 - self.left_upper.0
    }
    /// Returns height of bounded part.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::datacore::images::ImageArea;
    /// let area: ImageArea = ImageArea::from(((100, 100), (200, 200)));
    /// assert_eq!(area.height(), 100);
    /// ```
    ///
    pub fn height(&self) -> u32 {
        self.right_lower.1 - self.left_upper.1
    }
}
impl From<((u32, u32), (u32, u32))> for ImageArea {
    /// Constructs [`ImageArea`] using given coordinates that represent a part of image that is bounded by them,
    /// so even for inputs like `((200, 200), (100, 100))` or `((200, 100), (100, 200))`
    /// actual base for area would be `((x_min, y_min), (x_max, y_max))`.
    ///
    fn from(value: ((u32, u32), (u32, u32))) -> Self {
        ImageArea {
            left_upper: (value.0 .0.min(value.1 .0), value.0 .1.min(value.1 .1)),
            right_lower: (value.0 .0.max(value.1 .0), value.0 .1.max(value.1 .1)),
        }
    }
}

/// [`Image`] struct is used to represent images and manipulate them.
///
/// It supports loading images from disk, saving them, redacting, blitting and many other transformations.
/// This struct is widely used throughout game engine.
///
pub struct Image<'a> {
    /// Name of a loaded image file (`PathBuf` is empty only if image was manually created).
    ///
    filename: PathBuf,
    /// Underlying `sdl2` surface.
    ///
    surface: ImageSurface<'a>,
}
impl<'a> Image<'a> {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Constructs [`Image`] from `sdl2` surface.
    ///
    pub(crate) fn from_sdl_surface(filename: PathBuf, surface: ImageSurface<'a>) -> Self {
        Self { filename, surface }
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Destructures itself by consuming [`Image`].
    ///
    pub(crate) fn destructure(self) -> (PathBuf, ImageSurface<'a>) {
        (self.filename, self.surface)
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns reference to underlying `ImageSurface`.
    ///
    pub(crate) fn get_sdl_surface(&self) -> &ImageSurface<'a> {
        &self.surface
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns mutable reference to underlying `ImageSurface`.
    ///
    pub(crate) fn get_sdl_surface_mut(&mut self) -> &mut ImageSurface<'a> {
        &mut self.surface
    }

    /// Returns name of file from which [`Image`] was initialized or empty `Path`, if it was created manually.
    ///
    pub fn filename(&self) -> &Path {
        self.filename.as_path()
    }

    /// Initializes new empty image from given size and format.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::datacore::images::{Image, PixelFormat};
    /// let image: Image = Image::new(100, 100, PixelFormat::RGBA32);
    /// ```
    ///
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        Self {
            filename: PathBuf::new(),
            surface: ImageSurface::new(width, height, format.to_sdl_pixel_format_enum())
                .expect("All `PixelFormat` variants have valid representations in `sdl2`."),
        }
    }
    /// Initializes [`Image`] in given format from buffer which will be leaked to acquire static reference.
    ///
    /// No additional conversions will be made.
    ///
    pub fn from_raw_buffer(
        buffer: Box<[u8]>,
        width: u32,
        height: u32,
        pitch: u32,
        format: PixelFormat,
    ) -> Result<Self, Error> {
        Ok(Self {
            filename: PathBuf::new(),
            surface: ImageSurface::from_data(
                Box::leak::<'a>(buffer),
                width,
                height,
                pitch,
                format.to_sdl_pixel_format_enum(),
            )
            .map_err(|message| Error::new(ErrorKind::InvalidData, message))?,
        })
    }
    /// Copies the surface into a new one of a specified pixel format.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::{Image, PixelFormat};
    /// let image: Image = Image::new(100, 100, PixelFormat::RGBA32);
    /// let new_image: Image = image.convert(PixelFormat::RGBA8888);
    /// ```
    ///
    pub fn convert(&self, format: PixelFormat) -> Self {
        Image {
            filename: self.filename.clone(),
            surface: self
                .surface
                .convert_format(format.to_sdl_pixel_format_enum())
                .expect("All conversions should not fail."),
        }
    }

    /// Returns byte offset of a pixel (x, y) for image data.
    ///
    pub fn pixel_offset(&self, x: u32, y: u32) -> usize {
        x as usize * self.surface.pixel_format_enum().byte_size_per_pixel()
            + y as usize * self.surface.pitch() as usize
    }
    /// Applies function to inner data of image and returns result of this function.
    ///
    /// Inner data of image is represented by `u8` slice.
    /// To find offset of exact pixel you can use `pixel_offset` function. `data[image.pixel_offset(x, y)]` points on the first byte of pixel,
    /// however pixel size 1/2/3/4 byte long and the order of pixel components are dependent on image format and endianness.
    /// Use `PixelFormat` enum functions to get info about image format. To get values for colors use bit operations.
    ///
    /// This function might cause performance issues, so it should not be used frequently.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::Image;
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let image: Image = Image::from_file(Path::new("i.png")).expect("Filename should be correct.");
    /// let (x, y): (u32, u32) = (100, 200);  // Accesses (100, 200) pixel
    /// println!("{}", image.access_data(|data|
    ///     data[image.pixel_offset(x, y)]) & 0b11
    /// );
    /// ```
    ///
    pub fn access_data<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        match self.surface.must_lock() {
            true => self.surface.with_lock(f),
            false => f(self
                .surface
                .without_lock()
                .expect("Surface should not be locked at this branch.")),
        }
    }
    /// Applies function to inner data of image and returns result of this function.
    ///
    /// Inner data of image is represented by `u8` slice.
    /// To find offset of exact pixel you can use `pixel_offset` function. `data[image.pixel_offset(x, y)]` points on the first byte of pixel,
    /// however pixel size 1/2/3/4 byte long and the order of pixel components are dependent on image format and endianness.
    /// Use `PixelFormat` enum functions to get info about image format. To get values for colors use bit operations.
    ///
    /// This function might cause performance issues, so it should not be used frequently.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::Image;
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let mut image: Image = Image::from_file(Path::new("i.png"))
    ///     .expect("Filename should be correct.");
    /// let (x, y): (u32, u32) = (100, 200);  // Accesses (100, 200) pixel
    /// let coord: usize = image.pixel_offset(x, y);
    /// println!("{:?}",
    ///     image.access_data_mut(|data| {
    ///         data[coord] = 255;
    ///         data[coord]
    ///     })
    /// );
    /// ```
    ///
    ///
    pub fn access_data_mut<R>(&mut self, f: impl FnOnce(&mut [u8]) -> R) -> R {
        match self.surface.must_lock() {
            true => self.surface.with_lock_mut(f),
            false => f(self
                .surface
                .without_lock_mut()
                .expect("Surface should not be locked at this branch.")),
        }
    }

    /// Crops image using given area which will be left after cropping.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::{ImageArea, Image};
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let image1: Image = Image::from_file(Path::new("i.png")).expect("Filename should be correct.");
    /// let image2: Image = image1.crop(ImageArea::from(((50, 50), (100, 100))));
    /// ```
    ///
    pub fn crop(&self, area: ImageArea) -> Image {
        let mut result = ImageSurface::new(area.width(), area.height(), self.surface.pixel_format_enum())
            .expect("`ImageSystem::init` should be called before using anything else from `ggengine::datacore::image` submodule..");
        let _ = self
            .surface
            .blit(Some(area.to_sdl_rect()), &mut result, None)
            .expect("Cropping should be possible..");
        Image {
            filename: PathBuf::new(),
            surface: result,
        }
    }
    /// Blits (copies) part of source image to part of destination image.
    ///
    /// Blitting can be thought of as overlaying parts of image with part of another.
    /// If `src_area` is `None` then whole part of source image will be used.
    /// If `dst_area` is `None` then source part will be positioned at left upper corner.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::{ImageArea, Image, PixelFormat};
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let source: Image = Image::from_file(Path::new("i.png")).expect("Filename should be correct.");
    /// let mut destination: Image = Image::new(100, 100, PixelFormat::RGBA32);
    /// source.blit_to(Some(ImageArea::from(((50, 50), (100, 100)))), &mut destination, None);
    /// ```
    ///
    /// # Panics
    /// Blitting can panic or behave wrongly if any [`ImageArea`] bounded part coordinates exceed `i32::MAX`.
    /// Best efforts to clamp width and height are done, but if the most left upper coordinate exceeds, it's impossible to handle.
    ///
    pub fn blit_to(
        &self,
        src_area: Option<ImageArea>,
        dst_image: &mut Image,
        dst_area: Option<ImageArea>,
    ) {
        let _ = self
            .surface
            .blit(
                src_area.map(|area| area.to_sdl_rect()),
                &mut dst_image.surface,
                dst_area.map(|area| area.to_sdl_rect()),
            )
            .expect("Blitting should not fail.");
    }
    /// Blits part of another image on this image.
    ///
    /// This function is an alias with changed order of arguments for `blit_to` function.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::{ImageArea, Image, PixelFormat};
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let source: Image = Image::from_file(Path::new("i.png")).expect("Filename should be correct.");
    /// let mut destination: Image = Image::new(100, 100, PixelFormat::RGBA32);
    /// destination.blit_from(None, &source, Some(ImageArea::from(((50, 50), (100, 100)))));
    /// ```
    ///
    pub fn blit_from(
        &mut self,
        dst_area: Option<ImageArea>,
        src_image: &Image,
        src_area: Option<ImageArea>,
    ) {
        src_image.blit_to(src_area, self, dst_area);
    }

    /// Returns width of image in pixels.
    ///
    pub fn width(&self) -> u32 {
        self.surface.width()
    }
    /// Returns height of image in pixels.
    ///
    pub fn height(&self) -> u32 {
        self.surface.height()
    }
    /// Returns size of image in pixels (width and height).
    ///
    pub fn size(&self) -> (u32, u32) {
        self.surface.size()
    }
    /// Returns image area that covers whole image.
    ///
    pub fn image_area(&self) -> ImageArea {
        ImageArea {
            left_upper: (0, 0),
            right_lower: self.size(),
        }
    }
    /// Returns pitch of image.
    ///
    pub fn pitch(&self) -> u32 {
        self.surface.pitch()
    }
    /// Returns image's pixel format or `None`, if format wasn't recognised.
    ///
    /// Even if format was not recognised, all `Image` methods would still work.
    ///
    pub fn pixel_format(&self) -> Option<PixelFormat> {
        PixelFormat::from_sdl_pixel_format_enum(self.surface.pixel_format_enum())
    }
}
impl FromFile for Image<'_> {
    /// Initializes [`Image`] from given file.
    ///
    /// Only RGB-based formats are supported.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::Image;
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let image: Image = Image::from_file(Path::new("i.png")).expect("Filename should be correct.");
    /// ```
    ///
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let surface = ImageSurface::from_file(path.as_ref())
            .map_err(|message| Error::new(ErrorKind::NotFound, message))?;
        if PixelFormat::from_sdl_pixel_format_enum(surface.pixel_format_enum()).is_none() {
            return Err(Error::new(ErrorKind::InvalidData, "Wrong image format"));
        }
        Ok(Image {
            filename: path.as_ref().to_path_buf(),
            surface,
        })
    }
}
impl ToFile for Image<'_> {
    /// Saves image to '*.png' file.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::images::{Image, PixelFormat};
    /// # use ggengine::datacore::assets::ToFile;
    /// let image: Image = Image::new(100, 100, PixelFormat::RGBA32);
    /// image.to_file("i.png").expect("Filename should be correct.");
    /// ```
    ///
    fn to_file(&self, filename: impl AsRef<Path>) -> Result<(), Error> {
        self.surface
            .save(filename)
            .map_err(|message| Error::new(ErrorKind::InvalidData, message))
    }
}
impl fmt::Debug for Image<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("filename", &self.filename)
            .finish()
    }
}

bitflags!(
    /// [`ImageFormat`] bitflag struct lists supported image formats.
    ///
    pub struct ImageFormat : u32 {
        /// JPG image format.
        ///
        const JPG = 1 << 0;
        /// PNG image format.
        ///
        const PNG = 1 << 1;
        /// TIF image format.
        ///
        const TIF = 1 << 2;
        /// WEBP image format.
        ///
        const WEBP = 1 << 3;
    }
);

/// [`IMAGE_CONTEXT`] global static variable handles `sdl2::image` context.
///
static IMAGE_CONTEXT: OnceLock<ImageContext> = OnceLock::new();
/// [`ImageSystem`] is a global handler for image metadata.
///
/// ### `ImageSystem::init` should be called before using anything else from this submodule.
///
#[derive(Copy, Clone, Debug)]
pub enum ImageSystem {}
impl ImageSystem {
    /// Initializes image system, prepares libraries for use and allows different formats to be opened.
    /// If system is already initialized, does nothing; don't fear to 're-init' when in doubt.
    ///
    /// ### `ImageSystem::init` should be called before using anything else from `ggengine::datacore::image` submodule.
    ///
    pub fn init(image_format: ImageFormat) {
        let _ =
            IMAGE_CONTEXT.set(
                image_init(ImageInitFlag::from_bits(image_format.bits()).expect(
                    "`ImageFormat` constants are the same as in `InitFlag` bitflags struct",
                ))
                .expect("Image driver should be available."),
            );
    }
}
