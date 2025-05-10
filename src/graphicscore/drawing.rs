//! `graphicscore::drawing` submodule implements several canvases
//! ([`WindowCanvas`], [`TextureCanvas`] and [`TextureCanvas`]) that allow drawing on them.
//!
//! This submodule provides [`Canvas`] trait that defines drawing interface for canvases,
//! [`WindowCanvas`] struct as main canvas that is a base for all other canvases, because
//! drawing on images or textures is implemented by managing those inside [`WindowCanvas`].
//!
//! # Model
//! Canvases are the top of the following hierarchy:
//! canvases create texture creators,
//! texture creators create textures,
//! textures can be blitted to canvases.
//!
//! This hierarchy implies some constraints.
//!
//! For example, no texture can outlive its texture creator,
//! but this is not true for canvases and texture creators.
//! Texture creator can outlive its canvas and can still produce texturess - it is, however, useless.
//! Also, you can create multiple texture creators from the same canvas and all of them are valid.
//!
//! But there is one constraint that can cause undefined behaviour if broken - textures should only
//! be used in the canvas that created their texture creator.
//!
//! Implementation of managing functions makes its best efforts are made to ensure this:
//! you cannot instantiate most of the canvases, and you are only given access to them in managing functions;
//! closures that capture external values (and thus can bring external textures) are prohibited;
//! absence of return value of managing functions makes it
//! impossible to return texture that is created by underlying canvas.
//!
//! However, `ggengine` cannot track textures if they come from different window canvases,
//! because enforcing proper constraints will make managing lifetimes and borrows really hard and will severely hurt the ergonomics.
//! Realistically, you should only be using one window in a game, and thus you will be operating with only one window canvas.
//! That said, it is important for user of `ggengine` to know about this caveat and to do his best
//! to avoid causing this problem.
//!

use crate::{
    datacore::images::Image,
    graphicscore::{
        textures::{AccessType, Texture, TextureCreator},
        {Blendable, BlendingType},
    },
    mathcore::{
        shapes::{PolygonLike, Rect, Segment},
        transforms::{Rotatable, Scalable, Translatable},
        vectors::Point,
        Color,
    },
    utils::Window,
};
use sdl2::{
    rect::{FRect as SdlFRect, Rect as SdlRect},
    render::{
        SurfaceCanvas as RenderSurfaceCanvas, SurfaceCanvas, WindowCanvas as RenderWindowCanvas,
    },
};
use std::fmt;

/// [`Canvas`] trait defines drawing methods that should be implemented on any canvas.
///
/// Every canvas allows drawing graphical primitives (points, lines, polygons) with selected color,
/// pasting textures into canvas.
///
/// This trait requires [`Blendable`] trait to be implemented.
///
/// # Example
/// ```rust, no_run
/// # use ggengine::GGEngine;
/// # use ggengine::utils::Window;
/// # use ggengine::graphicscore::{textures::{Texture, TextureCreator}, drawing::{Canvas, WindowCanvas}};
/// # use ggengine::datacore::{assets::ToFile, images::{Image, PixelFormat}};
/// # use ggengine::mathcore::{{Angle, Size, Color}, vectors::Point, shapes::{Segment, Rect}};
/// let engine: GGEngine = GGEngine::init();
/// let window: Window = engine.build_window("ggengine", 1000, 1000, Default::default());
/// let mut canvas: WindowCanvas = WindowCanvas::from_window(window, true);
/// let image: Image = canvas.manage_image(
///     Image::new(100, 100, PixelFormat::RGBA8888),
///     |image_canvas| {
///         image_canvas.set_draw_color(Color::RED);
///         assert_eq!(image_canvas.get_draw_color(), Color::RED);
///
///         image_canvas.draw_point(Point { x: 0.0, y: 0.0 });
///         image_canvas.draw_segment(
///             Segment {
///                 point1: Point { x: 10.0, y: 10.0 },
///                 point2: Point { x: 50.0, y: 50.0 }
///             }
///         );
///         image_canvas.draw_rect(
///             Rect::from_origin(
///                 Point { x: 100.0, y: 100.0 },
///                 Angle::DEG45,
///                 Size::try_from(30.0).expect("Value is in correct range."),
///                 Size::try_from(30.0).expect("Value is in correct range."),
///             )
///         );
///         image_canvas.draw_polygon(&[
///             Point { x: 200.0, y: 200.0 },
///             Point { x: 300.0, y: 300.0 },
///             Point { x: 400.0, y: 100.0 },
///         ]);
///
///         image_canvas.clear();
///
///         let texture_creator: TextureCreator = image_canvas.texture_creator();
///         image_canvas.blit_from_texture(
///             Some(Rect::from_origin(
///                 Point { x: 600.0, y: 600.0 },
///                 Angle::DEG60,
///                 Size::try_from(100.0).expect("Value is in correct range."),
///                 Size::try_from(100.0).expect("Value is in correct range."),
///             )),
///             &texture_creator.create_texture_from_file("texture.png")
///                 .expect("Filename should be correct"),
///             None,
///         );
///     }
/// );
/// image.to_file("image.png").expect("File creation or truncation should not fail");
/// ```
///
pub trait Canvas<'a>: Blendable {
    /// Sets new drawing color to the canvas.
    ///
    /// This will affect color of graphical primitives that are drawn and
    /// clearing color (`clear` method fills entire canvas with this color).
    ///
    fn set_draw_color(&mut self, color: Color);
    /// Returns color that is currently used for drawing.
    ///
    fn get_draw_color(&self) -> Color;

    /// Draws point on the canvas.
    ///
    /// Points coordinates are truncated towards integers.
    ///
    fn draw_point(&mut self, point: Point);
    /// Draws segment on the canvas.
    ///
    /// Points coordinates are truncated towards integers.
    ///
    fn draw_segment(&mut self, segment: Segment);
    /// Draws rectangle on the canvas.
    ///
    /// Points coordinates are truncated towards integers.
    ///
    fn draw_rect(&mut self, rect: Rect) {
        let vertices = rect.vertices();
        let length = vertices.len();
        for i in 1..=length {
            self.draw_segment(Segment {
                point1: vertices[i - 1],
                point2: vertices[i % length],
            });
        }
    }
    /// Draws polygon on the canvas.
    ///
    /// Points coordinates are truncated towards integers.
    ///
    fn draw_polygon(&mut self, polygon: &[Point]) {
        let length = polygon.len();
        for i in 1..=length {
            self.draw_segment(Segment {
                point1: polygon[i - 1],
                point2: polygon[i % length],
            });
        }
    }

    /// Clears canvas by filling it out with current draw color.
    ///
    fn clear(&mut self);

    /// Returns canvas's texture creator.
    ///
    fn texture_creator(&self) -> TextureCreator<'a>;
    /// Blits texture to the canvas.
    ///
    /// `dst_area` represents area on the canvas to which texture will be blitted.
    /// Rotation of underlying [`Rect`] will determine rotation of texture.
    /// If `dst_area` is `None`, texture will be stretched to fill canvas and rotation won't be applied.
    ///
    /// `src_area` represents area of the texture that will be used for blitting.
    /// Rotation of `src_area` is irrelevant to the rotation of pasted texture,
    /// thus passing rotated rectangle can lead to unexpected results.
    /// If rectangle is rotated, this function will only use part of texture in axis-aligned
    /// bounding box of the rectangle.
    /// If `src_area` is `None`, whole texture will be used for blitting.
    ///
    fn blit_from_texture(
        &mut self,
        dst_area: Option<Rect>,
        texture: &Texture,
        src_area: Option<Rect>,
    );
}
/// [`impl_canvas`] macro implements [`Blendable`] and [`Canvas`] traits
/// for [`WindowCanvas`], [`TextureCanvas`] and [`ImageCanvas`].
///
/// Canvas must have `canvas` field.
///
macro_rules! impl_canvas {
    ($struct:ty, $texture_creator_fn:path) => {
        impl<'a> Blendable for $struct {
            fn set_blend_mode(&mut self, blend_mode: BlendingType) {
                self.canvas.set_blend_mode(blend_mode.to_sdl_blend_mode());
            }
            fn blend_mode(&self) -> BlendingType {
                BlendingType::from_sdl_blend_mode(self.canvas.blend_mode())
            }
        }
        impl<'a> Canvas<'a> for $struct {
            fn set_draw_color(&mut self, color: Color) {
                self.canvas.set_draw_color(color.to_rgba());
            }
            fn get_draw_color(&self) -> Color {
                let (r, g, b, a) = self.canvas.draw_color().rgba();
                Color::from_rgba(r, g, b, a)
            }

            fn draw_point(&mut self, point: Point) {
                self.canvas
                    .draw_fpoint((point.x, point.y))
                    .expect("`ggengine` renderer should be able to draw a point");
            }
            fn draw_segment(&mut self, segment: Segment) {
                self.canvas
                    .draw_fline(
                        (segment.point1.x, segment.point1.y),
                        (segment.point2.x, segment.point2.y),
                    )
                    .expect("`ggengine` renderer should be able to draw a point");
            }

            fn clear(&mut self) {
                self.canvas.clear();
            }

            fn texture_creator(&self) -> TextureCreator<'a> {
                $texture_creator_fn(self.canvas.texture_creator())
            }
            fn blit_from_texture(
                &mut self,
                dst_area: Option<Rect>,
                texture: &Texture,
                src_area: Option<Rect>,
            ) {
                self.canvas
                    .copy_ex_f(
                        texture.get_sdl_texture(),
                        src_area.map(|rect| {
                            let [point1, point2] = rect.aabb();
                            let (width, height) = {
                                let diff = point2 - point1;
                                (diff.x as u32, diff.y as u32)
                            };
                            SdlRect::new(point1.x as i32, point1.y as i32, width, height)
                        }),
                        dst_area.map(|rect| {
                            let origin = rect.origin();
                            let size = rect.size();
                            SdlFRect::from_center((origin.x, origin.y), size.0.get(), size.1.get())
                        }),
                        dst_area.map_or(0.0, |rect| f64::from(rect.angle().degrees())),
                        None,
                        false,
                        false,
                    )
                    .expect("`ggengine` renderer should be able to perform texture blitting");
            }
        }
    };
}

/// [`ImageCanvas`] struct represents canvas that allows drawing on an [`Image`].
///
/// [`ImageCanvas`] cannot be instantiated manually and is instead obtained by managing
/// image within [`WindowCanvas`].
///
pub struct ImageCanvas<'a> {
    /// Underlying `sdl2` canvas.
    ///
    canvas: RenderSurfaceCanvas<'a>,
}
impl fmt::Debug for ImageCanvas<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ImageCanvas")
    }
}
impl_canvas!(
    ImageCanvas<'a>,
    TextureCreator::from_sdl_texture_creator_image
);

/// [`TextureCanvas`] struct represents canvas that allows drawing on a [`Texture`].
///
/// [`TextureCanvas`] cannot be instantiated manually and is instead obtained by managing
/// texture within [`WindowCanvas`].
///
pub struct TextureCanvas<'a> {
    /// Underlying mutable reference to `sdl2` canvas.
    ///
    /// Instance of canvas is borrowed from the [`WindowCanvas`] due to `sdl2` implementation.
    ///
    canvas: &'a mut RenderWindowCanvas,
}
impl fmt::Debug for TextureCanvas<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextureCanvas")
    }
}
impl_canvas!(
    TextureCanvas<'a>,
    TextureCreator::from_sdl_texture_creator_window
);

/// [`WindowCanvas`] struct represents canvas that allows drawing on a [`Window`].
///
/// [`WindowCanvas`] is instantiated from [`Window`] struct by consuming it (OS shell of window is not destroyed).
/// You can destroy this to obtain your [`Window`] instance back.
///
/// All other canvases are based on [`WindowCanvas`], you can use both the [`ImageCanvas`]
/// and the [`TextureCanvas`] by managing corresponding structs ([`Image`]s and [`Texture`]s) within
/// [`WindowCanvas`].
///
pub struct WindowCanvas {
    /// Underlying `sdl2` canvas.
    ///
    canvas: RenderWindowCanvas,
}
impl WindowCanvas {
    /// Constructs [`WindowCanvas`] from the [`Window`] by consuming it (OS shell of window is not destroyed)..
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::GGEngine;
    /// # use ggengine::utils::Window;
    /// # use ggengine::graphicscore::drawing::{Canvas, WindowCanvas};
    /// let engine: GGEngine = GGEngine::init();
    /// let window: Window = engine.build_window("ggengine", 1000, 1000, Default::default());
    /// let canvas: WindowCanvas = WindowCanvas::from_window(window, true);
    /// ```
    ///
    pub fn from_window(window: Window, vsync: bool) -> Self {
        let builder = {
            let builder = window.destructure().into_canvas().target_texture();
            if vsync {
                builder.present_vsync()
            } else {
                builder
            }
        };
        WindowCanvas {
            canvas: builder
                .build()
                .expect("`ggengine` should be able to initialize canvas from the window"),
        }
    }
    /// Consumes [`WindowCanvas`] to get back [`Window`] instance from which it was created.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::GGEngine;
    /// # use ggengine::utils::Window;
    /// # use ggengine::graphicscore::drawing::{Canvas, WindowCanvas};
    /// let engine: GGEngine = GGEngine::init();
    /// let window: Window = engine.build_window("ggengine", 1000, 1000, Default::default());
    /// let canvas: WindowCanvas = WindowCanvas::from_window(window, true);
    /// let window: Window = canvas.into_window();
    /// ```
    ///
    pub fn into_window(self) -> Window {
        Window::from_sdl_window(self.canvas.into_window())
    }

    /// [`WindowCanvas`] manages [`Image`] by consuming it and allowing drawing on [`ImageCanvas`]
    /// inside passed function. New image is returned after drawing.
    ///
    /// # Note
    /// Type of `f` is function pointer, because it disallows passing closures that capture external values.
    /// This constraint is necessary due to fact that every canvas should work only with [`Texture`]s that
    /// are obtained from [`TextureCreator`] whose parent is that canvas.
    /// You can read more about this in the docs for `graphicscore::drawing` submodule.
    ///
    /// # Example
    ///
    /// This example fills entire image with red color and saves it to file.
    ///
    /// ```rust, no_run
    /// # use ggengine::GGEngine;
    /// # use ggengine::utils::Window;
    /// # use ggengine::graphicscore::drawing::{Canvas, WindowCanvas};
    /// # use ggengine::datacore::{assets::ToFile, images::{Image, PixelFormat}};
    /// # use ggengine::mathcore::Color;
    /// let engine: GGEngine = GGEngine::init();
    /// let window: Window = engine.build_window("ggengine", 1000, 1000, Default::default());
    /// let mut canvas: WindowCanvas = WindowCanvas::from_window(window, true);
    ///
    /// let image: Image = canvas.manage_image(
    ///     Image::new(100, 100, PixelFormat::RGBA8888),
    ///     |image_canvas| {
    ///         image_canvas.set_draw_color(Color::RED);
    ///         image_canvas.clear();
    ///     }
    /// );
    /// image.to_file("image.png").expect("File creation or truncation should not fail");
    /// ```
    ///
    pub fn manage_image<'image>(
        &mut self,
        image: Image<'image>,
        f: fn(&mut ImageCanvas) -> (),
    ) -> Image<'image> {
        let (filename, surface) = image.destructure();
        let canvas = SurfaceCanvas::from_surface(surface)
            .expect("`ggengine` should be able to initialize canvas from the image");
        let mut image_canvas = ImageCanvas { canvas };
        f(&mut image_canvas);
        image_canvas.canvas.present();
        Image::from_sdl_surface(filename, image_canvas.canvas.into_surface())
    }

    /// Returns whether the canvas supports texture management or not.
    ///
    /// Checks for this property are performed automatically.
    ///
    pub fn supports_texture_management(&self) -> bool {
        self.canvas.render_target_supported()
    }
    /// [`WindowCanvas`] manages [`Texture`] by borrowing it and allowing drawing on [`TextureCanvas`]
    /// inside passed function. [`Texture`] is changed in place.
    ///
    /// This function is no-op if [`WindowCanvas`] or passed [`Texture`] do not support texture management
    /// (`AccessType::Targeted` should be set for texture to allow management).
    ///
    /// Calling this function in a loop is not optimal, because canvas resets its target back to
    /// itself (although this issue is not applicable to image managing).
    /// If you want to manage multiple textures, use `manage_textures` function.
    ///
    /// # Note
    /// Type of `f` is function pointer, because it disallows passing closures that capture external values.
    /// This constraint is necessary due to fact that every canvas should work only with [`Texture`]s that
    /// are obtained from [`TextureCreator`] whose parent is that canvas.
    /// You can read more about this in the docs for `graphicscore::drawing` submodule.
    ///
    /// # Example
    ///
    /// This example fills entire texture with red color and then blits it upon window.
    /// ```rust, no_run
    /// # use ggengine::GGEngine;
    /// # use ggengine::utils::Window;
    /// # use ggengine::graphicscore::drawing::{Canvas, WindowCanvas};
    /// # use ggengine::graphicscore::textures::{Texture, TextureCreator, AccessType};
    /// # use ggengine::mathcore::Color;
    /// let engine: GGEngine = GGEngine::init();
    /// let window: Window = engine.build_window("ggengine", 1000, 1000, Default::default());
    /// let mut canvas: WindowCanvas = WindowCanvas::from_window(window, true);
    ///
    /// let texture_creator: TextureCreator = canvas.texture_creator();
    /// let mut texture: Texture = texture_creator.create_texture(
    ///     100, 100,
    ///     texture_creator.default_pixel_format(),
    ///     AccessType::Targeted
    /// );
    ///
    /// canvas.manage_texture(
    ///     &mut texture,
    ///     |texture_canvas| {
    ///         texture_canvas.set_draw_color(Color::RED);
    ///         texture_canvas.clear();
    ///     }
    /// );
    ///
    /// canvas.blit_from_texture(None, &texture, None);
    /// canvas.update();
    /// ```
    ///
    pub fn manage_texture<'managing, 'texture: 'managing>(
        &mut self,
        texture: &'managing mut Texture<'texture>,
        f: fn(&mut TextureCanvas) -> (),
    ) {
        if texture.access_type() != AccessType::Targeted || !self.supports_texture_management() {
            return;
        }
        self.canvas
            .with_texture_canvas(texture.get_sdl_texture_mut(), |canvas| {
                f(&mut TextureCanvas { canvas })
            })
            .expect("`ggengine` should be able to initialize canvas from the texture");
    }
    /// [`WindowCanvas`] manages [`Texture`]s by borrowing them and allowing drawing on [`TextureCanvas`]
    /// inside passed function. [`Texture`]s are changed in place.
    /// This function also implements additional 'indexing' of textures that allows marking
    /// them and differentiating them inside passed function.
    /// If you do not want to index your texture, you can just use `()` type for index.
    ///
    /// This function is no-op if [`WindowCanvas`] does not support texture management, and
    /// it skips all textures that are not suitable for texture management
    /// (`AccessType::Targeted` should be set for texture to allow management).
    ///
    /// # Note
    /// Type of `f` is function pointer, because it disallows passing closures that capture external values.
    /// This constraint is necessary due to fact that every canvas should work only with [`Texture`]s that
    /// are obtained from [`TextureCreator`] whose parent is that canvas.
    /// You can read more about this in the docs for `graphicscore::drawing` submodule.
    ///
    /// # Example
    ///
    /// This example fills two textures with different colors and then blits them on window.
    /// Note the usage of additional 'indexing' of the textures!
    ///
    /// ```rust, no_run
    /// # use ggengine::GGEngine;
    /// # use ggengine::utils::Window;
    /// # use ggengine::graphicscore::drawing::{Canvas, WindowCanvas};
    /// # use ggengine::graphicscore::textures::{Texture, TextureCreator, AccessType};
    /// # use ggengine::mathcore::{{Angle, Size, Color}, vectors::Point, shapes::Rect};
    /// let engine: GGEngine = GGEngine::init();
    /// let window: Window = engine.build_window("ggengine", 1000, 1000, Default::default());
    /// let mut canvas: WindowCanvas = WindowCanvas::from_window(window, true);
    ///
    /// let texture_creator: TextureCreator = canvas.texture_creator();
    /// let mut texture1: Texture = texture_creator.create_texture(
    ///     100, 100,
    ///     texture_creator.default_pixel_format(),
    ///     AccessType::Targeted
    /// );
    /// let mut texture2: Texture = texture_creator.create_texture(
    ///     100, 100,
    ///     texture_creator.default_pixel_format(),
    ///     AccessType::Targeted
    /// );
    ///
    /// let mut package: Vec<(Color, &mut Texture)> = vec![
    ///     (Color::RED, &mut texture1),
    ///     (Color::GREEN, &mut texture2)
    /// ];
    /// canvas.manage_textures(
    ///     &mut package,
    ///     |texture_canvas, index| {
    ///         texture_canvas.set_draw_color(*index);
    ///         texture_canvas.clear();
    ///     }
    /// );
    ///
    /// canvas.blit_from_texture(
    ///     Some(Rect::from_origin(
    ///         Point { x: 100.0, y: 100.0 },
    ///         Angle::ZERO,
    ///         Size::try_from(100.0).expect("Value is in correct range."),
    ///         Size::try_from(100.0).expect("Value is in correct range.")
    ///     )),
    ///     &texture1,
    ///     None
    /// );
    /// canvas.blit_from_texture(
    ///     Some(Rect::from_origin(
    ///         Point { x: 400.0, y: 400.0 },
    ///         Angle::ZERO,
    ///         Size::try_from(100.0).expect("Value is in correct range."),
    ///         Size::try_from(100.0).expect("Value is in correct range.")
    ///     )),
    ///     &texture2,
    ///     None
    /// );
    /// canvas.update();
    /// ```
    ///
    pub fn manage_textures<'managing, 'texture: 'managing, Index: 'managing>(
        &mut self,
        textures: &'managing mut [(Index, &'managing mut Texture<'texture>)],
        f: fn(&mut TextureCanvas, &Index) -> (),
    ) {
        if !self.supports_texture_management() {
            return;
        }
        let textures = textures
            .iter_mut()
            .filter(|(_, texture)| texture.access_type() == AccessType::Targeted)
            .map(|(ref index, ref mut texture)| (texture.get_sdl_texture_mut(), index))
            .collect::<Vec<_>>();
        self.canvas
            .with_multiple_texture_canvas(textures.iter(), |canvas, index| {
                f(&mut TextureCanvas { canvas }, *index)
            })
            .expect("`ggengine` should be able to initialize canvas from the texture");
    }

    /// Updates the image on the window.
    ///
    /// `ggengine` does not draw directly to the window, it draws to the canvas buffer.
    /// To commit your work you need to call `update`
    /// (this function is called automatically for images and textures after your work).
    ///
    pub fn update(&mut self) {
        self.canvas.present();
    }
}
impl fmt::Debug for WindowCanvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WindowCanvas")
    }
}
impl_canvas!(
    WindowCanvas,
    TextureCreator::from_sdl_texture_creator_window
);
