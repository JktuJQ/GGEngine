//! `gamecore::scenes` submodule implements [`Scene`] - struct that handles and manages
//! all game objects, components and systems that are bound to that [`Scene`].
//!

use crate::gamecore::components::ComponentId;
use crate::gamecore::{
    components::{bundled_components, Bundle, Component, Resource, ResourceId},
    entities::{EntityId, EntityMut, EntityRef},
    storages::{EntityComponentStorage, ResourceStorage},
};

/// [`Scene`] struct composes all structs that implement ECS architecture.
///
/// Parts of [`Scene`] are fairly low-level, and so this struct tries to
/// provide nice typed API for interfaces of those storages.
///
/// [`EntityComponentStorage`]: [`Scene`] provides only interface for functions that do not operate on entities exactly,
/// see [`EntityMut`]/[`EntityRef`] for typed API for component access.
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
// entity-component scene
impl Scene {
    /// inserts entity with components that are given in a [`Bundle`]
    /// into [`Scene`] and returns mutable reference to it,
    /// so it could be further modified.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// let player: EntityId = scene.spawn_entity((Player,)).id();
    /// ```
    ///
    /// You can spawn empty entity to defer initialization by passing `()` as a [`Bundle`]:
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::entities::EntityId;
    /// let mut scene: Scene = Scene::new();
    /// let player: EntityId = scene.spawn_entity(()).id();
    /// ```
    ///
    pub fn spawn_entity<B: Bundle<N>, const N: usize>(&mut self, bundle: B) -> EntityMut {
        self.entity_component_storage
            .insert_entity(bundled_components(bundle).into_iter())
    }
    /// Inserts multiple entities with components that are given in [`Bundle`]s
    /// into [`Scene`] and returns immutable references to those entities.
    ///
    /// It is more efficient than calling `Scene::spawn_entity` in a loop.
    ///
    /// # Note
    /// This function can only insert entities with the same [`Bundle`] type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// let npcs: Vec<EntityRef> = scene.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]).collect::<Vec<EntityRef>>();
    /// ```
    ///
    pub fn spawn_entities<B: Bundle<N>, const N: usize>(
        &mut self,
        bundles: impl IntoIterator<Item = B>,
    ) -> impl Iterator<Item = EntityRef> {
        self.entity_component_storage.insert_entities(
            bundles
                .into_iter()
                .map(|bundle| bundled_components(bundle).into_iter()),
        )
    }
    /// Removes entity from [`Scene`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let player: EntityId = scene.spawn_entity((Player,)).id();
    /// scene.despawn_entity(player);
    /// ```
    ///
    pub fn despawn_entity(&mut self, entity_id: EntityId) -> bool {
        self.entity_component_storage.remove_entity(entity_id)
    }
    /// Returns number of entities that are currently present in the scene.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::{Component, bundled_components};
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let npcs: Vec<EntityRef> = scene.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]).collect::<Vec<EntityRef>>();
    /// let npc: EntityId = npcs[0].id();
    /// assert_eq!(scene.entities_count(), 3);
    /// scene.despawn_entity(npc);
    /// assert_eq!(scene.entities_count(), 2);
    ///
    pub fn entities_count(&self) -> usize {
        self.entity_component_storage.entities_count()
    }
    /// Returns whether an entity with given id is currently stored or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let player: EntityId = scene.spawn_entity((Player,)).id();
    /// assert!(scene.contains_entity(player));
    ///
    /// scene.despawn_entity(player);
    /// assert!(!scene.contains_entity(player));
    /// ```
    ///
    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        self.entity_component_storage.contains_entity(entity_id)
    }
    /// Returns immutable reference to entity in [`Scene`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// let mut scene: Scene = Scene::new();
    ///
    /// let player: EntityId = scene.spawn_entity(()).id();
    /// let player_ref: EntityRef = scene.entity(player).expect("Entity was spawned.");
    /// ```
    ///
    pub fn entity(&self, entity_id: EntityId) -> Option<EntityRef> {
        self.entity_component_storage.entity(entity_id)
    }
    /// Returns references to all entities that are inserted in the scene.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let npcs1: Vec<EntityId> = scene.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]).map(|entity| entity.id()).collect::<Vec<EntityId>>();
    /// let npcs2: Vec<EntityId> = scene.all_entities()
    ///     .map(|entity| entity.id()).collect::<Vec<EntityId>>();
    /// for (id1, id2) in npcs1.iter().zip(npcs2.iter()) {
    ///     assert_eq!(id1, id2);
    /// }
    /// ```
    ///
    pub fn all_entities(&self) -> impl Iterator<Item = EntityRef> {
        self.entity_component_storage.all_entities()
    }
    /// Returns mutable reference to entity in [`Scene`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::entities::{EntityId, EntityMut};
    /// let mut scene: Scene = Scene::new();
    ///
    /// let player: EntityId = scene.spawn_entity(()).id();
    /// let player_mut: EntityMut = scene.entity_mut(player).expect("Entity was spawned.");
    /// ```
    ///
    pub fn entity_mut(&mut self, entity_id: EntityId) -> Option<EntityMut> {
        self.entity_component_storage.entity_mut(entity_id)
    }

    /// Extracts all component of one type from scene.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let npcs: Vec<EntityRef> = scene.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]).collect::<Vec<EntityRef>>();
    /// let names: Vec<Name> = scene.components_take::<Name>()
    ///     .expect("Component is present")
    ///     .collect::<Vec<Name>>();
    /// assert_eq!(names.len(), 3);
    /// ```
    ///
    pub fn components_take<C: Component>(
        &mut self,
    ) -> Option<impl Iterator<Item = C> + use<'_, C>> {
        self.entity_component_storage
            .components_take(ComponentId::of::<C>())
            .map(|components| {
                components.map(|component| {
                    *(component
                        .downcast::<C>()
                        .expect("This type should correspond to this value"))
                })
            })
    }
    /// Returns immutable references to all components of one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let npcs: Vec<EntityRef> = scene.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]).collect::<Vec<EntityRef>>();
    /// let names: Vec<&Name> = scene.components::<Name>()
    ///     .expect("Component is present")
    ///     .collect::<Vec<&Name>>();
    /// assert_eq!(names.len(), 3);
    /// ```
    ///
    pub fn components<C: Component>(&self) -> Option<impl Iterator<Item = &C>> {
        self.entity_component_storage
            .components(ComponentId::of::<C>())
            .map(|components| {
                components.map(|component| {
                    component
                        .downcast_ref::<C>()
                        .expect("This type should correspond to this value")
                })
            })
    }
    /// Returns mutable references to all components of one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let npcs: Vec<EntityRef> = scene.spawn_entities(vec![
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]).collect::<Vec<EntityRef>>();
    /// let names: Vec<&mut Name> = scene.components_mut::<Name>()
    ///     .expect("Component is present")
    ///     .collect::<Vec<&mut Name>>();
    /// assert_eq!(names.len(), 3);
    /// ```
    ///
    pub fn components_mut<C: Component>(&mut self) -> Option<impl Iterator<Item = &mut C>> {
        self.entity_component_storage
            .components_mut(ComponentId::of::<C>())
            .map(|components| {
                components.map(|component| {
                    component
                        .downcast_mut::<C>()
                        .expect("This type should correspond to this value")
                })
            })
    }
}
// resource scene
impl Scene {
    /// Inserts a new resource with the given value.
    ///
    /// Resources are unique data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data and this function will return old value.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// scene.insert_resource::<Score>(Score(0));
    /// ```
    ///
    pub fn insert_resource<R: Resource>(&mut self, value: R) -> Option<R> {
        self.resource_storage
            .insert_resource(ResourceId::of::<R>(), Box::new(value))
            .map(|boxed_resource| {
                *(boxed_resource
                    .downcast::<R>()
                    .expect("This type corresponds to this value."))
            })
    }
    /// Removes the resource of a given type and returns it if present.
    /// Otherwise, returns `None`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// scene.insert_resource::<Score>(Score(0));
    /// assert_eq!(scene.remove_resource::<Score>().expect("Resource was inserted.").0, 0);
    /// ```
    ///
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resource_storage
            .remove_resource(ResourceId::of::<R>())
            .map(|boxed_resource| {
                *(boxed_resource
                    .downcast::<R>()
                    .expect("This type corresponds to this value."))
            })
    }
    /// Returns whether a resource of given type exists or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// assert!(!scene.contains_resource::<Score>());
    ///
    /// scene.insert_resource::<Score>(Score(0));
    /// assert!(scene.contains_resource::<Score>());
    ///
    /// scene.remove_resource::<Score>();
    /// assert!(!scene.contains_resource::<Score>());
    /// ```
    ///
    pub fn contains_resource<R: Resource>(&mut self) -> bool {
        self.resource_storage
            .contains_resource(ResourceId::of::<R>())
    }
    /// Gets a reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// assert!(scene.resource::<Score>().is_none());
    ///
    /// scene.insert_resource::<Score>(Score(0));
    /// assert_eq!(scene.resource::<Score>().expect("Resource was inserted.").0, 0);
    /// ```
    ///
    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resource_storage
            .resource(ResourceId::of::<R>())
            .map(|resource_ref| {
                resource_ref
                    .downcast_ref::<R>()
                    .expect("This type corresponds to this value")
            })
    }
    /// Gets a mutable reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// assert!(scene.resource_mut::<Score>().is_none());
    ///
    /// scene.insert_resource::<Score>(Score(0));
    /// scene.resource_mut::<Score>().expect("Resource was inserted.").0 = 10;
    /// assert_eq!(scene.resource_mut::<Score>().expect("Resource was inserted.").0, 10);
    /// ```
    ///
    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resource_storage
            .resource_mut(ResourceId::of::<R>())
            .map(|resource_mut| {
                resource_mut
                    .downcast_mut::<R>()
                    .expect("This type corresponds to this value")
            })
    }
    /// Gets a mutable reference to the resource of given type if present,
    /// otherwise inserts the resource that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// assert!(!scene.contains_resource::<Score>());
    ///
    /// let _ = scene.resource_get_or_insert_with(|| Score(10));
    /// assert!(scene.contains_resource::<Score>());
    /// ```
    pub fn resource_get_or_insert_with<R: Resource>(&mut self, f: impl FnOnce() -> R) -> &mut R {
        self.resource_storage
            .resource_get_or_insert_with(ResourceId::of::<R>(), || Box::new(f()))
            .downcast_mut::<R>()
            .expect("This type corresponds to this value.")
    }
}
