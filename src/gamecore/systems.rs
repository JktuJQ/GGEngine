//! `gamecore::systems` submodule provides systems - functions that operate on
//! queries and implement whole behaviour of an application.
//!

// submodules and public re-exports
use crate::gamecore::scenes::Scene;
use std::any::{Any, TypeId};

/// [`SystemId`] id struct is needed to identify [`System`]s in [`SystemStorage`].
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by `ggengine`.
///
/// Storages internally operate on ids, which allows them to provide more flexible interface.
///
/// # Note
/// This id struct is different from others such as
/// [`ComponentId`](crate::gamecore::components::ComponentId),
/// [`ResourceId`](crate::gamecore::components::ResourceId) and
/// [`EventId`](crate::gamecore::components::EventId).
/// That is because Rust does not allow to write type of function,
/// and [`SystemId`]s should be derived from functions.
/// Because of that, [`SystemId`] takes a reference to `impl Any` value
/// from which it would obtain id.
/// That means that [`SystemId`] could be derived from objects that are not functions.
///
/// Although that is undesirable, there is a case in which that could present useful.
/// See docs of `System::id` for more information.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(TypeId);
impl SystemId {
    /// Obtains [`SystemId`] from type that is passed behind reference.
    ///
    pub fn of(value: &impl Any) -> Self {
        SystemId((*value).type_id())
    }
}
/// [`System`] trait represents functions that could be used to implement behaviour of the `ggengine` [`Scene`].
///
/// There is a resemblance of this trait and the `Fn*` family traits,
/// and that is because any [`System`] is just a function.
/// The main feature that those functions should have to be a [`System`]
/// is that its arguments could be derived from `&mut Scene`.
/// That said, [`System`] trait is implemented for all functions
/// that take `&mut Scene` or operate on queries
/// ([`ComponentQuery`], [`ResourceQuery`], [`EventQuery`] and [`SystemQuery`]).
///
/// See more examples of system usage in the module docs.
///
pub trait System<Args>: 'static {
    /// Type of the output of the system.
    ///
    type Output;

    /// Id of a system.
    ///
    /// # Note
    /// Currently in Rust one type could theoretically implement `Fn*` trait multiple times with different arguments.
    /// For that type, [`SystemId`] would be the same and it could lead to some unexpected behaviour.
    ///
    /// For example, storing that type in [`SystemStorage`] as [`System`] with arguments `Args1`
    /// and as [`System`] with arguments `Args2`
    /// would make storage to view those systems as one, because [`SystemId`] of those are equal.
    ///
    /// Fortunately, such type is quite rare and is currently possible only on `nightly` through manual implementation,
    /// so this issue should not be of any concern most of the time.
    /// If you really need to support that case, you should use newtypes for those different [`System`] interpretations
    /// and provide different [`SystemId`]s
    /// (which could be achieved with proxying [`SystemId`] from dummy struct).
    ///
    fn id(&self) -> SystemId;

    /// Runs system function.
    ///
    /// It is easy to see rom the signature of this function
    /// that every system is isomorphic to `FnMut(&mut Scene) -> Output`
    /// (that is, that all of its arguments could be derived from `&mut Scene`).
    ///
    fn run(&mut self, scene: &mut Scene) -> Self::Output;
}
/// Type alias for `Box<dyn System>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of system is needed.
///
/// `Box<dyn System>` also allows combining multiple different [`System`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedSystem<Args, Output> = Box<dyn System<Args, Output = Output>>;

impl<Output, F: FnMut(&mut Scene) -> Output + 'static> System<(&mut Scene,)> for F {
    type Output = Output;

    fn id(&self) -> SystemId {
        SystemId(self.type_id())
    }
    
    fn run(&mut self, scene: &mut Scene) -> Self::Output {
        self(scene)
    }
}

pub use crate::gamecore::{querying::system_query::*, storages::system_storage::*};
