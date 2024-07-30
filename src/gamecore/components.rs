//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types
//! to exact [`GameObject`](super::gameobjects::GameObject),
//! and implements several basic components used in games.
//!

use std::{any::type_name, fmt};

pub trait Component: 'static {}
impl<T: 'static> Component for T {}

impl fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Component ({:?})", type_name::<Self>())
    }
}
