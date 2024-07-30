//! `gamecore::storages` submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!

use crate::gamecore::{
    components::Component,
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
/// [`Component`](super::components::Component)s and [`GameObject`](super::gameobjects::GameObject)s.
/// Id structs are indices for navigating their counterparts in [`Scene`](super::scenes::Scene) storage,
/// and those are implemented as newtype-wrappers of `usize`.
///
/// This hasher allows for those `usize`s to be used as keys in collections that require hashing
/// but without overhead of hashing. Those indices are already high-quality hashes because their
/// uniqueness is ensured, so this `Hasher` implementation can give an actual performance boost.
///
/// [`NoOpHasher`] also passes `u64` without hashing
/// (but still conversion to `usize` has to be made).
/// That is made to target `TypeId` struct and map it to ids.
///
/// **This hasher only passes `usize` and `u64` as a no-op hashing,
/// `write` function should not be used (it's implementation is not suited for usage);
/// use `write_usize` or `write_u64` instead.**
///
#[derive(Copy, Clone, Debug)]
pub struct NoOpHasher(usize);
impl Hasher for NoOpHasher {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = bytes.iter().fold(self.0, |hash, x| {
            hash.rotate_right(8).wrapping_add(*x as usize)
        })
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i as usize;
    }
    fn write_usize(&mut self, i: usize) {
        self.0 = i;
    }
}

/// [`NoOpHasherState`] struct implements `BuildHasher` trait that produces [`NoOpHasher`].
///
/// This should be passed to collections interfaces (e.g. `HashMap::with_hasher(NoOpHasherState))`.
///
#[derive(Copy, Clone, Debug)]
pub struct NoOpHasherState;
impl BuildHasher for NoOpHasherState {
    type Hasher = NoOpHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher(0)
    }
}

/// Type alias for `HashMap<K, V, NoOpHasherState>`.
///
/// [`IdMap`] should be used wherever id structs are keys in a `HashMap`.
///
pub type IdMap<K, V> = HashMap<K, V, NoOpHasherState>;

/// [`ComponentMap`] struct handles `Component` initialization by binding specific `TypeId`s to exact `ComponentId`.
/// This approach allows for describing `Component`s as Rust types.
///
/// This struct is a wrapper of `IdMap<TypeId, ComponentId>`.
///
#[derive(Debug)]
pub struct ComponentMap {
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
    pub fn new() -> ComponentMap {
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
    pub fn with_capacity(capacity: usize) -> ComponentMap {
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
    pub fn get_or_insert<T: Component>(&mut self) -> ComponentId {
        let type_id: TypeId = TypeId::of::<T>();
        let component_id: ComponentId = ComponentId::new(self.components.len());
        *self.components.entry(type_id).or_insert(component_id)
    }
    /// Removes `Component` type from [`ComponentMap`] and returns its previous `ComponentId`.
    /// If this `Component` was not initialized, returns `None`.
    ///
    /// # Complexity
    /// Removal in the [`ComponentMap`] is amortized `O(1)`.
    ///
    pub fn remove<T: Component>(&mut self) -> Option<ComponentId> {
        self.components.remove(&TypeId::of::<T>())
    }
    /// Returns `ComponentId` that corresponds to `Component` type.
    /// If this `Component` was not initialized, returns `None`.
    ///
    /// # Complexity
    /// Lookup in the [`ComponentMap`] are both amortized `O(1)`.
    ///
    pub fn get<T: Component>(&self) -> Option<ComponentId> {
        self.components.get(&TypeId::of::<T>()).copied()
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the [`ComponentMap`] might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    ///
    pub fn capacity(&self) -> usize {
        self.components.capacity()
    }
    /// Returns the number of elements in the map.
    ///
    pub fn len(&self) -> usize {
        self.components.len()
    }
    /// Returns `true` if the map contains no elements, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

