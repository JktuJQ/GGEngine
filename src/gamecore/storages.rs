//! `gamecore::storages` hidden submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!

use crate::gamecore::{
    components::{
        BoxedComponent, BoxedResource, Bundle, BundledComponent, Component, ComponentId,
        Downcastable, Resource, ResourceId,
    },
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
/// and those are implemented as newtype-wrappers of `u64`.
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
        panic!("`write` method should not be used on `NoOpHasher`.");
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
/// that maps entities to their `Component`s.
///
/// Conceptually, [`EntityComponentStorage`] can be thought of as an `HashMap<ComponentId, Vec<impl Component>>`,
/// where each `Vec` contains components of one type that belong to different entities.
///
/// Each row corresponds to a single entity
/// (i.e. equal indices of `Vec`s point to different components on the same entity)
/// and each column corresponds to a single `Component`
/// (i.e. every `Vec` contains all `Component`s of one type that belong to different entities).
///
/// Fetching components from a table involves fetching the associated column for a `Component` type
/// (via its `ComponentId`), then fetching the entity's row within that column.
///
/// # Performance
/// [`EntityComponentStorage`] uses [`NoOpHasher`], because ids are reliable hashes due to implementation.
/// This speeds up those 'amortized `O(1)`' even more.
///
/// Since components are stored in columnar contiguous blocks of memory, table is optimized for fast querying,
/// but frequent insertion and removal can be relatively slow.
///
/// Implementation with an actual table (represented by `Vec` that emulates `Vec<Vec<impl Component>>`)
/// could be a bit faster on querying due to cache locality, but insertion and removal would be very slow
/// (insertion would require shifting most of the table and removal would too,
/// unless we decide to just 'forget' deleted data, but this will hurt cache locality badly).
/// Chosen approach is a good trade-off between speed of lookups/querying and speed of insertion/removal,
/// with the accent on the former.
///
/// # Note
/// This collection is designed to provide more fine-grained control over entity-component storage.
/// Most of the time you should use [`EntityMut`] or its readonly counterpart to get nice API
/// of editing entities.
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

    /// Inserts new entity supplying it with bundle.
    ///
    /// Main feature of this function is that it allows to pass additional capacity,
    /// which will allow for allocation optimizations.
    ///
    fn spawn_entity_with_capacity(
        &mut self,
        bundle: impl Bundle,
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

        for bundled_component in bundle.bundled_components() {
            let _ = self.insert_bundled_component_with_capacity(
                entity_id,
                bundled_component,
                entities_count_capacity,
            );
        }

        EntityMut::new(entity_id, self)
    }

    /// Inserts bundled component into existing entity.
    ///
    /// Main feature of this function is that it allows to pass additional capacity,
    /// which will allow for allocation optimizations.
    ///
    fn insert_bundled_component_with_capacity(
        &mut self,
        entity_id: EntityId,
        bundled_component: BundledComponent,
        entities_count_capacity: usize,
    ) -> Option<BoxedComponent> {
        let (component_id, boxed_component) = bundled_component.destructure();

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

    /// inserts entity with components that are given in a [`Bundle`]
    /// into [`EntityComponentStorage`] and returns mutable reference to it,
    /// so it could be further modified.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player,)).id();
    /// ```
    ///
    /// You can spawn empty entity to defer initialization by passing `()` as a [`Bundle`]:
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::entities::EntityId;
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity(()).id();
    /// ```
    ///
    pub fn spawn_entity(&mut self, bundle: impl Bundle) -> EntityMut {
        self.spawn_entity_with_capacity(bundle, 0)
    }
    /// Inserts multiple entities with components that are given in [`Bundle`]s
    /// into [`EntityComponentStorage`] and returns ids of those entities.
    ///
    /// It is more efficient than calling `EntityComponentStorage::spawn_entity` in a loop.
    ///
    /// # Note
    /// This function can only insert entities with the same [`Bundle`] type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let npcs: Vec<EntityId> = storage.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]);
    /// ```
    ///
    pub fn spawn_entities(
        &mut self,
        bundles: impl IntoIterator<Item = impl Bundle>,
    ) -> Vec<EntityId> {
        let bundles_iter = bundles.into_iter();
        let adding = {
            let (lower, upper) = bundles_iter.size_hint();
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
        for bundle in bundles_iter {
            ids.push(self.spawn_entity_with_capacity(bundle, resizing).id());
        }
        ids
    }
    /// Removes entity from [`EntityComponentStorage`] by removing all of its components.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player,)).id();
    /// storage.despawn_entity(player);
    /// ```
    ///
    pub fn despawn_entity(&mut self, entity_id: EntityId) -> bool {
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
    /// Returns whether an entity with given id is currently stored or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player,)).id();
    /// assert!(storage.contains_entity(player));
    /// storage.despawn_entity(player);
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
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity(()).id();
    /// let player_ref: EntityRef = storage.entity(player).expect("Entity was spawned.");
    /// ```
    ///
    pub fn entity(&self, entity_id: EntityId) -> Option<EntityRef> {
        if self.contains_entity(entity_id) {
            Some(EntityRef::new(entity_id, self))
        } else {
            None
        }
    }
    /// Returns mutable reference to entity in [`EntityComponentStorage`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::entities::{EntityId, EntityMut};
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity(()).id();
    /// let player_mut: EntityMut = storage.entity_mut(player).expect("Entity was spawned.");
    /// ```
    ///
    pub fn entity_mut(&mut self, entity_id: EntityId) -> Option<EntityMut> {
        if self.contains_entity(entity_id) {
            Some(EntityMut::new(entity_id, self))
        } else {
            None
        }
    }

    /// Inserts component to given entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity(()).id();
    /// storage.insert_component(player, Player);
    /// ```
    ///
    pub fn insert_component<C: Component>(
        &mut self,
        entity_id: EntityId,
        component: C,
    ) -> Option<C> {
        self.insert_bundled_component_with_capacity(
            entity_id,
            BundledComponent::bundle(component),
            0,
        )
        .map(|boxed_component| {
            boxed_component
                .downcast_to_value::<C>()
                .expect("This type corresponds to this value.")
        })
    }
    /// Inserts bundle of components to given entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity(()).id();
    /// storage.insert_bundle(player, (Player, Health(10)));
    /// ```
    ///
    pub fn insert_bundle(&mut self, entity_id: EntityId, bundle: impl Bundle) {
        for bundled_component in bundle.bundled_components() {
            let _ = self.insert_bundled_component_with_capacity(entity_id, bundled_component, 0);
        }
    }
    /// Removes component from an entity and returns the old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player,)).id();
    /// storage.remove_component::<Player>(player);
    /// assert!(storage.contains_entity(player));
    /// ```
    ///
    pub fn remove_component<C: Component>(&mut self, entity_id: EntityId) -> Option<C> {
        self.table
            .get_mut(&ComponentId::of::<C>())?
            .get_mut(entity_id.0)?
            .take()
            .map(|boxed_component| {
                boxed_component
                    .downcast_to_value::<C>()
                    .expect("This type corresponds to this value.")
            })
    }
    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player, Health(10))).id();
    /// storage.remove_all_components(player);
    /// assert!(storage.contains_entity(player));
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
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player,)).id();
    /// assert!(storage.contains_component::<Player>(player));
    /// storage.remove_component::<Player>(player);
    /// assert!(!storage.contains_component::<Player>(player));
    /// ```
    ///
    pub fn contains_component<C: Component>(&self, entity_id: EntityId) -> bool {
        self.table
            .get(&ComponentId::of::<C>())
            .and_then(|component_column| component_column.get(entity_id.0))
            .map(|component| component.is_some())
            .is_some_and(|present| present)
    }
    /// Returns immutable reference to the component of given entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player, Health(10))).id();
    /// assert_eq!(storage.component::<Health>(player).expect("Component is present.").0, 10);
    /// storage.remove_component::<Player>(player);
    /// assert!(storage.component::<Player>(player).is_none());
    /// ```
    ///
    pub fn component<C: Component>(&self, entity_id: EntityId) -> Option<&C> {
        self.table
            .get(&ComponentId::of::<C>())?
            .get(entity_id.0)?
            .as_ref()?
            .downcast_to_ref::<C>()
    }
    /// Returns mutable reference to the component of given entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player, Health(10))).id();
    /// let health: &mut Health = storage.component_mut::<Health>(player).expect("Component is present.");
    /// health.0 = 20;
    /// assert_eq!(storage.component::<Health>(player).expect("Component is present").0, 20);
    /// storage.remove_component::<Health>(player);
    /// assert!(storage.component_mut::<Health>(player).is_none());
    /// ```
    ///
    pub fn component_mut<C: Component>(&mut self, entity_id: EntityId) -> Option<&mut C> {
        self.table
            .get_mut(&ComponentId::of::<C>())?
            .get_mut(entity_id.0)?
            .as_mut()?
            .downcast_to_mut::<C>()
    }
    /// Gets a mutable reference to the component of given type if present,
    /// otherwise inserts the component that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EntityComponentStorage;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: EntityComponentStorage = EntityComponentStorage::new();
    /// let player: EntityId = storage.spawn_entity((Player,)).id();
    /// let _ = storage.component_get_or_insert_with(player, || Health(10));
    /// assert!(storage.contains_component::<Health>(player));
    /// ```
    ///
    pub fn component_get_or_insert_with<C: Component>(
        &mut self,
        entity_id: EntityId,
        f: impl FnOnce() -> C,
    ) -> &mut C {
        if !self.contains_component::<C>(entity_id) {
            let _ = self.insert_bundled_component_with_capacity(
                entity_id,
                BundledComponent::bundle(f()),
                0,
            );
        }
        self.component_mut::<C>(entity_id)
            .expect("Component was added if it was not already present.")
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.max_vacant_index = 0;
        self.removed_entities.clear();
        self.table.clear();
    }
}

/// [`ResourceStorage`] struct provides API for a storage of [`Resource`]s.
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

    /// Inserts a new resource with the given value.
    ///
    /// Resources are unique data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data and this function will return old value.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// storage.insert_resource::<Score>(Score(0));
    /// ```
    ///
    pub fn insert_resource<R: Resource>(&mut self, value: R) -> Option<R> {
        self.resources
            .insert(ResourceId::of::<R>(), Box::new(value))
            .map(|boxed_resource| {
                boxed_resource
                    .downcast_to_value::<R>()
                    .expect("This type corresponds to this value.")
            })
    }

    /// Removes the resource of a given type and returns it if present.
    /// Otherwise, returns `None`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// storage.insert_resource::<Score>(Score(0));
    /// assert_eq!(storage.remove_resource::<Score>().expect("Resource was inserted.").0, 0);
    /// ```
    ///
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources
            .remove(&ResourceId::of::<R>())
            .map(|boxed_resource| {
                boxed_resource
                    .downcast_to_value::<R>()
                    .expect("This type corresponds to this value.")
            })
    }

    /// Returns whether a resource of given type exists or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains_resource::<Score>());
    /// storage.insert_resource::<Score>(Score(0));
    /// assert!(storage.contains_resource::<Score>());
    /// storage.remove_resource::<Score>();
    /// assert!(!storage.contains_resource::<Score>());
    /// ```
    ///
    pub fn contains_resource<R: Resource>(&mut self) -> bool {
        self.resources.contains_key(&ResourceId::of::<R>())
    }

    /// Gets a reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(storage.resource::<Score>().is_none());
    /// storage.insert_resource::<Score>(Score(0));
    /// assert_eq!(storage.resource::<Score>().expect("Resource was inserted.").0, 0);
    /// ```
    ///
    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resources
            .get(&ResourceId::of::<R>())?
            .downcast_to_ref::<R>()
    }
    /// Gets a mutable reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(storage.resource_mut::<Score>().is_none());
    /// storage.insert_resource::<Score>(Score(0));
    /// storage.resource_mut::<Score>().expect("Resource was inserted.").0 = 10;
    /// assert_eq!(storage.resource::<Score>().expect("Resource was inserted.").0, 10);
    /// ```
    ///
    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&ResourceId::of::<R>())?
            .downcast_to_mut::<R>()
    }

    /// Gets a mutable reference to the resource of given type if present,
    /// otherwise inserts the resource that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains_resource::<Score>());
    /// let _ = storage.resource_get_or_insert_with(|| Score(10));
    /// assert!(storage.contains_resource::<Score>());
    /// ```
    pub fn resource_get_or_insert_with<R: Resource>(&mut self, f: impl FnOnce() -> R) -> &mut R {
        self.resources
            .entry(ResourceId::of::<R>())
            .or_insert_with(|| Box::new(f()))
            .downcast_to_mut::<R>()
            .expect("This type corresponds to this value.")
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.resources.clear();
    }
}
