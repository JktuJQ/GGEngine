//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types to entity,
//! and implements several basic components used in games.
//!

use std::{
    any::{type_name, Any, TypeId},
    array::from_fn,
    fmt,
    mem::swap,
};
use seq_macro::seq;

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
/// Since most types implement `Any`, defining new [`Component`]s could be done like so:
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
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// struct Player;
/// impl Component for Player {}
///
/// struct Name(&'static str);
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
pub trait Component: Any {}
/// Type alias for `Box<dyn Component>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of components is needed.
///
/// `Box<dyn Component>` also allows combining multiple different [`Component`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedComponent = Box<dyn Component>;
impl dyn Component {
    /// Returns true if the inner type is the same as `C`.
    ///
    pub fn is<C: Component>(&self) -> bool {
        let as_any: &dyn Any = self;
        as_any.is::<C>()
    }

    /// Attempts to downcast the box to a concrete type.
    ///
    /// # Note
    /// `downcast` consumes initial `Box`,
    /// but on failure it does not need to, and so it returns it in upcasted form (`Box<dyn Any>`).
    /// Although it would be preferrable to return initial type (`Box<dyn Resource>`),
    /// it is impossible to do so.
    ///
    pub fn downcast<C: Component>(self: Box<Self>) -> Result<Box<C>, Box<dyn Any>> {
        let as_any: Box<dyn Any> = self;
        as_any.downcast::<C>()
    }
    /// Returns some reference to the inner value if it is of type `C`, or `None` if it isn’t.
    ///
    pub fn downcast_ref<C: Component>(&self) -> Option<&C> {
        let as_any: &dyn Any = self;
        as_any.downcast_ref::<C>()
    }
    /// Returns some mutable reference to the inner value if it is of type `C`, or `None` if it isn’t.
    ///
    pub fn downcast_mut<C: Component>(&mut self) -> Option<&mut C> {
        let as_any: &mut dyn Any = self;
        as_any.downcast_mut::<C>()
    }
}
impl fmt::Debug for dyn Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", type_name::<Self>())
    }
}
/// [`ComponentId`] id struct is needed to identify [`Component`]s in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in
/// which entity with this [`Component`] is registered.
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by [`Scene`](super::scenes::Scene).
///
/// Storages operate on ids, which allows them to provide more flexible interface.
/// You can also try to trick type system by providing data that does not correspond to Rust type
/// through id of existing 'fake' type.
///
/// That said, you should use typed API that `ggengine` exposes through several structs,
/// not the API of `ggengine::storages` (unless absolutely needed).
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentId(TypeId);
impl ComponentId {
    /// Returns [`ComponentId`] of given [`Component`] type.
    ///
    pub fn of<C: Component>() -> Self {
        ComponentId(TypeId::of::<C>())
    }
}

/// [`Bundle`] trait defines a static set of [`Component`]s.
///
/// In ECS, components define objects and systems operate on combinations of components.
/// [`Bundle`] trait provides a way to create a set of [`Component`]s that are coupled
/// by some logic, and it just makes sense to use those together.
///
/// Bundles are only a convenient way to initialize new entities and they cannot be used to fetch from those.
/// That is because [`Component`]s in entity are unique
/// (you can't have two components of one type in one entity).
/// As a result, removing one of intersecting bundles might invalidate the other one,
/// which would be rather unexpected in a system that is operating on unremoved bundle.
///
/// # Examples
/// Every [`Component`] is a [`Bundle`], because component is basically a set (bundle) of one component.
/// Additionally, tuples of bundles are also [`Bundle`]
/// (with up to 16 items; if you need more, consider implementing your own [`Bundle`]).
/// This allows you to combine the necessary components into a [`Bundle`].
///
/// For example defining a `PlayerBundle` containing components that describe the player
/// can be written as follows:
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// #[derive(Default)]
/// struct Player;
/// impl Component for Player {}
///
/// #[derive(Default)]
/// struct Name(&'static str);
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
/// # struct Name(&'static str);
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
/// That is where you have three options.
/// 1. You can use extension trait pattern to define constructors for tuples:
/// ```rust
/// # use ggengine::gamecore::components::Component;
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(&'static str);
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
///     fn with_name(name: &'static str) -> Self;
/// }
/// impl WithName for PlayerBundle {
///     fn with_name(name: &'static str) -> PlayerBundle {
///         let mut result: PlayerBundle = PlayerBundle::default();
///         result.1 = Name(name);
///         result
///     }
/// }
///
/// let player: PlayerBundle = PlayerBundle::with_name("Player");
/// ```
///
/// 2. You can leverage provided implementation to construct your own:
/// ```rust
/// # use ggengine::gamecore::components::{Bundle, Component, ComponentId, BoxedComponent};
/// # #[derive(Default)]
/// # struct Player;
/// # impl Component for Player {}
/// #
/// # #[derive(Default)]
/// # struct Name(&'static str);
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
/// impl Bundle<3> for PlayerBundle {
///     fn component_ids() -> [ComponentId; 3] {
///         <(Player, Name, Position)>::component_ids()
///     }
///     fn boxed_components(self) -> [BoxedComponent; 3] {
///         (self.player, self.name, self.position).boxed_components()
///     }
/// }
///
/// let player: PlayerBundle = PlayerBundle {
///     name: Name("Player"),
///     ..Default::default()
/// };
/// ```
/// That approach allows to free yourself from all restrictions,
/// and just 'pack a bundle' at the very end.
///
/// 3. You can manually implement [`Bundle`] trait:
/// ```rust
/// # use ggengine::gamecore::components::{Bundle, Component, ComponentId, BoxedComponent};
/// # use std::iter::once;
/// struct PackedBundle<T> {
///     inner_component: T
/// }
/// impl<T: Component> Bundle<1> for PackedBundle<T> {
///     fn component_ids() -> [ComponentId; 1] {
///         [ComponentId::of::<T>()]
///     }
///     fn boxed_components(self) -> [BoxedComponent; 1] {
///         [Box::new(self.inner_component)]
///     }
/// }
/// ```
///
/// Manual implementations (even those that leverage existing implementations) are rather clunky
/// and susceptible to errors (fairly easy to mistype).
/// You can also provide multiple bundle implementations (of different length) to the same structure,
/// which could be misleading and will prevent Rust from implicitly choosing right implementation
/// for generic functions.
/// With that in mind, you should use implementation for tuples
/// (which is protected by orphan rules from multiple implementations).
///
pub trait Bundle<const N: usize> {
    /// Returns ids of all components that are in the bundle.
    ///
    /// Since that can be done statically ([`Bundle`] is a static set), this function does not take `self`.
    /// Although that requires splitting [`Bundle`] functionality
    /// into two functions (which is more susceptible to errors),
    /// that allows operating on [`Bundle`]s as on types (statically) through `Bundle::component_ids`.
    ///
    fn component_ids() -> [ComponentId; N];
    /// Consumes itself and returns iterator of [`BoxedComponent`]s.
    ///
    fn boxed_components(self) -> [BoxedComponent; N];
}
/// Type alias for `(ComponentId, BoxedComponent)`.
///
/// When [`Bundle`] is used, most of the time both the [`ComponentId`] and the [`BoxedComponent`]
/// are needed; this type alias is specifically for those situations.
///
/// [`bundled_components`] function expresses that need - check docs for more information.
///
pub type BundledComponent = (ComponentId, BoxedComponent);
/// [`bundled_components`] function zips two arrays of [`Bundle`] together.
///
/// Although functionality of [`Bundle`] is split in two functions
/// (`Bundle::component_ids` does not require `self`, which allows operating on [`Bundle`]s as on types),
/// it is still common to use those two arrays simultaneously, which could be done through this function.
///
/// If you only need an `Iterator` that is a result of a zip, manual zipping would be faster.
///
pub fn bundled_components<B: Bundle<N>, const N: usize>(bundle: B) -> [BundledComponent; N] {
    struct NoOpComponent;
    impl Component for NoOpComponent {}

    let component_ids = B::component_ids();
    let mut boxed_components = bundle.boxed_components();
    from_fn(|i| {
        let mut component: Box<dyn Component> = Box::new(NoOpComponent);
        swap(&mut component, &mut boxed_components[i]);
        (component_ids[i], component)
    })
}
impl<T: Component> Bundle<1> for T {
    fn component_ids() -> [ComponentId; 1] {
        [ComponentId::of::<T>()]
    }
    fn boxed_components(self) -> [BoxedComponent; 1] {
        [Box::new(self)]
    }
}
/// [`impl_bundle`] macro implements [`Bundle`] trait for tuples.
///
macro_rules! impl_bundle {
    ($size:tt: $(($t:ident, $index:tt),)*) => {
        impl<$($t: Component,)*> Bundle<$size> for ($($t,)*) {
            fn component_ids() -> [ComponentId; $size] {
                [$(ComponentId::of::<$t>(),)*]
            }
            fn boxed_components(self) -> [BoxedComponent; $size] {
                [$(Box::new(self.$index),)*]
            }
        }
    };
}
seq!(SIZE in 0..=16 {
    #(seq!(N in 0..SIZE { impl_bundle!(SIZE: #((C~N, N),)*); });)*
});

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
/// Since most types implement `Any`, defining new [`Resource`]s could be done like so:
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
pub trait Resource: Any {}
/// Type alias for `Box<dyn Resource>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of resource is needed.
///
/// `Box<dyn Resource>` also allows combining multiple different [`Resource`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedResource = Box<dyn Resource>;
impl dyn Resource {
    /// Returns true if the inner type is the same as `R`.
    ///
    pub fn is<R: Resource>(&self) -> bool {
        let as_any: &dyn Any = self;
        as_any.is::<R>()
    }

    /// Attempts to downcast the box to a concrete type.
    ///
    /// # Note
    /// `downcast` consumes initial `Box`,
    /// but on failure it does not need to, and so it returns it in upcasted form (`Box<dyn Any>`).
    /// Although it would be preferrable to return initial type (`Box<dyn Resource>`),
    /// it is impossible to do so.
    ///
    pub fn downcast<R: Resource>(self: Box<Self>) -> Result<Box<R>, Box<dyn Any>> {
        let as_any: Box<dyn Any> = self;
        as_any.downcast::<R>()
    }
    /// Returns some reference to the inner value if it is of type `R`, or `None` if it isn’t.
    ///
    pub fn downcast_ref<R: Resource>(&self) -> Option<&R> {
        let as_any: &dyn Any = self;
        as_any.downcast_ref::<R>()
    }
    /// Returns some mutable reference to the inner value if it is of type `R`, or `None` if it isn’t.
    ///
    pub fn downcast_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let as_any: &mut dyn Any = self;
        as_any.downcast_mut::<R>()
    }
}
impl fmt::Debug for dyn Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", type_name::<Self>())
    }
}
/// [`ResourceId`] id struct is needed to identify [`Resource`]s in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which this [`Resource`] is registered.
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by [`Scene`](super::scenes::Scene).
///
/// Storages operate on ids, which allows them to provide more flexible interface.
/// You can also try to trick type system by providing data that does not correspond to Rust type
/// through id of existing 'fake' type.
///
/// That said, you should use typed API that `ggengine` exposes through several structs,
/// not the API of `ggengine::storages` (unless absolutely needed).
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(TypeId);
impl ResourceId {
    /// Returns [`ResourceId`] of given [`Resource`] type.
    ///
    pub fn of<R: Resource>() -> Self {
        ResourceId(TypeId::of::<R>())
    }
}
