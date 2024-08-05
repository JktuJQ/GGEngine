//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types
//! to exact [`GameObject`](super::gameobjects::GameObject),
//! and implements several basic components used in games.
//!

use std::{
    any::{type_name, Any},
    fmt,
};

/// `as_any` hidden module is needed to hide trait upcasting workaround.
/// 
/// Currently, trait upcasting is not stable,
/// but it is needed to use runtime reflextion (`&dyn Component` to `&dyn Any`).
/// Workaround is implemented in `AsAny` trait
/// 
pub(in crate::gamecore) mod as_any {
    use std::any::Any;

    /// [`AsAny`] trait is a workaround for trait upcasting.
    /// 
    /// [`AsAny`] trait blanked implementation for all types that implement `Any`
    /// allows for `&self` to coerce to `&dyn Any`.
    /// 
    pub trait AsAny {
        /// Method that coerces `&self` to `&dyn Any`.
        /// 
        fn as_any(&self) -> &dyn Any;
    }
    impl<T: Any> AsAny for T {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
}

pub trait Component: Any + as_any::AsAny {}
impl<T: Any> Component for T { }

impl fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Component ({:?})", type_name::<Self>())
    }
}
