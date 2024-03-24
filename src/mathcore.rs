//! `ggengine::mathcore` module is a core that implements all math functionality for engine.
//!
//! # Prelude
//! `ggengine::mathcore` prelude can be imported with `use ggengine::mathcore::prelude::*`.
//!
//! # Model
//! There are several very important constructs that are essential to game engine.
//! Vectors usually represent directions and coordinates.
//! With that in mind, it's natural to implement transformations of objects as
//! matrices.
//! Shapes are represented by their vertices (vectors), so translation, rotation and other transformations are easy to apply.
//! Shape collision is checked using geometry.
//!
//! That model is quite common for many game engines,
//! so almost everyone with basic knowledge of game developing should be familiar with `ggengine`.
//!

// submodules and public re-exports
mod ext;
pub use ext::*;

pub mod collisions;
pub mod floats;
pub mod matrices;
pub mod shapes;
pub mod transforms;
pub mod vectors;

// prelude
pub mod prelude;
