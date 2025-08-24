//! Submodule that implement [`ResourceStorage`].
//!

use super::{NoOpHasherState, TypeIdMap};
use crate::gamecore::resources::{Resource, ResourceId};

/// [`ResourceStorage`] struct provides API for a storage of [`Resource`]s.
///
/// Conceptually, [`ResourcesStorage`] can be thought of as an `HashMap<ResourceId, R>`,
/// where each separate `R` represents resource of one type.
///
#[derive(Debug, Default)]
pub struct ResourceStorage {
    /// Map that stores resources.
    ///
    resources: TypeIdMap<ResourceId, Box<dyn Resource>>,
}
impl ResourceStorage {
    /// Initializes new [`ResourceStorage`].
    ///
    /// Created [`ResourceStorage`] will not allocate until first insertions.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// let storage: ResourceStorage = ResourceStorage::new();
    /// ```
    ///
    pub fn new() -> Self {
        ResourceStorage {
            resources: TypeIdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.resources.clear();
    }
}
// resources
impl ResourceStorage {
    /// Inserts a new resource with the given value.
    ///
    /// Resources are unique data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data and this function will return old value.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    ///
    /// storage.insert(Score(0));
    /// ```
    ///
    pub fn insert<R: Resource>(&mut self, resource: R) -> Option<R> {
        self.resources
            .insert(ResourceId::of::<R>(), Box::new(resource))
            .map(|boxed| {
                *(boxed
                    .downcast::<R>()
                    .expect("`Resource` is of correct type"))
            })
    }

    /// Removes the resource of a given type and returns it if present.
    /// Otherwise, returns `None`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::Resource;
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    ///
    /// storage.insert(Score(0));
    /// assert_eq!(storage.remove::<Score>().expect("`Score` was inserted").0, 0);
    /// ```
    ///
    pub fn remove<R: Resource>(&mut self) -> Option<R> {
        self.resources.remove(&ResourceId::of::<R>()).map(|boxed| {
            *(boxed
                .downcast::<R>()
                .expect("`Resource` is of correct type"))
        })
    }

    /// Returns whether a resource of given type is present or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains::<Score>());
    ///
    /// storage.insert(Score(0));
    /// assert!(storage.contains::<Score>());
    ///
    /// let _ = storage.remove::<Score>();
    /// assert!(!storage.contains::<Score>());
    /// ```
    ///
    pub fn contains<R: Resource>(&mut self) -> bool {
        self.resources.contains_key(&ResourceId::of::<R>())
    }

    /// Gets a reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(storage.resource::<Score>().is_none());
    ///
    /// storage.insert(Score(0));
    /// assert_eq!(storage.resource::<Score>().expect("`Score` was inserted").0, 0);
    /// ```
    ///
    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get(&ResourceId::of::<R>()).map(|boxed| {
            boxed
                .downcast_ref::<R>()
                .expect("`Resource` is of correct type")
        })
    }
    /// Gets a mutable reference to the resource of the given type if present.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(storage.resource_mut::<Score>().is_none());
    ///
    /// storage.insert(Score(0));
    /// let score = storage.resource_mut::<Score>().expect("`Score` was isnerted");
    /// score.0 = 15;
    /// assert_eq!(storage.resource::<Score>().expect("`Score` was inserted").0, 15);
    /// ```
    ///
    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources.get_mut(&ResourceId::of::<R>()).map(|boxed| {
            boxed
                .downcast_mut::<R>()
                .expect("`Resource` is of correct type")
        })
    }
}
