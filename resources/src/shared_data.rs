use std::{
    any::TypeId,
    collections::HashMap,
    path::Path,
    sync::{Arc, RwLock},
};

use nrg_serialize::INVALID_UID;

use crate::{from_file, Data, Deserializable, Resource, ResourceId, ResourceTrait};

pub trait DynamicResource: ResourceTrait + 'static {}
pub trait DataResource: DynamicResource {
    type DataType;
    fn create_from_data(shared_data: &SharedDataRw, data: Self::DataType) -> Resource
    where
        Self: Sized;
}

pub trait SerializableResource: DataResource {
    fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> Resource
    where
        Self: Sized,
        Self::DataType: Deserializable,
    {
        let data = from_file::<Self::DataType>(filepath);
        Self::create_from_data(shared_data, data)
    }
}

pub trait FileResource: DynamicResource {
    fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> Resource
    where
        Self: Sized;
}

pub struct ResourceStorage {
    resources: Vec<Resource>,
}
unsafe impl Send for ResourceStorage {}
unsafe impl Sync for ResourceStorage {}

impl Default for ResourceStorage {
    #[inline]
    fn default() -> Self {
        Self {
            resources: Vec::new(),
        }
    }
}

impl ResourceStorage {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
    #[inline]
    pub fn add_resource<T: 'static + ResourceTrait>(&mut self, data: T) -> Resource {
        let resource = Arc::new(RwLock::new(data));
        self.resources.push(resource.clone());
        resource
    }
    pub fn flush(&mut self) {
        let mut to_remove = Vec::new();
        for r in self.resources.iter() {
            if Arc::strong_count(r) <= 1 {
                to_remove.push(r.read().unwrap().id());
            }
        }
        for id in to_remove {
            self.remove_resource(id);
        }
    }
    #[inline]
    fn remove_resource(&mut self, resource_id: ResourceId) {
        self.resources
            .retain(|resource| resource.read().unwrap().id() != resource_id);
    }
    #[inline]
    pub fn remove_resources(&mut self) {
        self.resources.clear();
    }
    #[inline]
    pub fn has_resource(&self, resource_id: ResourceId) -> bool {
        if let Some(_index) = self
            .resources
            .iter()
            .position(|x| x.read().unwrap().id() == resource_id)
        {
            return true;
        }
        false
    }
    #[inline]
    pub fn match_resource<T, F>(&self, f: F) -> ResourceId
    where
        T: 'static + ResourceTrait + Sized,
        F: Fn(&T) -> bool,
    {
        if let Some(item) = self.resources.iter().find(|&e| {
            let resource = &*e.read().unwrap() as *const dyn ResourceTrait as *const T;
            let resource = unsafe { &*resource };
            f(resource)
        }) {
            return item.read().unwrap().id();
        }
        INVALID_UID
    }
    #[inline]
    pub fn get_resource(&self, resource_id: ResourceId) -> Resource {
        if let Some(item) = self
            .resources
            .iter()
            .find(|&x| x.read().unwrap().id() == resource_id)
        {
            item.clone()
        } else {
            panic!(
                "Unable to find requested resource {}",
                resource_id.to_simple().to_string().as_str(),
            )
        }
    }
    #[inline]
    pub fn get_resources_of_type(&self) -> Vec<Resource> {
        let mut vec = Vec::new();
        self.resources.iter().for_each(|e| {
            vec.push(e.clone());
        });
        vec
    }
}

pub struct SharedData {
    resources: HashMap<TypeId, ResourceStorage>,
}
unsafe impl Send for SharedData {}
unsafe impl Sync for SharedData {}

impl Data for SharedData {}

impl Default for SharedData {
    #[inline]
    fn default() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

impl SharedData {
    #[inline]
    fn clear(&mut self) {
        for (&_t, rs) in self.resources.iter_mut() {
            rs.remove_resources();
        }
        self.resources.clear();
    }
    #[inline]
    pub fn add_resource<T: DynamicResource>(&mut self, data: T) -> Resource {
        let rs = self
            .resources
            .entry(TypeId::of::<T>())
            .or_insert_with(ResourceStorage::default);
        rs.add_resource(data)
    }
    #[inline]
    pub fn flush_resources(&mut self) {
        for (_, rs) in self.resources.iter_mut() {
            rs.flush();
        }
    }
    #[inline]
    pub fn has_resource<T: 'static>(shared_data: &SharedDataRw, resource_id: ResourceId) -> bool {
        let data = shared_data.read().unwrap();
        if let Some(rs) = data.resources.get(&TypeId::of::<T>()) {
            return rs.has_resource(resource_id);
        }
        false
    }
    #[inline]
    pub fn has_resources_of_type<T: 'static>(shared_data: &SharedDataRw) -> bool {
        let data = shared_data.read().unwrap();
        data.resources.contains_key(&TypeId::of::<T>())
    }
    #[inline]
    pub fn match_resource<T, F>(shared_data: &SharedDataRw, f: F) -> ResourceId
    where
        T: DynamicResource + Sized,
        F: Fn(&T) -> bool,
    {
        let data = shared_data.read().unwrap();
        if let Some(rs) = data.resources.get(&TypeId::of::<T>()) {
            return rs.match_resource(f);
        }
        INVALID_UID
    }
    #[inline]
    pub fn get_resource<T: DynamicResource>(
        shared_data: &SharedDataRw,
        resource_id: ResourceId,
    ) -> Resource {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        rs.get_resource(resource_id)
    }
    #[inline]
    pub fn get_resources_of_type<T: DynamicResource>(shared_data: &SharedDataRw) -> Vec<Resource> {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        rs.get_resources_of_type()
    }
    #[inline]
    pub fn get_num_resources_of_type<T: DynamicResource>(shared_data: &SharedDataRw) -> usize {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        rs.get_resources_of_type().len()
    }
    #[inline]
    pub fn remove_resource_of_type(
        shared_data: &SharedDataRw,
        type_id: TypeId,
        resource_id: ResourceId,
    ) {
        let mut data = shared_data.write().unwrap();
        if let Some(vec) = data.resources.get_mut(&type_id) {
            vec.remove_resource(resource_id)
        }
    }
    #[inline]
    pub fn get_resourceid_at_index<T: DynamicResource>(
        shared_data: &SharedDataRw,
        index: usize,
    ) -> ResourceId {
        let data = shared_data.read().unwrap();
        let rs = data.resources.get(&TypeId::of::<T>()).unwrap();
        let vec = rs.get_resources_of_type();
        debug_assert!(index < vec.len());
        let resource_id = vec[index].read().unwrap().id();
        resource_id
    }
}

impl Drop for SharedData {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

pub type SharedDataRw = Arc<RwLock<SharedData>>;
