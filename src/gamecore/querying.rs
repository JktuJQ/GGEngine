//! `systems::querying` submodule defines several traits and structs
//! which allow rich and concise representation of system queries.
//!
//! This submodule leverages Rust type system a lot,
//! and it may be a bit much to take for an end user of the `ggengine`.
//! Encapsulating all of that into separate submodule would unload the docs on `gamecore::systems`
//! and better convey intentions.
//!

use seq_macro::seq;
use std::any::Any;

/// [`QueryParameterMarker`] trait is defined for all query types that `ggengine` supports.
///
pub trait QueryParameterMarker {}

/// [`QueryParameter`] trait defines types that may be used for querying.
///
pub trait QueryParameter<M: QueryParameterMarker> {
    /// Type of parameter without indirections.
    ///
    type Inner: Any;
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

// submodules and public re-exports
pub(super) mod component_query;
pub(super) mod event_query;
pub(super) mod resource_query;
pub(super) mod system_query;
