//! `ggengine::mathcore` module is a core that implements all math functionality for engine.
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
