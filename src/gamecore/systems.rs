//! `gamecore::systems` submodule provides systems - functions that operate on
//! entities and their components and implement whole behaviour of an application.
//!

// submodules and public re-exports
pub mod querying;
pub use querying::{ComponentGroup, ComponentsQuery, ResourcesQuery};
use querying::{ComponentGroupsTuple, ResourcesTuple};

use crate::gamecore::scenes::Scene;
use std::{
    any::{Any, TypeId},
    error::Error,
    fmt,
    marker::PhantomData,
};

#[derive(Debug)]
pub enum SystemPreparationError {}
impl fmt::Display for SystemPreparationError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => unreachable!("`SystemPreparationError` enum has no variants."),
        }
    }
}
impl Error for SystemPreparationError {}
pub trait PrepareSystem {
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError>;
}

pub struct SystemId(TypeId);
pub trait PreparedSystem {
    fn id(&self) -> SystemId;

    fn run(&mut self, scene: &mut Scene);
}
pub type BoxedPreparedSystem = Box<dyn PreparedSystem>;

pub struct BareSystem<F, S> {
    state: S,
    f: F,
}
impl<F, S> BareSystem<F, S>
where
    F: for<'a> FnMut(&'a mut S, &'a mut Scene) + Any,
{
    pub fn with_state(state: S, f: F) -> Self {
        Self { state, f }
    }
}
impl<F, S> BareSystem<F, S>
where
    S: Default,

    F: for<'a> FnMut(&'a mut S, &'a mut Scene) + Any,
{
    pub fn with_default_state(f: F) -> Self {
        Self {
            state: S::default(),
            f,
        }
    }
}
impl<F, S> PrepareSystem for BareSystem<F, S>
where
    F: for<'a> FnMut(&'a mut S, &'a mut Scene) + Any,
{
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError> {
        Ok(self)
    }
}
impl<F, S> PreparedSystem for BareSystem<F, S>
where
    F: for<'a> FnMut(&'a mut S, &'a mut Scene) + Any,
{
    fn id(&self) -> SystemId {
        SystemId(self.f.type_id())
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.f)(&mut self.state, scene)
    }
}

pub struct System<F, S, C, R> {
    state: S,
    f: F,

    _parameters: PhantomData<(C, R)>,
}
impl<F, S, C, R> System<F, S, C, R>
where
    C: ComponentGroupsTuple,
    R: ResourcesTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>) + Any,
{
    pub fn with_state(state: S, f: F) -> Self {
        Self {
            state,
            f,
            _parameters: PhantomData,
        }
    }
}
impl<F, S, C, R> System<F, S, C, R>
where
    S: Default,
    C: ComponentGroupsTuple,
    R: ResourcesTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>) + Any,
{
    pub fn with_default_state(f: F) -> Self {
        Self {
            state: S::default(),
            f,
            _parameters: PhantomData,
        }
    }
}

impl<F, S, C, R> PrepareSystem for System<F, S, C, R>
where
    C: ComponentGroupsTuple,
    R: ResourcesTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>) + Any,
{
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError> {
        Ok(SystemWrapper(self))
    }
}

struct SystemWrapper<F, S, C, R>(System<F, S, C, R>);
impl<F, S, C, R> PreparedSystem for SystemWrapper<F, S, C, R>
where
    C: ComponentGroupsTuple,
    R: ResourcesTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>) + Any,
{
    fn id(&self) -> SystemId {
        SystemId(self.0.f.type_id())
    }

    fn run(&mut self, _: &mut Scene) {}
}
