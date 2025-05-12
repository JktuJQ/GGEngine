//! `gamecore::entities` submodule implements [`Entity`] struct -
//! game object that has some characteristics (components) on which game engine operates.
//!

use crate::gamecore::{
    components::{Bundle, Component},
    storages::EntityComponentStorage,
};
use std::hash::{Hash, Hasher};

/// [`EntityId`] id struct is needed to identify entities
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this entity is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId(pub(super) usize);
impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64)
    }
}

/// [`EntityRef`] provides immutable access to a single entity and all of its components.
///
#[derive(Debug)]
pub struct EntityRef<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Entity storage which can be navigated by `entity_id`.
    ///
    entity_component_storage: &'a EntityComponentStorage,
}
impl<'a> EntityRef<'a> {
    /// Creates new [`EntityRef`], immutably borrowing [`EntityComponentStorage`].
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &'a EntityComponentStorage,
    ) -> EntityRef<'a> {
        EntityRef {
            entity_id,
            entity_component_storage,
        }
    }
}
impl<'a> From<EntityMut<'a>> for EntityRef<'a> {
    fn from(value: EntityMut<'a>) -> EntityRef<'a> {
        EntityRef {
            entity_id: value.entity_id,
            entity_component_storage: value.entity_component_storage,
        }
    }
}

/// [`EntityMut`] provides mutable access to a single entity and all of its components.
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
    ///
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &'a mut EntityComponentStorage,
    ) -> EntityMut<'a> {
        EntityMut {
            entity_id,
            entity_component_storage,
        }
    }

    pub fn despawn(self) {
        self.entity_component_storage.remove_entity(self.entity_id);
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn insert<C: Component>(&mut self, component: C) -> Option<C> {
        self.entity_component_storage
            .insert_component::<C>(self.entity_id, component)
    }
    pub fn insert_bundle(&mut self, bundle: impl Bundle) {
        self.entity_component_storage
            .insert_bundle(self.entity_id, bundle)
    }

    pub fn remove<C: Component>(&mut self) -> Option<C> {
        self.entity_component_storage
            .remove_component::<C>(self.entity_id)
    }

    pub fn contains<C: Component>(&self) -> bool {
        self.entity_component_storage
            .contains_component::<C>(self.entity_id)
    }

    pub fn get<C: Component>(&self) -> Option<&C> {
        self.entity_component_storage
            .get_component::<C>(self.entity_id)
    }
    pub fn get_mut<C: Component>(&mut self) -> Option<&mut C> {
        self.entity_component_storage
            .get_component_mut::<C>(self.entity_id)
    }
    pub fn get_or_insert<C: Component>(&mut self, f: impl FnOnce() -> C) -> &mut C {
        self.entity_component_storage
            .get_component_or_insert::<C>(self.entity_id, f)
    }
}
