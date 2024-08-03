//! # GGEngine
//!
//! **GGEngine** - 2d game engine written in pure Rust that implements Unity-like interface.
//!

#![warn(missing_docs, clippy::missing_docs_in_private_items)] // `missing_docs`
#![warn(unused_import_braces, unused_qualifications, unused_results)] // `unused_*`
#![warn(trivial_casts, trivial_numeric_casts)] // `casts`
#![warn(missing_copy_implementations, missing_debug_implementations)] // `missing_*_implementations`
#![warn(variant_size_differences, unreachable_pub)]

// crates
extern crate bitflags;

extern crate sdl2;

extern crate serde;
extern crate serde_big_array;
extern crate serde_cbor;

// utils
mod ggengine;
pub use crate::ggengine::*;

pub mod utils;

// cores
pub mod datacore;
pub mod gamecore;
pub mod graphicscore;
pub mod mathcore;
