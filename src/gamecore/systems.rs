//! `gamecore::systems` submodule provides systems - functions that operate on
//! queries and implement whole behaviour of an application.
//!

// submodules and public re-exports
use crate::gamecore::{
    querying::{
        component_query::{ComponentGroupsTuple, ComponentQuery},
        event_query::{EventQuery, EventsTuple},
        resource_query::{ResourceQuery, ResourcesTuple},
    },
    scenes::Scene,
};
use std::{
    any::{Any, TypeId},
    fmt,
    marker::PhantomData,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(TypeId);
pub trait System {
    fn id(&self) -> SystemId;

    fn run(&mut self, scene: &mut Scene);
}
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
            SystemQuery,
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
                SystemQuery,
            ) + 'static,
    > System for SystemFn<State, ComponentParams, ResourceParams, EventParams, F>
{
    fn id(&self) -> SystemId {
        SystemId(self.f.type_id())
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.f)(
            &mut self.state,
            ComponentQuery::new(&mut scene.component_storage),
            ResourceQuery::new(&mut scene.resource_storage),
            EventQuery::new(&mut scene.event_storage),
            SystemQuery::new(&mut scene.system_storage),
        )
    }
}

pub use crate::gamecore::{querying::system_query::*, storages::system_storage::*};
