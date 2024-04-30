//! `ggengine::datacore` module is a core that supplies structs and functions
//! that are needed to work with different formats of external data.
//!
//! Since this core heavily relies on `sdl2` library,
//! there are several places where those structs are just re-exported with new names as a part of `ggengine` crate,
//! although most of the interface is abstracted away.
//!
//! # Prelude
//! `ggengine::datacore` prelude can be imported with `use ggengine::datacore::prelude::*`.
//!
//! # Usage
//! `datacore` module provides several structs for work with external data, such as audio, images and
//! fonts. It also implements `AssetManager` that encapsulates work with filesystem.
//!
//! This module is similar to `mathcore` in that sense that both are 'helpers' for game engine
//! implementation.
//!

// submodules and public re-exports
pub mod assets;
pub mod audio;
pub mod fonts;
pub mod images;

// prelude
pub mod prelude;
