//! `gamecore::scenes` submodule implements [`Scene`] - struct that handles and manages
//! all game objects, components and systems that are bound to that [`Scene`].
//!

use crate::gamecore::storages::{EntityComponentStorage, ResourceStorage};

/// [`Scene`] struct composes all structs that implement ECS architecture.
///
#[derive(Debug, Default)]
pub struct Scene {
    /// Storage that contains entities and components.
    ///
    pub entity_component_storage: EntityComponentStorage,
    /// Storage that contains resources.
    ///
    pub resource_storage: ResourceStorage,
}
impl Scene {
    /// Initializes new [`Scene`].
    ///
    /// Created [`Scene`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// let scene: Scene = Scene::new();
    /// ```
    ///
    pub fn new() -> Scene {
        Scene {
            entity_component_storage: EntityComponentStorage::new(),
            resource_storage: ResourceStorage::new(),
        }
    }

    /// Clears scene, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.entity_component_storage.clear();
        self.resource_storage.clear();
    }
}
