//! Submodule that implements [`ComponentQuery`].
//!

use super::{QueryParameter, QueryParameterMarker, QueryParameterTuple};
use crate::gamecore::components::{Component, ComponentStorage};
use seq_macro::seq;
use std::marker::PhantomData;

/// [`ComponentMarker`] zero-sized type serves as a parameter marker
/// for queries that operate on [`Component`]s.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct ComponentMarker;
impl QueryParameterMarker for ComponentMarker {}

impl<C: Component> QueryParameter<ComponentMarker> for &C {
    type Inner = C;
}
impl<C: Component> QueryParameter<ComponentMarker> for &mut C {
    type Inner = C;
}

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

/// [`ComponentQuery`] struct represents a result of querying components from [`Scene`](crate::gamecore::scenes::Scene).
///
#[derive(Debug)]
pub struct ComponentQuery<'a, ComponentParams: ComponentGroupsTuple> {
    /// Storage of components.
    ///
    storage: &'a mut ComponentStorage,

    /// `PhantomData` for component parameters.
    ///
    _params: PhantomData<ComponentParams>,
}
impl<'a, ComponentParams: ComponentGroupsTuple> ComponentQuery<'a, ComponentParams> {
    pub fn new(storage: &'a mut ComponentStorage) -> Self {
        Self {
            storage,

            _params: PhantomData,
        }
    }
}
