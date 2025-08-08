//! Submodule that implement [`ResourceStorage`].
//!

use super::{IdMap, NoOpHasherState};
use crate::gamecore::resources::{BoxedResource, ResourceId};

/// [`ResourceStorage`] struct provides API
/// for a storage of [`Resource`](super::resources::Resource)s.
///
#[derive(Debug, Default)]
pub struct ResourceStorage {
    /// Map that stores resources.
    ///
    resources: IdMap<ResourceId, BoxedResource>,
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
            resources: IdMap::with_hasher(NoOpHasherState),
        }
    }

    /// Clears storage, removing all data. Keeps the allocated memory.
    ///
    pub fn clear(&mut self) {
        self.resources.clear();
    }

    /// Inserts a new resource with the given value.
    ///
    /// Resources are unique data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data and this function will return old value.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// ```
    ///
    pub fn insert_resource(
        &mut self,
        resource_id: ResourceId,
        boxed_resource: BoxedResource,
    ) -> Option<BoxedResource> {
        self.resources.insert(resource_id, boxed_resource)
    }
    /// Removes the resource of a given type and returns it if present.
    /// Otherwise, returns `None`.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.remove_resource(ResourceId::of::<Score>()).is_some());
    /// ```
    ///
    pub fn remove_resource(&mut self, resource_id: ResourceId) -> Option<BoxedResource> {
        self.resources.remove(&resource_id)
    }
    /// Returns whether a resource of given type exists or not.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains_resource(ResourceId::of::<Score>()));
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.contains_resource(ResourceId::of::<Score>()));
    ///
    /// let _ = storage.remove_resource(ResourceId::of::<Score>());
    /// assert!(!storage.contains_resource(ResourceId::of::<Score>()));
    /// ```
    ///
    pub fn contains_resource(&mut self, resource_id: ResourceId) -> bool {
        self.resources.contains_key(&resource_id)
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
    /// assert!(storage.resource(ResourceId::of::<Score>()).is_none());
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.resource(ResourceId::of::<Score>()).is_some());
    /// ```
    ///
    pub fn resource(&self, resource_id: ResourceId) -> Option<&BoxedResource> {
        self.resources.get(&resource_id)
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
    /// assert!(storage.resource_mut(ResourceId::of::<Score>()).is_none());
    ///
    /// storage.insert_resource(ResourceId::of::<Score>(), Box::new(Score(0)));
    /// assert!(storage.resource_mut(ResourceId::of::<Score>()).is_some());
    /// ```
    ///
    pub fn resource_mut(&mut self, resource_id: ResourceId) -> Option<&mut BoxedResource> {
        self.resources.get_mut(&resource_id)
    }
    /// Gets a mutable reference to the resource of given type if present,
    /// otherwise inserts the resource that is constructed by given closure and
    /// returns mutable reference to it.
    ///
    /// # Example
    /// ```rust
    /// # use ggengine::gamecore::storages::ResourceStorage;
    /// # use ggengine::gamecore::resources::{Resource, ResourceId};
    /// struct Score(u32);
    /// impl Resource for Score {}
    ///
    /// let mut storage: ResourceStorage = ResourceStorage::new();
    /// assert!(!storage.contains_resource(ResourceId::of::<Score>()));
    ///
    /// let _ = storage.resource_get_or_insert_with(ResourceId::of::<Score>(), || Box::new(Score(10)));
    /// assert!(storage.contains_resource(ResourceId::of::<Score>()));
    /// ```
    pub fn resource_get_or_insert_with(
        &mut self,
        resource_id: ResourceId,
        f: impl FnOnce() -> BoxedResource,
    ) -> &mut BoxedResource {
        self.resources.entry(resource_id).or_insert_with(|| f())
    }
}
