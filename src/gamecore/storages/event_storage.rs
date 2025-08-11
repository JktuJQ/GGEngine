//! Submodule that implements [`EventStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::events::{Event, EventId};
use std::any::Any;

/// In `event_storage`, [`DynVec`] represents type-erased `Vec<T>`.
///
#[derive(Debug)]
struct DynVec {
    /// Type-erased vec.
    ///
    vec: Box<dyn Any>,
}
impl DynVec {
    /// Ereates new [`DynVec`] that will represent type-erased `Vec<C>`.
    ///
    fn new<E: Event>() -> DynVec {
        DynVec {
            vec: Box::new(Vec::<E>::new()),
        }
    }

    /// Downcasts [`DynVec`] to vector.
    ///
    fn downcast<E: Event>(self) -> Result<Vec<E>, DynVec> {
        match self.vec.downcast::<Vec<E>>() {
            Ok(vec) => Ok(*vec),
            Err(vec) => Err(DynVec { vec }),
        }
    }
    /// Downcasts [`DynVec`] reference to `&Vec<E>`.
    ///
    fn downcast_ref<E: Event>(&self) -> Option<&Vec<E>> {
        self.vec.downcast_ref::<Vec<E>>()
    }
    /// Downcasts [`DynVec`] mutable reference to `&mut Vec<E>`.
    ///
    fn downcast_mut<E: Event>(&mut self) -> Option<&mut Vec<E>> {
        self.vec.downcast_mut::<Vec<E>>()
    }
}

/// [`EventStorage`] struct provides API for a storage of [`Event`]s.
///
#[derive(Debug, Default)]
pub struct EventStorage {
    /// Map that stores events.
    ///
    events: TypeIdMap<EventId, DynVec>,
}
impl EventStorage {
    /// Initializes new [`EventStorage`].
    ///
    /// Created [`EventStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EventStorage;
    /// let storage: EventStorage = EventStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        EventStorage {
            events: TypeIdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
// events
impl EventStorage {
    /// Inserts a new event with the given value.
    ///
    /// Since events of the same type could be inserted multiple times,
    /// this function just pushes new on in the internal vector.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EventStorage;
    /// # use ggengine::gamecore::events::Event;
    /// // Mock `EntityId`.
    /// struct EntityId(u64);
    ///
    /// struct InflictedDamage {
    ///     damage: u32,
    ///     target: EntityId,
    /// }
    /// impl Event for InflictedDamage {}
    ///
    /// let mut storage: EventStorage = EventStorage::new();
    ///
    /// storage.insert(InflictedDamage {
    ///     damage: 10,
    ///     target: EntityId(0)
    /// });
    /// storage.insert(InflictedDamage {
    ///     damage: 15,
    ///     target: EntityId(1)
    /// });
    /// ```
    ///
    pub fn insert<E: Event>(&mut self, event: E) {
        self.events
            .entry(EventId::of::<E>())
            .or_insert(DynVec::new::<E>())
            .downcast_mut::<E>()
            .expect("`DynVec` is of correct type")
            .push(event)
    }

    /// Removes all events of a given type and returns them if present.
    /// Otherwise, returns `None`.
    ///
    /// # Note
    /// This function behaviour is consistent with `EventStorage::contains`;
    /// it returns `None` even if the vector is present but is empty.
    /// Thus, `EventStorage::remove` never returns an empty vector.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EventStorage;
    /// # use ggengine::gamecore::events::Event;
    /// // Mock `EntityId`.
    /// #[derive(Copy, Clone, Debug, PartialEq)]
    /// struct EntityId(u64);
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq)]
    /// struct InflictedDamage {
    ///     damage: u32,
    ///     target: EntityId,
    /// }
    /// impl Event for InflictedDamage {}
    ///
    /// let mut storage: EventStorage = EventStorage::new();
    ///
    /// let damage1: InflictedDamage = InflictedDamage {
    ///     damage: 10,
    ///     target: EntityId(0)
    /// };
    /// let damage2: InflictedDamage = InflictedDamage {
    ///     damage: 15,
    ///     target: EntityId(1)
    /// };
    /// storage.insert(damage1);
    /// storage.insert(damage2);
    ///
    /// assert_eq!(storage.remove::<InflictedDamage>().expect("`InflictedDamage` was inserted"), vec![damage1, damage2]);
    /// assert!(storage.remove::<InflictedDamage>().is_none());
    /// ```
    ///
    pub fn remove<E: Event>(&mut self) -> Option<Vec<E>> {
        self.events.remove(&EventId::of::<E>()).and_then(|events| {
            let vec = events.downcast::<E>().expect("`DynVec` is of correct type");
            if vec.is_empty() {
                None
            } else {
                Some(vec)
            }
        })
    }

    /// Returns whether any event of given type is present or not.
    /// That means that if the event was at the storage and then was removed,
    /// this method won't count it as present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EventStorage;
    /// # use ggengine::gamecore::events::Event;
    /// // Mock `EntityId`.
    /// struct EntityId(u64);
    ///
    /// struct InflictedDamage {
    ///     damage: u32,
    ///     target: EntityId,
    /// }
    /// impl Event for InflictedDamage {}
    ///
    /// let mut storage: EventStorage = EventStorage::new();
    ///
    /// storage.insert(InflictedDamage {
    ///     damage: 10,
    ///     target: EntityId(0)
    /// });
    ///
    /// assert!(storage.contains::<InflictedDamage>());
    /// storage.remove::<InflictedDamage>();
    /// assert!(!storage.contains::<InflictedDamage>());
    /// ```
    ///
    pub fn contains<E: Event>(&self) -> bool {
        self.events.contains_key(&EventId::of::<E>())
            && !self
                .events
                .get(&EventId::of::<E>())
                .expect("Presence of this event type was checked")
                .downcast_ref::<E>()
                .expect("`DynVec` is of correct type")
                .is_empty()
    }

    /// Returns an reference to all events currently in the storage if present.
    /// Otherwise, returns `None`.
    ///
    /// # Note
    /// This function behaviour is consistent with `EventStorage::contains`;
    /// it returns `None` even if the vector is present but is empty.
    /// Thus, `EventStorage::events` never returns an empty vector.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EventStorage;
    /// # use ggengine::gamecore::events::Event;
    /// // Mock `EntityId`.
    /// #[derive(Copy, Clone, Debug, PartialEq)]
    /// struct EntityId(u64);
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq)]
    /// struct InflictedDamage {
    ///     damage: u32,
    ///     target: EntityId,
    /// }
    /// impl Event for InflictedDamage {}
    ///
    /// let mut storage: EventStorage = EventStorage::new();
    ///
    /// let damage1: InflictedDamage = InflictedDamage {
    ///     damage: 10,
    ///     target: EntityId(0)
    /// };
    /// let damage2: InflictedDamage = InflictedDamage {
    ///     damage: 15,
    ///     target: EntityId(1)
    /// };
    /// storage.insert(damage1);
    /// storage.insert(damage2);
    ///
    /// assert_eq!(storage.events::<InflictedDamage>().expect("`InflictedDamage` was inserted"), &vec![damage1, damage2]);
    /// ```
    ///
    pub fn events<E: Event>(&self) -> Option<&Vec<E>> {
        self.events.get(&EventId::of::<E>()).map(|events| {
            events
                .downcast_ref::<E>()
                .expect("`DynVec` is of correct type")
        })
    }
    /// Returns a mutable reference to all events currently in the storage if present.
    /// Otherwise, returns `None`.
    ///
    /// # Note
    /// This function behaviour is consistent with `EventStorage::contains`;
    /// it returns `None` even if the vector is present but is empty.
    /// Thus, `EventStorage::events_mut` never returns an empty vector.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::EventStorage;
    /// # use ggengine::gamecore::events::Event;
    /// // Mock `EntityId`.
    /// #[derive(Copy, Clone, Debug, PartialEq)]
    /// struct EntityId(u64);
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq)]
    /// struct InflictedDamage {
    ///     damage: u32,
    ///     target: EntityId,
    /// }
    /// impl Event for InflictedDamage {}
    ///
    /// let mut storage: EventStorage = EventStorage::new();
    ///
    /// let mut damage1: InflictedDamage = InflictedDamage {
    ///     damage: 10,
    ///     target: EntityId(0)
    /// };
    /// let damage2: InflictedDamage = InflictedDamage {
    ///     damage: 15,
    ///     target: EntityId(1)
    /// };
    /// storage.insert(damage1);
    /// storage.insert(damage2);
    ///
    /// let events = storage.events_mut::<InflictedDamage>().expect("`InflictedDamage` was inserted");
    /// events[0].damage *= 2;
    ///
    /// damage1.damage *= 2;
    /// assert_eq!(storage.events::<InflictedDamage>().expect("`InflictedDamage` was inserted"), &vec![damage1, damage2]);
    /// ```
    ///
    pub fn events_mut<E: Event>(&mut self) -> Option<&mut Vec<E>> {
        let events = self.events.get_mut(&EventId::of::<E>())?;
        let vec = events
            .downcast_mut::<E>()
            .expect("`DynVec` is of correct type");
        if vec.is_empty() {
            None
        } else {
            Some(vec)
        }
    }
}
