//! `gamecore::entities` submodule implements entities -
//! game objects that have some characteristics (components) on which game engine operates.
//!

use crate::gamecore::{
    components::{Bundle, Component},
    storages::EntityComponentStorage,
};
use std::hash::{Hash, Hasher};

/// [`EntityId`] id struct is needed to identify entities
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this entity is registered.
///
/// # Note
/// [`EntityId`] is only valid for the [`Scene`](super::scenes::Scene) it was obtained from,
/// and although you can use it for any other scene,
/// fetching will either fail or return unexpected results.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub(super) usize);
impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64)
    }
}

/// [`EntityRef`] provides immutable access to a single entity and all of its components.
///
/// This struct provides ergonomic access to [`EntityComponentStorage`] API,
/// and `ggengine` advises using [`EntityRef`] instead of using bare [`EntityComponentStorage`].
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
    /// Entity storage which can be navigated by `entity_id`.
    ///
    entity_component_storage: &'a EntityComponentStorage,
}
impl<'a> EntityRef<'a> {
    /// Creates new [`EntityRef`], immutably borrowing [`EntityComponentStorage`].
    ///
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &'a EntityComponentStorage,
    ) -> EntityRef<'a> {
        EntityRef {
            entity_id,
            entity_component_storage,
        }
    }

    /// Returns id of this entity.
    ///
    /// Usually you would call this method before dropping [`EntityRef`] to
    /// save id of entity so you could obtain reference to it later.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityRef, EntityId};
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let entity: EntityRef = EntityRef::from(storage.spawn_entity(()));
    /// let entity_id: EntityId = entity.id();
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
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityRef = EntityRef::from(storage.spawn_entity((Player,)));
    /// assert!(entity.contains::<Player>());
    /// ```
    ///
    pub fn contains<C: Component>(&self) -> bool {
        self.entity_component_storage
            .contains_component::<C>(self.entity_id)
    }
    /// Returns immutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityRef;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let entity: EntityRef = EntityRef::from(storage.spawn_entity((Player, Health(10))));
    /// assert_eq!(entity.get::<Health>().expect("Component was inserted.").0, 10);
    /// ```
    ///
    pub fn get<C: Component>(&self) -> Option<&C> {
        self.entity_component_storage.component::<C>(self.entity_id)
    }
}
impl<'a> From<EntityMut<'a>> for EntityRef<'a> {
    fn from(value: EntityMut<'a>) -> EntityRef<'a> {
        EntityRef {
            entity_id: value.entity_id,
            entity_component_storage: value.entity_component_storage,
        }
    }
}

/// [`EntityMut`] provides mutable access to a single entity and all of its components.
///
/// This struct provides ergonomic access to [`EntityComponentStorage`] API,
/// and `ggengine` advises using [`EntityMut`] instead of using bare [`EntityComponentStorage`].
/// 
#[derive(Debug)]
pub struct EntityMut<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Entity storage which can be navigated by `entity_id`.
    ///
    entity_component_storage: &'a mut EntityComponentStorage,
}
impl<'a> EntityMut<'a> {
    /// Creates new [`EntityMut`], mutably borrowing [`EntityComponentStorage`].
    ///
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &'a mut EntityComponentStorage,
    ) -> EntityMut<'a> {
        EntityMut {
            entity_id,
            entity_component_storage,
        }
    }

    /// Returns id of this entity.
    ///
    /// Usually you would call this method before dropping [`EntityMut`] to
    /// save id of entity so you could obtain reference to it later.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let entity: EntityMut = storage.spawn_entity(());
    /// let entity_id: EntityId = entity.id();
    /// ```
    ///
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    /// Consumes [`EntityMut`] and despawns its entity.
    ///
    /// When this functions is called,
    /// [`EntityId`] from `EntityMut::id` are no longer valid.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let entity: EntityMut = storage.spawn_entity(());
    /// let entity_id: EntityId = entity.id();
    /// entity.despawn();
    /// assert!(!storage.contains_entity(entity_id));
    /// ```
    ///
    pub fn despawn(self) {
        let _ = self.entity_component_storage.despawn_entity(self.entity_id);
    }

    /// Inserts component into entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity(());
    /// let _ = entity.insert(Player);
    /// ```
    ///
    pub fn insert<C: Component>(&mut self, component: C) -> Option<C> {
        self.entity_component_storage
            .insert_component::<C>(self.entity_id, component)
    }
    /// Inserts bundle of components into entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity(());
    /// let _ = entity.insert_bundle((Player, Health(10)));
    ///
    /// let entity_id: EntityId = entity.id();
    /// assert!(storage.contains_component::<Player>(entity_id));
    /// assert!(storage.contains_component::<Health>(entity_id));
    /// ```
    ///
    pub fn insert_bundle(&mut self, bundle: impl Bundle) {
        self.entity_component_storage
            .insert_bundle(self.entity_id, bundle)
    }

    /// Removes component from entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity((Player,));
    /// let player: Player = entity.remove::<Player>().expect("Component is present.");
    ///
    /// let entity_id: EntityId = entity.id();
    /// assert!(!storage.contains_component::<Player>(entity_id));
    /// ```
    pub fn remove<C: Component>(&mut self) -> Option<C> {
        self.entity_component_storage
            .remove_component::<C>(self.entity_id)
    }
    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity((Player, Health(10)));
    /// entity.clear();
    ///
    /// let entity_id: EntityId = entity.id();
    /// assert!(!storage.contains_component::<Player>(entity_id));
    /// assert!(!storage.contains_component::<Health>(entity_id));
    /// ```
    ///
    pub fn clear(&mut self) {
        self.entity_component_storage
            .remove_all_components(self.entity_id)
    }

    /// Returns whether this component is present in entity or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity(());
    /// assert!(!entity.contains::<Player>());
    ///
    /// let _ = entity.insert(Player);
    /// assert!(entity.contains::<Player>());
    /// ```
    ///
    pub fn contains<C: Component>(&self) -> bool {
        self.entity_component_storage
            .contains_component::<C>(self.entity_id)
    }
    /// Extracts component from this entity and returns it if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity((Player, Health(10)));
    /// let health: Health = entity.take::<Health>().expect("Component is present.");
    /// assert_eq!(health.0, 10);
    /// ```
    pub fn take<C: Component>(&mut self) -> Option<C> {
        self.entity_component_storage
            .component_take::<C>(self.entity_id)
    }
    /// Returns immutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let entity: EntityMut = storage.spawn_entity((Player, Health(10)));
    /// assert_eq!(entity.get::<Health>().expect("Component was inserted.").0, 10);
    /// ```
    ///
    pub fn get<C: Component>(&self) -> Option<&C> {
        self.entity_component_storage.component::<C>(self.entity_id)
    }
    /// Returns mutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity((Player, Health(10)));
    /// entity.get_mut::<Health>().expect("Component is present.").0 = 20;
    /// assert_eq!(entity.get::<Health>().expect("Component was inserted.").0, 20);
    /// ```
    ///
    pub fn get_mut<C: Component>(&mut self) -> Option<&mut C> {
        self.entity_component_storage
            .component_mut::<C>(self.entity_id)
    }
    /// Gets a mutable reference to the component of given type if present,
    /// otherwise inserts the component that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let mut entity: EntityMut = storage.spawn_entity((Player,));
    /// let _ = entity.get_or_insert_with(|| Health(10));
    /// assert!(entity.contains::<Health>());
    /// ```
    ///
    pub fn get_or_insert_with<C: Component>(&mut self, f: impl FnOnce() -> C) -> &mut C {
        self.entity_component_storage
            .component_get_or_insert_with::<C>(self.entity_id, f)
    }
}
