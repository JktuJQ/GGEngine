//! Submodule that implements [`ResourceQuery`].
//!

use super::{QueryParameter, QueryParameterMarker, QueryParameterTuple};
use crate::gamecore::resources::{Resource, ResourceStorage};
use std::marker::PhantomData;

/// [`ResourceMarker`] zero-sized type serves as a parameter marker
/// for queries that operate on [`Resource`]s.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct ResourceMarker;
impl QueryParameterMarker for ResourceMarker {}

impl<R: Resource> QueryParameter<ResourceMarker> for &R {
    type Inner = R;
}
impl<R: Resource> QueryParameter<ResourceMarker> for &mut R {
    type Inner = R;
}

/// [`ResourcesTuple`] trait is an alias for `QueryParameterTuple<ResourceMarker>`.
/// It is implemented for tuples of [`QueryParameter`]s which are marked as resources.
///
pub trait ResourcesTuple: QueryParameterTuple<ResourceMarker> {}
impl<T: QueryParameterTuple<ResourceMarker>> ResourcesTuple for T {}

/// [`ResourceQuery`] struct represents a result of querying resources from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct ResourceQuery<'a, ResourceParams: ResourcesTuple> {
    /// Storage of resources.
    ///
    storage: &'a mut ResourceStorage,

    /// `PhantomData` for resource parameters.
    ///
    _params: PhantomData<ResourceParams>,
}
impl<'a, ResourceParams: ResourcesTuple> ResourceQuery<'a, ResourceParams> {
    pub fn new(storage: &'a mut ResourceStorage) -> Self {
        Self {
            storage,

            _params: PhantomData,
        }
    }
}
