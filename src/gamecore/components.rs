//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types
//! to exact [`Entity`](super::entities::Entity),
//! and implements several basic components used in games.
//!

use std::{
    any::{type_name, Any, TypeId},
    fmt,
    iter::{empty, once},
};

/// `as_any` hidden module is needed to hide trait upcasting workaround.
///
/// Currently, trait upcasting is not stable,
/// but it is needed to properly use runtime reflection
/// (convert `&dyn Component`/`&dyn Resource` to `&dyn Any`).
/// Workaround is implemented in `AsAny` trait.
///
/// # Note
/// Although `as_any_ref` method could belong to the [`Component`]/[`Resource`] trait itself,
/// it would not fit ECS component meaning (even if it was hidden in docs)
/// and also would require for users to manually implement it (even if it is trivial),
/// so it is instead moved to the hidden trait.
/// This approach also helps to maintain SemVer compatibility. When trait upcasting
/// will be implemented, there will be no use for `as_any` method and it removing it
/// would break compatibility (if `as_any` belonged to [`Component`]/[`Resource`] trait), but
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
        fn as_any_ref(&self) -> &dyn Any;
        /// Method that coerces `&mut self` to `&mut dyn Any`.
        ///
        fn as_any_mut(&mut self) -> &mut dyn Any;
        /// Method that coerces `Box<dyn T>` to `Box<dyn Any>`.
        ///
        fn as_any_box(self: Box<Self>) -> Box<dyn Any>;
    }
    impl<T: Any> AsAny for T {
        fn as_any_ref(&self) -> &dyn Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
        fn as_any_box(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }
}

/// [`Component`] trait defines objects that are components by ECS terminology.
///
/// In ECS, components define objects, almost like in Rust
/// traits define structs. So basically, components just are parts of `Entity`
/// that are responsible for its functionality.
/// ECS pattern encourages clean, decoupled design that
/// splits up your app data and logic into its core components.
///
/// `ggengine` supports having only one component of each type binded to `Entity`.
/// Trying to add two components of one type to `Entity` could lead to unexpected
/// behaviour, as `Entity` will only use the latest component.
///
/// # Implementation
/// [`Component`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Component`] trait, and it requires `'static` trait bound.
///
/// There is also `AsAny` supertrait which may seem a 'seal' trait
/// that would forbid any external implementations - but it is not.
/// `AsAny` trait has blanket implementation for every type that has `Any` implemented
/// and so it is not a constraint at all.
///
/// That is why implementing [`Component`] trait is so easy:
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
/// Type alias for `Box<dyn Component>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of components is needed.
///
/// `Box<dyn Component>` also allows combining multiple different [`Component`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedComponent = Box<dyn Component>;
/// [`ComponentId`] id struct is needed to identify [`Component`]s in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in
/// which entity with this [`Component`] is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(pub(super) u64);

/// [`Bundle`] trait defines a set of [`Component`]s.
///
/// In ECS, components define objects and systems operate on combinations of components.
/// [`Bundle`] trait provides a way to create a set of [`Component`]s that are coupled
/// by some logic, and it just makes sense to use those together.
///
/// Bundles are only a convenient way to group components in a set, and they should
/// not be used as units of behaviour. That is because multiple bundles could contain
/// the same [`Component`] type, and adding both of them to one
/// [`Entity`](super::entities::Entity) would lead to unexpected behaviour
/// (see [`Component`] trait docs).
/// For this reason it is impossible to use [`Bundle`] for querying. Instead, you should
/// operate on [`Component`]s which define your game logic, querying those you need to use.
///
/// # Examples
/// Every [`Component`] is a [`Bundle`], because component is basically a set (bundle) of one component.
/// Additionally, tuples of bundles are also [`Bundle`] (with up to 12 items,
/// but those tuples can be nested, which practically removes that bound).
/// This allows you to combine the necessary components into a [`Bundle`].
///
/// [`Bundle`] specifically requires [`BoxedComponent`]s to be returned from `Bundle::components`,
/// and that is because iterator of one [`Component`] type would be
///
/// for example defining a `PlayerBundle` containing components that describe the player
/// can be written as follows:
///
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// #[derive(Default)]
/// struct Player;
/// impl Component for Player {}
///
/// #[derive(Default)]
/// struct Name(String);
/// impl Component for Name {}
///
/// #[derive(Default)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
/// impl Component for Position {}
///
/// type PlayerBundle = (Player, Name, Position);
/// ```
///
/// You might want to customize initialization of your [`Bundle`].
/// You can, of course, use `Default::default()`:
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(String);
/// # impl Component for Name {}
/// #
/// # #[derive(Default)]
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// # impl Component for Position {}
/// #
/// type PlayerBundle = (Player, Name, Position);
///
/// let player: PlayerBundle = Default::default();
/// ```
/// However, tuples do not support the struct update syntax
/// and for complex cases, their initialization is inconvenient.
///
/// That is where you have two options.
/// 1. You can use extension trait pattern to define constructors
/// for tuples:
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(String);
/// # impl Component for Name {}
/// #
/// # #[derive(Default)]
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// # impl Component for Position {}
/// #
/// type PlayerBundle = (Player, Name, Position);
///
/// trait WithName {
///     fn with_name(name: String) -> Self;
/// }
/// impl WithName for PlayerBundle {
///     fn with_name(name: String) -> PlayerBundle {
///         let mut result: PlayerBundle = PlayerBundle::default();
///         result.1 = Name(name);
///         result
///     }
/// }
///
/// let player: PlayerBundle = PlayerBundle::with_name("Player".to_string());
/// ```
///
/// 2. You can implement [`Bundle`] trait leveraging provided implementation to construct your own:
/// ```rust
/// # use ggengine::gamecore::components::{Bundle, Component, BoxedComponent};
/// # use std::any::TypeId;
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(String);
/// # impl Component for Name {}
/// #
/// # #[derive(Default)]
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// # impl Component for Position {}
/// #
/// #[derive(Default)]
/// struct PlayerBundle {
///     player: Player,
///     name: Name,
///     position: Position,
/// }
/// impl Bundle for PlayerBundle {
///     fn components(self) -> impl IntoIterator<Item = (TypeId, BoxedComponent)> {
///         (self.player, self.name, self.position).components()
///     }
/// }
///
/// let player: PlayerBundle = PlayerBundle {
///     name: Name("Player".to_string()),
///     ..Default::default()
/// };
/// ```
/// That approach allows to free yourself from all restrictions,
/// and just 'pack a bundle' at the very end.
///
/// # Manual implementation
/// `Bundle::components` function must return `impl IntoIterator<Item = (TypeId, BoxedComponent)>`.
/// That `TypeId` is needed for dynamic dispatch of [`Component`] types in the ECS storage.
/// Implementation of this trait should ensure that every `TypeId` matches with
/// the type of corresponding [`Component`] that is boxed.
///
/// Here is an example on how to correctly implement [`Bundle`] trait:
/// ```rust
/// # use ggengine::gamecore::components::{BoxedComponent, Bundle, Component};
/// # use std::any::TypeId;
/// # use std::iter::once;
/// struct PackedBundle<T> {
///     inner_component: T
/// }
/// impl<T: Component> Bundle for PackedBundle<T> {
///     fn components(self) -> impl IntoIterator<Item=(TypeId, BoxedComponent)> {
///         let boxed_component: BoxedComponent = Box::new(self.inner_component);
///         once((TypeId::of::<T>(), boxed_component))
///     }
/// }
/// ```
/// `ggengine` advises not to implement [`Bundle`] manually unless you really need it.
///
pub trait Bundle {
    /// Consumes itself and returns list of [`BoxedComponent`]s.
    ///
    fn components(self) -> impl IntoIterator<Item = (TypeId, BoxedComponent)>;
}
impl<T: Component> Bundle for T {
    fn components(self) -> impl IntoIterator<Item = (TypeId, BoxedComponent)> {
        let boxed_self: BoxedComponent = Box::new(self);
        once((TypeId::of::<T>(), boxed_self))
    }
}
/// [`impl_bundle`] macro implements [`Bundle`] trait for tuples of arity 12 or less.
///
macro_rules! impl_bundle {
    () => {
        impl Bundle for ()
        {
            fn components(self) -> impl IntoIterator<Item = (TypeId, BoxedComponent)> {
                empty()
            }
        }
    };
    (($t_start:ident, $index_start:tt), $(($t:ident, $index:tt),)*) => {
        impl<$t_start: Bundle, $($t: Bundle),*> Bundle for ($t_start, $($t,)*)
        {
            fn components(self) -> impl IntoIterator<Item = (TypeId, BoxedComponent)> {
                self.$index_start.components().into_iter()$(.chain(self.$index.components().into_iter()))*
            }
        }
        impl_bundle!($(($t, $index),)*);
    };
}
impl_bundle!(
    (T00, 11),
    (T01, 10),
    (T02, 9),
    (T03, 8),
    (T04, 7),
    (T05, 6),
    (T06, 5),
    (T07, 4),
    (T08, 3),
    (T09, 2),
    (T10, 1),
    (T11, 0),
);

/// [`Resource`] trait defines unique global data that is bounded to the `Scene`.
///
/// [`Resource`]s are very similar to [`Component`]s, with the only difference is that
/// [`Component`]s are bounded to `Entity`s and [`Resource`]s are bound to the `Scene`.
///
/// Applications often have some global data that they share, it could be time, score, asset collection, etc.
/// Although global resources could be implemented as components that belong to some 'global' Entity,
/// that would be confusing and would not convey intention logic.
/// [`Resource`] trait supports this pattern, enforcing it through type system and allowing
/// for data to be shared easily.
///
/// # Implementation
/// [`Resource`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Resource`] trait, and it requires `'static` trait bound.
///
/// There is also `AsAny` supertrait which may seem a 'seal' trait
/// that would forbid any external implementations - but it is not.
/// `AsAny` trait has blanket implementation for every type that has `Any` implemented
/// and so it is not a constraint at all.
///
/// That is why implementing [`Resource`] trait is so easy:
///
/// ```rust
/// use ggengine::gamecore::components::Resource;
/// struct T;
/// impl Resource for T {}
/// ```
///
/// Considering that [`Resource`] is basically a [`Component`], almost everything
/// that goes for [`Component`]s is also true for [`Resource`]s.
/// To further understand relations between those traits, it is encouraged to read docs for submodule items.
///
pub trait Resource: Any + as_any::AsAny {}
impl fmt::Debug for dyn Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Resource ({:?})", type_name::<Self>())
    }
}
/// Type alias for `Box<dyn Resource>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of resource is needed.
///
/// `Box<dyn Resource>` also allows combining multiple different [`Resource`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedResource = Box<dyn Resource>;
/// [`ResourceId`] id struct is needed to identify [`Resource`]s in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which this [`Resource`] is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(pub(super) u64);
