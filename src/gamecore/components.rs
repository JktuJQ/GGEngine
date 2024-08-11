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
/// but it is needed to properly use runtime reflection
/// (convert `&dyn Component` to `&dyn Any`).
/// Workaround is implemented in `AsAny` trait.
///
/// # Note
/// Although `as_any` method could belong to the [`Component`] trait itself,
/// it would not fit ECS component meaning (even if it was hidden in docs)
/// and also would require for users to manually implement it (even if it is trivial),
/// so it is instead moved to the hidden trait.
/// This approach also helps mantaining SemVer compatibility. When trait upcasting
/// will be implemented, there will be no use for `as_any` method and it removing it
/// would break compatibility (if `as_any` belonged to [`Component`] trait), but
/// current implementation would allow to just delete `AsAny` trait completely
/// without breaking compatibility.
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

/// [`Component`] trait defines objects that are components by ECS terminology.
///
/// In ECS, components define objects, almost like in Rust
/// traits define structs. So basically, components just are parts of `GameObject`
/// that are responsible for its functionality.
/// ECS pattern encourages clean, decoupled design that
/// splits up your app data and logic into its core components.
///
/// # Implementation
/// [`Component`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Component`] trait and it requires `'static` trait bound.
///
/// There is also `AsAny` supertrait which may seem a 'seal' trait
/// that would forbid any external implementations - but it is not!
/// `AsAny` trait has blanket implementation for every type that has `Any` implemented
/// and so it is not a constraint at all.
/// That why implementing [`Component`] trait is so easy:
///
/// ```rust
/// use ggengine::gamecore::components::Component;
/// struct T;
/// impl Component for T {}
/// ```
///
/// # Examples
/// Any Rust type that fits [`Component`]'s constraints can be a [`Component`].
/// They are usually structs, but can also be enums or zero sized types.
/// The following example shows how one might define components for RPG:
///
/// ```rust
/// # use ggengine::gamecore::components::Component;
///
/// struct Player;
/// impl Component for Player {}
///
/// struct Name(String);
/// impl Component for Name {}
///
/// struct Position {
///     x: f32,
///     y: f32,
/// }
/// impl Component for Position {}
///
/// enum Race {
///     Elf,
///     Dwarf,
///     Human,
///     Orc,
/// }
/// impl Component for Race {}
///
/// enum Weapon {
///     Sword,
///     Spear,
///     Bow {
///         arrows: u32,
///     },
/// }
/// impl Component for Weapon {}
/// ```
///
pub trait Component: Any + as_any::AsAny {}
impl fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Component ({:?})", type_name::<Self>())
    }
}
