//! Submodule that implement [`SystemStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::{scenes::Scene, systems::{System, SystemId}};
use std::fmt;

/// [`StoredSystem`] struct wraps boxed system that takes `&mut Scene` and returns nothing.
///
/// Since storage could only operate with systems of one kind,
/// arguments and return types of all systems need to be unifiable.
/// `&mut Scene` as argument type and `()` as return type are the only options
/// to which arguments and return type of any system could be unified.
/// With that in mind, every stored system is just a `FnMut(&mut Scene)` object.
///
struct StoredSystem(Box<dyn FnMut(&mut Scene)>);
impl StoredSystem {
    /// Unifies any system to `FnMut(&mut Scene)` representation and wraps it in [`StoredSystem`].
    ///
    fn from_system<Args, F: System<Args>>(mut system: F) -> Self {
        StoredSystem(Box::new(move |scene: &mut Scene| { let _ = system.run(scene); }))
    }

    /// Runs underlying unified system.
    ///
    fn run(&mut self, scene: &mut Scene) {
        self.0.run(scene)
    }
}
impl fmt::Debug for StoredSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stored system")
    }
}
/// [`SystemNode`] struct serves as the emulation of node in doubly linked list.
///
#[derive(Debug)]
struct SystemNode {
    /// Index of previous system.
    ///
    prev: usize,
    /// Boxed system.
    ///
    system: StoredSystem,
    /// Index of next system.
    ///
    next: usize,
}
/// [`SystemStorage`] struct implements schedule of [`System`]s.
///
/// Conceptually, [`SystemStorage`] can be thought of as an `LinkedList<S>`
/// (more precisely, doubly linked list with efficient O(1) random access).
///
/// That allows to store systems in a sequence and
/// additionally have the ability to change their positions very quickly.
///
#[derive(Debug, Default)]
pub struct SystemStorage {
    /// `TypeIdMap` that maps [`SystemId`]s to indices in which corresponding system is in a schedule.
    /// That allows O(1) random access to the linked list nodes.
    ///
    system_positions: TypeIdMap<SystemId, usize>,
    /// `Vec` that emulates `LinkedList<S>`.
    /// That allows sequenced access through systems schedule.
    ///
    schedule: Vec<SystemNode>,
}
impl SystemStorage {
    /// Initializes new [`SystemStorage`].
    ///
    /// Created [`SystemStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::SystemStorage;
    /// let storage: SystemStorage = SystemStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        SystemStorage {
            system_positions: TypeIdMap::with_hasher(NoOpHasherState),
            schedule: Vec::new(),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.system_positions.clear();
        self.schedule.clear();
    }
}
