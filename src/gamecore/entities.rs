//! `gamecore::entities` submodule implements entities -
//! game objects that have some characteristics (components) on which game engine operates.
//!

use crate::gamecore::components::{Component, ComponentSet, ComponentStorage};
use std::hash::{Hash, Hasher};

/// [`EntityId`] id struct is needed to identify entities
/// in [`storage`](super::storages::ComponentStorage).
///
/// It is assigned by the [`ComponentStorage`] in which
/// this entity is registered.
///
/// [`EntityId`] is only valid for the [`ComponentStorage`] it was obtained from,
/// and although you can use it for any other storage,
/// fetching will either fail or return unexpected results.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntityId(pub(super) usize);
impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64)
    }
}

/// [`EntityRef`] provides immutable access to a single entity and all of its components.
///
/// This struct provides ergonomic access to [`ComponentStorage`] API,
///
/// # Note
/// If you want to downgrade [`EntityMut`] to [`EntityRef`] without manual dropping,
/// you can use `EntityRef::from` to perform that conversion.
///
#[derive(Debug)]
pub struct EntityRef<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Component storage which can be navigated by `entity_id`.
    ///
    storage: &'a ComponentStorage,
}
impl EntityRef<'_> {
    /// Creates new [`EntityRef`], immutably borrowing [`ComponentStorage`].
    ///
    pub(super) fn new(entity_id: EntityId, storage: &ComponentStorage) -> EntityRef {
        EntityRef { entity_id, storage }
    }

    /// Returns id of this entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let entity: EntityId = EntityRef::from(storage.insert_empty_entity()).id();
    /// ```
    ///
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    /// Returns whether this component is present in entity or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityRef;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityRef = EntityRef::from(storage.insert_entity(Player));
    /// assert!(entity.contains_component::<Player>());
    /// ```
    ///
    pub fn contains_component<C: Component>(&self) -> bool {
        self.storage.contains_component::<C>(self.entity_id)
    }

    /// Returns immutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityRef;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    /// let entity: EntityRef = EntityRef::from(storage.insert_entity((Player, Health(10))));
    /// assert_eq!(entity.component::<Health>().expect("Component was inserted").0, 10);
    /// ```
    ///
    pub fn component<C: Component>(&self) -> Option<&C> {
        self.storage.component::<C>(self.entity_id)
    }
}
impl<'a> From<EntityMut<'a>> for EntityRef<'a> {
    fn from(value: EntityMut<'a>) -> EntityRef<'a> {
        EntityRef {
            entity_id: value.entity_id,
            storage: value.storage,
        }
    }
}

/// [`EntityMut`] provides mutable access to a single entity and all of its components.
///
/// This struct provides ergonomic access to [`ComponentStorage`] API.
///
#[derive(Debug)]
pub struct EntityMut<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Component storage which can be navigated by `entity_id`.
    ///
    storage: &'a mut ComponentStorage,
}
impl EntityMut<'_> {
    /// Creates new [`EntityMut`], immutably borrowing [`ComponentStorage`].
    ///
    pub(super) fn new(entity_id: EntityId, storage: &mut ComponentStorage) -> EntityMut {
        EntityMut { entity_id, storage }
    }

    /// Returns id of this entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityId;
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let entity: EntityId = storage.insert_empty_entity().id();
    /// ```
    ///
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    /// Consumes [`EntityMut`] and removes its entity.
    ///
    /// When this function is called, [`EntityId`]s that were obtained from `EntityMut::id` are no longer valid.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let entity: EntityMut = storage.insert_empty_entity();
    /// let entity_id: EntityId = entity.id();
    /// entity.remove();
    /// assert!(!storage.contains_entity(entity_id));
    /// ```
    ///
    pub fn remove(self) {
        let _ = self.storage.remove_entity(self.entity_id);
    }

    /// Inserts component into entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_empty_entity();
    /// let _ = entity.insert_component(Player);
    /// let id: EntityId = entity.id();
    /// assert!(storage.contains_component::<Player>(id));
    /// ```
    ///
    pub fn insert_component<C: Component>(&mut self, component: C) -> Option<C> {
        self.storage.insert_component(self.entity_id, component)
    }
    /// Inserts set of components into entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_empty_entity();
    /// let _ = entity.insert_many_components((Player, Health(10)));
    /// assert!(entity.contains_component::<Player>());
    /// assert!(entity.contains_component::<Health>());
    /// ```
    ///
    pub fn insert_many_components<CS: ComponentSet>(&mut self, components: CS) {
        self.storage
            .insert_many_components(self.entity_id, components)
    }

    /// Removes component from entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_entity(Player);
    /// let player: Player = entity.remove_component::<Player>().expect("Component is present");
    /// assert!(!entity.contains_component::<Player>());
    /// ```
    ///
    pub fn remove_component<C: Component>(&mut self) -> Option<C> {
        self.storage.remove_component::<C>(self.entity_id)
    }
    /// Removes components that are in set from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_entity((Player, Name("Alice"), Health(10)));
    /// entity.remove_many_components::<(Player, Health)>();
    /// assert!(!entity.contains_component::<Player>());
    /// assert!(entity.contains_component::<Name>());
    /// assert!(!entity.contains_component::<Health>());
    /// ```
    ///
    pub fn remove_many_components<CS: ComponentSet>(&mut self) {
        self.storage.remove_many_components::<CS>(self.entity_id)
    }

    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_entity((Player, Health(10)));
    /// entity.clear();
    /// assert!(!entity.contains_component::<Player>());
    /// assert!(!entity.contains_component::<Health>());
    /// ```
    ///
    pub fn clear(&mut self) {
        self.storage.clear_entity(self.entity_id);
    }

    /// Returns whether this component is present in entity or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_empty_entity();
    /// assert!(!entity.contains_component::<Player>());
    ///
    /// let _ = entity.insert_component(Player);
    /// assert!(entity.contains_component::<Player>());
    /// ```
    ///
    pub fn contains_component<C: Component>(&self) -> bool {
        self.storage.contains_component::<C>(self.entity_id)
    }
    /// Returns immutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_entity((Player, Health(10)));
    /// assert_eq!(entity.component::<Health>().expect("Component was inserted").0, 10);
    /// ```
    ///
    pub fn component<C: Component>(&self) -> Option<&C> {
        self.storage.component::<C>(self.entity_id)
    }
    /// Returns mutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.insert_entity((Player, Health(10)));
    /// entity.component_mut::<Health>().expect("Component was inserted").0 = 20;
    /// assert_eq!(entity.component::<Health>().expect("Component was inserted").0, 20);
    /// ```
    ///
    pub fn component_mut<C: Component>(&mut self) -> Option<&mut C> {
        self.storage.component_mut::<C>(self.entity_id)
    }
}
