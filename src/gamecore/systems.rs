//! `gamecore::systems` submodule provides systems - functions that operate on
//! entities and their components and implement whole behaviour of an application.
//!

// submodules and public re-exports
pub mod querying;
pub use querying::{ComponentGroup, ComponentsQuery, EventsQuery, ResourcesQuery};
use querying::{ComponentGroupsTuple, EventsTuple, ResourcesTuple};

use crate::gamecore::scenes::Scene;
use std::{
    any::{Any, TypeId},
    error::Error,
    fmt,
    marker::PhantomData,
};

/// [`SystemPreparationError`] enum lists all errors that could occur
/// during preparation of the system function.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SystemPreparationError {}
impl fmt::Display for SystemPreparationError {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("`SystemPreparationError` enum has no variants.")
    }
}
impl Error for SystemPreparationError {}
/// [`PrepareSystemFn`] trait is implemented for functions that could be
/// transformed into `ggengine` systems.
///
/// Functions that take high level abstractions that could be derived from the [`Scene`]
/// are considered unprepared, because they parameters may be ill-formed and incoherent.
/// Additional checks are needed to ensure correctness,
/// and `PrepareSystemFn::prepare` function is specifically for that.
///
pub trait PrepareSystemFn {
    /// Prepares system function, ensuring parameters correctness.
    ///
    fn prepare(self) -> Result<impl System, SystemPreparationError>;
}

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

pub struct RawSystemFn<F, S> {
    state: S,
    f: F,
}
impl<F, S> RawSystemFn<F, S>
where
    F: for<'a> FnMut(&'a mut S, &'a mut Scene) + Any,
{
    pub fn with_state(state: S, f: F) -> Self {
        Self { state, f }
    }
}
impl<F, S> RawSystemFn<F, S>
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
impl<F, S> PrepareSystemFn for RawSystemFn<F, S>
where
    F: for<'a> FnMut(&'a mut S, &'a mut Scene) + Any,
{
    fn prepare(self) -> Result<impl System, SystemPreparationError> {
        Ok(self)
    }
}
impl<F, S> System for RawSystemFn<F, S>
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

pub struct SystemFn<F, S, C, R, E> {
    state: S,
    f: F,

    _parameters: PhantomData<(C, R, E)>,
}
impl<F, S, C, R, E> SystemFn<F, S, C, R, E>
where
    C: ComponentGroupsTuple,
    R: ResourcesTuple,
    E: EventsTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>, EventsQuery<'a, E>)
        + Any,
{
    pub fn with_state(state: S, f: F) -> Self {
        Self {
            state,
            f,
            _parameters: PhantomData,
        }
    }
}
impl<F, S, C, R, E> SystemFn<F, S, C, R, E>
where
    S: Default,
    C: ComponentGroupsTuple,
    R: ResourcesTuple,
    E: EventsTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>, EventsQuery<'a, E>)
        + Any,
{
    pub fn with_default_state(f: F) -> Self {
        Self {
            state: S::default(),
            f,
            _parameters: PhantomData,
        }
    }
}

impl<F, S, C, R, E> PrepareSystemFn for SystemFn<F, S, C, R, E>
where
    C: ComponentGroupsTuple,
    R: ResourcesTuple,
    E: EventsTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>, EventsQuery<'a, E>)
        + Any,
{
    fn prepare(self) -> Result<impl System, SystemPreparationError> {
        Ok(SystemFnWrapper(self))
    }
}

struct SystemFnWrapper<F, S, C, R, E>(SystemFn<F, S, C, R, E>);
impl<F, S, C, R, E> System for SystemFnWrapper<F, S, C, R, E>
where
    C: ComponentGroupsTuple,
    R: ResourcesTuple,
    E: EventsTuple,

    F: for<'a> FnMut(&'a mut S, ComponentsQuery<'a, C>, ResourcesQuery<'a, R>, EventsQuery<'a, E>)
        + Any,
{
    fn id(&self) -> SystemId {
        SystemId(self.0.f.type_id())
    }

    fn run(&mut self, _: &mut Scene) {}
}
