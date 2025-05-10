//! `gamecore::storages` hidden submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!

use crate::gamecore::{
    components::{
        BoxedComponent, BoxedResource, Bundle, BundledComponent, Component, ComponentId,
        Downcastable, Resource, ResourceId,
    },
    entities::{EntityId, EntityMut},
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
/// Conceptually, [`EntityComponentStorage`] can be thought of as an `HashMap<ComponentId, Vec<C: Component>>`,
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
/// Implementation with an actual table (represented by `Vec` that emulates `Vec<Vec<C: Component>>`)
/// could be a bit faster on querying due to cache locality, but insertion and removal would be very slow
/// (insertion would require shifting most of the table and removal would too,
/// unless we decide to just 'forget' deleted data, but this will hurt cache locality badly).
/// Chosen approach is a good trade-off between speed of lookups/querying and speed of insertion/removal,
/// with the accent on the former.
///
/// To see more on complexity topic, you can read docs for corresponding methods.
///
#[derive(Debug)]
pub(super) struct EntityComponentStorage {
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
    /// If you know how many components and entities you are going to use,
    /// use methods that initialize [`EntityComponentStorage`] with capacity.
    /// That could greatly increase performance, especially if [`EntityComponentStorage`]
    /// will need to handle frequent insertions and deletions.
    ///
    pub(super) fn new() -> Self {
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
    fn insert_entity_with_capacity(
        &mut self,
        bundle: impl Bundle,
        entities_count_capacity: usize,
    ) -> EntityMut {
        let entity_id: EntityId = match self.removed_entities.iter().next().copied() {
            Some(id) => {
                let _ = self.removed_entities.remove(&id);
                id
            }
            None => {
                let reserved_id: EntityId = EntityId(self.max_vacant_index);
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
        let (component_id, boxed_component): (ComponentId, BoxedComponent) =
            bundled_component.destructure();

        let components: &mut Vec<Option<BoxedComponent>> = self
            .table
            .entry(component_id)
            .or_insert(Vec::with_capacity(entities_count_capacity));

        let entity_index: usize = entity_id.0;
        if components.len() <= entity_index {
            components.resize_with(
                if entities_count_capacity == 0 {
                    entity_index
                } else {
                    entities_count_capacity
                },
                || None,
            );
        }

        components[entity_index].replace(boxed_component)
    }

    /// Inserts empty entity into [`EntityComponentStorage`] and returns mutable reference to it,
    /// so it could be further modified.
    ///
    pub(super) fn insert_entity_empty(&mut self) -> EntityMut {
        self.insert_entity(())
    }
    /// Inserts entity with components that are given in a [`Bundle`]
    /// into [`EntityComponentStorage`] and returns mutable reference to it,
    /// so it could be further modified.
    ///
    pub(super) fn insert_entity(&mut self, bundle: impl Bundle) -> EntityMut {
        self.insert_entity_with_capacity(bundle, 0)
    }
    /// Inserts multiple entities with components that are given in [`Bundle`]s
    /// into [`EntityComponentStorage`] and returns mutable reference to it,
    /// so it could be further modified.
    ///
    /// It is more efficient than calling `EntityComponentStorage::insert_entity` in a loop.
    ///
    pub(super) fn insert_entities(
        &mut self,
        bundles: impl IntoIterator<Item = impl Bundle> + ExactSizeIterator,
    ) {
        let adding: usize = bundles.len();
        let resizing: usize = if adding >= self.removed_entities.len() {
            let vacant: usize = self.removed_entities.len();
            let reserved: usize = self.max_vacant_index - vacant;
            reserved + (adding - vacant)
        } else {
            0
        };

        for bundle in bundles {
            let _ = self.insert_entity_with_capacity(bundle, resizing);
        }
    }
    /// Removes entity from [`EntityComponentStorage`] by removing all of its components.
    ///
    pub(super) fn remove_entity(&mut self, entity_id: EntityId) {
        if self.removed_entities.contains(&entity_id) {
            return;
        }
        let _ = self.removed_entities.insert(entity_id);

        let entity_index: usize = entity_id.0;
        for components in self.table.values_mut() {
            if components.len() > entity_index {
                components[entity_index] = None;
            }
        }
    }

    /// Inserts component to given entity.
    ///
    pub(super) fn insert_component<C: Component>(
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
    pub(super) fn insert_bundle(&mut self, entity_id: EntityId, bundle: impl Bundle) {
        for bundled_component in bundle.bundled_components() {
            self.insert_bundled_component_with_capacity(entity_id, bundled_component, 0);
        }
    }
    /// Removes component from an entity and returns the old value if present.
    ///
    pub(super) fn remove_component<C: Component>(&mut self, entity_id: EntityId) -> Option<C> {
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
    /// Returns immutable reference to the component of given entity if present.
    ///
    pub(super) fn get_component<C: Component>(&self, entity_id: EntityId) -> Option<&C> {
        self.table
            .get(&ComponentId::of::<C>())?
            .get(entity_id.0)?
            .as_ref()?
            .downcast_to_ref::<C>()
    }
    /// Returns mutable reference to the component of given entity if present.
    ///
    pub(super) fn get_component_mut<C: Component>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut C> {
        self.table
            .get_mut(&ComponentId::of::<C>())?
            .get_mut(entity_id.0)?
            .as_mut()?
            .downcast_to_mut::<C>()
    }
}

/// [`ResourceStorage`] struct provides API for a storage of `Resource`s.
///
/// Commonly, you will use this struct through the `Scene` which has its own [`ResourceStorage`].
/// `ggengine` still provides tools for manual constructions - you might want to use them to implement
/// efficient application reload or other specialized scenarios.
///
#[derive(Debug, Default)]
pub(super) struct ResourceStorage {
    /// Map that stores resources.
    ///
    resources: IdMap<ResourceId, BoxedResource>,
}
impl ResourceStorage {
    /// Initializes new [`ResourceStorage`].
    ///
    /// Created [`ResourceStorage`] will not allocate until first insertions.
    ///
    pub(super) fn new() -> Self {
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
    pub(super) fn insert_resource<R: Resource>(&mut self, value: R) -> Option<R> {
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
    pub(super) fn remove_resource<R: Resource>(&mut self) -> Option<R> {
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
    pub(super) fn contains_resource<R: Resource>(&mut self) -> bool {
        self.resources.contains_key(&ResourceId::of::<R>())
    }

    /// Gets a reference to the resource of the given type if present.
    ///
    pub(super) fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.resources
            .get(&ResourceId::of::<R>())?
            .downcast_to_ref::<R>()
    }
    /// Gets a mutable reference to the resource of the given type if present.
    ///
    pub(super) fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&ResourceId::of::<R>())?
            .downcast_to_mut::<R>()
    }

    /// Gets a mutable reference to the resource of given type if present,
    /// otherwise inserts the resource that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    pub(super) fn get_resource_or_insert_with<R: Resource>(
        &mut self,
        f: impl FnOnce() -> R,
    ) -> &mut R {
        self.resources
            .entry(ResourceId::of::<R>())
            .or_insert_with(|| Box::new(f()))
            .downcast_to_mut::<R>()
            .expect("This type corresponds to this value.")
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub(super) fn clear(&mut self) {
        self.resources.clear();
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::gamecore::components::{Component, Resource};
    use std::any::TypeId;

    impl Component for u8 {}
    impl Component for i8 {}

    impl Resource for u8 {}
    impl Resource for i8 {}

    #[test]
    fn component_map() {
        use super::{ComponentId, ComponentMap};

        let mut component_map: ComponentMap = ComponentMap::new();

        let component_id_u8: ComponentId = component_map.get_or_insert(TypeId::of::<u8>());
        assert_eq!(
            component_map.get_or_insert(TypeId::of::<u8>()),
            component_id_u8
        );

        assert_eq!(
            component_map.remove(&TypeId::of::<u8>()),
            Some(component_id_u8)
        );
        assert!(component_map.is_empty());
        assert!(component_map.remove(&TypeId::of::<u8>()).is_none());

        let component_id_i8: ComponentId = component_map.get_or_insert(TypeId::of::<i8>());
        let component_id_u8: ComponentId = component_map.get_or_insert(TypeId::of::<u8>());
        assert_eq!(
            component_map.get_or_insert(TypeId::of::<u8>()),
            component_id_u8
        );
        assert_eq!(
            component_map.get_or_insert(TypeId::of::<i8>()),
            component_id_i8
        );

        let _ = component_map.remove(&TypeId::of::<i8>());

        let component_id_i8: ComponentId = component_map.get_or_insert(TypeId::of::<i8>());
        assert_eq!(
            component_map.get_or_insert(TypeId::of::<i8>()),
            component_id_i8
        );

        assert_ne!(
            component_map.get_or_insert(TypeId::of::<u8>()),
            component_id_i8
        );
    }

    #[test]
    fn resource_map() {
        use super::{ResourceId, ResourceMap};

        let mut resource_map: ResourceMap = ResourceMap::new();

        let resource_id_u8: ResourceId = resource_map.get_or_insert(TypeId::of::<u8>());
        assert_eq!(
            resource_map.get_or_insert(TypeId::of::<u8>()),
            resource_id_u8
        );

        assert_eq!(
            resource_map.remove(&TypeId::of::<u8>()),
            Some(resource_id_u8)
        );
        assert!(resource_map.is_empty());
        assert!(resource_map.remove(&TypeId::of::<u8>()).is_none());

        let resource_id_i8: ResourceId = resource_map.get_or_insert(TypeId::of::<i8>());
        let resource_id_u8: ResourceId = resource_map.get_or_insert(TypeId::of::<u8>());
        assert_eq!(
            resource_map.get_or_insert(TypeId::of::<u8>()),
            resource_id_u8
        );
        assert_eq!(
            resource_map.get_or_insert(TypeId::of::<i8>()),
            resource_id_i8
        );

        let _ = resource_map.remove(&TypeId::of::<i8>());

        let resource_id_i8: ResourceId = resource_map.get_or_insert(TypeId::of::<i8>());
        assert_eq!(
            resource_map.get_or_insert(TypeId::of::<i8>()),
            resource_id_i8
        );

        assert_ne!(
            resource_map.get_or_insert(TypeId::of::<u8>()),
            resource_id_i8
        );
    }

    #[test]
    fn entity_component_storage() {
        use super::{BoxedComponent, ComponentId, EntityComponentStorage, EntityId};
        use std::ops::Deref;

        let entity_id0: EntityId = EntityId(0);
        let entity_id1: EntityId = EntityId(1);

        let component_id0: ComponentId = ComponentId(0);
        const COMPONENT0: u8 = 0;
        let component_id1: ComponentId = ComponentId(1);
        const COMPONENT1: i8 = 0;

        let mut component_table: EntityComponentStorage = EntityComponentStorage::new();

        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .is_none());
        let _ = component_table.insert_entity(entity_id0);

        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .is_none());
        let _ = component_table.insert_component(component_id0);

        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .is_some());
        assert!(component_table
            .get_entity_component(entity_id0, component_id1)
            .is_none());
        assert!(component_table
            .get_entity_component(entity_id1, component_id0)
            .is_none());
        assert!(component_table
            .get_entity_component(entity_id1, component_id1)
            .is_none());

        component_table.add_component_to_entity((component_id0, Box::new(COMPONENT0)), entity_id0);
        let retrieval: &Option<BoxedComponent> = component_table
            .get_entity_component(entity_id0, component_id0)
            .expect("Component was added.");
        let retrieved_component: &BoxedComponent =
            retrieval.as_ref().expect("Component was added.");
        assert_eq!(
            retrieved_component
                .deref()
                .as_any_ref()
                .downcast_ref::<u8>()
                .expect("u8 was packed."),
            &COMPONENT0
        );

        component_table.insert_component(component_id1);
        assert!(component_table
            .get_entity_component(entity_id0, component_id1)
            .expect("EntityId and ComponentId were inserted.")
            .as_ref()
            .is_none());

        component_table.remove_entity(entity_id0);
        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .is_none());

        let _ = component_table.insert_entity(entity_id0);
        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .is_some());
        let _ = component_table.insert_entity(entity_id1);
        assert!(component_table
            .get_entity_component(entity_id1, component_id0)
            .is_some());

        component_table.add_component_to_entity((component_id1, Box::new(COMPONENT1)), entity_id1);
        assert!(component_table
            .get_entity_component(entity_id0, component_id1)
            .expect("EntityId and ComponentId were inserted.")
            .as_ref()
            .is_none());

        assert!(component_table
            .get_entity_component(entity_id1, component_id1)
            .expect("EntityId and ComponentId were inserted.")
            .as_ref()
            .is_some());

        component_table.add_component_to_entity((component_id0, Box::new(COMPONENT0)), entity_id0);
        component_table.add_component_to_entity((component_id0, Box::new(COMPONENT0)), entity_id1);
        assert_eq!(
            component_table
                .get_entity_component(entity_id0, component_id0)
                .expect("Component was added.")
                .as_ref()
                .expect("Component was added.")
                .deref()
                .as_any_ref()
                .downcast_ref::<u8>()
                .expect("u8 was packed."),
            component_table
                .get_entity_component(entity_id1, component_id0)
                .expect("Component was added.")
                .as_ref()
                .expect("Component was added.")
                .deref()
                .as_any_ref()
                .downcast_ref::<u8>()
                .expect("u8 was packed.")
        );

        component_table.remove_component_from_entity(component_id0, entity_id0);
        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .expect("EntityId and ComponentId were inserted.")
            .as_ref()
            .is_none());

        let _ = component_table.remove_component(component_id0);
        assert!(component_table
            .get_entity_component(entity_id0, component_id0)
            .is_none());
        assert!(component_table
            .get_entity_component(entity_id1, component_id0)
            .is_none());
    }

    #[test]
    fn resource_storage() {
        use super::ResourceStorage;

        let mut resource_storage: ResourceStorage = ResourceStorage::new();

        let _ = resource_storage.insert_resource(0u8);
        let _ = resource_storage.insert_resource(0i8);

        assert!(resource_storage.contains_resource::<u8>());
        assert_eq!(resource_storage.insert_resource(1u8), Some(0u8));
        let resource: &mut u8 = resource_storage
            .get_resource_mut::<u8>()
            .expect("`u8` resource was added");

        assert_eq!(*resource, 1u8);
        *resource = 2u8;

        assert_eq!(
            resource_storage
                .remove_resource::<u8>()
                .expect("`u8` resource was added"),
            2u8
        );

        resource_storage.clear_resources();
        assert!(resource_storage.get_resource::<i8>().is_none());

        let resource: &mut u8 = resource_storage.get_resource_or_insert_with(|| 0u8);
        *resource = 1u8;
        assert_eq!(*resource_storage.get_resource_or_insert_with(|| 5u8), 1u8);
        assert_eq!(
            *resource_storage
                .get_resource::<u8>()
                .expect("`u8` resource was added"),
            1u8
        );
    }
}
*/
