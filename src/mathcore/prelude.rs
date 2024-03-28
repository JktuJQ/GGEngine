//! Prelude module: `mathcore::prelude` re-exports all `ggengine::mathcore` items.
//!
//! # Examples
//! Import all the exports.
//!
//! ```rust
//! use ggengine::mathcore::prelude::*;
//! ```
//!

// re-exports
pub use crate::mathcore::collisions::*;
pub use crate::mathcore::floats::*;
pub use crate::mathcore::matrices::*;
pub use crate::mathcore::shapes::*;
pub use crate::mathcore::transforms::*;
pub use crate::mathcore::vectors::*;
pub use crate::mathcore::*;
