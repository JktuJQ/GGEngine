//! `ggengine` hidden submodule implements [`GGEngine`] struct that handles
//! all subsystems that are needed for `ggengine` work.
//!

use sdl2::{init as sdl_initialization, Sdl, VideoSubsystem as SdlVideoSubsystem};
use std::fmt;

/// [`GGEngine`] struct handles global context for `ggengine`.
///
/// This struct uses underlying handler that is local to the main thread, so
/// most of the functionality of engine is going to work only on main thread,
/// which, for example, ensures that event handling is bound to the thread where [`GGEngine`] was
/// initialized.
///
/// [`GGEngine`] struct initializes **ONLY** basic subsystems that are needed for work -
/// that includes video system and event system;
/// other subsystems such as audio, images and fonts systems should be initialized manually.
///
/// # Example
/// ```rust, no_run
/// # use ggengine::{GGEngine, utils::Window};
/// let engine: GGEngine = GGEngine::init();
/// let window: Window = engine.build_window("GGENGINE", 1600, 900, Default::default());
/// ```
///
pub struct GGEngine {
    /// Underlying `sdl2` context handler.
    ///
    sdl: Sdl,
    /// Underlying video subsystem.
    ///
    video: SdlVideoSubsystem,
}
impl GGEngine {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns reference to underlying `VideoSubsystem` handler.
    ///
    pub(crate) fn get_sdl_videosubsystem(&self) -> &SdlVideoSubsystem {
        &self.video
    }

    /// Internally initializes global handler for `ggengine` library.
    ///
    /// This function loads and prepares all submodules for usage.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::GGEngine;
    /// let engine: GGEngine = GGEngine::init();
    /// ```
    ///
    pub fn init() -> GGEngine {
        let sdl = sdl_initialization()
            .expect("`ggengine` should be able to initialize underlying `sdl2` handler");
        let video = sdl
            .video()
            .expect("`ggengine` should be able to initialize underlying `video` handler");
        GGEngine { sdl, video }
    }
}
impl fmt::Debug for GGEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GGEngine").finish()
    }
}
