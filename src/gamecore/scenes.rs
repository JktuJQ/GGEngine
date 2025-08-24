//! `gamecore::scenes` submodule implements [`Scene`] - struct that handles and manages
//! all game objects, components and systems that are bound to that [`Scene`].
//!

use crate::gamecore::storages::{ComponentStorage, EventStorage, ResourceStorage, SystemStorage};

/// [`Scene`] struct composes all structs that implement ECS architecture.
///
#[derive(Debug, Default)]
pub struct Scene {
    /// Storage that contains components.
    ///
    pub component_storage: ComponentStorage,
    /// Storage that contains resources.
    ///
    pub resource_storage: ResourceStorage,
    /// Storage that contains events.
    ///
    pub event_storage: EventStorage,
    /// Storage that contains systems.
    ///
    pub system_storage: SystemStorage,
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
            component_storage: ComponentStorage::new(),
            resource_storage: ResourceStorage::new(),
            event_storage: EventStorage::new(),
            system_storage: SystemStorage::new(),
        }
    }

    /// Clears scene, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.component_storage.clear();
        self.resource_storage.clear();
        self.event_storage.clear();
        self.system_storage.clear();
    }
}
