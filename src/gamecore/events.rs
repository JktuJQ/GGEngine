//! `gamecore::events` submodule defines [`Event`] trait
//! that allows representing game logic flow in form of Rust types,
//! and implements several basic events used in games.
//!

use std::{
    any::{type_name, Any, TypeId},
    fmt,
};

/// [`Event`] trait defines data that is used by `ggengine` parts to communicate.
///
/// [`Event`]s are usually small objects which convey a message that someone is expected to handle.
/// In games it could be an event of pressing a button, event of inflicting damage or event of level ending.
/// Some of them are singular, some of them could be send multiple times.
/// [`Event`]s are a tool that allows user to affect the data flow.
///
/// # Implementation
/// [`Event`] trait requires `'static` trait bound, because `Any`
/// is a supertrait of [`Event`] trait, and it requires `'static` trait bound.
///
/// Since most types implement `Any`, defining new [`Event`]s could be done like so:
/// ```rust
/// use ggengine::gamecore::events::Event;
/// struct T;
/// impl Event for T {}
/// ```
///
pub trait Event: Any {}
/// Type alias for `Box<dyn Event>`.
///
/// This type alias will be frequently used in situations in which
/// ownership of resource is needed.
///
/// `Box<dyn Event>` also allows combining multiple different [`Event`]s in one structure
/// (`Vec`, iterator, etc.).
///
pub type BoxedEvent = Box<dyn Event>;
impl dyn Event {
    /// Returns true if the inner type is the same as `E`.
    ///
    pub fn is<E: Event>(&self) -> bool {
        let as_any: &dyn Any = self;
        as_any.is::<E>()
    }

    /// Attempts to downcast the box to a concrete type.
    ///
    /// # Note
    /// `downcast` consumes initial `Box`,
    /// but on failure it does not need to, and so it returns it in upcasted form (`Box<dyn Any>`).
    /// Although it would be preferrable to return initial type (`Box<dyn Event>`),
    /// it is impossible to do so.
    ///
    pub fn downcast<E: Event>(self: Box<Self>) -> Result<Box<E>, Box<dyn Any>> {
        let as_any: Box<dyn Any> = self;
        as_any.downcast::<E>()
    }
    /// Returns some reference to the inner value if it is of type `E`, or `None` if it isn’t.
    ///
    pub fn downcast_ref<E: Event>(&self) -> Option<&E> {
        let as_any: &dyn Any = self;
        as_any.downcast_ref::<E>()
    }
    /// Returns some mutable reference to the inner value if it is of type `E`, or `None` if it isn’t.
    ///
    pub fn downcast_mut<E: Event>(&mut self) -> Option<&mut E> {
        let as_any: &mut dyn Any = self;
        as_any.downcast_mut::<E>()
    }
}
impl fmt::Debug for dyn Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", type_name::<Self>())
    }
}
/// [`EventId`] id struct is needed to identify [`Event`]s in [`EventStorage`].
///
/// # Usage
/// Usage of this struct is fairly advanced.
/// Most of the time you should use convenient statically typed API,
/// which is provided by `ggengine`.
///
/// Storages operate on ids, which allows them to provide more flexible interface.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventId(TypeId);
impl EventId {
    /// Returns [`EventId`] of given [`Event`] type.
    ///
    pub fn of<E: Event>() -> Self {
        EventId(TypeId::of::<E>())
    }
}

// submodules and public re-exports
pub use super::storages::EventStorage;
