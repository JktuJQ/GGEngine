//! `gamecore::storages` submodule implements several collections that
//! are used to store ECS-related data for game engine.
//!

use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

// In `ggengine` `usize` is used in id structs because
// those really only represent internal indices, and `usize` is the best fit,
// but `Hasher` implementation requires `u64`, so this conversion has to be made.
// This can hurt performance on targets where `usize` > `u64` due to hash collisions,
// but this scenario is very unlikely.
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
/// **This hasher only passes `usize` as a no-op hashing,
/// `write` function should not be used (it's implementation is not suited for usage);
/// use `write_usize` instead.**
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

    fn write_usize(&mut self, i: usize) {
        self.0 = i;
    }
}

/// [`NoOpHasherState`] struct implements `BuildHasher` that produces [`NoOpHasher`].
///
/// This should be passed to collections interfaces (e.g. `HashMap::with_hasher(NoOpHasherState)`).
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
