//! `gamecore::components` submodule defines [`Component`] trait
//! that allows binding game logic that is represented in form of Rust types to entity,
//! and implements several common components used in games.
//!

use crate::gamecore::entities::EntityId;
use seq_macro::seq;
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
/// # Example
/// Any Rust type that fits [`Component`]'s constraints can be a [`Component`].
/// They are usually structs, but can also be enums or zero-sized types.
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
/// [`ComponentId`] id struct is needed to identify [`Component`]s in [`EntityComponentStorage`].
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by `ggengine`.
///
/// Storages internally operate on ids, which allows them to provide more flexible interface.
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
/// # Examples
/// Every [`Component`] is a [`Bundle`], because component is basically a set (bundle) of one component.
/// Additionally, tuples of bundles are also [`Bundle`]
/// (with up to 16 items, but can be nested indefinetely; if you need more, consider implementing your own [`Bundle`]).
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
/// # use ggengine::gamecore::components::{Bundle, Component, ComponentId};
/// # use ggengine::gamecore::storages::EntityComponentStorage;
/// # use ggengine::gamecore::entities::EntityId;
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
/// impl Bundle for PlayerBundle {
///     const SIZE: usize = 3;
///
///     fn component_ids() -> impl Iterator<Item = ComponentId> {
///         <(Player, Name, Position)>::component_ids()
///     }
///     fn add_to_entity(
///         self,
///         entity_id: EntityId,
///         entity_component_storage: &mut EntityComponentStorage
///     ) {
///         (self.player, self.name, self.position)
///             .add_to_entity(entity_id, entity_component_storage)
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
/// # use ggengine::gamecore::components::{Bundle, Component, ComponentId};
/// # use ggengine::gamecore::storages::EntityComponentStorage;
/// # use ggengine::gamecore::entities::EntityId;
/// # use std::iter::once;
/// struct PackedBundle<T> {
///     inner_component: T
/// }
/// impl<T: Component> Bundle for PackedBundle<T> {
///     const SIZE: usize = 1;
///
///     fn component_ids() -> impl Iterator<Item = ComponentId> {
///         once(ComponentId::of::<T>())
///     }
///     fn add_to_entity(
///         self,
///         entity_id: EntityId,
///         entity_component_storage: &mut EntityComponentStorage
///     ) {
///         let _ = entity_component_storage.insert_component(entity_id, self.inner_component);
///     }
/// }
/// ```
///
/// Manual implementations (even those that leverage existing implementations) are rather clunky
/// and susceptible to errors (fairly easy to mistype).
/// With that in mind, you should use implementation for tuples.
///
pub trait Bundle {
    /// Size of the [`Bundle`].
    ///
    const SIZE: usize;

    /// Returns ids of all components that are in the bundle.
    ///
    /// This function should return iterator with length of `<self as Bundle>::SIZE`.
    ///
    fn component_ids() -> impl Iterator<Item = ComponentId>;

    /// This method should add all of the components of a bundle to the entity.
    /// Tha could be done by sequentially calling `EntityComponentStorage::insert_component` for each component in a bundle.
    /// Since that requires statically knowing component types, this could only be done from this function.
    ///
    fn add_to_entity(
        self,
        entity_id: EntityId,
        entity_component_storage: &mut EntityComponentStorage,
    );
}
impl<T: Component> Bundle for T {
    const SIZE: usize = 1;

    fn component_ids() -> impl Iterator<Item = ComponentId> {
        once(ComponentId::of::<T>())
    }

    fn add_to_entity(
        self,
        entity_id: EntityId,
        entity_component_storage: &mut EntityComponentStorage,
    ) {
        let _ = entity_component_storage.insert_component(entity_id, self);
    }
}
/// [`impl_bundle`] macro implements [`Bundle`] trait for tuples.
///
macro_rules! impl_bundle {
    ($(($t:ident, $index:tt)),* $(,)?) => {
        impl<$($t,)*> Bundle for ($($t,)*)
        where
            $($t: Bundle,)*
        {
            const SIZE: usize = $($t::SIZE + )* 0;

            fn component_ids() -> impl Iterator<Item = ComponentId> {
                empty()$(.chain($t::component_ids()))*
            }

            fn add_to_entity(
                self,
                _entity_id: EntityId,
                _entity_component_storage: &mut EntityComponentStorage
            ) {
                $(let _ = self.$index.add_to_entity(_entity_id, _entity_component_storage);)*
            }
        }
    };
}
seq!(SIZE in 0..=16 {
    #(
        seq!(N in 0..SIZE {
            impl_bundle!(#((C~N, N),)*);
        });
    )*
});

// submodules and public re-exports
pub use super::storages::EntityComponentStorage;
