//! `gamecore::entities` submodule implements [`Entity`] struct -
//! game object that has some characteristics (components) on which game engine operates.
//!

/// [`EntityId`] id struct is needed to identify [`Entity`](super::entities::Entity)s
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this [`Entity`](super::entities::Entity) is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub(super) u64);
