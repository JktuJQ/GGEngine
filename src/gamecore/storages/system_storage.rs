//! Submodule that implement [`SystemStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::{
    scenes::Scene,
    systems::{System, SystemId},
};
use std::fmt;

/// [`SystemPosition`] enum lists possible positions
/// in which new system could be inserted into [`SystemStorage`].
///
/// Docs on [`SystemPosition`] variants describe specifics of each position.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    Tail,
}

/// [`DecomposedSystem`] struct is what any system system could be coerced to.
///
/// To store different systems in one generic container,
/// systems arguments and return types must be coerced to some common ground types.
/// `&mut Scene` as argument type and `()` as return type are the only options
/// to which arguments and return type of any system could be coerced.
/// So, [`DecomposedSystem`] is just [`SystemId`] and `Box<dyn FnMut(&mut Scene)>` stored together
/// (basically a [`System`] v-table representation).
///
struct DecomposedSystem {
    /// Id of a system which was coerced to [`DecomposedSystem`].
    ///
    id: SystemId,
    /// Boxed system function.
    ///
    f: Box<dyn FnMut(&mut Scene)>,
}
impl DecomposedSystem {
    /// Decomposes any system.
    ///
    fn from_system<Args, F: System<Args>>(mut system: F) -> Self {
        DecomposedSystem {
            id: system.id(),
            f: Box::new(move |scene: &mut Scene| {
                let _ = system.run(scene);
            }),
        }
    }
}
impl System<(&mut Scene,)> for DecomposedSystem {
    type Output = ();

    fn id(&self) -> SystemId {
        self.id
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.f)(scene)
    }
}
impl fmt::Debug for DecomposedSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decomposed system with {:?}", self.id)
    }
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
#[derive(Debug, Default)]
pub struct SystemStorage {
    schedule: Vec<SystemNode>,
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
            schedule: Vec::new(),
            positions: TypeIdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.schedule.clear();
        self.positions.clear();
    }
}
// systems
impl SystemStorage {
    /// Returns the index of the tail (last) system.
    ///
    fn tail_index(&self) -> usize {
        self.schedule.get(0).map_or(0, |node| node.prev)
    }
    /// Updates the position mapping for a system at the given index.
    ///
    fn update_position(&mut self, index: usize) {
        let _ = self.positions.insert(self.schedule[index].system.id, index);
    }
    /// Inserts a system at the head (start) of the schedule.
    ///
    fn insert_at_head(&mut self, system: DecomposedSystem) {
        let new_index = self.schedule.len();

        let tail_index = self.tail_index();

        self.schedule.push(SystemNode {
            prev: tail_index,
            system,
            next: new_index,
        });

        self.schedule.swap(0, new_index);

        self.update_position(0);
        self.update_position(new_index);

        let second_index = self.schedule[new_index].next;
        self.schedule[second_index].prev = new_index;
        self.schedule[new_index].prev = 0;
    }
    /// Inserts a system before the specified anchor system.
    ///
    fn insert_before(&mut self, anchor_index: usize, system: DecomposedSystem) {
        if anchor_index == 0 {
            return self.insert_at_head(system);
        }

        let new_index = self.schedule.len();

        let before_anchor_index = self.schedule[anchor_index].prev;

        self.schedule.push(SystemNode {
            prev: before_anchor_index,
            system,
            next: anchor_index,
        });

        self.update_position(new_index);

        self.schedule[before_anchor_index].next = new_index;
        self.schedule[anchor_index].prev = new_index;
    }
    /// Inserts a system after the specified anchor system.
    ///
    fn insert_after(&mut self, anchor_index: usize, system: DecomposedSystem) {
        let new_index = self.schedule.len();

        let after_anchor_index = self.schedule[anchor_index].next;

        self.schedule.push(SystemNode {
            prev: anchor_index,
            system,
            next: after_anchor_index,
        });

        self.update_position(new_index);

        self.schedule[anchor_index].next = new_index;
        self.schedule[after_anchor_index].prev = new_index;
    }
    /// Inserts a system at the tail (end) of the schedule.
    ///
    fn insert_at_tail(&mut self, system: DecomposedSystem) {
        let new_index = self.schedule.len();

        let tail_index = self.tail_index();

        self.schedule.push(SystemNode {
            prev: tail_index,
            system,
            next: 0,
        });

        self.update_position(new_index);

        self.schedule[tail_index].next = new_index;
        self.schedule[0].prev = new_index;
    }
    /// Inserts a system into the storage at the specified position.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, SystemPosition, System, SystemId};
    /// let mut storage = SystemStorage::new();
    ///
    /// fn unused_system() {}
    /// fn system1() {}
    /// fn system2() {}
    /// fn system3() {}
    /// fn system4() {}
    /// fn system5() {}
    /// fn system6() {}
    /// fn system7() {}
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

        let system = DecomposedSystem::from_system(system);
        match position {
            SystemPosition::Head => self.insert_at_head(system),
            SystemPosition::Before(anchor_system_id) => {
                if !self.positions.contains_key(&anchor_system_id) {
                    self.insert_at_head(system)
                } else {
                    self.insert_before(self.positions[&anchor_system_id], system)
                }
            }
            SystemPosition::After(anchor_system_id) => {
                if !self.positions.contains_key(&anchor_system_id) {
                    self.insert_at_tail(system)
                } else {
                    self.insert_after(self.positions[&anchor_system_id], system)
                }
            }
            SystemPosition::Tail => self.insert_at_tail(system),
        }
    }

    /// Removes a system from the storage by its [`SystemId`].
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{SystemStorage, SystemPosition, System, SystemId};
    /// let mut storage = SystemStorage::new();
    ///
    /// fn unused_system() {}
    /// fn system1() {}
    /// fn system2() {}
    /// fn system3() {}
    /// fn system4() {}
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
        let _ = self.positions.remove(&system.id);

        if index != last_index {
            self.update_position(index);
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

    pub fn system_order(&self) -> Vec<SystemId> {
        let mut order = Vec::new();
        if self.schedule.is_empty() {
            return order;
        }
        let mut current_index = 0;

        loop {
            order.push(self.schedule[current_index].system.id);
            current_index = self.schedule[current_index].next;

            if current_index == 0 {
                break;
            }
        }
        order
    }
}
