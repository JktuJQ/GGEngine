//! Submodule that implement [`SystemStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::{
    scenes::Scene,
    systems::{DecomposedSystem, System, SystemId},
};
use std::{
    mem::{replace, swap},
    num::Wrapping,
};

/// [`SystemPosition`] enum lists possible positions
/// in which new system could be inserted into [`SystemStorage`].
///
/// Docs on [`SystemPosition`] variants describe specifics of each position.
///
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SystemPosition {
    /// Head position (system will be inserted at the start of a [`SystemStorage`]'s schedule).
    ///
    Head,
    /// System will be inserted right before system with provided [`SystemId`].
    ///
    /// Case where no system with such [`SystemId`] is present in [`SystemStorage`]
    /// will be handled as the `SystemPosition::Head` case.
    ///
    Before(SystemId),
    /// System will be inserted right after system with provided [`SystemId`].
    ///
    /// Case where no system with such [`SystemId`] is present in [`SystemStorage`]
    /// will be handled as the `SystemPosition::Tail` case.
    ///
    After(SystemId),
    /// Tail position (system will be inserted at the end of a [`SystemStorage`]'s schedule).
    ///
    #[default]
    Tail,
}

/// [`SystemNode`] struct serves as the emulation of node in doubly linked list.
///
#[derive(Debug)]
struct SystemNode {
    /// Index of previous system.
    ///
    prev: usize,
    /// Decomposed system.
    ///
    system: DecomposedSystem,
    /// Index of next system.
    ///
    next: usize,
}
/// [`SystemStorage`] struct implements schedule of [`System`]s.
///
/// Conceptually, [`SystemStorage`] can be thought of as an `LinkedList<S>`
/// (more precisely, doubly linked list with O(1) random access).
///
/// That allows to store systems in a sequence and
/// additionally have the ability to change their positions very quickly.
///
/// # Note
/// [`SystemStorage`] is different from other storages in a sense that
/// it heavily utilizes ids, not generic typed interface.
/// That is due to the nature of [`System`]s - they are functions,
/// and it is impossible to name a type of a function;
/// so the only way to identify them is to use [`SystemId`]s,
/// which are bound to unnameable type of concrete [`System`].
///
/// # Usage
/// [`System`]s require `&mut Scene` to run, but since they are stored inside [`SystemStorage`]
/// which is inside [`Scene`], it becomes very hard to call [`System`].
/// To call it, you would need to move that [`System`] from [`SystemStorage`].
/// `SystemStorage::take_system` function does exactly that - it moves the system
/// out of the storage, leaving placeholder system behind.
///
/// That (along with [`SystemQuery`](crate::gamecore::systems::SystemQuery)) allows
/// systems to operate on other systems in schedule.
///
/// Taking and returning, placeholder values are a fairly advanced topic.
/// Most of the time user of the [`SystemStorage`] would only work
/// with `SystemStorage::run_system_schedule`.
///
/// # Example
/// ```rust
/// # use ggengine::gamecore::systems::{SystemStorage, System, SystemId};
/// # use ggengine::gamecore::scenes::Scene;
/// fn system1() {
///     println!("system1");
/// }
/// fn system2() {
///     println!("system2");
/// }
/// fn system3() {
///     println!("system3");
/// }
///
/// let mut scene: Scene = Scene::new();
/// scene.system_storage.insert_system(system1, Default::default());
/// scene.system_storage.insert_system(system2, Default::default());
/// scene.system_storage.insert_system(system3, Default::default());
///
/// SystemStorage::run_system_schedule(&mut scene);
/// // prints "system1", "system2", "system3"
/// ```
///
#[derive(Debug, Default)]
pub struct SystemStorage {
    /// Index of schedule head (first system).
    ///
    /// Is equal to `0` if schedule is empty.
    ///
    schedule_head: usize,
    /// Schedule of functions.
    ///
    /// `Vec` represents doubly linked list,
    /// where head and tail are linked to each other as well.
    ///
    schedule: Vec<SystemNode>,
    /// `TypeIdMap` that holds indices of all systems.
    ///
    /// `TypeIdMap` allows implementing O(1) random access
    /// to the doubly linked list.
    ///
    positions: TypeIdMap<SystemId, usize>,
}
impl SystemStorage {
    /// Initializes new [`SystemStorage`].
    ///
    /// Created [`SystemStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::SystemStorage;
    /// let storage: SystemStorage = SystemStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        SystemStorage {
            schedule_head: 0,
            schedule: Vec::new(),
            positions: TypeIdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.schedule_head = 0;
        self.schedule.clear();
        self.positions.clear();
    }
}
// systems
impl SystemStorage {
    /// Returns the index of the tail (last) system.
    ///
    fn tail_index(&self) -> usize {
        self.schedule
            .get(self.schedule_head)
            .map_or(0, |head| head.prev)
    }
    /// Inserts a system into the storage at the specified position.
    ///
    /// Docs on [`SystemPosition`] list all cases and implications of each position.
    ///
    /// # Performance
    /// Although [`SystemStorage`] is optimized for insertion at random positions,
    /// the best position is at the tail - that way storage maintains its
    /// internal sequence logically and that improves cache locality.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, SystemPosition, System, SystemId};
    /// fn unused_system() {}
    /// fn system1() {}
    /// fn system2() {}
    /// fn system3() {}
    /// fn system4() {}
    /// fn system5() {}
    /// fn system6() {}
    /// fn system7() {}
    ///
    /// let mut storage = SystemStorage::new();
    ///
    /// storage.insert_system(system1, SystemPosition::Head);
    /// storage.insert_system(system2, SystemPosition::Tail);
    /// storage.insert_system(system3, SystemPosition::Head);
    /// storage.insert_system(system4, SystemPosition::Before(system2.id()));
    /// storage.insert_system(system5, SystemPosition::After(system1.id()));
    /// assert_eq!(storage.system_order(), vec![
    ///     system3.id(),
    ///     system1.id(),
    ///     system5.id(),
    ///     system4.id(),
    ///     system2.id(),
    /// ]);
    ///
    /// storage.insert_system(system6, SystemPosition::Before(unused_system.id()));
    /// storage.insert_system(system7, SystemPosition::After(unused_system.id()));
    /// assert_eq!(storage.system_order(), vec![
    ///     system6.id(),
    ///     system3.id(),
    ///     system1.id(),
    ///     system5.id(),
    ///     system4.id(),
    ///     system2.id(),
    ///     system7.id(),
    /// ]);
    /// ```
    ///
    pub fn insert_system<Args, S: System<Args>>(&mut self, system: S, position: SystemPosition) {
        if self.positions.contains_key(&system.id()) {
            return;
        }

        let last_index = self.schedule.len();

        let (prev, next) = match position {
            SystemPosition::Head => {
                let (prev, next) = (self.tail_index(), self.schedule_head);
                self.schedule_head = last_index;
                (prev, next)
            }
            SystemPosition::Before(anchor_system_id) => {
                let anchor_index = *self
                    .positions
                    .get(&anchor_system_id)
                    .unwrap_or(&self.schedule_head);

                if anchor_index == self.schedule_head {
                    self.schedule_head = last_index;
                }
                (self.schedule[anchor_index].prev, anchor_index)
            }
            SystemPosition::After(anchor_system_id) => {
                let anchor_index = *self
                    .positions
                    .get(&anchor_system_id)
                    .unwrap_or(&self.tail_index());
                (anchor_index, self.schedule[anchor_index].next)
            }
            SystemPosition::Tail => (self.tail_index(), self.schedule_head),
        };

        self.schedule.push(SystemNode {
            prev,
            system: DecomposedSystem::from_system(system),
            next,
        });

        let _ = self
            .positions
            .insert(self.schedule[last_index].system.id(), last_index);

        self.schedule[prev].next = last_index;
        self.schedule[next].prev = last_index;
    }

    /// Removes a system from the storage by its [`SystemId`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, SystemPosition, System, SystemId};
    /// fn unused_system() {}
    /// fn system1() {}
    /// fn system2() {}
    /// fn system3() {}
    /// fn system4() {}
    ///
    /// let mut storage = SystemStorage::new();
    ///
    /// storage.insert_system(system1, SystemPosition::Head);
    /// storage.insert_system(system2, SystemPosition::Tail);
    /// storage.insert_system(system3, SystemPosition::Tail);
    /// storage.insert_system(system4, SystemPosition::Tail);
    /// assert_eq!(storage.system_order(), vec![
    ///     system1.id(),
    ///     system2.id(),
    ///     system3.id(),
    ///     system4.id(),
    /// ]);
    ///
    /// storage.remove_system(system2.id());
    /// assert_eq!(storage.system_order(), vec![
    ///     system1.id(),
    ///     system3.id(),
    ///     system4.id(),
    /// ]);
    ///
    /// storage.remove_system(system1.id());
    /// assert_eq!(storage.system_order(), vec![
    ///     system3.id(),
    ///     system4.id(),
    /// ]);
    ///
    /// storage.remove_system(system4.id());
    /// assert_eq!(storage.system_order(), vec![
    ///     system3.id(),
    /// ]);
    ///
    /// storage.remove_system(system3.id());
    /// assert!(storage.system_order().is_empty());
    ///
    /// storage.remove_system(unused_system.id());
    /// assert!(storage.system_order().is_empty());
    /// ```
    ///
    pub fn remove_system(&mut self, system_id: SystemId) {
        let Some(&index) = self.positions.get(&system_id) else {
            return;
        };

        let last_index = self.schedule.len() - 1;
        if last_index == 0 {
            return self.clear();
        }

        let (prev, system, next) = {
            let SystemNode { prev, system, next } = self.schedule.swap_remove(index);
            (
                if prev == last_index { index } else { prev },
                system,
                if next == last_index { index } else { next },
            )
        };
        let _ = self.positions.remove(&system.id());

        if index != last_index {
            let _ = self
                .positions
                .insert(self.schedule[index].system.id(), index);
        }
        if index == self.schedule_head {
            self.schedule_head = next;
        }

        self.schedule[prev].next = next;
        self.schedule[next].prev = prev;

        if index != last_index {
            let swapped_node = &self.schedule[index];
            let (swapped_prev, swapped_next) = (swapped_node.prev, swapped_node.next);

            self.schedule[swapped_prev].next = index;
            self.schedule[swapped_next].prev = index;
        }
    }

    /// Returns whether a system with given [`SystemId`] is present or not.
    ///
    /// Placeholder systems are not considered present in the storage,
    /// check docs on [`SystemStorage`] for more info.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, System, SystemId};
    /// fn system1() {}
    /// fn system2() {}
    ///
    /// let mut storage: SystemStorage = SystemStorage::new();
    ///
    /// storage.insert_system(system1, Default::default());
    /// assert!(storage.contains_system(system1.id()));
    /// assert!(!storage.contains_system(system2.id()));
    /// ```
    ///
    pub fn contains_system(&self, system_id: SystemId) -> bool {
        self.is_system_taken(system_id) == Some(false)
    }

    /// Placeholder system that will replace systems when they are taken from [`SystemStorage`].
    ///
    /// This function is a no-op.
    ///
    /// Check docs on [`SystemStorage`] and `SystemStorage::take_system` for more info.
    ///
    pub fn placeholder_system() {}
    /// Returns whether a system with given [`SystemId`] was taken out from [`SystemStorage`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, System, SystemId};
    /// fn system() {}
    ///
    /// let mut system_storage: SystemStorage = SystemStorage::new();
    ///
    /// assert!(system_storage.is_system_taken(system.id()).is_none());
    ///
    /// system_storage.insert_system(system, Default::default());
    /// assert_eq!(system_storage.is_system_taken(system.id()), Some(false));
    ///
    /// system_storage.take_system(system.id());
    /// assert_eq!(system_storage.is_system_taken(system.id()), Some(true));
    ///
    /// system_storage.remove_system(system.id());
    /// assert!(system_storage.is_system_taken(system.id()).is_none());
    /// ```
    ///
    pub fn is_system_taken(&self, system_id: SystemId) -> Option<bool> {
        let &index = self.positions.get(&system_id)?;
        Some(self.schedule[index].system.id() == SystemStorage::placeholder_system.id())
    }
    /// Takes system out of the storage, leaving placeholder behind.
    ///
    /// [`System`]s require `&mut Scene` to run, but since they are stored inside [`SystemStorage`]
    /// which is inside [`Scene`], it becomes very hard to call [`System`].
    /// To call it, you would need to move that [`System`] from [`SystemStorage`].
    /// `SystemStorage::take_system` function does exactly that - it moves the system
    /// out of the storage, leaving placeholder system behind.
    ///
    /// Leaving placeholder system is convenient because then returning system
    /// would be trivial - there would be no need to remember position of the system
    /// (and storage won't need to rearrange systems internally, maintaining order).
    ///
    /// Trying to move placeholder system is prohibited.
    ///
    /// If the system should be taken permanently from the storage, consider `SystemStorage::remove_system`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, System, SystemId, DecomposedSystem};
    /// # use ggengine::gamecore::scenes::Scene;
    /// fn system() {
    ///     println!("system");
    /// }
    ///
    /// let mut system_storage: SystemStorage = SystemStorage::new();
    ///
    /// assert!(system_storage.take_system(system.id()).is_none());
    ///
    /// system_storage.insert_system(system, Default::default());
    /// let mut system: DecomposedSystem = system_storage.take_system(system.id())
    ///     .expect("System was inserted");
    /// system.run(&mut Scene::new());  // prints "system"
    ///
    /// assert!(system_storage.take_system(system.id()).is_none());
    /// ```
    ///
    pub fn take_system(&mut self, system_id: SystemId) -> Option<DecomposedSystem> {
        let &index = self.positions.get(&system_id)?;

        let system = &mut self.schedule[index].system;
        if system.id() == SystemStorage::placeholder_system.id() {
            return None;
        }

        Some(replace(
            system,
            DecomposedSystem::from_system(SystemStorage::placeholder_system),
        ))
    }
    /// Returns system into the storage from which it was previously moved.
    ///
    /// If the system is not present in the storage in placeholder form
    /// (which could be checked by `SystemStorage::is_system_taken`),
    /// an error is returned.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, System, SystemId, DecomposedSystem};
    /// # use ggengine::gamecore::scenes::Scene;
    /// fn system1() {}
    /// fn system2() {}
    /// fn system3() {}
    ///
    /// let mut system_storage: SystemStorage = SystemStorage::new();
    ///
    /// system_storage.insert_system(system1, Default::default());
    /// system_storage.insert_system(system2, Default::default());
    /// system_storage.insert_system(system3, Default::default());
    ///
    /// let system: DecomposedSystem = system_storage.take_system(system2.id())
    ///     .expect("System was inserted");
    /// let _ = system_storage.return_taken_system(system)
    ///     .expect("System was taken");
    ///
    /// assert_eq!(system_storage.system_order(), vec![
    ///     system1.id(),
    ///     system2.id(),
    ///     system3.id(),
    /// ]);
    /// ```
    ///
    pub fn return_taken_system(
        &mut self,
        system: DecomposedSystem,
    ) -> Result<(), DecomposedSystem> {
        let Some(&index) = self.positions.get(&system.id()) else {
            return Err(system);
        };

        let storage_system = &mut self.schedule[index].system;
        if storage_system.id() != SystemStorage::placeholder_system.id() {
            return Err(system);
        }

        let _ = replace(storage_system, system);
        Ok(())
    }

    /// Runs [`Scene`]'s [`SystemStorage`] schedule.
    ///
    /// This function runs every function in [`Scene`]'s [`SystemStorage`] schedule.
    /// It is more efficient than calling
    /// `SystemStorage::take_system` and `SystemStorage::return_taken_system` sequentially.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, System, SystemId};
    /// # use ggengine::gamecore::scenes::Scene;
    /// fn system1() {
    ///     println!("system1");
    /// }
    /// fn system2() {
    ///     println!("system2");
    /// }
    /// fn system3() {
    ///     println!("system3");
    /// }
    ///
    /// let mut scene: Scene = Scene::new();
    /// scene.system_storage.insert_system(system1, Default::default());
    /// scene.system_storage.insert_system(system2, Default::default());
    /// scene.system_storage.insert_system(system3, Default::default());
    ///
    /// SystemStorage::run_system_schedule(&mut scene);
    /// // prints "system1", "system2", "system3"
    /// ```
    ///
    pub fn run_system_schedule(scene: &mut Scene) {
        let mut system = DecomposedSystem::from_system(SystemStorage::placeholder_system);

        let mut schedule_index = scene.system_storage.schedule_head;
        for _ in 0..scene.system_storage.schedule.len() {
            schedule_index = scene.system_storage.schedule[schedule_index].next;

            swap(
                &mut scene.system_storage.schedule[schedule_index].system,
                &mut system,
            );
            system.run(scene);
            swap(
                &mut scene.system_storage.schedule[schedule_index].system,
                &mut system,
            );
        }
    }

    /// Returns [`SystemId`]s in the order they appear in the schedule.
    ///
    /// # Example
    /// ```
    /// # use ggengine::gamecore::systems::{SystemStorage, SystemPosition, System, SystemId};
    /// # use ggengine::gamecore::scenes::Scene;
    /// fn system1() {}
    /// fn system2() {}
    /// fn system3() {}
    /// fn system4() {}
    ///
    /// let mut storage = SystemStorage::new();
    ///
    /// storage.insert_system(system1, SystemPosition::Head);
    /// storage.insert_system(system2, SystemPosition::Tail);
    /// storage.insert_system(system3, SystemPosition::Head);
    /// storage.insert_system(system4, SystemPosition::After(system1.id()));
    ///
    /// assert_eq!(storage.system_order(), vec![
    ///     system3.id(),
    ///     system1.id(),
    ///     system4.id(),
    ///     system2.id(),
    /// ]);
    /// ```
    ///
    pub fn system_order(&self) -> Vec<SystemId> {
        let mut order = Vec::with_capacity(self.schedule.len());

        let mut schedule_index = self.schedule_head;
        for _ in 0..self.schedule.len() {
            let node = &self.schedule[schedule_index];
            schedule_index = node.next;

            order.push(node.system.id());
        }
        order
    }
    /// Rearranges the systems in the [`SystemStorage`] to match their logical schedule order.
    /// This can improve cache locality and simplify debugging.
    ///
    pub fn reorder(&mut self) {
        let mut reordered_schedule = Vec::with_capacity(self.schedule.capacity());
        let len = self.schedule.len();

        let mut schedule_index = self.schedule_head;
        for index in 0..len {
            let mut node = replace(
                &mut self.schedule[schedule_index],
                SystemNode {
                    prev: 0,
                    system: DecomposedSystem::from_system(|| {}),
                    next: 0,
                },
            );
            schedule_index = node.next;

            node.prev = (Wrapping(index) - Wrapping(1)).0;
            node.next = (Wrapping(index) + Wrapping(1)).0;

            reordered_schedule.push(node);
            let _ = self
                .positions
                .insert(reordered_schedule[index].system.id(), index);
        }
        if len > 0 {
            reordered_schedule[0].prev = len - 1;
            reordered_schedule[len - 1].next = 0;
        }
        self.schedule_head = 0;
        self.schedule = reordered_schedule;
    }
}
