//! Submodule that implements [`SystemQuery`].
//!

use crate::gamecore::storages::SystemStorage;
use std::{error::Error, fmt};

/// [`SystemQueryValidationError`] enum lists all errors that could occur
/// during validation of [`SystemQuery`] parameters.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SystemQueryValidationError {}
impl fmt::Display for SystemQueryValidationError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("`SystemQueryValidationError` enum has no variants")
    }
}
impl Error for SystemQueryValidationError {}

/// [`SystemQuery`] struct represents a result of querying events from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct SystemQuery<'a> {
    /// Storage of resources.
    ///
    storage: &'a mut SystemStorage,
}
impl<'a> SystemQuery<'a> {
    pub fn is_valid() -> Result<(), SystemQueryValidationError> {
        Ok(())
    }

    pub fn new(storage: &'a mut SystemStorage) -> Result<Self, SystemQueryValidationError> {
        Self::is_valid()?;
        Ok(Self::new_validated(storage))
    }
    pub fn new_validated(storage: &'a mut SystemStorage) -> Self {
        Self { storage }
    }
}
