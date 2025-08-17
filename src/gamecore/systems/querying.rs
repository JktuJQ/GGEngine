//! `systems::querying` submodule defines several traits and structs
//! which allow rich and concise representation of system queries.
//!
//! This submodule leverages Rust type system a lot,
//! and it may be a bit much to take for an end user of the `ggengine`.
//! Encapsulating all of that into separate submodule would unload the docs on `gamecore::systems`
//! and better convey intentions.
//!

use crate::gamecore::{
    components::Component,
    events::Event,
    resources::Resource,
    storages::{EntityComponentStorage, EventStorage, ResourceStorage},
};
use seq_macro::seq;
use std::{any::Any, marker::PhantomData};

/// [`QueryParameterMarker`] trait is defined for all query types that `ggengine` supports.
///
pub trait QueryParameterMarker {}

/// [`ComponentMarker`] zero-sized type serves as a parameter marker
/// for queries that operate on [`Component`]s.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct ComponentMarker;
impl QueryParameterMarker for ComponentMarker {}

/// [`ResourceMarker`] zero-sized type serves as a parameter marker
/// for queries that operate on [`Resource`]s.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct ResourceMarker;
impl QueryParameterMarker for ResourceMarker {}

/// [`EventMarker`] zero-sized type serves as a parameter marker
/// for queries that operate on [`Event`]s.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct EventMarker;
impl QueryParameterMarker for EventMarker {}

/// [`QueryParameter`] trait defines types that may be used for querying.
///
pub trait QueryParameter<M: QueryParameterMarker> {
    /// Type of parameter without indirections.
    ///
    type Inner: Any;
}

impl<C: Component> QueryParameter<ComponentMarker> for &C {
    type Inner = C;
}
impl<C: Component> QueryParameter<ComponentMarker> for &mut C {
    type Inner = C;
}

impl<R: Resource> QueryParameter<ResourceMarker> for &R {
    type Inner = R;
}
impl<R: Resource> QueryParameter<ResourceMarker> for &mut R {
    type Inner = R;
}

impl<E: Event> QueryParameter<EventMarker> for &E {
    type Inner = E;
}
impl<E: Event> QueryParameter<EventMarker> for &mut E {
    type Inner = E;
}

/// [`QueryParameterTuple`] trait is defined for tuples of [`QueryParameter`]s which have the same marker.
///
pub trait QueryParameterTuple<M: QueryParameterMarker> {
    /// The size of a tuple.
    ///
    const SIZE: usize;
}
/// `impl_query_parameter_tuple` macro implements [`QueryParameterTuple`] trait for tuples.
///
macro_rules! impl_query_parameter_tuple {
    ($size:expr => $($t:ident),* $(,)?) => {
        impl<M: QueryParameterMarker, $($t: QueryParameter<M>,)*> QueryParameterTuple<M> for ($($t,)*) {
            const SIZE: usize = $size;
        }
    };
}
seq!(SIZE in 0..=16 {
    #(seq!(N in 0..SIZE { impl_query_parameter_tuple!(SIZE => #(T~N,)*); });)*
});

/// [`ComponentsTuple`] trait is an alias for `QueryParameterTuple<ComponentMarker>`.
/// It is implemented for tuples of [`QueryParameter`]s which are marked as components.
///
pub trait ComponentsTuple: QueryParameterTuple<ComponentMarker> {}
impl<T: QueryParameterTuple<ComponentMarker>> ComponentsTuple for T {}

/// [`ComponentGroup`] struct is a type that allows 'wrapping' [`ComponentsTuple`] as its generic parameter.
/// It is used only in typing of [`ComponentsQuery`].
///
#[derive(Debug)]
pub struct ComponentGroup<T: ComponentsTuple>(PhantomData<T>);

/// [`ComponentGroupsTuple`] trait is implemented for tuples of [`ComponentGroup`]s.
///
pub trait ComponentGroupsTuple {
    /// Size of a tuple.
    ///
    const SIZE: usize;
    /// Total number of components in all groups.
    ///
    const TOTAL_COMPONENTS: usize;
}
/// `impl_component_groups_tuple` macro implements [`ComponentGroupsTuple`] trait for tuples.
///
macro_rules! impl_component_groups_tuple {
    ($size:expr => $($t:ident),* $(,)?) => {
        impl<$($t: ComponentsTuple,)*> ComponentGroupsTuple for ($(ComponentGroup<$t>,)*) {
            const SIZE: usize = $size;
            const TOTAL_COMPONENTS: usize = $($t::SIZE + )* 0;
        }
    };
}
seq!(SIZE in 0..=16 {
    #(seq!(N in 0..SIZE { impl_component_groups_tuple!(SIZE => #(Q~N,)*); });)*
});

/// [`ComponentsQuery`] struct represents a result of querying components from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct ComponentsQuery<'a, T: ComponentGroupsTuple> {
    /// Storage of components.
    ///
    storage: &'a mut EntityComponentStorage,

    /// `PhantomData` for 'unused' generic parameter.
    ///
    _tuples: PhantomData<T>,
}

/// [`ResourcesTuple`] trait is an alias for `QueryParameterTuple<ResourceMarker>`.
/// It is implemented for tuples of [`QueryParameter`]s which are marked as resources.
///
pub trait ResourcesTuple: QueryParameterTuple<ResourceMarker> {}
impl<T: QueryParameterTuple<ResourceMarker>> ResourcesTuple for T {}

/// [`ResourcesQuery`] struct represents a result of querying resources from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct ResourcesQuery<'a, T: ResourcesTuple> {
    /// Storage of resources.
    ///
    storage: &'a mut ResourceStorage,

    /// `PhantomData` for 'unused' generic parameter.
    ///
    _tuples: PhantomData<T>,
}

/// [`EventsTuple`] trait is an alias for `QueryParameterTuple<EventMarker>`.
/// It is implemented for tuples of [`QueryParameter`]s which are marked as events.
///
pub trait EventsTuple: QueryParameterTuple<EventMarker> {}
impl<T: QueryParameterTuple<EventMarker>> EventsTuple for T {}

/// [`EventsQuery`] struct represents a result of querying events from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct EventsQuery<'a, T: EventsTuple> {
    /// Storage of resources.
    ///
    storage: &'a mut EventStorage,

    /// `PhantomData` for 'unused' generic parameter.
    ///
    _tuples: PhantomData<T>,
}
