//! `gamecore::storages` submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!
//! This submodule is fairly low-level; it uses plain ids to implement structures,
//! which allows providing the most flexible interface.
//! `ggengine` provides typed API on top of this module through several structs; and
//! you should use that, not the API of this submodule (unless absolutely needed).
//!

use crate::gamecore::{
    components::{BoxedComponent, BoxedResource, BundledComponent, ComponentId, ResourceId},
    entities::{EntityId, EntityMut, EntityRef},
};
use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hasher},
};

/// [`NoOpHasher`] struct is a hasher that removes overhead of hashing by directly passing
/// its internals through.
///
/// # Usage
/// ECS model heavily relies on fast querying and indexation of components and entities.
/// Id structs are indices for navigating their counterparts in [`Scene`](super::scenes::Scene) storage,
/// and those are implemented as wrappers of `u64`.
///
/// This hasher allows for those `u64`s to be used as keys in collections that require hashing
/// but without overhead of hashing. Those indices are already high-quality hashes because their
/// uniqueness is ensured, so this `Hasher` implementation can give an actual performance boost.
///
/// **This hasher only passes `u64` as a no-op hashing, `write` function will panic.**
///
#[derive(Copy, Clone, Debug, Default)]
struct NoOpHasher(u64);
impl Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!("`write` method should not be used on `NoOpHasher`");
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

/// [`NoOpHasherState`] struct implements `BuildHasher` trait that produces [`NoOpHasher`].
///
/// This should be passed to collections interfaces (e.g. `HashMap::with_hasher(NoOpHasherState)`).
///
#[derive(Copy, Clone, Debug, Default)]
struct NoOpHasherState;
impl BuildHasher for NoOpHasherState {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher::default()
    }
}

/// Type alias for `HashSet<T, NoOpHasherState>`.
///
/// [`IdSet`] should be used wherever id structs are keys in a `HashSet`.
///
type IdSet<T> = HashSet<T, NoOpHasherState>;
/// Type alias for `HashMap<K, V, NoOpHasherState>`.
///
/// [`IdMap`] should be used wherever id structs are keys in a `HashMap`.
///
type IdMap<K, V> = HashMap<K, V, NoOpHasherState>;
/// [`EntityComponentStorage`] is a column-oriented structure-of-arrays based storage
/// that maps entities to their [`Component`](super::components::Component)s.
///
/// Conceptually, [`EntityComponentStorage`] can be thought of as an `HashMap<ComponentId, Vec<Option<BoxedComponent>>>`,
/// where each `Vec` contains components of one type that belong to different entities.
///
/// Each row corresponds to a single entity
/// (i.e. equal indices of `Vec`s point to different components on the same entity)
/// and each column corresponds to a single `Component`
/// (i.e. every `Vec` contains all `Component`s of one type that belong to different entities).
///
/// Fetching components from a table involves fetching the associated column for a `Component` type
/// (via its [`ComponentId`]), then fetching the entity's row within that column.
///
/// # Performance
/// [`EntityComponentStorage`] uses no-op hasher, because ids are reliable hashes due to implementation.
///
/// Since components are stored in columnar contiguous blocks of memory, table is optimized for fast querying,
/// but frequent insertion and removal can be relatively slow.
/// Chosen approach is a good trade-off between speed of lookups/querying and speed of insertion/removal,
/// with the accent on the former.
///
/// # Note
/// This collection is designed to provide more fine-grained control over entity-component storage.
/// That results in very verbose interface (heavy usage of `impl Iterator` to allow dynamic collections).
/// Most of the time you should use [`EntityMut`] or its readonly counterpart and [`Scene`](super::scenes::Scene)
/// to get nice typed API for operations.
/// Although examples in docs for [`EntityComponentStorage`] show usage of bare storage interface,
/// it is not the recommended way of doing such operations.
///
#[derive(Debug, Default)]
pub struct EntityComponentStorage {
    /// Maximal index that is vacant for entity insertion.
    ///
    max_vacant_index: usize,
    /// Set of removed entities.
    ///
    removed_entities: IdSet<EntityId>,

    /// Table that holds all components.
    ///
    table: IdMap<ComponentId, Vec<Option<BoxedComponent>>>,
}
impl EntityComponentStorage {
    /// Initializes new [`EntityComponentStorage`].
    ///
    /// Created [`EntityComponentStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// let storage: EntityComponentStorage = EntityComponentStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        EntityComponentStorage {
            max_vacant_index: 0,
            removed_entities: IdSet::with_hasher(NoOpHasherState),

            table: IdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.max_vacant_index = 0;
        self.removed_entities.clear();
        self.table.clear();
    }

    /// Inserts new entity supplying it with components.
    ///
    /// Main feature of this function is that it allows to pass additional capacity,
    /// which will allow for allocation optimizations.
    ///
    fn insert_entity_with_capacity(
        &mut self,
        components: impl Iterator<Item = BundledComponent>,
        entities_count_capacity: usize,
    ) -> EntityMut {
        let entity_id = match self.removed_entities.iter().next().copied() {
            Some(id) => {
                let _ = self.removed_entities.remove(&id);
                id
            }
            None => {
                let reserved_id = EntityId(self.max_vacant_index);
                self.max_vacant_index += 1;
                reserved_id
            }
        };

        for (component_id, boxed_component) in components {
            let _ = self.insert_bundled_component_with_capacity(
                entity_id,
                component_id,
                boxed_component,
                entities_count_capacity,
            );
        }

        EntityMut::new(entity_id, self)
    }

    /// Inserts component into existing entity.
    ///
    /// Main feature of this function is that it allows to pass additional capacity,
    /// which will allow for allocation optimizations.
    ///
    fn insert_bundled_component_with_capacity(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        boxed_component: BoxedComponent,
        entities_count_capacity: usize,
    ) -> Option<BoxedComponent> {
        let components_column = self
            .table
            .entry(component_id)
            .or_insert(Vec::with_capacity(entities_count_capacity));

        let entity_index = entity_id.0;
        if components_column.len() <= entity_index {
            components_column.resize_with(
                if entities_count_capacity == 0 {
                    entity_index + 1
                } else {
                    entities_count_capacity
                },
                || None,
            );
        }

        components_column[entity_index].replace(boxed_component)
    }

    /// Inserts entity with components into [`EntityComponentStorage`]
    /// and returns mutable reference to it, so it could be further modified.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player,)).into_iter()
    /// ).id();
    /// ```
    ///
    /// You can insert empty entity to defer initialization by passing `empty()`:
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::entities::EntityId;
    /// # use std::iter::empty;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     empty()
    /// ).id();
    /// ```
    ///
    pub fn insert_entity(
        &mut self,
        components: impl Iterator<Item = BundledComponent>,
    ) -> EntityMut {
        self.insert_entity_with_capacity(components, 0)
    }
    /// Inserts multiple entities with components into [`EntityComponentStorage`]
    /// and returns immutable references to those entities.
    ///
    /// It is more efficient than calling `EntityComponentStorage::insert_entity` in a loop.
    ///
    /// # Note
    /// This function can only insert entities with same components type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs: Vec<EntityRef> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).collect::<Vec<EntityRef>>();
    /// ```
    ///
    pub fn insert_entities(
        &mut self,
        many_components: impl IntoIterator<Item = impl Iterator<Item = BundledComponent>>,
    ) -> impl Iterator<Item = EntityRef> {
        let many_components_iter = many_components.into_iter();
        let adding = {
            let (lower, upper) = many_components_iter.size_hint();
            upper.unwrap_or(lower)
        };
        let resizing = if adding >= self.removed_entities.len() {
            let vacant = self.removed_entities.len();
            let reserved = self.max_vacant_index - vacant;
            reserved + (adding - vacant)
        } else {
            0
        };

        let mut ids = Vec::new();
        for components in many_components_iter {
            ids.push(self.insert_entity_with_capacity(components, resizing).id());
        }
        ids.into_iter().map(|id| EntityRef::new(id, self))
    }
    /// Removes entity from [`EntityComponentStorage`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player,)).into_iter()
    /// ).id();
    /// storage.remove_entity(player);
    /// ```
    ///
    pub fn remove_entity(&mut self, entity_id: EntityId) -> bool {
        if self.removed_entities.contains(&entity_id) {
            return false;
        }
        let _ = self.removed_entities.insert(entity_id);

        let entity_index = entity_id.0;
        for component_column in self.table.values_mut() {
            if component_column.len() > entity_index {
                component_column[entity_index] = None;
            }
        }
        true
    }
    /// Returns number of entities that are currently present in the storage.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs: Vec<EntityRef> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).collect::<Vec<EntityRef>>();
    /// let npc: EntityId = npcs[0].id();
    /// assert_eq!(storage.entities_count(), 3);
    /// storage.remove_entity(npc);
    /// assert_eq!(storage.entities_count(), 2);
    ///
    pub fn entities_count(&self) -> usize {
        self.max_vacant_index - self.removed_entities.len()
    }
    /// Returns whether an entity with given id is currently stored or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player,)).into_iter()
    /// ).id();
    /// assert!(storage.contains_entity(player));
    ///
    /// storage.remove_entity(player);
    /// assert!(!storage.contains_entity(player));
    /// ```
    ///
    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        entity_id.0 < self.max_vacant_index && !self.removed_entities.contains(&entity_id)
    }
    /// Returns immutable reference to entity in [`EntityComponentStorage`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// # use std::iter::empty;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     empty()
    /// ).id();
    /// let player_ref: EntityRef = storage.entity(player).expect("Entity was inserted");
    /// ```
    ///
    pub fn entity(&self, entity_id: EntityId) -> Option<EntityRef> {
        if self.contains_entity(entity_id) {
            Some(EntityRef::new(entity_id, self))
        } else {
            None
        }
    }
    /// Returns references to all entities that are inserted in the storage.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs1: Vec<EntityId> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).map(|entity| entity.id()).collect::<Vec<EntityId>>();
    /// let npcs2: Vec<EntityId> = storage.all_entities()
    ///     .map(|entity| entity.id()).collect::<Vec<EntityId>>();
    /// for (id1, id2) in npcs1.iter().zip(npcs2.iter()) {
    ///     assert_eq!(id1, id2);
    /// }
    /// ```
    ///
    pub fn all_entities(&self) -> impl Iterator<Item = EntityRef> {
        (0..self.max_vacant_index).filter_map(|id| self.entity(EntityId(id)))
    }
    /// Returns mutable reference to entity in [`EntityComponentStorage`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::entities::{EntityId, EntityMut};
    /// # use std::iter::empty;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     empty()
    /// ).id();
    /// let player_mut: EntityMut = storage.entity_mut(player).expect("Entity was inserted");
    /// ```
    ///
    pub fn entity_mut(&mut self, entity_id: EntityId) -> Option<EntityMut> {
        if self.contains_entity(entity_id) {
            Some(EntityMut::new(entity_id, self))
        } else {
            None
        }
    }

    /// Inserts component into given entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId};
    /// # use ggengine::gamecore::entities::EntityId;
    /// # use std::iter::empty;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     empty()
    /// ).id();
    /// storage.insert_component(player, ComponentId::of::<Player>(), Box::new(Player));
    /// ```
    ///
    pub fn insert_component(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        boxed_component: BoxedComponent,
    ) -> Option<BoxedComponent> {
        self.insert_bundled_component_with_capacity(entity_id, component_id, boxed_component, 0)
    }
    /// Inserts components into given entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// # use std::iter::empty;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     empty()
    /// ).id();
    /// storage.insert_components(
    ///     player,
    ///     bundled_components((Player, Health(10))).into_iter()
    /// );
    /// ```
    ///
    pub fn insert_components(
        &mut self,
        entity_id: EntityId,
        components: impl Iterator<Item = BundledComponent>,
    ) {
        for (component_id, boxed_component) in components {
            let _ = self.insert_bundled_component_with_capacity(
                entity_id,
                component_id,
                boxed_component,
                0,
            );
        }
    }
    /// Removes component from an entity and returns the old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player,)).into_iter()
    /// ).id();
    /// storage.remove_component(player, ComponentId::of::<Player>());
    /// assert!(storage.contains_entity(player));
    /// ```
    ///
    pub fn remove_component(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> Option<BoxedComponent> {
        self.table
            .get_mut(&component_id)?
            .get_mut(entity_id.0)?
            .take()
    }
    /// Removes multiple components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Name(String);
    /// impl Component for Name {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player, Name("Alice".to_string()), Health(10),)).into_iter()
    /// ).id();
    /// storage.remove_components(
    ///     player,
    ///     [ComponentId::of::<Player>(), ComponentId::of::<Health>()].into_iter()
    /// );
    /// assert!(storage.contains_entity(player));
    /// assert!(!storage.contains_component(player, ComponentId::of::<Player>()));
    /// assert!(storage.contains_component(player, ComponentId::of::<Name>()));
    /// assert!(!storage.contains_component(player, ComponentId::of::<Health>()));
    /// ```
    ///
    pub fn remove_components(
        &mut self,
        entity_id: EntityId,
        component_ids: impl Iterator<Item = ComponentId>,
    ) {
        for component_id in component_ids {
            let _ = self.remove_component(entity_id, component_id);
        }
    }
    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player, Health(10))).into_iter()
    /// ).id();
    /// storage.remove_all_components(player);
    /// assert!(storage.contains_entity(player));
    /// assert!(!storage.contains_component(player, ComponentId::of::<Player>()));
    /// assert!(!storage.contains_component(player, ComponentId::of::<Health>()));
    /// ```
    ///
    pub fn remove_all_components(&mut self, entity_id: EntityId) {
        let entity_index = entity_id.0;
        for component_column in self.table.values_mut() {
            if entity_index < component_column.len() {
                component_column[entity_index] = None;
            }
        }
    }
    /// Returns whether this component is present in an entity or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player,)).into_iter()
    /// ).id();
    /// assert!(storage.contains_component(player, ComponentId::of::<Player>()));
    ///
    /// storage.remove_component(player, ComponentId::of::<Player>());
    /// assert!(!storage.contains_component(player, ComponentId::of::<Player>()));
    /// ```
    ///
    pub fn contains_component(&self, entity_id: EntityId, component_id: ComponentId) -> bool {
        self.table
            .get(&component_id)
            .and_then(|component_column| component_column.get(entity_id.0))
            .is_some_and(|component| component.is_some())
    }
    /// Extracts all component of one type from storage.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components, ComponentId, BoxedComponent};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs: Vec<EntityRef> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).collect::<Vec<EntityRef>>();
    /// let names: Vec<BoxedComponent> = storage.components_take(ComponentId::of::<Name>())
    ///     .expect("Component is present")
    ///     .collect::<Vec<BoxedComponent>>();
    /// assert_eq!(names.len(), 3);
    /// ```
    ///
    pub fn components_take(
        &mut self,
        component_id: ComponentId,
    ) -> Option<impl Iterator<Item = BoxedComponent> + use<'_>> {
        Some(
            self.table
                .remove(&component_id)?
                .into_iter()
                .enumerate()
                .filter_map(|(id, component)| {
                    if !self.removed_entities.contains(&EntityId(id)) {
                        component
                    } else {
                        None
                    }
                }),
        )
    }
    /// Returns immutable reference to the component of given entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player, Health(10))).into_iter()
    /// ).id();
    /// assert!(storage.component(player, ComponentId::of::<Health>()).is_some());
    /// ```
    ///
    pub fn component(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> Option<&BoxedComponent> {
        self.table.get(&component_id)?.get(entity_id.0)?.as_ref()
    }
    /// Returns immutable references to all components of one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components, ComponentId, BoxedComponent};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs: Vec<EntityRef> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).collect::<Vec<EntityRef>>();
    /// let names: Vec<&BoxedComponent> = storage.components(ComponentId::of::<Name>())
    ///     .expect("Component is present")
    ///     .collect::<Vec<&BoxedComponent>>();
    /// assert_eq!(names.len(), 3);
    /// ```
    ///
    pub fn components(
        &self,
        component_id: ComponentId,
    ) -> Option<impl Iterator<Item = &BoxedComponent>> {
        Some(
            self.table
                .get(&component_id)?
                .iter()
                .enumerate()
                .filter_map(|(id, component)| {
                    if !self.removed_entities.contains(&EntityId(id)) {
                        component.as_ref()
                    } else {
                        None
                    }
                }),
        )
    }
    /// Returns mutable reference to the component of given entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player, Health(10))).into_iter()
    /// ).id();
    /// assert!(storage.component_mut(player, ComponentId::of::<Health>()).is_some());
    /// ```
    ///
    pub fn component_mut(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> Option<&mut BoxedComponent> {
        self.table
            .get_mut(&component_id)?
            .get_mut(entity_id.0)?
            .as_mut()
    }
    /// Returns mutable references to all components of one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components, ComponentId, BoxedComponent};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs: Vec<EntityRef> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).collect::<Vec<EntityRef>>();
    /// let names: Vec<&mut BoxedComponent> = storage.components_mut(ComponentId::of::<Name>())
    ///     .expect("Component is present")
    ///     .collect::<Vec<&mut BoxedComponent>>();
    /// assert_eq!(names.len(), 3);
    /// ```
    ///
    pub fn components_mut(
        &mut self,
        component_id: ComponentId,
    ) -> Option<impl Iterator<Item = &mut BoxedComponent>> {
        Some(
            self.table
                .get_mut(&component_id)?
                .iter_mut()
                .enumerate()
                .filter_map(|(id, component)| {
                    if !self.removed_entities.contains(&EntityId(id)) {
                        component.as_mut()
                    } else {
                        None
                    }
                }),
        )
    }
    /// This function is the `HashMap::get_disjoint_mut` analogue.
    ///
    /// This method has no typed counterpart, and thus, is only usable through the [`EntityComponentStorage`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, bundled_components, ComponentId, BoxedComponent};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let npcs: Vec<EntityRef> = storage.insert_entities(vec![
    ///     bundled_components((NPC, Name("Alice"))).into_iter(),
    ///     bundled_components((NPC, Name("Bob"))).into_iter(),
    ///     bundled_components((NPC, Name("Charlie"))).into_iter()
    /// ]).collect::<Vec<EntityRef>>();
    /// let components: [Vec<&mut BoxedComponent>; 2] = storage.components_disjoint_mut(
    ///     [ComponentId::of::<NPC>(), ComponentId::of::<Name>()]
    /// ).map(|components|
    ///     components.expect("Component is present")
    ///         .collect::<Vec<&mut BoxedComponent>>()
    /// );
    /// ```
    ///
    pub fn components_disjoint_mut<const N: usize>(
        &mut self,
        component_ids: [ComponentId; N],
    ) -> [Option<impl Iterator<Item = &mut BoxedComponent>>; N] {
        self.table
            .get_disjoint_mut(component_ids.each_ref())
            .map(|option_component_column| {
                option_component_column.map(|component_column| {
                    component_column
                        .iter_mut()
                        .enumerate()
                        .filter_map(|(id, component)| {
                            if !self.removed_entities.contains(&EntityId(id)) {
                                component.as_mut()
                            } else {
                                None
                            }
                        })
                })
            })
    }
    /// Gets a mutable reference to the component of given type if present,
    /// otherwise inserts the component that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::{Component, ComponentId, bundled_components};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     bundled_components((Player,)).into_iter()
    /// ).id();
    /// let _ = storage.component_get_or_insert_with(
    ///     player,
    ///     ComponentId::of::<Health>(),
    ///     || Box::new(Health(10))
    /// );
    /// assert!(storage.contains_component(player, ComponentId::of::<Health>()));
    /// ```
    ///
    pub fn component_get_or_insert_with(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
        f: impl FnOnce() -> BoxedComponent,
    ) -> &mut BoxedComponent {
        if !self.contains_component(entity_id, component_id) {
            let _ = self.insert_bundled_component_with_capacity(entity_id, component_id, f(), 0);
        }
        self.component_mut(entity_id, component_id)
            .expect("Component was added if it was not already present")
    }
}

/// [`ResourceStorage`] struct provides API
/// for a storage of [`Resource`](super::components::Resource)s.
///
#[derive(Debug, Default)]
pub struct ResourceStorage {
    /// Map that stores resources.
    ///
    resources: IdMap<ResourceId, BoxedResource>,
}
impl ResourceStorage {
    /// Initializes new [`ResourceStorage`].
    ///
    /// Created [`ResourceStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// let storage: ResourceStorage = ResourceStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        ResourceStorage {
            resources: IdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    /// Inserts a new resource with the given value.
    ///
    /// Resources are unique data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data and this function will return old value.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// ```
    ///
    pub fn insert_resource(
        &mut self,
        resource_id: ResourceId,
        boxed_resource: BoxedResource,
    ) -> Option<BoxedResource> {
        self.resources.insert(resource_id, boxed_resource)
    }
    /// Removes the resource of a given type and returns it if present.
    /// Otherwise, returns `None`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.remove_resource(ResourceId::of::<Score>()).is_some());
    /// ```
    ///
    pub fn remove_resource(&mut self, resource_id: ResourceId) -> Option<BoxedResource> {
        self.resources.remove(&resource_id)
    }
    /// Returns whether a resource of given type exists or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains_resource(ResourceId::of::<Score>()));
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.contains_resource(ResourceId::of::<Score>()));
    ///
    /// let _ = storage.remove_resource(ResourceId::of::<Score>());
    /// assert!(!storage.contains_resource(ResourceId::of::<Score>()));
    /// ```
    ///
    pub fn contains_resource(&mut self, resource_id: ResourceId) -> bool {
        self.resources.contains_key(&resource_id)
    }
    /// Gets a reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(storage.resource(ResourceId::of::<Score>()).is_none());
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.resource(ResourceId::of::<Score>()).is_some());
    /// ```
    ///
    pub fn resource(&self, resource_id: ResourceId) -> Option<&BoxedResource> {
        self.resources.get(&resource_id)
    }
    /// Gets a mutable reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(storage.resource_mut(ResourceId::of::<Score>()).is_none());
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.resource_mut(ResourceId::of::<Score>()).is_some());
    /// ```
    ///
    pub fn resource_mut(&mut self, resource_id: ResourceId) -> Option<&mut BoxedResource> {
        self.resources.get_mut(&resource_id)
    }
    /// Gets a mutable reference to the resource of given type if present,
    /// otherwise inserts the resource that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains_resource(ResourceId::of::<Score>()));
    ///
    /// let _ = storage.resource_get_or_insert_with(ResourceId::of::<Score>(), || Box::new(Score(10)));
    /// assert!(storage.contains_resource(ResourceId::of::<Score>()));
    /// ```
    pub fn resource_get_or_insert_with(
        &mut self,
        resource_id: ResourceId,
        f: impl FnOnce() -> BoxedResource,
    ) -> &mut BoxedResource {
        self.resources.entry(resource_id).or_insert_with(|| f())
    }
}
