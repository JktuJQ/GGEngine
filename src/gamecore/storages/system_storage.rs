//! Submodule that implement [`SystemStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::systems::{DecomposedSystem, System, SystemId};

/// [`SystemPosition`] enum lists possible positions
/// in which new system could be inserted into [`SystemStorage`].
///
/// Docs on [`SystemPosition`] variants describe specifics of each position.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SystemPosition {
    /// Start position (system will be inserted in the start of a [`SystemStorage`]'s schedule).
    ///
    Start,
    /// System will be inserted right before system with provided [`SystemId`].
    ///
    /// Case where no system with such [`SystemId`] is present in [`SystemStorage`]
    /// will be handled as the `SystemPosition::Start` case.
    ///
    Before(SystemId),
    /// System will be inserted right after system with provided [`SystemId`].
    ///
    /// Case where no system with such [`SystemId`] is present in [`SystemStorage`]
    /// will be handled as the `SystemPosition::End` case.
    ///
    After(SystemId),
    /// End position (system will be inserted in the end of a [`SystemStorage`]'s schedule).
    ///
    End,
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
/// (more precisely, doubly linked list with efficient O(1) random access).
///
/// That allows to store systems in a sequence and
/// additionally have the ability to change their positions very quickly.
///
#[derive(Debug, Default)]
pub struct SystemStorage {
    /// `Vec` that emulates `LinkedList<S>`.
    /// That allows sequenced access through systems schedule.
    ///
    schedule: Vec<SystemNode>,
    /// `TypeIdMap` that maps [`SystemId`]s to indices in which corresponding system is in a schedule.
    /// That allows O(1) random access to the linked list nodes.
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
