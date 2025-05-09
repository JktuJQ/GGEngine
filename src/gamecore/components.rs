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
pub trait Component: Any {}
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ComponentId(TypeId);
impl ComponentId {
    /// Returns [`ComponentId`] of given [`Component`] type.
    ///
    pub(super) fn of<C: Component>() -> Self {
        ComponentId(TypeId::of::<C>())
    }
}

/// [`BundledComponent`] represents component with its type metadata.
///
/// `ggengine` uses dynamic dispatch to implement ECS architecture,
/// and so, it needs information about types.
/// [`BoxedComponent`] allows erasing type to restore it later,
/// but without type hint that will be impossible.
/// [`BundledComponent`] solves that problem - constructing this struct
/// automatically records necessary metadata.
///
/// # Usage
/// This struct is most commonly used in [`Bundle`] implementations,
/// because it ensures that the implementations could not mismatch type information.
///
#[derive(Debug)]
pub struct BundledComponent {
    /// Type metadata of component.
    ///
    component_id: ComponentId,
    /// Boxed component.
    ///
    boxed_component: BoxedComponent,
}
impl BundledComponent {
    /// Constructs [`BundledComponent`] from [`Component`],
    /// additionally recording necessary metadata.
    ///
    pub fn bundle<C: Component>(component: C) -> BundledComponent {
        BundledComponent {
            component_id: ComponentId::of::<C>(),
            boxed_component: Box::new(component),
        }
    }

    /// Destructures [`BundledComponent`],
    /// returning type information of component and its boxed value.
    ///
    pub(super) fn destructure(self) -> (ComponentId, BoxedComponent) {
        (self.component_id, self.boxed_component)
    }
}
/// [`BundledIterator`] is a wrapper that provides trivial implementation of [`Bundle`]
/// for anything that implements `IntoIterator<Item = BundledComponent>`.
/// This struct does not provide any other functionality.
///
/// # Example
/// ```rust
/// # use ggengine::gamecore::components::{BundledIterator, Bundle};
/// fn take_bundle(bundle: impl Bundle) {}
///
/// take_bundle(BundledIterator(vec![]));  // compiles
/// ```
///
pub struct BundledIterator<T>(pub T);
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
/// For example defining a `PlayerBundle` containing components that describe the player
/// can be written as follows:
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
/// That is where you have three options.
/// 1. You can use extension trait pattern to define constructors for tuples:
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
/// # use ggengine::gamecore::components::{Bundle, Component, BundledComponent};
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
///     fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
///         (self.player, self.name, self.position).bundled_components()
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
/// 3. You can manually implement [`Bundle`] trait by using [`BundledComponent`]:
/// ```rust
/// # use ggengine::gamecore::components::{Bundle, Component, BundledComponent};
/// struct PackedBundle<T> {
///     inner_component: T
/// }
/// impl<T: Component> Bundle for PackedBundle<T> {
///     fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
///         BundledComponent::bundle(self.inner_component).bundled_components()
///     }
/// }
/// ```
/// [`BundledComponent`] ensures that the implementation is correct,
/// because it automatically records necessary metadata of component.
///
pub trait Bundle {
    /// Consumes itself and returns iterator of [`BundledComponent`]s.
    ///
    fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent>;
}
impl<T: Component> Bundle for T {
    fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
        BundledComponent::bundle(self).bundled_components()
    }
}
impl Bundle for BundledComponent {
    fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
        once(self)
    }
}
impl<T: IntoIterator<Item = BundledComponent>> Bundle for BundledIterator<T> {
    fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
        self.0
    }
}
/// [`impl_bundle`] macro implements [`Bundle`] trait for tuples of arity 12 or less.
///
macro_rules! impl_bundle {
    () => {
        impl Bundle for ()
        {
            fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
                empty()
            }
        }
    };
    (($t_start:ident, $index_start:tt), $(($t:ident, $index:tt),)*) => {
        impl<$t_start: Bundle, $($t: Bundle),*> Bundle for ($t_start, $($t,)*)
        {
            fn bundled_components(self) -> impl IntoIterator<Item = BundledComponent> {
                self.$index_start.bundled_components().into_iter()$(.chain(self.$index.bundled_components().into_iter()))*
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ResourceId(TypeId);
impl ResourceId {
    /// Returns [`ResourceId`] of given [`Resource`] type.
    ///
    pub(super) fn of<R: Resource>() -> Self {
        ResourceId(TypeId::of::<R>())
    }
}

/// [`Downcastable`] trait allows [`Component`]s and [`Resource`]s
/// to be downcasted to concrete types from behind `dyn` + indirection.
///
/// Ideally this functionality would be implemented with a bunch of independent functions,
/// but that is impossible to do generically
/// since `Box<T>` where `T: ?Sized` is not `Sized` itself, and thus cannot be cast to `Any`.
///
/// Concrete implementations of this trait on `dyn Component` and `dyn Resource`
/// (which is all that is needed anyway) allow doing that conversion.
///
/// # Note
/// `downcast_to_value` consumes initial `Box`,
/// but on failure it does not need to, and so it returns it in upcasted form (`Box<dyn Any>`).
/// Although it would be preferrable to return initial type, it is impossible to do so from trait.
///
/// `ggengine` always uses this function in a context where conversion cannot fail and
/// that makes this issue practically non-existent.
///
pub(super) trait Downcastable {
    /// Method that coerces `Box<Self>` to `T`.
    ///
    fn downcast_to_value<T: Any>(self: Box<Self>) -> Result<T, Box<dyn Any>>;
    /// Method that coerces `&Self` to `&T`.
    ///
    fn downcast_to_ref<T: Any>(&self) -> Option<&T>;
    /// Method that coerces `&mut Self` to `&mut T`.
    ///
    fn downcast_to_mut<T: Any>(&mut self) -> Option<&mut T>;
}
impl Downcastable for dyn Component {
    fn downcast_to_value<T: Any>(self: Box<Self>) -> Result<T, Box<dyn Any>> {
        let as_any: Box<dyn Any> = self;
        as_any.downcast::<T>().map(|boxed| *boxed)
    }
    fn downcast_to_ref<T: Any>(&self) -> Option<&T> {
        let as_any: &dyn Any = self;
        as_any.downcast_ref::<T>()
    }
    fn downcast_to_mut<T: Any>(&mut self) -> Option<&mut T> {
        let as_any: &mut dyn Any = self;
        as_any.downcast_mut::<T>()
    }
}
impl Downcastable for dyn Resource {
    fn downcast_to_value<T: Any>(self: Box<Self>) -> Result<T, Box<dyn Any>> {
        let as_any: Box<dyn Any> = self;
        as_any.downcast::<T>().map(|boxed| *boxed)
    }
    fn downcast_to_ref<T: Any>(&self) -> Option<&T> {
        let as_any: &dyn Any = self;
        as_any.downcast_ref::<T>()
    }
    fn downcast_to_mut<T: Any>(&mut self) -> Option<&mut T> {
        let as_any: &mut dyn Any = self;
        as_any.downcast_mut::<T>()
    }
}