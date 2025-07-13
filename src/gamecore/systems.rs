//! `gamecore::systems` submodule provides systems - functions that operate on
//! entities and their components and implement whole behaviour of an application.
//!

use crate::gamecore::{
    components::{Component, Resource},
    scenes::Scene,
    storages::{EntityComponentStorage, ResourceStorage},
};
use seq_macro::seq;
use std::{any::{TypeId, Any}, marker::PhantomData};

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

pub struct Components<'a, C: ComponentQueries> {
    storage: &'a mut EntityComponentStorage,
    _marker: PhantomData<C>,
}
pub struct Resources<'a, R: SceneData<ResourceMarker>> {
    storage: &'a mut ResourceStorage,
    _marker: PhantomData<R>,
}

pub enum SystemPreparationError {}
pub trait PrepareSystem {
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError>;
}
pub struct SystemId(TypeId);
pub trait PreparedSystem {
    fn id(&self) -> SystemId;

    fn run(&mut self, scene: &mut Scene);
}
pub type BoxedPreparedSystem = Box<dyn PreparedSystem>;

pub struct BareSystem<F: for<'scene> FnMut(&'scene mut Scene) -> () + 'static>(pub F);
impl<F> PrepareSystem for BareSystem<F>
where
    F: for<'scene> FnMut(&'scene mut Scene) -> () + 'static
{
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError> {
        Ok(self)
    }
}
impl<F> PreparedSystem for BareSystem<F>
where
    F: for<'scene> FnMut(&'scene mut Scene) -> () + 'static
{
    fn id(&self) -> SystemId {
        SystemId(self.0.type_id())
    }

    fn run(&mut self, scene: &mut Scene) {
        self.0(scene)
    }
}

pub struct System<
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
> {
    system: F,
    _markers: PhantomData<(C, R)>,
}
impl<F, C, R> System<F, C, R>
where
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>
{
    pub fn new(f: F) -> Self {
        System {
            system: f,
            _markers: PhantomData
        }
    }
}
impl<F, C, R> From<F> for System<F, C, R>
where
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn from(value: F) -> Self {
        System {
            system: value,
            _markers: PhantomData,
        }
    }
}
impl<F, C, R> PrepareSystem for System<F, C, R>
where
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError> {
        todo!("prepare system");
        Ok(PreparedSystemWrapper::from(self))
    }
}
struct PreparedSystemWrapper<
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
> {
    system: F,
    _markers: PhantomData<(C, R)>,
}
impl<F, C, R> From<System<F, C, R>> for PreparedSystemWrapper<F, C, R>
where
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn from(value: System<F, C, R>) -> Self {
        PreparedSystemWrapper {
            system: value.system,
            _markers: value._markers,
        }
    }
}
impl<F, C, R> PreparedSystem for PreparedSystemWrapper<F, C, R>
where
    F: for<'scene> FnMut(Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn id(&self) -> SystemId {
        SystemId(self.system.type_id())
    }
    
    fn run(&mut self, scene: &mut Scene) {
        (self.system)(
            Components {
                storage: &mut scene.entity_component_storage,
                _marker: PhantomData,
            },
            Resources {
                storage: &mut scene.resource_storage,
                _marker: PhantomData,
            },
        )
    }
}

pub struct StatefulSystem<
    T: 'static,
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
> {
    state: T,
    system: F,
    _markers: PhantomData<(C, R)>,
}
impl<T, F, C, R> StatefulSystem<T, F, C, R>
where
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>
{
    pub fn new(f: F, state: T) -> Self {
        StatefulSystem {
            state,
            system: f,
            _markers: PhantomData
        }
    }
}
impl<T, F, C, R> From<F> for StatefulSystem<T, F, C, R>
where
    T: Default + 'static,
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn from(value: F) -> Self {
        StatefulSystem {
            state: Default::default(),
            system: value,
            _markers: PhantomData,
        }
    }
}
impl<T, F, C, R> PrepareSystem for StatefulSystem<T, F, C, R>
where
    T: 'static,
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn prepare(self) -> Result<impl PreparedSystem, SystemPreparationError> {
        todo!("prepare system");
        Ok(PreparedStatefulSystemWrapper::from(self))
    }
}
struct PreparedStatefulSystemWrapper<
    T: 'static,
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
> {
    state: T,
    system: F,
    _markers: PhantomData<(C, R)>,
}
impl<T, F, C, R> From<StatefulSystem<T, F, C, R>> for PreparedStatefulSystemWrapper<T, F, C, R>
where
    T: 'static,
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn from(value: StatefulSystem<T, F, C, R>) -> Self {
        PreparedStatefulSystemWrapper {
            state: value.state,
            system: value.system,
            _markers: value._markers,
        }
    }
}
impl<T, F, C, R> PreparedSystem for PreparedStatefulSystemWrapper<T, F, C, R>
where
    T: 'static,
    F: for<'scene> FnMut(&'scene mut T, Components<'scene, C>, Resources<'scene, R>) -> () + 'static,
    C: ComponentQueries,
    R: SceneData<ResourceMarker>,
{
    fn id(&self) -> SystemId {
        SystemId(self.system.type_id())
    }

    fn run(&mut self, scene: &mut Scene) {
        (self.system)(
            &mut self.state,
            Components {
                storage: &mut scene.entity_component_storage,
                _marker: PhantomData,
            },
            Resources {
                storage: &mut scene.resource_storage,
                _marker: PhantomData,
            },
        )
    }
}
