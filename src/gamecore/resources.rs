//! `gamecore::resources` submodule defines [`Resource`] trait
//! that allows representing global game state and logic in form of Rust types,
//! and implements several basic resources used in games.
//!

use std::{
    any::{type_name, Any, TypeId},
    fmt,
};

/// [`Resource`] trait defines unique global data that is bounded to the `Scene`.
///
/// [`Resource`]s are very similar to [`Component`](crate::gamecore::components::Component)s,
/// with the only difference is that
/// [`Component`](crate::gamecore::components::Component)s are bounded to `Entity`s and
/// [`Resource`]s are bound to the [`Scene`](crate::gamecore::scenes::Scene).
///
/// Applications often have some global data that they share, it could be time, score, asset collection, etc.
/// Although global resources could be implemented as components that belong to some 'global' Entity,
/// that would be confusing and would not convey intention logic.
/// [`Resource`] trait supports this pattern, enforcing it through type system and allowing
/// for data to be shared easily.
///
/// # Implementation
/// [`Resource`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Resource`] trait, and it requires `'static` trait bound.
///
/// Since most types implement `Any`, defining new [`Resource`]s could be done like so:
/// ```rust
/// use ggengine::gamecore::resources::Resource;
/// struct T;
/// impl Resource for T {}
/// ```
///
pub trait Resource: Any {}
/// Type alias for `Box<dyn Resource>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of resource is needed.
///
/// `Box<dyn Resource>` also allows combining multiple different [`Resource`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedResource = Box<dyn Resource>;
impl dyn Resource {
    /// Returns true if the inner type is the same as `R`.
    ///
    pub fn is<R: Resource>(&self) -> bool {
        let as_any: &dyn Any = self;
        as_any.is::<R>()
    }

    /// Attempts to downcast the box to a concrete type.
    ///
    /// # Note
    /// `downcast` consumes initial `Box`,
    /// but on failure it does not need to, and so it returns it in upcasted form (`Box<dyn Any>`).
    /// Although it would be preferrable to return initial type (`Box<dyn Resource>`),
    /// it is impossible to do so.
    ///
    pub fn downcast<R: Resource>(self: Box<Self>) -> Result<Box<R>, Box<dyn Any>> {
        let as_any: Box<dyn Any> = self;
        as_any.downcast::<R>()
    }
    /// Returns some reference to the inner value if it is of type `R`, or `None` if it isn’t.
    ///
    pub fn downcast_ref<R: Resource>(&self) -> Option<&R> {
        let as_any: &dyn Any = self;
        as_any.downcast_ref::<R>()
    }
    /// Returns some mutable reference to the inner value if it is of type `R`, or `None` if it isn’t.
    ///
    pub fn downcast_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let as_any: &mut dyn Any = self;
        as_any.downcast_mut::<R>()
    }
}
impl fmt::Debug for dyn Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", type_name::<Self>())
    }
}
/// [`ResourceId`] id struct is needed to identify [`Resource`]s in [`ResourceStorage`].
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by `ggengine`.
///
/// Storages internally operate on ids, which allows them to provide more flexible interface.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(TypeId);
impl ResourceId {
    /// Returns [`ResourceId`] of given [`Resource`] type.
    ///
    pub fn of<R: Resource>() -> Self {
        ResourceId(TypeId::of::<R>())
    }
}

pub use crate::gamecore::{querying::resource_query::*, storages::resource_storage::*};
