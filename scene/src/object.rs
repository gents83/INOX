use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::PathBuf,
};

use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

pub type ComponentId = ResourceId;
pub type ObjectId = ResourceId;

pub struct Object {
    id: ResourceId,
    filepath: PathBuf,
    components: HashMap<TypeId, ComponentId>,
}

impl ResourceTrait for Object {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        self.filepath.clone()
    }
}

impl Default for Object {
    fn default() -> Self {
        Self {
            id: generate_random_uid(),
            filepath: PathBuf::default(),
            components: HashMap::new(),
        }
    }
}

impl Object {
    pub fn create(shared_data: &SharedDataRw) -> ObjectId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(Object::default())
    }
    pub fn add_component<C>(shared_data: &SharedDataRw, object_id: ObjectId) -> ComponentId
    where
        C: ResourceTrait + Default + 'static,
    {
        let object = SharedData::get_resource::<Self>(shared_data, object_id);
        let object = &mut object.get_mut();
        debug_assert!(
            !object.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        let resource_id = {
            let mut data = shared_data.write().unwrap();
            let component = C::default();
            let resource_id = component.id();
            data.add_resource(component);
            resource_id
        };
        object.components.insert(TypeId::of::<C>(), resource_id);
        resource_id
    }
}
