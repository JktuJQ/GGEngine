//! Submodule that implement [`ComponentStorage`].
//!

use super::{NoOpHasherState, TypeIdMap, TypeIdSet};
use crate::gamecore::{
    components::{Component, ComponentId, ComponentSet},
    entities::{EntityId, EntityMut, EntityRef},
};
use std::{any::Any, array::from_fn};

/// In `entity_component_storage`, [`DynVec`] represents type-erased `Vec<Option<T>>`.
///
/// This serves as an efficient replacement for `Vec<Option<BoxedComponent>>` in situations
/// where multiple such vectors are used and each separate vector is homogenous in its component type.
///
#[derive(Debug)]
struct DynVec {
    /// Type-erased vec.
    ///
    vec: Box<dyn Any>,

    /// Function that allows removing item at exact position in type-erased vec.
    ///
    /// This function is created when the [`DynVec`] is initialized,
    /// and so it 'records' the type information while remaining type-erased for the end user of [`DynVec`].
    ///
    remove_at_fn: fn(&mut DynVec, usize),
}
impl DynVec {
    /// Creates function that could operate on type-erased vec by internally recording required type.
    ///
    fn remove_at_fn<T: Component>() -> fn(&mut DynVec, usize) {
        |this: &mut DynVec, i: usize| {
            let vec = this
                .vec
                .downcast_mut::<Vec<Option<T>>>()
                .expect("Correct type was recorded at initialization");
            if i < vec.len() {
                vec[i] = None;
            }
        }
    }

    /// Creates new [`DynVec`] that will represent type-erased `Vec<Option<C>>`.
    ///
    fn new<C: Component>() -> DynVec {
        DynVec {
            vec: Box::new(Vec::<Option<C>>::new()),

            remove_at_fn: DynVec::remove_at_fn::<C>(),
        }
    }

    /// Downcasts [`DynVec`] to `Vec<Option<C>>`.
    ///
    fn downcast<C: Component>(self) -> Result<Vec<Option<C>>, DynVec> {
        match self.vec.downcast::<Vec<Option<C>>>() {
            Ok(vec) => Ok(*vec),
            Err(vec) => Err(DynVec {
                vec,
                remove_at_fn: self.remove_at_fn,
            }),
        }
    }
    /// Downcasts [`DynVec`] reference to `&Vec<Option<C>>`.
    ///
    fn downcast_ref<C: Component>(&self) -> Option<&Vec<Option<C>>> {
        self.vec.downcast_ref::<Vec<Option<C>>>()
    }
    /// Downcasts [`DynVec`] mutable reference to `&mut Vec<Option<C>>`.
    ///
    fn downcast_mut<C: Component>(&mut self) -> Option<&mut Vec<Option<C>>> {
        self.vec.downcast_mut::<Vec<Option<C>>>()
    }
}
/// [`ComponentStorage`] is a column-oriented structure-of-arrays based storage
/// that maps entities to their [`Component`]s.
///
/// Conceptually, [`ComponentStorage`] can be thought of as an `HashMap<ComponentId, Vec<Option<C>>>`,
/// where each separate `Vec` contains components of one type that belong to different entities.
///
/// # Note
/// This collection is designed to provide more fine-grained control over entity-component storage.
/// Most of the time you should use [`EntityMut`] or its readonly counterpart.
/// For example, most storage functions that require [`EntityId`] do nothing
/// if the entity with that id is not present.
/// Checking whether that function early returned is left for the user
/// (`ComponentStorage::contains_entity` will tell if the entity id is correct).
/// Using [`EntityMut`]/[`EntityRef`], however, resolves that ambiguity,
/// because those references always point at correct entities with correct [`EntityId`]s.
///
#[derive(Debug, Default)]
pub struct ComponentStorage {
    /// Maximal index that is vacant for entity insertion.
    ///
    max_vacant_index: usize,

    /// Set of removed entities.
    ///
    removed_entities: TypeIdSet<EntityId>,

    /// Table that holds all components.
    ///
    /// For each individual key/value pair where key is `TypeId::of::<T>()`,
    /// value would be `DynVec` with internal type of `Vec<T>`.
    ///
    table: TypeIdMap<ComponentId, DynVec>,
}
impl ComponentStorage {
    /// Initializes new [`ComponentStorage`].
    ///
    /// Created [`ComponentStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// let storage: ComponentStorage = ComponentStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        ComponentStorage {
            max_vacant_index: 0,
            removed_entities: TypeIdSet::with_hasher(NoOpHasherState),

            table: TypeIdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.max_vacant_index = 0;
        self.removed_entities.clear();
        self.table.clear();
    }
}
// entities
impl ComponentStorage {
    /// Finds suitable [`EntityId`]s for new entities.
    ///
    fn obtain_entity_ids<const N: usize>(&mut self) -> [EntityId; N] {
        from_fn(|_| match self.removed_entities.iter().next().copied() {
            Some(id) => {
                let _ = self.removed_entities.remove(&id);
                id
            }
            None => {
                let new_id = EntityId(self.max_vacant_index);
                self.max_vacant_index += 1;
                new_id
            }
        })
    }

    /// Inserts empty entity.
    /// Equivalent of `entity_component_storage.insert_entity(())`.
    ///
    pub fn insert_empty_entity(&mut self) -> EntityMut {
        self.insert_entity(())
    }
    /// Inserts entity with components into [`ComponentStorage`]
    /// and returns mutable reference to it, so it could be further modified.
    ///
    /// # Examples
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity((Player, Health(10))).id();
    /// ```
    ///
    pub fn insert_entity<CS: ComponentSet>(&mut self, components: CS) -> EntityMut {
        let entity_id = self.obtain_entity_ids::<1>()[0];
        components.insert_set(entity_id, self);
        EntityMut::new(entity_id, self)
    }
    /// Inserts multiple entities with components into [`ComponentStorage`]
    /// and returns immutable references to those entities.
    ///
    /// It is slightly more efficient than calling `ComponentStorage::insert_entity` in a loop.
    ///
    /// # Note
    /// This function can only insert entities with same components type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let npcs: [EntityRef; 3] = storage.insert_many_entities([
    ///     (NPC, Name("Alice"), Health(5)),
    ///     (NPC, Name("Bob"), Health(10)),
    ///     (NPC, Name("Charlie"), Health(15))
    /// ]);
    /// ```
    ///
    pub fn insert_many_entities<CS: ComponentSet, const N: usize>(
        &mut self,
        many_components: [CS; N],
    ) -> [EntityRef; N] {
        let ids = self.obtain_entity_ids::<N>();
        for (entity_id, components) in ids.into_iter().rev().zip(many_components.into_iter().rev())
        {
            components.insert_set(entity_id, self);
        }
        ids.map(|entity_id| EntityRef::new(entity_id, self))
    }

    /// Removes entity from [`ComponentStorage`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(Player).id();
    ///
    /// assert!(storage.contains_entity(player));
    /// storage.remove_entity(player);
    /// assert!(!storage.contains_entity(player));
    /// ```
    ///
    pub fn remove_entity(&mut self, entity_id: EntityId) -> bool {
        if !self.contains_entity(entity_id) {
            return false;
        }
        let _ = self.removed_entities.insert(entity_id);

        self.clear_entity(entity_id);
        true
    }

    /// Removes all components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity((Player, Health(10))).id();
    /// storage.clear_entity(player);
    /// assert!(storage.contains_entity(player));
    /// assert!(!storage.contains_component::<Player>(player));
    /// assert!(!storage.contains_component::<Health>(player));
    /// ```
    ///
    pub fn clear_entity(&mut self, entity_id: EntityId) {
        if !self.contains_entity(entity_id) {
            return;
        }
        for component_column in self.table.values_mut() {
            (component_column.remove_at_fn)(component_column, entity_id.0)
        }
    }

    /// Returns whether an entity with given id is currently stored or not.
    ///
    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        entity_id.0 < self.max_vacant_index && !self.removed_entities.contains(&entity_id)
    }

    /// Returns immutable reference to entity in [`ComponentStorage`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_empty_entity().id();
    /// let player_ref: EntityRef = storage.entity(player).expect("Entity was inserted");
    /// ```
    ///
    pub fn entity(&self, entity_id: EntityId) -> Option<EntityRef> {
        if self.contains_entity(entity_id) {
            Some(EntityRef::new(entity_id, self))
        } else {
            None
        }
    }
    /// Returns mutable reference to entity in [`ComponentStorage`] if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// # use ggengine::gamecore::entities::{EntityId, EntityMut};
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_empty_entity().id();
    /// let player_mut: EntityMut = storage.entity_mut(player).expect("Entity was inserted");
    /// ```
    ///
    pub fn entity_mut(&mut self, entity_id: EntityId) -> Option<EntityMut> {
        if self.contains_entity(entity_id) {
            Some(EntityMut::new(entity_id, self))
        } else {
            None
        }
    }

    /// Returns all the [`EntityId`] that are valid.
    /// That allows iterating over all entities in a storage.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::ComponentStorage;
    /// # use ggengine::gamecore::entities::EntityId;
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let id1 = storage.insert_empty_entity().id();
    /// let id2 = storage.insert_empty_entity().id();
    /// let id3 = storage.insert_empty_entity().id();
    ///
    /// let ids: Vec<EntityId> = storage.entity_ids();
    /// assert_eq!(ids, vec![id1, id2, id3]);
    /// ```
    ///
    pub fn entity_ids(&self) -> Vec<EntityId> {
        let mut vec = Vec::new();
        for index in 0..self.max_vacant_index {
            let entity_id = EntityId(index);
            if self.contains_entity(entity_id) {
                vec.push(entity_id);
            }
        }
        vec
    }

    /// Returns number of entities that are currently present in the storage.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::{EntityId, EntityRef};
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let npcs: [EntityRef; 3] = storage.insert_many_entities([
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]);
    /// let npc: EntityId = npcs[0].id();
    /// assert_eq!(storage.entity_count(), 3);
    /// storage.remove_entity(npc);
    /// assert_eq!(storage.entity_count(), 2);
    /// ```
    ///
    pub fn entity_count(&self) -> usize {
        self.max_vacant_index - self.removed_entities.len()
    }
}
// components
impl ComponentStorage {
    /// Inserts component into given entity and returns old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_empty_entity().id();
    /// storage.insert_component(player, Player);
    /// ```
    ///
    pub fn insert_component<C: Component>(
        &mut self,
        entity_id: EntityId,
        component: C,
    ) -> Option<C> {
        if !self.contains_entity(entity_id) {
            return None;
        }

        let component_column = self
            .table
            .entry(ComponentId::of::<C>())
            .or_insert(DynVec::new::<C>())
            .downcast_mut::<C>()
            .expect("`DynVec` is of correct type");

        let entity_index = entity_id.0;
        if component_column.len() <= entity_index {
            component_column.resize_with(entity_index + 1, || None);
        }
        component_column[entity_index].replace(component)
    }
    /// Inserts components into given entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_empty_entity().id();
    /// storage.insert_many_components(
    ///     player,
    ///     (Player, Health(10))
    /// );
    /// ```
    ///
    pub fn insert_many_components<CS: ComponentSet>(
        &mut self,
        entity_id: EntityId,
        components: CS,
    ) {
        if !self.contains_entity(entity_id) {
            return;
        }
        components.insert_set(entity_id, self)
    }

    /// Removes component from an entity and returns the old value if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(Player).id();
    /// storage.remove_component::<Player>(player);
    /// assert!(storage.contains_entity(player));
    /// assert!(!storage.contains_component::<Player>(player));
    /// ```
    ///
    pub fn remove_component<C: Component>(&mut self, entity_id: EntityId) -> Option<C> {
        if !self.contains_entity(entity_id) {
            return None;
        }
        self.table
            .get_mut(&ComponentId::of::<C>())?
            .downcast_mut::<C>()
            .expect("`DynVec` is of correct type")
            .get_mut(entity_id.0)?
            .take()
    }
    /// Removes multiple components from entity.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     (Player, Name("Alice"), Health(10))
    /// ).id();
    /// storage.remove_many_components::<(Player, Health)>(player);
    /// assert!(storage.contains_entity(player));
    /// assert!(!storage.contains_component::<Player>(player));
    /// assert!(storage.contains_component::<Name>(player));
    /// assert!(!storage.contains_component::<Health>(player));
    /// ```
    ///
    pub fn remove_many_components<B: ComponentSet>(&mut self, entity_id: EntityId) {
        if !self.contains_entity(entity_id) {
            return;
        }
        for component_id in B::component_ids() {
            let Some(component_column) = self.table.get_mut(&component_id) else {
                continue;
            };
            (component_column.remove_at_fn)(component_column, entity_id.0);
        }
    }

    /// Returns whether this component is present in an entity or not.
    ///
    pub fn contains_component<C: Component>(&self, entity_id: EntityId) -> bool {
        !self.removed_entities.contains(&entity_id)
            && self
                .table
                .get(&ComponentId::of::<C>())
                .and_then(|component_column| {
                    component_column
                        .downcast_ref::<C>()
                        .expect("`DynVec` is of correct type")
                        .get(entity_id.0)
                })
                .is_some_and(|component| component.is_some())
    }

    /// Returns immutable reference to the component of given entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     (Player, Health(10))
    /// ).id();
    /// assert_eq!(storage.component::<Health>(player).expect("`Health` was inserted").0, 10);
    /// ```
    ///
    pub fn component<C: Component>(&self, entity_id: EntityId) -> Option<&C> {
        if !self.contains_entity(entity_id) {
            return None;
        }
        self.table
            .get(&ComponentId::of::<C>())?
            .downcast_ref::<C>()
            .expect("`DynVec` is of correct type")
            .get(entity_id.0)?
            .as_ref()
    }
    /// Returns mutable reference to the component of given entity if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityId;
    /// struct Player;
    /// impl Component for Player {}
    ///
    /// struct Health(u32);
    /// impl Component for Health {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let player: EntityId = storage.insert_entity(
    ///     (Player, Health(10))
    /// ).id();
    /// let component = storage.component_mut::<Health>(player).expect("`Health` was inserted");
    /// component.0 = 15;
    /// assert_eq!(storage.component::<Health>(player).expect("`Health` was inserted").0, 15);
    /// ```
    ///
    pub fn component_mut<C: Component>(&mut self, entity_id: EntityId) -> Option<&mut C> {
        if !self.contains_entity(entity_id) {
            return None;
        }
        self.table
            .get_mut(&ComponentId::of::<C>())?
            .downcast_mut::<C>()
            .expect("`DynVec` is of correct type")
            .get_mut(entity_id.0)?
            .as_mut()
    }

    /// Removes all components of one type from all entities and returns them in an iterator.
    /// Returns `None` if components of this type were never present in the storage or were removed by this function previously.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// #[derive(Debug, PartialEq)]
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let npcs: [EntityRef; 3] = storage.insert_many_entities([
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]);
    /// let names: Vec<Name> = storage.remove_components::<Name>()
    ///     .expect("Component is present")
    ///     .collect::<Vec<Name>>();
    /// assert_eq!(names, vec![Name("Alice"), Name("Bob"), Name("Charlie")]);
    /// for entity_id in storage.entity_ids() {
    ///     assert!(!storage.contains_component::<Name>(entity_id));
    /// }
    /// ```
    ///
    pub fn remove_components<C: Component>(&mut self) -> Option<impl Iterator<Item = C>> {
        self.table.remove(&ComponentId::of::<C>()).map(|dynvec| {
            dynvec
                .downcast::<C>()
                .expect("`DynVec` is of correct type")
                .into_iter()
                .flatten()
        })
    }

    /// Returns immutable references to all components of one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// #[derive(Debug, PartialEq)]
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let npcs: [EntityRef; 3] = storage.insert_many_entities([
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]);
    /// let names: Vec<&Name> = storage.components::<Name>()
    ///     .expect("Component is present")
    ///     .collect::<Vec<&Name>>();
    /// assert_eq!(names, vec![&Name("Alice"), &Name("Bob"), &Name("Charlie")]);
    /// ```
    ///
    pub fn components<C: Component>(&self) -> Option<impl Iterator<Item = &C>> {
        let components = self
            .table
            .get(&ComponentId::of::<C>())?
            .downcast_ref::<C>()
            .expect("`DynVec` is of correct type")
            .iter()
            .enumerate()
            .filter_map(|(index, component)| {
                if !self.removed_entities.contains(&EntityId(index)) {
                    component.as_ref()
                } else {
                    None
                }
            });
        Some(components)
    }
    /// Returns immutable references to all components of one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::components::{Component, ComponentStorage};
    /// # use ggengine::gamecore::entities::EntityRef;
    /// #[derive(Debug, PartialEq)]
    /// struct NPC;
    /// impl Component for NPC {}
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Name(&'static str);
    /// impl Component for Name {}
    ///
    /// let mut storage: ComponentStorage = ComponentStorage::new();
    ///
    /// let npcs: [EntityRef; 3] = storage.insert_many_entities([
    ///     (NPC, Name("Alice")),
    ///     (NPC, Name("Bob")),
    ///     (NPC, Name("Charlie"))
    /// ]);
    /// let mut names: Vec<&mut Name> = storage.components_mut::<Name>()
    ///     .expect("Component is present")
    ///     .collect::<Vec<&mut Name>>();
    /// names[1].0 = "Brad";
    /// assert_eq!(names, vec![&Name("Alice"), &Name("Brad"), &Name("Charlie")]);
    /// ```
    ///
    pub fn components_mut<C: Component>(&mut self) -> Option<impl Iterator<Item = &mut C>> {
        let components = self
            .table
            .get_mut(&ComponentId::of::<C>())?
            .downcast_mut::<C>()
            .expect("`DynVec` is of correct type")
            .iter_mut()
            .enumerate()
            .filter_map(|(index, component)| {
                if !self.removed_entities.contains(&EntityId(index)) {
                    component.as_mut()
                } else {
                    None
                }
            });
        Some(components)
    }
}
