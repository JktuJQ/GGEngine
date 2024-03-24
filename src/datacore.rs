//! `ggengine::datacore` module is a core that supplies structs and functions
//! that are needed to work with different formats of external data.
//!
//! Since this core heavily relies on sdl2 library,
//! there are several places where those structs are just re-exported with new names as a part of `ggengine` crate,
//! although most of the interface is abstracted away.
//!

// submodules and public re-exports
pub mod audio;
