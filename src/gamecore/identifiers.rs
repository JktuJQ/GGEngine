//! `gamecore::identifiers` submodule provides several id structs that
//! uniquely identify objects and allows easy querying on them.
//!

/// [`impl_id`] macro implements basic id struct interface.
///
/// For most id structs, some kind of uniqueness is required for their sound usage.
/// That uniqueness is not provided by this struct, and that is why `new` associated function is
/// exposed only to crate visibility - it is caller's job to provide soundness.
///
macro_rules! impl_id {
    ($struct:ident) => {
        impl $struct {
            /// Creates new id with given value.
            ///
            pub(crate) fn new(id: u64) -> Self {
                Self(id)
            }

            /// Returns underlying id.
            ///
            pub fn value(&self) -> u64 {
                self.0
            }
        }
    };
}

/// [`ComponentId`] id struct is needed to identify [`Component`](super::components::Component)s
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// [`GameObject`](super::gameobjects::GameObject) with
/// this [`Component`](super::components::Component) is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(u64);
impl_id!(ComponentId);

/// [`ResourceId`] id struct is needed to identify [`Resource`](super::components::Resource)s
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this [`Resource`](super::components::Component) is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(u64);
impl_id!(ResourceId);

/// [`GameObjectId`] id struct is needed to identify [`GameObject`](super::gameobjects::GameObject)s
/// in [`Scene`](super::scenes::Scene).
///
/// It is assigned by the [`Scene`](super::scenes::Scene) in which
/// this [`GameObject`](super::gameobjects::GameObject) is registered.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameObjectId(u64);
impl_id!(GameObjectId);
