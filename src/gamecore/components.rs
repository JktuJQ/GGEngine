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
/// [`ComponentId`] id struct is needed to identify [`Component`]s in [`ComponentStorage`].
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

/// [`ComponentSet`] trait defines a static set of [`Component`]s.
///
/// In ECS, components define objects and systems operate on combinations of components.
/// [`ComponentSet`] trait provides a way to create a set of [`Component`]s that are coupled
/// by some logic, and it just makes sense to use those together.
///
/// Although absence of repeating elements is not enforced for the set, it it still expected from it.
/// If there is repeating component in the set, only one of those will be added to the storage,
/// which could lead to unexpected behaviour.
///
/// Due to that, you should not use [`ComponentSet`]s as units of behaviour -
/// adding two sets to the entity and then removing one of them does not necessarily mean
/// that the other is still present if those two intersect.
/// For that reason, there is intentionally no way to check whether the set is present at entity
/// or to query based on set components.
/// That said, you should only use [`ComponentSet`]s as a convenient way to add/remove multiple components
/// at once, not thinking about sets as abstraction.
///
/// # Examples
/// Every [`Component`] is a [`ComponentSet`], because component is basically a set of one component.
/// Additionally, tuples of sets are also [`ComponentSet`]
/// (with up to 16 items, but can be nested indefinetely; if you need more, consider implementing your own [`ComponentSet`]).
/// This allows you to combine the necessary components into a [`ComponentSet`].
///
/// For example defining a `PlayerComponentSet` containing components that describe the player
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
/// type PlayerComponentSet = (Player, Name, Position);
/// ```
///
/// You might want to customize initialization of your [`ComponentSet`].
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
/// type PlayerComponentSet = (Player, Name, Position);
///
/// let player: PlayerComponentSet = Default::default();
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
/// type PlayerComponentSet = (Player, Name, Position);
///
/// trait WithName {
///     fn with_name(name: &'static str) -> Self;
/// }
/// impl WithName for PlayerComponentSet {
///     fn with_name(name: &'static str) -> PlayerComponentSet {
///         let mut result: PlayerComponentSet = PlayerComponentSet::default();
///         result.1 = Name(name);
///         result
///     }
/// }
///
/// let player: PlayerComponentSet = PlayerComponentSet::with_name("Player");
/// ```
///
/// 2. You can leverage provided implementation to construct your own:
/// ```rust
/// # use ggengine::gamecore::components::{ComponentSet, Component, ComponentId, ComponentStorage};
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
/// struct PlayerComponentSet {
///     player: Player,
///     name: Name,
///     position: Position,
/// }
/// impl ComponentSet for PlayerComponentSet {
///     const SIZE: usize = 3;
///
///     fn component_ids() -> impl Iterator<Item = ComponentId> {
///         <(Player, Name, Position)>::component_ids()
///     }
///     fn insert_set(self, entity_id: EntityId, storage: &mut ComponentStorage) {
///         (self.player, self.name, self.position).insert_set(entity_id, storage)
///     }
/// }
///
/// let player: PlayerComponentSet = PlayerComponentSet {
///     name: Name("Player"),
///     ..Default::default()
/// };
/// ```
/// That approach allows to free yourself from all restrictions,
/// and just 'pack a bundle' at the very end.
///
/// 3. You can manually implement [`ComponentSet`] trait:
/// ```rust
/// # use ggengine::gamecore::components::{ComponentSet, Component, ComponentId, ComponentStorage};
/// # use ggengine::gamecore::entities::EntityId;
/// # use std::iter::once;
/// struct PackedComponentSet<T> {
///     inner_component: T
/// }
/// impl<T: Component> ComponentSet for PackedComponentSet<T> {
///     const SIZE: usize = 1;
///
///     fn component_ids() -> impl Iterator<Item = ComponentId> {
///         once(ComponentId::of::<T>())
///     }
///     fn insert_set(self, entity_id: EntityId, storage: &mut ComponentStorage) {
///         let _ = storage.insert_component(entity_id, self.inner_component);
///     }
/// }
/// ```
///
/// Manual implementations (even those that leverage existing implementations) are rather clunky
/// and susceptible to errors (fairly easy to mistype).
/// With that in mind, you should use implementation for tuples.
///
pub trait ComponentSet {
    /// Size of the [`ComponentSet`].
    ///
    const SIZE: usize;

    /// Returns ids of all components that are in the set.
    ///
    /// This function should return iterator with length of `ComponentSet::SIZE`.
    /// When `const_generics` will land,
    /// this function should return `[ComponentId; Self::Size]` to prevent buggy implementations.
    ///
    fn component_ids() -> impl Iterator<Item = ComponentId>;

    /// This method should insert all of the components of a set to the entity.
    /// That could be done by sequentially calling `ComponentStorage::insert_component` for each component in a set.
    /// Since that requires statically knowing component types, this could only be done from this function.
    ///
    /// Normally this function would not be called directly,
    /// instead `ComponentStorage::insert_components` would be used.
    ///
    fn insert_set(self, entity_id: EntityId, component_storage: &mut ComponentStorage);
}
impl<C: Component> ComponentSet for C {
    const SIZE: usize = 1;

    fn component_ids() -> impl Iterator<Item = ComponentId> {
        once(ComponentId::of::<C>())
    }

    fn insert_set(self, entity_id: EntityId, storage: &mut ComponentStorage) {
        let _ = storage.insert_component(entity_id, self);
    }
}
/// [`impl_component_set`] macro implements [`ComponentSet`] trait for tuples.
///
macro_rules! impl_component_set {
    ($(($t:ident, $index:tt)),* $(,)?) => {
        impl<$($t,)*> ComponentSet for ($($t,)*)
        where
            $($t: ComponentSet,)*
        {
            const SIZE: usize = $($t::SIZE + )* 0;

            fn component_ids() -> impl Iterator<Item = ComponentId> {
                empty()$(.chain($t::component_ids()))*
            }

            fn insert_set(self, _entity_id: EntityId, _storage: &mut ComponentStorage) {
                $(let _ = self.$index.insert_set(_entity_id, _storage);)*
            }
        }
    };
}
seq!(SIZE in 0..=16 {
    #(
        seq!(N in 0..SIZE {
            impl_component_set!(#((C~N, N),)*);
        });
    )*
});

pub use crate::gamecore::{querying::component_query::*, storages::component_storage::*};
