//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types
//! to exact [`GameObject`](super::gameobjects::GameObject),
//! and implements several basic components used in games.
//!

use std::{
    any::{type_name, Any},
    fmt,
};

pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any> Component for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Component ({:?})", type_name::<Self>())
    }
}
