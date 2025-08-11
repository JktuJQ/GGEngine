//! `gamecore::storages` submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!

use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasher, Hasher},
};

/// [`NoOpHasher`] struct is a hasher that removes overhead of hashing by directly passing
/// its internals through.
///
/// # Usage
/// ECS model heavily relies on fast querying and indexation of components and entities.
/// Id structs are indices for navigating their counterparts in storages,
/// and those are implemented as wrappers of `u64`.
///
/// This hasher allows for those `u64`s to be used as keys in collections that require hashing
/// but without overhead of hashing.
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

    fn write_u64(&mut self, x: u64) {
        self.0 = x;
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
/// [`TypeIdSet`] should be used wherever id structs are keys in a `HashSet`.
///
type TypeIdSet<T> = HashSet<T, NoOpHasherState>;
/// Type alias for `HashMap<K, V, NoOpHasherState>`.
///
/// [`TypeIdMap`] should be used wherever id structs are keys in a `HashMap`.
///
type TypeIdMap<K, V> = HashMap<K, V, NoOpHasherState>;

// submodules and public re-exports
mod entity_component_storage;
pub use entity_component_storage::*;

mod resource_storage;
pub use resource_storage::*;

mod event_storage;
pub use event_storage::*;
