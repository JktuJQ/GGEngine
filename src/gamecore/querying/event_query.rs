//! Submodule that implements [`EventQuery`].
//!

use super::{QueryParameter, QueryParameterMarker, QueryParameterTuple};
use crate::gamecore::events::{Event, EventStorage};
use std::{error::Error, fmt, marker::PhantomData};

/// [`EventMarker`] zero-sized type serves as a parameter marker
/// for queries that operate on [`Event`]s.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct EventMarker;
impl QueryParameterMarker for EventMarker {}

impl<E: Event> QueryParameter<EventMarker> for &E {
    type Inner = E;
}
impl<E: Event> QueryParameter<EventMarker> for &mut E {
    type Inner = E;
}

/// [`EventsTuple`] trait is an alias for `QueryParameterTuple<EventMarker>`.
/// It is implemented for tuples of [`QueryParameter`]s which are marked as events.
///
pub trait EventsTuple: QueryParameterTuple<EventMarker> {}
impl<T: QueryParameterTuple<EventMarker>> EventsTuple for T {}

/// [`EventQueryValidationError`] enum lists all errors that could occur
/// during validation of [`EventQuery`] parameters.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EventQueryValidationError {}
impl fmt::Display for EventQueryValidationError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("`EventQueryValidationError` enum has no variants")
    }
}
impl Error for EventQueryValidationError {}

/// [`EventQuery`] struct represents a result of querying events from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct EventQuery<'a, EventParams: EventsTuple> {
    /// Storage of resources.
    ///
    storage: &'a mut EventStorage,

    /// `PhantomData` for event parameters.
    ///
    _params: PhantomData<EventParams>,
}
impl<'a, EventParams: EventsTuple> EventQuery<'a, EventParams> {
    pub fn is_valid() -> Result<(), EventQueryValidationError> {
        todo!("perform validation based on `EventParams`")
    }

    pub fn new(storage: &'a mut EventStorage) -> Result<Self, EventQueryValidationError> {
        Self::is_valid()?;
        Ok(Self::new_validated(storage))
    }
    pub fn new_validated(storage: &'a mut EventStorage) -> Self {
        Self {
            storage,

            _params: PhantomData,
        }
    }
}
