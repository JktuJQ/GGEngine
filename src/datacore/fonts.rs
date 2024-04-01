//! `datacore::fonts` submodule supplies instruments that help in work with truetype fonts.
//!
//! There is not much to say about details, because everything that can be not obvious is just font terms,
//! which you can find in the internet.
//! In other things (e.g. [`FontSystem`]) this submodule is very similar to `audio` and `images`.
//!
//! ## Important
//! This module works **ONLY** with truetype fonts.
//!

use crate::{
    datacore::images::Image,
    mathcore::{vectors::Vector2Int, Color},
};
use bitflags::bitflags;
use sdl2::ttf::{
    init as ttf_init, Font as TTFont, FontError as TTFontError, FontStyle as TTFontStyle,
    Hinting as TTFontHinting, PartialRendering as TTFPartialRendering, Sdl2TtfContext,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{
    fmt,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// [`FontShowMode`] enum lists possible modes for showing truetype fonts.
///
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontShowMode {
    /// Allows showing text in a single line with given color.
    ///
    /// Text would not be anti-aliased and font will be shown on 8-bit image.
    ///
    Solid {
        /// Text is going to be colored in this color.
        ///
        color: Color,
    },
    /// Allows showing text on a background in a single line with given colors.
    ///
    /// Text would be anti-aliased and font will be shown on 8-bit image.
    ///
    Shaded {
        /// Text is going to be colored in this color.
        ///
        color: Color,
        /// Background of text is going to be colored in this color.
        ///
        background: Color,
    },
    /// Allows showing text in a single line using alpha blending to dither the font with the given color.
    ///
    /// Text would be anti-aliased and font will be shown on 32-bit image.
    ///
    Blended {
        /// Text is going to be colored in this color.
        ///
        color: Color,
    },
    /// Allows showing text in a multiple lines using alpha blending to dither the font with the given color.
    ///
    /// Text would be anti-aliased and font will be shown on 32-bit image.
    ///
    BlendedAndWrapped {
        /// Text is going to be colored in this color.
        ///
        color: Color,
        /// That's how much letters can be at one line at max
        /// (exceeding this threshold will lead to wrapping remaining part to the next line).
        wrap_max_width: u32,
    },
}
impl FontShowMode {
    /// Applies showing mode to font to obtain image.
    ///
    fn apply<'a>(self, show_object: TTFPartialRendering) -> Result<Image<'a>, Error> {
        (match self {
            FontShowMode::Solid { color } => show_object.solid(color.to_rgba()),
            FontShowMode::Shaded { color, background } => {
                show_object.shaded(color.to_rgba(), background.to_rgba())
            }
            FontShowMode::Blended { color } => show_object.blended(color.to_rgba()),
            FontShowMode::BlendedAndWrapped {
                color,
                wrap_max_width,
            } => show_object.blended_wrapped(color.to_rgba(), wrap_max_width),
        })
        .map(|surface| Image::from_sdl_surface(PathBuf::new(), surface))
        .map_err(|error| {
            let message: String = match error {
                TTFontError::InvalidLatin1Text(_) => String::from("Invalid Latin-1 text"),
                TTFontError::SdlError(message) => message,
            };
            Error::new(ErrorKind::InvalidData, message)
        })
    }
}
bitflags!(
    /// [`FontStyle`] bitflag struct lists truetype font styles.
    ///
    pub struct FontStyle : u32 {
        /// Normal font.
        ///
        const NORMAL = 0;
        /// Bold font.
        ///
        const BOLD = 1 << 0;
        /// Italic font.
        ///
        const ITALIC = 1 << 1;
        /// Underline font.
        ///
        const UNDERLINE = 1 << 2;
        /// Strikethrough font.
        ///
        const STRIKETHROUGH = 1 << 3;
    }
);
/// [`FontHinting`] enum lists possible hintings for truetype fonts.
///
#[derive(Copy, Clone, Debug)]
pub enum FontHinting {
    /// Normal font.
    ///
    Normal,
    /// Light font.
    ///
    Light,
    /// Mono font.
    ///
    Mono,
    /// No hintings.
    ///
    Nothing,
}
impl FontHinting {
    /// Converts `sdl2` `Hinting` to [`FontHinting`].
    ///
    fn from_sdl2enum(hinting: TTFontHinting) -> Self {
        match hinting {
            TTFontHinting::Normal => FontHinting::Normal,
            TTFontHinting::Light => FontHinting::Light,
            TTFontHinting::Mono => FontHinting::Mono,
            TTFontHinting::None => FontHinting::Nothing,
        }
    }
    /// Returns `sdl2` representation of this enum.
    ///
    fn to_sdl2enum(self) -> TTFontHinting {
        match self {
            FontHinting::Normal => TTFontHinting::Normal,
            FontHinting::Light => TTFontHinting::Light,
            FontHinting::Mono => TTFontHinting::Mono,
            FontHinting::Nothing => TTFontHinting::None,
        }
    }
}
/// [`GlyphMetrics`] struct stores information about glyphs of a font.
///
#[derive(Copy, Clone, Debug)]
pub struct GlyphMetrics {
    /// Minimal coordinate of glyph.
    ///
    pub min: Vector2Int,
    /// Maximal coordinate of glyph.
    ///
    pub max: Vector2Int,
    /// Advance of a glyph.
    ///
    pub advance: i32,
}
/// [`Font`] struct handles loaded font data.
///
/// # Examples
/// ```rust, no_run
/// # use ggengine::datacore::fonts::{Font, FontShowMode, FontSystem};
/// # use ggengine::mathcore::Color;
/// # use std::path::Path;
/// FontSystem::init();
/// let font: Font = FontSystem::load_font(Path::new("font.ttf"), 14)
///     .expect("Filename should be correct");
/// font.show_text(FontShowMode::Solid { color: Color::BLACK }, "ggengine")
///     .expect("Conversion should not fail");
/// ```
///
#[derive(Serialize)]
pub struct Font {
    /// Name of a loaded font.
    ///
    filename: PathBuf,
    /// Point size of font.
    ///
    point_size: u16,
    /// Underlying sdl font.
    ///
    #[serde(skip_serializing)]
    font: TTFont<'static, 'static>,
}
impl<'de> Deserialize<'de> for Font {
    fn deserialize<D>(deserializer: D) -> Result<Font, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (path_buf, point_size): (PathBuf, u16) =
            <(PathBuf, u16) as Deserialize>::deserialize(deserializer)?;
        FontSystem::load_font(path_buf.as_path(), point_size).map_err(|err| {
            de::Error::invalid_value(de::Unexpected::Str(&err.to_string()), &"Wrong filename")
        })
    }
}
impl Font {
    /// Returns name of file from which [`Font`] was initialized.
    ///
    pub fn filename(&self) -> &Path {
        self.filename.as_path()
    }

    /// Transforms given UTF-8 text using this font and given [`FontShowMode`] into image.
    ///
    /// # Examples
    /// ```rust, no_run
    /// # use ggengine::datacore::fonts::{Font, FontShowMode, FontSystem};
    /// # use ggengine::mathcore::Color;
    /// # use std::path::Path;
    /// # FontSystem::init();
    /// let font: Font = FontSystem::load_font(Path::new("font.ttf"), 14)
    ///     .expect("Filename should be correct");
    /// font.show_text(FontShowMode::Solid { color: Color::BLACK }, "ggengine")
    ///     .expect("Conversion should not fail");
    /// ```
    ///
    pub fn show_text(&self, mode: FontShowMode, text: &str) -> Result<Image, Error> {
        mode.apply(self.font.render(text))
    }
    /// Transforms given character using this font and given [`FontShowMode`] into image.
    ///
    /// # Examples
    /// ```rust, no_run
    /// # use ggengine::datacore::fonts::{Font, FontShowMode, FontSystem};
    /// # use ggengine::mathcore::Color;
    /// # use std::path::Path;
    /// # FontSystem::init();
    /// let font: Font = FontSystem::load_font(Path::new("font.ttf"), 14)
    ///     .expect("Filename should be correct");
    /// font.show_character(FontShowMode::Solid { color: Color::BLACK }, 'a')
    ///     .expect("Conversion should not fail");
    /// ```
    ///
    pub fn show_character(&self, mode: FontShowMode, character: char) -> Result<Image, Error> {
        mode.apply(self.font.render_char(character))
    }
    /// Transforms given Latin-1 text using this font and given [`FontShowMode`] into image.
    ///
    /// # Examples
    /// ```rust, no_run
    /// # use ggengine::datacore::fonts::{Font, FontShowMode, FontSystem};
    /// # use ggengine::mathcore::Color;
    /// # use std::path::Path;
    /// # FontSystem::init();
    /// let font: Font = FontSystem::load_font(Path::new("font.ttf"), 14)
    ///     .expect("Filename should be correct");
    /// font.show_latin1_text(FontShowMode::Solid { color: Color::BLACK },
    ///     &[0xC4, 0x70, 0x70, 0x6C, 0x65]
    /// ).expect("Conversion should not fail");
    /// ```
    ///
    pub fn show_latin1_text(&self, mode: FontShowMode, latin1_text: &[u8]) -> Result<Image, Error> {
        mode.apply(self.font.render_latin1(latin1_text))
    }

    /// Returns the index of the given character in this font face.
    ///
    pub fn find_glyph(&self, character: char) -> Option<u16> {
        self.font.find_glyph(character)
    }
    /// Returns the glyph metrics of the given character in this font face.
    ///
    pub fn find_glyph_metrics(&self, character: char) -> Option<GlyphMetrics> {
        self.font
            .find_glyph_metrics(character)
            .map(|sdl_metrics| GlyphMetrics {
                min: Vector2Int::from([sdl_metrics.minx, sdl_metrics.miny]),
                max: Vector2Int::from([sdl_metrics.maxx, sdl_metrics.maxy]),
                advance: sdl_metrics.advance,
            })
    }

    /// Returns the width and height of the given UTF-8 text when rendered using this font.
    ///
    pub fn size_of_text(&self, text: &str) -> Option<(u32, u32)> {
        match self.font.size_of(text) {
            Ok(size) => Some(size),
            Err(_) => None,
        }
    }
    /// Returns the width and height of the given character when rendered using this font.
    ///
    pub fn size_of_char(&self, character: char) -> Option<(u32, u32)> {
        match self.font.size_of_char(character) {
            Ok(size) => Some(size),
            Err(_) => None,
        }
    }
    /// Returns the width and height of the given Latin-1 text in bytes when rendered using this font.
    ///
    pub fn size_of_latin1_text(&self, latin1_text: &[u8]) -> Option<(u32, u32)> {
        match self.font.size_of_latin1(latin1_text) {
            Ok(size) => Some(size),
            Err(_) => None,
        }
    }

    /// Returns this font's maximum total height.
    ///
    pub fn height(&self) -> u32 {
        self.font.height() as u32
    }
    /// Returns this font’s highest ascent (height above base).
    ///
    pub fn ascent(&self) -> u32 {
        self.font.ascent() as u32
    }
    /// Returns this font’s lowest descent (height below base).
    ///
    pub fn descent(&self) -> u32 {
        self.font.descent().unsigned_abs()
    }
    /// Returns this font's size in points.
    ///
    pub fn point_size(&self) -> u16 {
        self.point_size
    }

    /// Returns the number of faces in this font.
    ///
    pub fn face_count(&self) -> u16 {
        self.font.face_count()
    }
    /// Returns whether this font is monospaced or not.
    pub fn is_face_fixed_width(&self) -> bool {
        self.font.face_is_fixed_width()
    }
    /// Returns the family name of the current font face.
    ///
    pub fn face_family_name(&self) -> Option<String> {
        self.font.face_family_name()
    }
    /// Returns the name of the current font face.
    ///
    pub fn face_style_name(&self) -> Option<String> {
        self.font.face_style_name()
    }

    /// Sets new outline width for this font.
    ///
    pub fn set_outline_width(&mut self, width: u16) {
        self.font.set_outline_width(width);
    }
    /// Returns current outline width of this font.
    ///
    pub fn get_outline_width(&self) -> u16 {
        self.font.get_outline_width()
    }

    /// Sets new kerning for this font.
    ///
    pub fn set_kerning(&mut self, kerning: bool) {
        self.font.set_kerning(kerning);
    }
    /// Returns current kerning of this font.
    ///
    pub fn get_kerning(&self) -> bool {
        self.font.get_kerning()
    }

    /// Sets new styling for this font.
    ///
    pub fn set_style(&mut self, style: FontStyle) {
        self.font
            .set_style(TTFontStyle::from_bits(style.bits() as i32).expect(
                "`FontStyle` constants are the same as in SDL `FontStyle` bitflags struct",
            ));
    }
    /// Returns current styling of this font.
    ///
    pub fn get_style(&self) -> FontStyle {
        FontStyle::from_bits(self.font.get_style().bits() as u32)
            .expect("`FontStyle` constants are the same as in SDL `FontStyle` bitflags struct")
    }

    /// Sets new hinting for this font.
    ///
    pub fn set_hinting(&mut self, hinting: FontHinting) {
        self.font.set_hinting(hinting.to_sdl2enum());
    }
    /// Returns current hinting of this font.
    ///
    pub fn get_hinting(&self) -> FontHinting {
        FontHinting::from_sdl2enum(self.font.get_hinting())
    }
}
impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Font")
            .field("filename", &self.filename)
            .finish()
    }
}

/// [`TTF_CONTEXT`] global static variable handles `sdl2::ttf` context.
///
static TTF_CONTEXT: OnceLock<Sdl2TtfContext> = OnceLock::new();
/// [`FontSystem`] is a global handler for truetype fonts metadata.
///
/// ### `FontSystem::init` should be called before using anything else from this submodule.
///
#[derive(Copy, Clone, Debug)]
pub enum FontSystem {}
impl FontSystem {
    /// Initializes truetype font system, prepares libraries for use and allows different formats to be opened.
    ///
    /// ### `FontSystem::init` should be called before using anything else from `ggengine::datacore::fonts` submodule.
    ///
    pub fn init() {
        if TTF_CONTEXT.get().is_some() {
            return;
        }
        let _ = TTF_CONTEXT.set(ttf_init().expect("Font driver should be available"));
    }

    /// Loads truetype font from path with given size in points.
    ///
    /// # Examples
    /// ```rust, no_run
    /// # use ggengine::datacore::fonts::FontSystem;
    /// # use std::path::Path;
    /// FontSystem::init();
    /// let font = FontSystem::load_font(Path::new("font.ttf"), 14)
    ///     .expect("Filename should be correct");
    /// ```
    ///
    pub fn load_font(path: impl AsRef<Path>, point_size: u16) -> Result<Font, Error> {
        Ok(Font {
            filename: path.as_ref().to_path_buf(),
            point_size,
            font: TTF_CONTEXT
                .get()
                .expect("`FontSystem::init` should be called before using anything else from `ggengine::datacore::fonts` submodule")
                .load_font(path, point_size).map_err(|message| Error::new(ErrorKind::NotFound, message))?,
        })
    }
    /// Loads truetype font from path at exact index in file with given size in points.
    ///
    /// # Examples
    /// ```rust, no_run
    /// # use ggengine::datacore::fonts::FontSystem;
    /// # use std::path::Path;
    /// FontSystem::init();
    /// let font = FontSystem::load_font_at_index(Path::new("font.ttf"), 14, 10)
    ///     .expect("Filename should be correct");
    /// ```
    ///
    pub fn load_font_at_index(
        path: impl AsRef<Path>,
        point_size: u16,
        index: u32,
    ) -> Result<Font, Error> {
        Ok(Font {
            filename: path.as_ref().to_path_buf(),
            point_size,
            font: TTF_CONTEXT
                .get()
                .expect("`FontSystem::init` should be called before using anything else from `ggengine::datacore::fonts` submodule")
                .load_font_at_index(path, index, point_size).map_err(|message| Error::new(ErrorKind::NotFound, message))?,
        })
    }
}
