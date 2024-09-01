//! `gamecore::storages` hidden submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!

use crate::gamecore::{
    components::{BoxedComponent, Component},
    identifiers::{ComponentId, GameObjectId},
};
use std::{
    any::TypeId,
    collections::HashMap,
    hash::{BuildHasher, Hasher},
};

/// [`NoOpHasher`] struct is a hasher that removes overhead of hashing by directly passing
/// its internals through.
///
/// # Usage
/// ECS model heavily relies on fast querying and indexation of
/// [`Component`](Component)s and [`GameObject`](super::gameobjects::GameObject)s.
/// Id structs are indices for navigating their counterparts in [`Scene`](super::scenes::Scene) storage,
/// and those are implemented as newtype-wrappers of `u64`.
///
/// This hasher allows for those `u64`s to be used as keys in collections that require hashing
/// but without overhead of hashing. Those indices are already high-quality hashes because their
/// uniqueness is ensured, so this `Hasher` implementation can give an actual performance boost.
///
/// **This hasher only passes `u64` as a no-op hashing, `write` function will panic. **
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
/// This should be passed to collections interfaces (e.g. `HashMap::with_hasher(NoOpHasherState))`.
///
#[derive(Copy, Clone, Debug, Default)]
struct NoOpHasherState;
impl BuildHasher for NoOpHasherState {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher::default()
    }
}

/// Type alias for `HashMap<K, V, NoOpHasherState>`.
///
/// [`IdMap`] should be used wherever id structs are keys in a `HashMap`.
///
type IdMap<K, V> = HashMap<K, V, NoOpHasherState>;

/// [`ComponentMap`] struct handles `Component` initialization by binding specific `TypeId`s to exact `ComponentId`.
/// This approach allows for describing `Component`s as Rust types.
///
/// This struct is a wrapper of `IdMap<TypeId, ComponentId>`.
///
#[derive(Debug, Default)]
pub(super) struct ComponentMap {
    /// Map that binds `TypeId`s to `ComponentId`s.
    ///
    components: IdMap<TypeId, ComponentId>,
}
impl ComponentMap {
    /// Initializes an empty [`ComponentMap`] that uses [`NoOpHasher`] internally.
    ///
    /// The [`ComponentMap`] is initially created with a capacity of 0,
    /// so it will not allocate until it is first inserted into.
    ///
    pub(super) fn new() -> ComponentMap {
        ComponentMap {
            components: IdMap::with_hasher(NoOpHasherState),
        }
    }
    /// Initializes an empty [`ComponentMap`] that uses [`NoOpHasher`] internally
    /// with at least the specified capacity.
    ///
    /// The [`ComponentMap`] will be able to hold at least `capacity` elements without reallocating.
    /// This method is allowed to allocate for more elements than `capacity`.
    /// If `capacity` is 0, it will not allocate.
    ///
    pub(super) fn with_capacity(capacity: usize) -> ComponentMap {
        ComponentMap {
            components: IdMap::with_capacity_and_hasher(capacity, NoOpHasherState),
        }
    }

    /// Initializes `Component` type in [`ComponentMap`] and returns assigned `ComponentId`.
    /// If `Component` was already initialized it returns `ComponentId` that was assigned previously.
    ///
    /// # Complexity
    /// Insertion and lookup in the [`ComponentMap`] are both amortized `O(1)`.
    ///
    pub(super) fn get_or_insert<T: Component>(&mut self) -> ComponentId {
        let type_id: TypeId = TypeId::of::<T>();
        let component_id: ComponentId = ComponentId::new(self.components.len() as u64);
        *self.components.entry(type_id).or_insert(component_id)
    }
    /// Removes `Component` type from [`ComponentMap`] and returns its previous `ComponentId`.
    /// If this `Component` was not initialized, returns `None`.
    ///
    /// # Complexity
    /// Removal in the [`ComponentMap`] is amortized `O(1)`.
    ///
    pub(super) fn remove<T: Component>(&mut self) -> Option<ComponentId> {
        self.components.remove(&TypeId::of::<T>())
    }
    /// Returns `ComponentId` that corresponds to `Component` type.
    /// If this `Component` was not initialized, returns `None`.
    ///
    /// # Complexity
    /// Lookup in the [`ComponentMap`] are both amortized `O(1)`.
    ///
    pub(super) fn get<T: Component>(&self) -> Option<ComponentId> {
        self.components.get(&TypeId::of::<T>()).copied()
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the [`ComponentMap`] might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///
    pub(super) fn capacity(&self) -> usize {
        self.components.capacity()
    }
    /// Returns the number of elements in the map.
    ///
    pub(super) fn len(&self) -> usize {
        self.components.len()
    }
    /// Returns `true` if the map contains no elements, otherwise `false`.
    ///
    pub(super) fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

/// [`ComponentTable`] is a column-oriented structure-of-arrays based storage
/// that maps `GameObject`s to their `Component`s.
///
/// Conceptually, [`ComponentTable`] can be thought of as an `HashMap<ComponentId, Vec<T: Component>>`,
/// where each `Vec` contains components of one type that belong to different `GameObject`s.
///
/// Each row corresponds to a single `GameObject`
/// (i.e. equal indices of `Vec`s point to different components on the same entity)
/// and each column corresponds to a single `Component`
/// (i.e. every `Vec` contains all `Component`s of one type that belong to different `GameObject`s).
///
/// Fetching components from a table involves fetching the associated column for a `Component` type
/// (via its `ComponentId`), then fetching the `GameObject`'s row within that column.
///
/// # Performance
/// [`ComponentTable`]'s maps use [`NoOpHasher`], because ids are reliable hashes due to implementation.
/// This speeds up those 'amortized `O(1)`' even more.
///
/// Since components are stored in columnar contiguous blocks of memory, table is optimized for fast querying,
/// but frequent insertion and removal can be relatively slow.
///
/// Implementation with an actual table (represented by `Vec` that emulates `Vec<Vec<T: Component>>`)
/// could be a bit faster on querying due to cache locality, but insertion and removal would be very slow
/// (insertion would require shifting most of the table and removal would too,
/// unless we decide to just 'forget' deleted data, but this will hurt cache locality badly).
/// Chosen approach is a good trade-off between speed of lookups/querying and speed of insertion/removal,
/// with the accent on the former.
///
/// To see more on complexity topic, you can read docs for corresponding methods.
///
#[derive(Debug, Default)]
pub(super) struct ComponentTable {
    /// Map that tracks which position in table belongs to exact `GameObjectId`.
    ///
    /// Usage of this map and `removed` vector allows packing `GameObject`s in the table tightly,
    /// which optimises cache locality and memory usage.
    ///
    gameobject_map: IdMap<GameObjectId, usize>,
    /// Vector that holds indices that were binded to removed `GameObjectId`s.
    ///
    /// Insertion of `GameObjectId`s will use removed indices if possible,
    /// so that blanks caused by removal of `GameObjectId`s are filled with
    /// following insertions.
    ///
    removed: Vec<usize>,

    /// Table that holds all components.
    ///
    component_table: IdMap<ComponentId, Vec<Option<BoxedComponent>>>,
}
impl ComponentTable {
    /// Initializes new [`ComponentTable`].
    ///
    /// Created [`ComponentTable`] will not allocate until first insertions.
    ///
    /// If you know how much `Component`s and `GameObject`s you are going to use,
    /// use methods that initialize [`ComponentTable`] with capacity.
    /// That could greatly increase performance, especially if [`ComponentTable`]
    /// will need to handle frequent insertions and deletions.
    ///
    pub(super) fn new() -> ComponentTable {
        ComponentTable {
            gameobject_map: IdMap::with_hasher(NoOpHasherState),
            removed: Vec::new(),

            component_table: IdMap::with_hasher(NoOpHasherState),
        }
    }
    /// Initializes [`ComponentTable`] with specified capacity for both `GameObject` and `Component` storage.
    ///
    /// Usage of this associated function should be preferred, because it can greatly increase performance
    /// by decreasing number of allocations.
    ///
    /// Use this associated function if you have an estimation on how much
    /// `GameObject`s and `Component`s you are going to use.
    /// If you are unsure of one of the capacities, pass 0 to it.
    ///
    pub(super) fn with_capacity(
        gameobject_capacity: usize,
        component_capacity: usize,
    ) -> ComponentTable {
        ComponentTable {
            gameobject_map: IdMap::with_capacity_and_hasher(gameobject_capacity, NoOpHasherState),
            removed: Vec::with_capacity(gameobject_capacity),

            component_table: IdMap::with_capacity_and_hasher(component_capacity, NoOpHasherState),
        }
    }

    /// Inserts `GameObjectId` in the [`ComponentTable`].
    ///
    /// If any `GameObjectId`s were removed from [`ComponentTable`] and
    /// their places are not yet filled, inserted `GameObjectId` gets one of theirs place.
    /// Filling gaps between `GameObjectId`s ensures contiguity of data, and thus
    /// provides fast querying and efficient memory usage.
    ///
    /// # Complexity
    /// Insertion requires checking last item in `removed` vector which is `O(1)`
    /// and insertion of this value in a map if it is not already present is amortized `O(1)`.
    /// Overall complexity is amortized `O(1)`.
    ///
    pub(super) fn insert_gameobject(&mut self, gameobject_id: GameObjectId) {
        let new_index: usize = self.removed.pop().unwrap_or(self.gameobject_count());
        let _ = self
            .gameobject_map
            .entry(gameobject_id)
            .or_insert(new_index);
    }
    /// Removes `GameObjectId` from the [`ComponentTable`].
    ///
    /// Gaps that `GameObjectId`s leave after removal are filled with upcoming insertions.
    ///
    /// # Complexity
    /// Removal requires removal of `GameObjectId` from map which is amortized `O(1)`
    /// and removal of components from columns which requires iterating through `self.component_count()` columns,
    /// so it is `O(self.component_count())`.
    /// Overall complexity is `O(self.component_count())`.
    ///
    pub(super) fn remove_gameobject(&mut self, gameobject_id: GameObjectId) {
        let Some(deleted_index) = self.gameobject_map.remove(&gameobject_id) else {
            return;
        };
        self.removed.push(deleted_index);
        for components in self.component_table.values_mut() {
            if let Some(component) = components.get_mut(deleted_index) {
                *component = None;
            }
        }
    }

    /// Inserts column that corresponds to given `ComponentId` if not present.
    ///
    /// This function allocates column with capacity that is equal to `self.gameobject_capacity()`.
    ///
    /// # Complexity
    /// Insertion requires insertion to map which is amortized `O(1)`.
    /// Overall complexity is amortized `O(1)`.
    ///
    pub(super) fn insert_component(&mut self, component_id: ComponentId) {
        let gameobjects: usize = self.gameobject_count();
        let gameobject_capacity: usize = self.gameobject_capacity();
        self.component_table
            .entry(component_id)
            .or_insert(Vec::with_capacity(gameobject_capacity))
            .resize_with(gameobjects, || None);
    }
    /// Adds component to `GameObjectId` if both `GameObjectId` and `ComponentId` are tracked by [`ComponentTable`].
    /// If either `GameObjectId` or `ComponentId` are not present, does nothing.
    ///
    /// This function can be also used to replace component if needed.
    ///
    /// # Complexity
    /// Insertion requires 2 lookups on maps which are amortized `O(1)`
    /// and changing value in a vector which is `O(1)`.
    /// Overall complexity is amortized `O(1)`.
    ///
    pub(super) fn add_component_to_gameobject(
        &mut self,
        component_id: ComponentId,
        component: BoxedComponent,
        gameobject_id: GameObjectId,
    ) {
        let Some(&gameobject_index) = self.gameobject_map.get(&gameobject_id) else {
            return;
        };
        let Some(components) = self.component_table.get_mut(&component_id) else {
            return;
        };
        if gameobject_index >= components.len() {
            components.resize_with(gameobject_index + 1, || None);
        }

        let place: &mut Option<BoxedComponent> = components
            .get_mut(gameobject_index)
            .expect("Existence of index has been ensured.");
        *place = Some(component);
    }
    /// Removes column that corresponds to given `ComponentId` if present.
    ///
    /// # Complexity
    /// Removal requires removing key from map which is amortized `O(1)`.
    /// Overall complexity is amortized `O(1)`.
    ///
    pub(super) fn remove_component(&mut self, component_id: ComponentId) {
        let _ = self.component_table.remove(&component_id);
    }
    /// Removes component from `GameObjectId` if both `GameObjectId` and `ComponentId` are tracked by [`ComponentTable`].
    /// If either `GameObjectId` or `ComponentId` are not present, does nothing.
    ///
    /// # Complexity
    /// Removal requires 2 lookups on maps which are amortized `O(1)`
    /// and changing value in a vector which is `O(1)`.
    /// Overall complexity is amortized `O(1)`.
    ///
    pub(super) fn remove_component_from_gameobject(
        &mut self,
        component_id: ComponentId,
        gameobject_id: GameObjectId,
    ) {
        let Some(&gameobject_index) = self.gameobject_map.get(&gameobject_id) else {
            return;
        };
        let Some(components) = self.component_table.get_mut(&component_id) else {
            return;
        };
        let Some(component) = components.get_mut(gameobject_index) else {
            return;
        };
        *component = None;
    }

    /// Returns component that has given `ComponentId` and is assigned to `GameObject` with given id if present,
    /// otherwise `None`.
    ///
    /// # Complexity
    /// Retrieval requires 2 lookups on maps which are amortized `O(1)`
    /// and retrieving value from a vector which is `O(1)`.
    /// Overall complexity is amortized `O(1)`.
    ///    
    pub(super) fn get_gameobject_component(
        &self,
        gameobject_id: GameObjectId,
        component_id: ComponentId,
    ) -> Option<&Option<BoxedComponent>> {
        let gameobject_index: &usize = self.gameobject_map.get(&gameobject_id)?;
        let components: &Vec<Option<BoxedComponent>> = self.component_table.get(&component_id)?;
        Some(components.get(*gameobject_index).unwrap_or_else(|| &None))
    }

    /// Returns the number of `GameObject`s the table can hold without reallocating.
    ///
    /// This number is a lower bound; the [`ComponentTable`] might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///     
    pub(super) fn gameobject_capacity(&self) -> usize {
        self.gameobject_map.capacity()
    }
    /// Returns the number of `Component`s the table can hold without reallocating.
    ///
    /// This number is a lower bound; the [`ComponentTable`] might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///
    pub(super) fn component_capacity(&self) -> usize {
        self.component_table.capacity()
    }

    /// Returns the number of `GameObject`s in the map.
    ///
    pub(super) fn gameobject_count(&self) -> usize {
        self.gameobject_map.len()
    }
    /// Returns the number of `Component`s in the map.
    ///
    pub(super) fn component_count(&self) -> usize {
        self.component_table.len()
    }

    /// Checks whether given `GameObjectId` is tracked by [`ComponentTable`] or not.
    ///
    pub(super) fn has_gameobject(&self, gameobject_id: GameObjectId) -> bool {
        self.gameobject_map.contains_key(&gameobject_id)
    }
    /// Checks whether given `ComponentId` is tracked by [`ComponentTable`] or not.
    ///
    pub(super) fn has_component(&self, component_id: ComponentId) -> bool {
        self.component_table.contains_key(&component_id)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn component_map() {
        use super::ComponentMap;
        use crate::gamecore::identifiers::ComponentId;

        let mut component_map: ComponentMap = ComponentMap::new();

        let component_id_u8: ComponentId = component_map.get_or_insert::<u8>();
        assert_eq!(component_map.get_or_insert::<u8>(), component_id_u8);

        assert_eq!(component_map.remove::<u8>(), Some(component_id_u8));
        assert!(component_map.is_empty());
        assert!(component_map.remove::<u8>().is_none());

        let component_id_i8: ComponentId = component_map.get_or_insert::<i8>();
        let component_id_u8: ComponentId = component_map.get_or_insert::<u8>();
        assert_eq!(component_map.get_or_insert::<u8>(), component_id_u8);
        assert_eq!(component_map.get_or_insert::<i8>(), component_id_i8);

        assert_ne!(component_map.get_or_insert::<u8>(), component_id_i8);
    }

    #[test]
    fn component_table() {
        use super::{BoxedComponent, ComponentTable};
        use crate::gamecore::{
            components::Component,
            identifiers::{ComponentId, GameObjectId},
        };
        use std::ops::Deref;

        let gameobject_id0: GameObjectId = GameObjectId::new(0);
        let gameobject_id1: GameObjectId = GameObjectId::new(1);

        let component_id0: ComponentId = ComponentId::new(0);
        const COMPONENT0: u8 = 0;
        impl Component for u8 {}
        let component_id1: ComponentId = ComponentId::new(1);
        const COMPONENT1: i8 = 0;
        impl Component for i8 {}

        let mut component_table: ComponentTable = ComponentTable::new();

        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .is_none());
        component_table.insert_gameobject(gameobject_id0);

        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .is_none());
        component_table.insert_component(component_id0);

        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .is_some());
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id1)
            .is_none());
        assert!(component_table
            .get_gameobject_component(gameobject_id1, component_id0)
            .is_none());
        assert!(component_table
            .get_gameobject_component(gameobject_id1, component_id1)
            .is_none());

        component_table.add_component_to_gameobject(
            component_id0,
            Box::new(COMPONENT0),
            gameobject_id0,
        );
        let retrieval: &Option<BoxedComponent> = component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .expect("Component was added.");
        let retrieved_component: &BoxedComponent =
            retrieval.as_ref().expect("Component was added.");
        assert_eq!(
            retrieved_component
                .deref()
                .as_any()
                .downcast_ref::<u8>()
                .expect("u8 was packed."),
            &COMPONENT0
        );

        component_table.insert_component(component_id1);
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id1)
            .expect("GameObjectId and ComponentId were inserted.")
            .as_ref()
            .is_none());

        component_table.remove_gameobject(gameobject_id0);
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .is_none());

        component_table.insert_gameobject(gameobject_id0);
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .is_some());
        component_table.insert_gameobject(gameobject_id1);
        assert!(component_table
            .get_gameobject_component(gameobject_id1, component_id0)
            .is_some());

        component_table.add_component_to_gameobject(
            component_id1,
            Box::new(COMPONENT1),
            gameobject_id1,
        );
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id1)
            .expect("GameObjectId and ComponentId were inserted.")
            .as_ref()
            .is_none());

        assert!(component_table
            .get_gameobject_component(gameobject_id1, component_id1)
            .expect("GameObjectId and ComponentId were inserted.")
            .as_ref()
            .is_some());

        component_table.add_component_to_gameobject(
            component_id0,
            Box::new(COMPONENT0),
            gameobject_id0,
        );
        component_table.add_component_to_gameobject(
            component_id0,
            Box::new(COMPONENT0),
            gameobject_id1,
        );
        assert_eq!(
            component_table
                .get_gameobject_component(gameobject_id0, component_id0)
                .expect("Component was added.")
                .as_ref()
                .expect("Component was added.")
                .deref()
                .as_any()
                .downcast_ref::<u8>()
                .expect("u8 was packed."),
            component_table
                .get_gameobject_component(gameobject_id1, component_id0)
                .expect("Component was added.")
                .as_ref()
                .expect("Component was added.")
                .deref()
                .as_any()
                .downcast_ref::<u8>()
                .expect("u8 was packed.")
        );

        component_table.remove_component_from_gameobject(component_id0, gameobject_id0);
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .expect("GameObjectId and ComponentId were inserted.")
            .as_ref()
            .is_none());

        component_table.remove_component(component_id0);
        assert!(component_table
            .get_gameobject_component(gameobject_id0, component_id0)
            .is_none());
        assert!(component_table
            .get_gameobject_component(gameobject_id1, component_id0)
            .is_none());
    }
}
