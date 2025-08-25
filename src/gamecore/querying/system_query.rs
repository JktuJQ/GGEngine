//! Submodule that implements [`SystemQuery`].
//!

use crate::gamecore::storages::system_storage::SystemStorage;

/// [`SystemQuery`] struct represents a result of querying events from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct SystemQuery<'a> {
    /// Storage of resources.
    ///
    storage: &'a mut SystemStorage,
}
impl<'a> SystemQuery<'a> {
    pub fn new(storage: &'a mut SystemStorage) -> Self {
        Self { storage }
    }
}
