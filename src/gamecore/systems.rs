//! `gamecore::systems` submodule provides systems - functions that operate on
//! entities and their components and implement whole behaviour of an application.
//!

use crate::gamecore::{
    components::{Component, Resource},
    scenes::Scene,
};
use seq_macro::seq;
use std::any::Any;

pub trait SystemState: Any {}
pub struct LocalState<'a, T>(pub &'a mut T);

struct ComponentMarker;
struct ResourceMarker;
trait SceneDataElement<M> {
    type Inner;
}
impl<C: Component> SceneDataElement<ComponentMarker> for &C {
    type Inner = C;
}
impl<C: Component> SceneDataElement<ComponentMarker> for &mut C {
    type Inner = C;
}
impl<R: Resource> SceneDataElement<ResourceMarker> for &R {
    type Inner = R;
}
impl<R: Resource> SceneDataElement<ResourceMarker> for &mut R {
    type Inner = R;
}

pub trait SceneData<M> {
    const ELEMENTS: usize;
}
macro_rules! impl_query_data {
    ($size:tt: $($t:ident,)*) => {
        impl<M, $($t: SceneDataElement<M>,)*> SceneData<M> for ($($t,)*) {
            const ELEMENTS: usize = $size;
        }
    };
}
seq!(SIZE in 0..=16 {
    #(seq!(N in 0..SIZE { impl_query_data!(SIZE: #(T~N,)*); });)*
});
pub struct Query<D: SceneData<ComponentMarker>>(D);

pub trait ComponentQueries {
    const QUERIES: usize;
    const QUERIES_COMPONENTS: usize;
}
macro_rules! impl_component_queries {
    ($size:tt: $($t:ident,)*) => {
        impl<$($t: SceneData<ComponentMarker>,)*> ComponentQueries for ($(Query<$t>,)*) {
            const QUERIES: usize = $size;
            const QUERIES_COMPONENTS: usize = $($t::ELEMENTS + )* 0;
        }
    };
}
seq!(SIZE in 0..=16 {
    #(seq!(N in 0..SIZE { impl_component_queries!(SIZE: #(Q~N,)*); });)*
});
pub struct Components<Q: ComponentQueries>(Q);

pub struct Resources<D: SceneData<ResourceMarker>>(D);

pub trait UnpreparedSystem<S: SystemState, C: ComponentQueries, R: SceneData<ResourceMarker>> {
    fn prepare_system(self) -> System;
}
impl<F, S, C, R> UnpreparedSystem<S, C, R> for F
where
    F: FnMut(LocalState<S>, Components<C>, Resources<R>) -> (),

    S: SystemState,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn prepare_system(self) -> System {
        System(|scene| ())
    }
}

#[derive(Debug)]
pub struct System(pub fn(&mut Scene) -> ());
