//! `utils` module defines several constructs that are essential to the
//! application powered by `ggengine`.
//!
//! This module can be thought of as a collection of abstractions over OS.
//!

use crate::{
    datacore::images::{Image, PixelFormat},
    mathcore::vectors::Vector2Int,
    GGEngine,
};
use sdl2::video::{
    DisplayMode as SdlDisplayMode, FlashOperation as SdlFlashOperation,
    FullscreenType as SdlFullscreenType, Window as SdlWindow, WindowBuilder as SdlWindowBuilder,
    WindowPos as SdlWindowPos,
};
use std::fmt;

/// [`Position`] enum encapsulates possible position settings.
///
/// Example of usage is shown in [`WindowSettings`] docs.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Position {
    /// Exact position that is given by coordinate (left top corner is `(0, 0)`).
    ///
    Exact(Vector2Int),
    /// Centered position.
    ///
    Centered,
}
/// [`FullscreenType`] lists types of fullscreen that are applicable to window.
///
/// Example of usage is shown in [`WindowSettings`] docs.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FullscreenType {
    /// Fullscreen is a real fullscreen mode - OS changes video mode for your application, but
    /// tabbing to another program might switch video mode back.
    ///
    Fullscreen,
    /// Desktop fullscreen is a 'fake' fullscreen - the actual video mode would not change.
    ///
    DesktopFullscreen,
}
impl FullscreenType {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Converts `sdl2` SdlFullscreenType to [`FullscreenType`].
    ///
    /// `None` corresponds to fullscreen not being enabled.
    ///
    pub(crate) fn from_sdl_fullscreen_type(
        fullscreen_type: SdlFullscreenType,
    ) -> Option<FullscreenType> {
        match fullscreen_type {
            SdlFullscreenType::Off => None,
            SdlFullscreenType::True => Some(FullscreenType::Fullscreen),
            SdlFullscreenType::Desktop => Some(FullscreenType::DesktopFullscreen),
        }
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns `sdl2` representation of this enum.
    ///
    pub(crate) fn to_sdl_fullscreen_type(this: Option<FullscreenType>) -> SdlFullscreenType {
        match this {
            Some(FullscreenType::Fullscreen) => SdlFullscreenType::True,
            Some(FullscreenType::DesktopFullscreen) => SdlFullscreenType::Desktop,
            None => SdlFullscreenType::Off,
        }
    }
}
/// [`InitialSizing`] enum lists possible states for window initial sizing.
///
/// Size that was given to window will be preserved, but window itself can be minimized or maximized.
///
/// Example of usage is shown in [`WindowSettings`] docs.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum InitialSizing {
    /// Window will be minimized upon initialization.
    ///
    Minimized,
    /// Window will be maximized upon initialization.
    ///
    Maximized,
}
/// [`WindowSettings`] struct carries data that is needed for window configuration.
///
/// If you do not want to tweak settings, just pass `..Default::default()` to fill up remaining options.
///
/// # Examples
/// ```rust
/// # use ggengine::utils::{WindowSettings, Position, FullscreenType, InitialSizing};
/// let window: WindowSettings = WindowSettings {
///     position: Some(Position::Centered),
///     initial_fullscreen: Some(FullscreenType::Fullscreen),
///     initial_sizing: Some(InitialSizing::Maximized),
///     ..WindowSettings::default()
/// };
/// ```
///
#[derive(Debug, Copy, Clone)]
pub struct WindowSettings {
    /// Initial window position.
    ///
    pub position: Option<Position>,

    /// Fullscreen mode of the window.
    ///
    pub initial_fullscreen: Option<FullscreenType>,
    /// Decides whether the window will always be on top or not.
    ///
    pub always_on_top: bool,

    /// Decides whether the window will be resizable or not.
    ///
    pub is_resizable: bool,
    /// Initial sizing of the window.
    ///
    pub initial_sizing: Option<InitialSizing>,

    /// Decides whether the window will be hidden or not.
    ///
    pub is_hidden: bool,
    /// Decides whether the window will be borderless or not.
    ///
    pub is_borderless: bool,
    /// Decides whether the window will allow high dpi or not.
    ///
    pub allow_high_dpi: bool,
}
impl WindowSettings {
    /// Applies settings to `sdl2` WindowBuilder.
    ///
    fn apply_to_builder(self, window_builder: &mut SdlWindowBuilder) -> &mut SdlWindowBuilder {
        if let Some(position) = self.position {
            let _ = match position {
                Position::Exact(Vector2Int { x, y }) => window_builder.position(x, y),
                Position::Centered => window_builder.position_centered(),
            };
        }
        if let Some(fullscreen_type) = self.initial_fullscreen {
            let _ = match fullscreen_type {
                FullscreenType::Fullscreen => window_builder.fullscreen(),
                FullscreenType::DesktopFullscreen => window_builder.fullscreen_desktop(),
            };
        }
        if self.always_on_top {
            let _ = window_builder.always_on_top();
        }
        if self.is_resizable {
            let _ = window_builder.resizable();
        }
        if let Some(initial_size) = self.initial_sizing {
            let _ = match initial_size {
                InitialSizing::Minimized => window_builder.minimized(),
                InitialSizing::Maximized => window_builder.maximized(),
            };
        }
        if self.is_hidden {
            let _ = window_builder.hidden();
        }
        if self.is_borderless {
            let _ = window_builder.borderless();
        }
        if self.allow_high_dpi {
            let _ = window_builder.allow_highdpi();
        }
        window_builder
    }
}
impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings {
            position: None,

            initial_fullscreen: None,
            always_on_top: false,

            is_resizable: true,
            initial_sizing: None,

            is_hidden: false,
            is_borderless: false,
            allow_high_dpi: true,
        }
    }
}

/// [`Ping`] enum lists possible ping modes.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Ping {
    /// Window will ping one time.
    Briefly,
    /// Window will be pinging until user focuses on it.
    ///
    UntilFocused,
}
impl Ping {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns `sdl2` representation of this enum.
    ///
    /// `None` corresponds to absence of pinging.
    ///
    pub(crate) fn to_sdl_flash_operation(this: Option<Ping>) -> SdlFlashOperation {
        match this {
            Some(Ping::Briefly) => SdlFlashOperation::Briefly,
            Some(Ping::UntilFocused) => SdlFlashOperation::UntilFocused,
            None => SdlFlashOperation::Cancel,
        }
    }
}

impl GGEngine {
    /// Builds window with given settings.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::{GGEngine, utils::Window};
    /// let engine: GGEngine = GGEngine::init();
    /// let window: Window = engine.build_window("GGENGINE", 1600, 900, Default::default());
    /// ```
    ///
    pub fn build_window(
        &self,
        title: &str,
        width: u32,
        height: u32,
        window_settings: WindowSettings,
    ) -> Window {
        Window {
            window: window_settings
                .apply_to_builder(&mut self.get_sdl_videosubsystem().window(title, width, height))
                .build()
                .expect("`ggengine` should be able to build a window (maybe incompatible symbols are given or given size is too big)"),
        }
    }
}
/// [`Window`] struct represents the shell of OS window.
///
/// This struct only allows manipulations with window properties, but it does not allow
/// direct pixel access.
/// To use window for drawing you should use corresponding rendering functions from `ggengine`.
///
/// # Example
/// ```rust, no_run
/// # use ggengine::{GGEngine, utils::Window};
/// let engine: GGEngine = GGEngine::init();
/// let window: Window = engine.build_window("GGENGINE", 1600, 900, Default::default());
/// ```
///
pub struct Window {
    /// Underlying `sdl2` window.
    ///
    window: SdlWindow,
}
impl Window {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Initializes [`Window`] from `sdl2` window.
    ///
    pub(crate) fn from_sdl_window(window: SdlWindow) -> Window {
        Window { window }
    }
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Destructures itself by consuming [`Window`].
    ///
    pub(crate) fn destructure(self) -> SdlWindow {
        self.window
    }

    /// Returns id of the window.
    ///
    pub fn id(&self) -> u32 {
        self.window.id()
    }

    /// Sets new refresh rate to the window.
    ///
    pub fn set_refresh_rate(&mut self, refresh_rate: u16) {
        self.window
            .set_display_mode(Some(SdlDisplayMode {
                refresh_rate: i32::from(refresh_rate),
                ..self
                    .window
                    .display_mode()
                    .expect("`ggengine` should be able to get display mode")
            }))
            .expect("`ggengine` should be able to set new refresh rate to window")
    }
    /// Returns current refresh rate of the window.
    ///
    pub fn refresh_rate(&self) -> u16 {
        u16::try_from(
            self.window
                .display_mode()
                .expect("`ggengine` should be able to get display mode")
                .refresh_rate,
        )
        .expect("Conversion should not fail")
    }

    /// Sets new pixel format for the window.
    ///
    /// You should call this function only if you really know what you are doing -
    /// default pixel format for window is the most optimised, so you can degrade performance severely.
    ///
    pub fn set_pixel_format(&mut self, pixel_format: PixelFormat) {
        self.window
            .set_display_mode(Some(SdlDisplayMode {
                format: PixelFormat::to_sdl_pixel_format_enum(pixel_format),
                ..self
                    .window
                    .display_mode()
                    .expect("`ggengine` should be able to get display mode")
            }))
            .expect("`ggengine` should be able to set pixel format to window")
    }
    /// Returns window's pixel format or `None`, if format wasn't recognised.
    ///
    /// Even if format was not recognised, all `Window` methods would still work.
    ///
    pub fn pixel_format(&self) -> Option<PixelFormat> {
        PixelFormat::from_sdl_pixel_format_enum(
            self.window
                .display_mode()
                .expect("`ggengine` should be able to get display mode")
                .format,
        )
    }

    /// Sets new title for the window.
    ///
    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title).expect(
            "`ggengine` should be able to rename title (maybe incompatible symbols are used)",
        );
    }
    /// Returns window's title.
    ///
    pub fn title(&self) -> &str {
        self.window.title()
    }

    /// Sets icon for the window.
    ///
    /// `ggengine` might not be able to display big icons, so you should experiment with icon size.
    /// 64x64 image size should be displayable, so it's preferred size.
    ///
    pub fn set_icon(&mut self, icon: &Image) {
        self.window.set_icon(icon.get_sdl_surface());
    }

    /// Sets new position of the window.
    ///
    pub fn set_position(&mut self, position: Vector2Int) {
        self.window.set_position(
            SdlWindowPos::Positioned(position.x),
            SdlWindowPos::Positioned(position.y),
        );
    }
    /// Returns current position of the window.
    ///
    pub fn position(&self) -> Vector2Int {
        let (x, y): (i32, i32) = self.window.position();
        Vector2Int::from([x, y])
    }

    /// Sets new size for the window.
    ///
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.window
            .set_size(width, height)
            .expect("`ggengine` should be able to resize window (maybe given size is too big)");
    }
    /// Returns current window size.
    ///
    pub fn size(&self) -> (u32, u32) {
        self.window.size()
    }

    /// Restores window size after minimisation or maximisation.
    ///
    pub fn restore_size(&mut self) {
        self.window.restore();
    }

    /// Sets minimal possible size for the window.
    ///
    pub fn set_minimal_size(&mut self, width: u32, height: u32) {
        self.window.set_minimum_size(width, height).expect("`ggengine` should be able to set minimal size for window (maybe given size is too big)");
    }
    /// Returns window's minimal size.
    ///
    pub fn minimal_size(&self) -> (u32, u32) {
        self.window.minimum_size()
    }
    /// Minimises the window.
    ///
    pub fn minimize(&mut self) {
        self.window.minimize()
    }
    /// Returns whether the window is minimised or not.
    ///
    pub fn is_minimized(&self) -> bool {
        self.window.is_minimized()
    }

    /// Sets maximal possible size for the window.
    ///
    pub fn set_maximal_size(&mut self, width: u32, height: u32) {
        self.window.set_maximum_size(width, height).expect("`ggengine` should be able to set maximal size for window (maybe given size is too big)");
    }
    /// Returns window's maximal size.
    ///
    pub fn maximal_size(&self) -> (u32, u32) {
        self.window.maximum_size()
    }
    /// Maximises the window.
    ///
    pub fn maximize(&mut self) {
        self.window.maximize();
    }
    /// Returns whether the window is maximised or not.
    ///
    pub fn is_maximized(&self) -> bool {
        self.window.is_maximized()
    }

    /// Sets new fullscreen type for the window.
    ///
    /// If `None` is passed, disables current fullscreen type.
    ///
    pub fn set_fullscreen_type(&mut self, fullscreen_type: Option<FullscreenType>) {
        self.window
            .set_fullscreen(FullscreenType::to_sdl_fullscreen_type(fullscreen_type))
            .expect("`ggengine` should be able to set fullscreen type");
    }
    /// Returns current window fullscreen type.
    ///
    /// `None` corresponds to fullscreen not being enabled.
    ///
    pub fn fullscreen_type(&self) -> Option<FullscreenType> {
        FullscreenType::from_sdl_fullscreen_type(self.window.fullscreen_state())
    }

    /// Sets the window always on top of everything else if `true` is passed.
    /// `false` disables it.
    ///
    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        self.window.set_always_on_top(always_on_top);
    }
    /// Returns whether the window would always be on top or not.
    ///
    pub fn is_always_on_top(&self) -> bool {
        self.window.is_always_on_top()
    }

    /// Requests a window to demand attention from the user by pinging.
    ///
    pub fn window_pinging(&mut self, ping: Option<Ping>) {
        self.window
            .flash(Ping::to_sdl_flash_operation(ping))
            .expect("`ggengine` should be able to ping window")
    }

    /// Grabs keyboard focus to the window if `true` is passed.
    /// `false` removes focus.
    ///
    pub fn grab_keyboard(&mut self, grab: bool) {
        self.window.set_keyboard_grab(grab);
    }
    /// Returns whether keyboard focus is grabbed onto window or not.
    ///
    pub fn is_keyboard_grabbed(&self) -> bool {
        self.window.keyboard_grab()
    }

    /// Grabs mouse focus to the window if `true` is passed.
    /// `false` removes focus.
    ///
    pub fn grab_mouse(&mut self, grab: bool) {
        self.window.set_mouse_grab(grab);
    }
    /// Returns whether mouse focus is grabbed onto window or not.
    ///
    pub fn is_mouse_grabbed(&self) -> bool {
        self.window.mouse_grab()
    }
}
impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!("Window {id}", id = self.id()))
            .finish()
    }
}
