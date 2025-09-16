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
    any::{type_name, Any, TypeId},
    fmt,
};

/// [`SystemId`] id struct is needed to identify [`System`]s in [`Systemconstructed_query`].
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by `ggengine`.
///
/// constructed_queries internally operate on ids, which allows them to provide more flexible interface.
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
    /// This function requires `self` to allow calling it on functions directly
    /// (there is no way to name function type and thus call static method on function).
    /// That also makes this trait dyn compatible, complying with other `ggengine` traits.
    ///
    /// # Note
    /// Currently in Rust one type could theoretically implement `Fn*` trait multiple times with different arguments.
    /// For that type, [`SystemId`] would be the same and it could lead to some unexpected behaviour.
    ///
    /// For example, storing that type in [`SystemStorage`] as [`System`] with arguments `Args1`
    /// and as [`System`] with arguments `Args2`
    /// would make constructed_query to view those systems as one, because [`SystemId`] of those are equal.
    ///
    /// Fortunately, such type is quite rare and is currently possible only on `nightly` through manual implementation,
    /// so this issue should not be of any concern most of the time.
    /// If you really need to support that case, you should use newtypes for those different [`System`] interpretations
    /// and provide different [`SystemId`]s
    /// (which could be achieved with proxying [`SystemId`] from dummy struct).
    ///
    fn id(&self) -> SystemId {
        SystemId(self.type_id())
    }

    /// Runs system function.
    ///
    /// It is easy to see from the signature of this function
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
impl<Args, Output> fmt::Debug for dyn System<Args, Output = Output> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} (args: {:?}, output: {:?})",
            type_name::<Self>(),
            type_name::<Args>(),
            type_name::<Output>()
        )
    }
}

impl<Output, F> System<(&mut Scene,)> for F
where
    F: FnMut(&mut Scene) -> Output + 'static,
{
    type Output = Output;

    fn run(&mut self, scene: &mut Scene) -> Self::Output {
        self(scene)
    }
}
/// [`impl_system`] macro implements [`System`] trait for functions,
/// which arguments could be derived from `&mut Scene`.
///
/// More specifically, this macro implements [`System`] trait for functions
/// where each argument is a query type
/// ([`ComponentQuery`]/[`ResourceQuery`]/[`EventQuery`]/[`SystemQuery`]).
/// Arguments are not allowed to be repeated, so functions with
/// up to 4 query type arguments are implementors of [`System`] trait.
///
macro_rules! impl_system {
    // base case that generates `impl` block
    (
        $scene:ident |
        generics => $($generic:ident with $generic_bound:ident,)* |
        arguments => $($query:ty,)* |
        constructed_queries => $($constructed_query:expr,)* ;
    ) => {
        impl<$($generic,)* Output, F> System<($($query,)*)> for F
        where
            $($generic: $generic_bound,)*

            F: FnMut($($query,)*) -> Output + 'static,
        {
            type Output = Output;

            fn run(&mut self, $scene: &mut Scene) -> Self::Output {
                self($($constructed_query,)*)
            }
        }
    };

    // cases
    (
        $scene:ident |
        generics => $($generic:ident with $generic_bound:ident,)* |
        arguments => $($query:ty,)* |
        constructed_queries => $($constructed_query:expr,)* ;
        components, $($parameter:ident,)*
    ) => {
        impl_system!(
            $scene |
            generics => $($generic with $generic_bound,)* ComponentParams with ComponentGroupsTuple, |
            arguments => $($query,)* ComponentQuery<'_, ComponentParams>, |
            constructed_queries => $($constructed_query,)* ComponentQuery::new(&mut $scene.component_storage), ;
            $($parameter,)*
        );
    };
    (
        $scene:ident |
        generics => $($generic:ident with $generic_bound:ident,)* |
        arguments => $($query:ty,)* |
        constructed_queries => $($constructed_query:expr,)* ;
        resources, $($parameter:ident,)*
    ) => {
        impl_system!(
            $scene |
            generics => $($generic with $generic_bound,)* ResourceParams with ResourcesTuple, |
            arguments => $($query,)* ResourceQuery<'_, ResourceParams>, |
            constructed_queries => $($constructed_query,)* ResourceQuery::new(&mut $scene.resource_storage), ;
            $($parameter,)*
        );
    };
    (   $scene:ident |
        generics => $($generic:ident with $generic_bound:ident,)* |
        arguments => $($query:ty,)* |
        constructed_queries => $($constructed_query:expr,)* ;
        events, $($parameter:ident,)*
    ) => {
        impl_system!(
            $scene |
            generics => $($generic with $generic_bound,)* EventParams with EventsTuple, |
            arguments => $($query,)* EventQuery<'_, EventParams>, |
            constructed_queries => $($constructed_query,)* EventQuery::new(&mut $scene.event_storage), ;
            $($parameter,)*
        );
    };
    (
        $scene:ident |
        generics => $($generic:ident with $generic_bound:ident,)* |
        arguments => $($query:ty,)* |
        constructed_queries => $($constructed_query:expr,)* ;
        systems, $($parameter:ident,)*
    ) => {
        impl_system!(
            $scene |
            generics => $($generic with $generic_bound,)* |
            arguments => $($query,)* SystemQuery<'_>, |
            constructed_queries => $($constructed_query,)* SystemQuery::new(&mut $scene.system_storage), ;
            $($parameter,)*
        );
    };

    (combination => ($($parameter:ident),*)) => {
        impl_system!(_scene | generics => | arguments => | constructed_queries => ; $($parameter,)*);
    };

    // 68 combinations of parameters
    (for all combinations) => {
        impl_system!(combination => ());

        impl_system!(combination => (components));
        impl_system!(combination => (resources));
        impl_system!(combination => (events));
        impl_system!(combination => (systems));

        impl_system!(combination => (components, resources));
        impl_system!(combination => (resources, components));
        impl_system!(combination => (components, events));
        impl_system!(combination => (events, components));
        impl_system!(combination => (components, systems));
        impl_system!(combination => (systems, components));
        impl_system!(combination => (resources, events));
        impl_system!(combination => (events, resources));
        impl_system!(combination => (resources, systems));
        impl_system!(combination => (systems, resources));
        impl_system!(combination => (events, systems));
        impl_system!(combination => (systems, events));

        impl_system!(combination => (components, resources, events));
        impl_system!(combination => (components, events, resources));
        impl_system!(combination => (resources, components, events));
        impl_system!(combination => (resources, events, components));
        impl_system!(combination => (events, components, resources));
        impl_system!(combination => (events, resources, components));
        impl_system!(combination => (components, resources, systems));
        impl_system!(combination => (components, systems, resources));
        impl_system!(combination => (resources, components, systems));
        impl_system!(combination => (resources, systems, components));
        impl_system!(combination => (systems, components, resources));
        impl_system!(combination => (systems, resources, components));
        impl_system!(combination => (components, events, systems));
        impl_system!(combination => (components, systems, events));
        impl_system!(combination => (events, components, systems));
        impl_system!(combination => (events, systems, components));
        impl_system!(combination => (systems, components, events));
        impl_system!(combination => (systems, events, components));
        impl_system!(combination => (resources, events, systems));
        impl_system!(combination => (resources, systems, events));
        impl_system!(combination => (events, resources, systems));
        impl_system!(combination => (events, systems, resources));
        impl_system!(combination => (systems, resources, events));
        impl_system!(combination => (systems, events, resources));

        impl_system!(combination => (components, resources, events, systems));
        impl_system!(combination => (components, resources, systems, events));
        impl_system!(combination => (components, events, resources, systems));
        impl_system!(combination => (components, events, systems, resources));
        impl_system!(combination => (components, systems, resources, events));
        impl_system!(combination => (components, systems, events, resources));
        impl_system!(combination => (resources, components, events, systems));
        impl_system!(combination => (resources, components, systems, events));
        impl_system!(combination => (resources, events, components, systems));
        impl_system!(combination => (resources, events, systems, components));
        impl_system!(combination => (resources, systems, components, events));
        impl_system!(combination => (resources, systems, events, components));
        impl_system!(combination => (events, components, resources, systems));
        impl_system!(combination => (events, components, systems, resources));
        impl_system!(combination => (events, resources, components, systems));
        impl_system!(combination => (events, resources, systems, components));
        impl_system!(combination => (events, systems, components, resources));
        impl_system!(combination => (events, systems, resources, components));
        impl_system!(combination => (systems, components, resources, events));
        impl_system!(combination => (systems, components, events, resources));
        impl_system!(combination => (systems, resources, components, events));
        impl_system!(combination => (systems, resources, events, components));
        impl_system!(combination => (systems, events, components, resources));
        impl_system!(combination => (systems, events, resources, components));
    };
}
impl_system!(for all combinations);

/// [`DecomposedSystem`] struct is what any system system could be coerced to.
///
/// To store different systems in one generic container,
/// systems arguments and return types must be coerced to some common ground types.
/// `&mut Scene` as argument type and `()` as return type are the only options
/// to which arguments and return type of any system could be coerced.
/// So, [`DecomposedSystem`] is just [`SystemId`] and `Box<dyn FnMut(&mut Scene)>` stored together
/// (basically a [`System`] v-table representation).
///
/// # Example
/// ```rust
/// # use ggengine::gamecore::systems::{System, DecomposedSystem};
/// # use ggengine::gamecore::scenes::Scene;
/// fn system() {
///     println!("system");
/// }
///
/// let mut decomposed_system: DecomposedSystem = DecomposedSystem::from_system(system);
/// assert_eq!(decomposed_system.id(), system.id());
/// decomposed_system.run(&mut Scene::new());  // prints "system"
/// ```
///
pub struct DecomposedSystem {
    /// Id of a system which was coerced to [`DecomposedSystem`].
    ///
    id: SystemId,
    /// Boxed system function.
    ///
    f: Box<dyn FnMut(&mut Scene)>,
}
impl DecomposedSystem {
    /// Decomposes any system.
    ///
    /// This function allows unifying different systems to one type.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{System, DecomposedSystem};
    /// # use ggengine::gamecore::scenes::Scene;
    /// fn system1() {}
    /// fn system2(_: &mut Scene) -> u32 { 42 }
    ///
    /// let systems: Vec<DecomposedSystem> = vec![
    ///     DecomposedSystem::from_system(system1),
    ///     DecomposedSystem::from_system(system2),
    /// ];
    /// ```
    ///
    pub fn from_system<Args, F: System<Args>>(mut system: F) -> Self {
        DecomposedSystem {
            id: system.id(),
            f: Box::new(move |scene: &mut Scene| {
                let _ = system.run(scene);
            }),
        }
    }

    /// Returns inner system function.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::systems::{System, DecomposedSystem};
    /// # use ggengine::gamecore::scenes::Scene;
    /// fn system() {
    ///     println!("system");
    /// }
    ///
    /// let decomposed_system: DecomposedSystem = DecomposedSystem::from_system(system);
    /// let mut system_fn: Box<dyn FnMut(&mut Scene)> = decomposed_system.system_fn();
    /// system_fn(&mut Scene::new())  // prints "system"
    /// ```
    pub fn system_fn(self) -> Box<dyn FnMut(&mut Scene)> {
        self.f
    }
}
impl System<(&mut Scene,)> for DecomposedSystem {
    type Output = ();

    fn id(&self) -> SystemId {
        self.id
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.f)(scene)
    }
}
impl fmt::Debug for DecomposedSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decomposed system with {:?}", self.id)
    }
}

pub use crate::gamecore::{querying::system_query::*, storages::system_storage::*};
