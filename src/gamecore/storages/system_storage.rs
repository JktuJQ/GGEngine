//! Submodule that implement [`SystemStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::systems::{System, SystemId};

/// [`SystemNode`] struct serves as the emulation of node in doubly linked list.
///
#[derive(Debug)]
struct SystemNode {
    /// Index of previous system.
    ///
    prev: usize,
    /// Boxed system.
    ///
    system: Box<dyn System>,
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
