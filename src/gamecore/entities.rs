//! `gamecore::entities` submodule implements entities -
//! game objects that have some characteristics (components) on which game engine operates.
//!

use crate::gamecore::{
    components::{bundled_components, Bundle, Component, ComponentId, Downcastable},
    storages::EntityComponentStorage,
};
use std::hash::{Hash, Hasher};

/// [`EntityId`] id struct is needed to identify entities
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this entity is registered.
///
/// [`EntityId`] is only valid for the [`Scene`](super::scenes::Scene) it was obtained from,
/// and although you can use it for any other scene,
/// fetching will either fail or return unexpected results.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntityId(pub(super) usize);
impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64)
    }
}

/// [`EntityRef`] provides immutable access to a single entity and all of its components.
///
/// This struct provides ergonomic access to [`Scene`](super::scenes::Scene) API,
/// and `ggengine` advises using [`EntityRef`] instead of using bare [`Scene`](super::scenes::Scene).
///
/// # Note
/// If you want to downgrade [`EntityMut`] to [`EntityRef`] without manual dropping,
/// you can use `EntityRef::from` to perform that conversion.
///
#[derive(Debug)]
pub struct EntityRef<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Entity scene which can be navigated by `entity_id`.
    ///
    entity_component_storage: &'a EntityComponentStorage,
}
impl EntityRef<'_> {
    /// Creates new [`EntityRef`], immutably borrowing [`EntityComponentStorage`].
    ///
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &EntityComponentStorage,
    ) -> EntityRef {
        EntityRef {
            entity_id,
            entity_component_storage,
        }
    }

    /// Returns id of this entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// # use ggengine::gamecore::scenes::Scene;
    /// let mut scene: Scene = Scene::new();
    ///
    /// let entity: EntityId = EntityRef::from(scene.spawn_entity(())).id();
    /// ```
    ///
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    /// Returns whether this component is present in entity or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityRef;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityRef = EntityRef::from(scene.spawn_entity((Player,)));
    /// assert!(entity.contains::<Player>());
    /// ```
    ///
    pub fn contains<C: Component>(&self) -> bool {
        self.entity_component_storage
            .contains_component(self.entity_id, ComponentId::of::<C>())
    }
    /// Returns immutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityRef;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    /// let entity: EntityRef = EntityRef::from(scene.spawn_entity((Player, Health(10))));
    /// assert_eq!(entity.get::<Health>().expect("Component was inserted").0, 10);
    /// ```
    ///
    pub fn get<C: Component>(&self) -> Option<&C> {
        self.entity_component_storage
            .component(self.entity_id, ComponentId::of::<C>())
            .map(|component_ref| {
                component_ref
                    .downcast_to_ref::<C>()
                    .expect("This type should correspond to this value")
            })
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
/// This struct provides ergonomic access to [`Scene`](super::scenes::Scene) API,
/// and `ggengine` advises using [`EntityMut`] instead of using bare [`Scene`](super::scenes::Scene).
///
#[derive(Debug)]
pub struct EntityMut<'a> {
    /// Entity id.
    ///
    entity_id: EntityId,
    /// Entity scene which can be navigated by `entity_id`.
    ///
    entity_component_storage: &'a mut EntityComponentStorage,
}
impl EntityMut<'_> {
    /// Creates new [`EntityMut`], immutably borrowing [`EntityComponentStorage`].
    ///
    pub(super) fn new(
        entity_id: EntityId,
        entity_component_storage: &mut EntityComponentStorage,
    ) -> EntityMut {
        EntityMut {
            entity_id,
            entity_component_storage,
        }
    }

    /// Returns id of this entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityId;
    /// # use ggengine::gamecore::scenes::Scene;
    /// let mut scene: Scene = Scene::new();
    ///
    /// let entity: EntityId = scene.spawn_entity(()).id();
    /// ```
    ///
    pub fn id(&self) -> EntityId {
        self.entity_id
    }

    /// Consumes [`EntityMut`] and despawns its entity.
    ///
    /// When this function is called,
    /// [`EntityId`] from `EntityMut::id` are no longer valid.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::{EntityMut, EntityId};
    /// # use ggengine::gamecore::scenes::Scene;
    /// let mut scene: Scene = Scene::new();
    ///
    /// let entity: EntityMut = scene.spawn_entity(());
    /// let entity_id: EntityId = entity.id();
    /// entity.despawn();
    /// assert!(!scene.contains_entity(entity_id));
    /// ```
    ///
    pub fn despawn(self) {
        let _ = self.entity_component_storage.remove_entity(self.entity_id);
    }

    /// Inserts component into entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity(());
    /// let _ = entity.insert(Player);
    /// ```
    ///
    pub fn insert<C: Component>(&mut self, component: C) -> Option<C> {
        self.entity_component_storage
            .insert_component(self.entity_id, ComponentId::of::<C>(), Box::new(component))
            .map(|boxed_component| {
                boxed_component
                    .downcast_to_value::<C>()
                    .expect("This type should correspond to this value")
            })
    }
    /// Inserts bundle of components into entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity(());
    /// let _ = entity.insert_many((Player, Health(10)));
    /// assert!(entity.contains::<Player>());
    /// assert!(entity.contains::<Health>());
    /// ```
    ///
    pub fn insert_many<B: Bundle<N>, const N: usize>(&mut self, bundle: B) {
        self.entity_component_storage
            .insert_components(self.entity_id, bundled_components(bundle).into_iter())
    }

    /// Removes component from entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity((Player,));
    /// let player: Player = entity.remove::<Player>().expect("Component is present");
    /// assert!(!entity.contains::<Player>());
    /// ```
    ///
    pub fn remove<C: Component>(&mut self) -> Option<C> {
        self.entity_component_storage
            .remove_component(self.entity_id, ComponentId::of::<C>())
            .map(|boxed_component| {
                boxed_component
                    .downcast_to_value::<C>()
                    .expect("This type should correspond to this value")
            })
    }
    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Name(String);
    /// impl Component for Name {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity((Player, Name("Alice".to_string()), Health(10),));
    /// entity.remove_many::<2, (Player, Health)>();
    /// assert!(!entity.contains::<Player>());
    /// assert!(entity.contains::<Name>());
    /// assert!(!entity.contains::<Health>());
    /// ```
    ///
    pub fn remove_many<const N: usize, B: Bundle<N>>(&mut self) {
        self.entity_component_storage
            .remove_components(self.entity_id, B::component_ids().into_iter())
    }
    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity((Player, Health(10)));
    /// entity.remove_all_components();
    /// assert!(!entity.contains::<Player>());
    /// assert!(!entity.contains::<Health>());
    /// ```
    ///
    pub fn remove_all_components(&mut self) {
        self.entity_component_storage
            .remove_all_components(self.entity_id)
    }

    /// Returns whether this component is present in entity or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity(());
    /// assert!(!entity.contains::<Player>());
    ///
    /// let _ = entity.insert(Player);
    /// assert!(entity.contains::<Player>());
    /// ```
    ///
    pub fn contains<C: Component>(&self) -> bool {
        self.entity_component_storage
            .contains_component(self.entity_id, ComponentId::of::<C>())
    }
    /// Returns immutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity((Player, Health(10)));
    /// assert_eq!(entity.get::<Health>().expect("Component was inserted").0, 10);
    /// ```
    ///
    pub fn get<C: Component>(&self) -> Option<&C> {
        self.entity_component_storage
            .component(self.entity_id, ComponentId::of::<C>())
            .map(|boxed_component| {
                boxed_component
                    .downcast_to_ref::<C>()
                    .expect("This type should correspond to this value")
            })
    }
    /// Returns mutable reference to the component of this entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity((Player, Health(10)));
    /// entity.get_mut::<Health>().expect("Component is present").0 = 20;
    /// assert_eq!(entity.get::<Health>().expect("Component was inserted").0, 20);
    /// ```
    ///
    pub fn get_mut<C: Component>(&mut self) -> Option<&mut C> {
        self.entity_component_storage
            .component_mut(self.entity_id, ComponentId::of::<C>())
            .map(|component_mut| {
                component_mut
                    .downcast_to_mut::<C>()
                    .expect("This type should correspond to this value")
            })
    }
    /// Gets a mutable reference to the component of given type if present,
    /// otherwise inserts the component that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::entities::EntityMut;
    /// # use ggengine::gamecore::scenes::Scene;
    /// # use ggengine::gamecore::components::Component;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut scene: Scene = Scene::new();
    ///
    /// let mut entity: EntityMut = scene.spawn_entity(Player);
    /// let health: &mut Health = entity.get_or_insert_with(|| Health(10));
    /// assert_eq!(health.0, 10);
    /// assert!(entity.contains::<Health>());
    /// ```
    ///
    pub fn get_or_insert_with<C: Component>(&mut self, f: impl FnOnce() -> C) -> &mut C {
        self.entity_component_storage
            .component_get_or_insert_with(self.entity_id, ComponentId::of::<C>(), || Box::new(f()))
            .downcast_to_mut::<C>()
            .expect("This type should correspond to this value")
    }
}
