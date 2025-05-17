//! `gamecore::scenes` submodule implements [`Scene`] - struct that handles and manages
//! all game objects, components and systems that are binded to that [`Scene`].
//!

use crate::gamecore::storages::{EntityComponentStorage, ResourceStorage};

/// [`Scene`] struct composes all structs that implement ECS architecture.
///
/// This struct does not provide much functionality on its own,
/// it is just a convenient abstraction that contains all ECS storages.
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
