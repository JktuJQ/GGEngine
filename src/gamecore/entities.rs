//! `gamecore::entities` submodule implements [`Entity`] struct -
//! game object that has some characteristics (components) on which game engine operates.
//!

use crate::gamecore::storages::EntityComponentStorage;

/// [`EntityId`] id struct is needed to identify [`Entity`](super::entities::Entity)s
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this [`Entity`](super::entities::Entity) is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u64);
impl EntityId {
    /// Creates new [`EntityId`] from id.
    ///
    pub(super) fn new(id: u64) -> EntityId {
        EntityId(id)
    }

    /// Returns id that corresponds to this [`EntityId`].
    ///
    pub(super) fn id(self) -> u64 {
        self.0
    }
}

/// [`EntityMut`] provides mutable access to a single entity and all of its components.
///
/// It is weaker than [`Entity`], because [`Entity`] has access to its [`Scene`](super::scenes::Scene),
/// while [`EntityMut`] does not.
///
#[derive(Debug)]
pub struct EntityMut<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Entity storage which can be navigated by `entity_id`.
    ///
    entity_component_storage: &'a mut EntityComponentStorage,
}
impl<'a> EntityMut<'a> {
    /// Creates new [`EntityMut`], mutably borrowing [`EntityComponentStorage`].
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &'a mut EntityComponentStorage,
    ) -> EntityMut<'a> {
        EntityMut {
            entity_id,
            entity_component_storage,
        }
    }
}
