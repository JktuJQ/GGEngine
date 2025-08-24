//! `gamecore::systems` submodule provides systems - functions that operate on
//! entities and their components and implement whole behaviour of an application.
//!

// submodules and public re-exports
use crate::gamecore::{
    querying::{
        ComponentGroupsTuple, ComponentQuery, EventQuery, EventsTuple, ResourceQuery,
        ResourcesTuple, SystemQuery,
    },
    scenes::Scene,
};
use std::{
    any::{Any, TypeId},
    error::Error,
    fmt,
    marker::PhantomData,
};

/// [`QueryValidationError`] enum lists all errors that could occur
/// during validation of query parameters of the system function.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum QueryValidationError {}
impl fmt::Display for QueryValidationError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("`QueryValidationError` enum has no variants")
    }
}
impl Error for QueryValidationError {}

/// [`SystemId`] id struct is needed to identify [`System`]s in [`SystemStorage`].
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by `ggengine`.
///
/// Storages internally operate on ids, which allows them to provide more flexible interface.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(TypeId);
/// [`System`] trait is implemented for prepared functions
/// that are ready to operate on [`Scene`]s.
///
pub trait System {
    /// Returns the id of the system.
    ///
    /// # Note
    /// On the stable Rust one type cannot implement `FnMut` trait multiple times with different arguments,
    /// but type system does not know that (and it is possible to do on nightly).
    /// Potentially, some type could be both `FnMut(A)` and `FnMut(A, B)`,
    /// and all systems derived from this type will have the same id.
    ///
    fn id(&self) -> SystemId;

    /// Runs the system.
    ///
    /// Basically, anything that works with the [`Scene`] could be considered as a [`System`].
    ///
    fn run(&mut self, scene: &mut Scene);
}
/// Type alias for `Box<dyn System>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of systems is needed.
///
/// `Box<dyn System>` also allows combining multiple different [`System`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedSystem = Box<dyn System>;
impl fmt::Debug for dyn System {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "System with id {:?}", self.id())
    }
}

pub struct RawSystemFn<State, F: FnMut(&mut State, &mut Scene) + 'static> {
    state: State,
    f: F,
}
impl<State, F: FnMut(&mut State, &mut Scene) + 'static> System for RawSystemFn<State, F> {
    fn id(&self) -> SystemId {
        SystemId(self.f.type_id())
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.f)(&mut self.state, scene)
    }
}

pub struct SystemFn<
    State,
    ComponentParams: ComponentGroupsTuple,
    ResourceParams: ResourcesTuple,
    EventParams: EventsTuple,
    F: FnMut(
            &mut State,
            ComponentQuery<ComponentParams>,
            ResourceQuery<ResourceParams>,
            EventQuery<EventParams>,
        ) + 'static,
> {
    state: State,
    f: F,

    _params: PhantomData<(ComponentParams, ResourceParams, EventParams)>,
}
impl<
        State,
        ComponentParams: ComponentGroupsTuple,
        ResourceParams: ResourcesTuple,
        EventParams: EventsTuple,
        F: FnMut(
                &mut State,
                ComponentQuery<ComponentParams>,
                ResourceQuery<ResourceParams>,
                EventQuery<EventParams>,
            ) + 'static,
    > System for SystemFn<State, ComponentParams, ResourceParams, EventParams, F>
{
    fn id(&self) -> SystemId {
        SystemId(self.f.type_id())
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.f)(
            &mut self.state,
            ComponentQuery::new_validated(&mut scene.component_storage),
            ResourceQuery::new_validated(&mut scene.resource_storage),
            EventQuery::new_validated(&mut scene.event_storage),
        )
    }
}
