use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
};

use nrg_graphics::MaterialInstance;
use nrg_resources::{from_file, ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::generate_random_uid;

use crate::{ObjectData, Transform};

pub type ComponentId = ResourceId;
pub type ObjectId = ResourceId;

pub struct Object {
    id: ResourceId,
    filepath: PathBuf,
    chilrden: Vec<ObjectId>,
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
            chilrden: Vec::new(),
        }
    }
}

impl Object {
    pub fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> ObjectId {
        let object_data = from_file::<ObjectData>(filepath);

        let object_id = {
            let mut data = shared_data.write().unwrap();
            let object = Object {
                filepath: filepath.to_path_buf(),
                ..Default::default()
            };
            data.add_resource(object)
        };

        let transform_id = Object::add_component::<Transform>(&shared_data, object_id);
        Transform::set(shared_data, transform_id, object_data.transform);

        if !object_data.material.clone().into_os_string().is_empty() {
            let material_id =
                MaterialInstance::create_from_file(shared_data, object_data.material.as_path());
            Object::add_component_with_id::<MaterialInstance>(&shared_data, object_id, material_id);
        }

        for child in object_data.children.iter() {
            let child_id = Object::create_from_file(shared_data, child.as_path());
            Object::add_child(shared_data, object_id, child_id);
        }

        object_id
    }
    pub fn create(shared_data: &SharedDataRw) -> ObjectId {
        let mut data = shared_data.write().unwrap();
        data.add_resource(Object::default())
    }

    pub fn add_child(shared_data: &SharedDataRw, parent_id: ObjectId, child_id: ObjectId) {
        let object = SharedData::get_resource::<Self>(shared_data, parent_id);
        let object = &mut object.get_mut();
        object.chilrden.push(child_id);
    }

    pub fn get_children(shared_data: &SharedDataRw, object_id: ObjectId) -> Vec<ObjectId> {
        let object = SharedData::get_resource::<Self>(shared_data, object_id);
        let object = &mut object.get_mut();
        object.chilrden.clone()
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
    pub fn add_component_with_id<C>(
        shared_data: &SharedDataRw,
        object_id: ObjectId,
        component_id: ComponentId,
    ) where
        C: ResourceTrait + 'static,
    {
        let object = SharedData::get_resource::<Self>(shared_data, object_id);
        let object = &mut object.get_mut();
        debug_assert!(
            !object.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        object.components.insert(TypeId::of::<C>(), component_id);
    }

    pub fn get_component_with_id<C>(
        shared_data: &SharedDataRw,
        object_id: ObjectId,
    ) -> Option<ComponentId>
    where
        C: ResourceTrait + 'static,
    {
        let object = SharedData::get_resource::<Self>(shared_data, object_id);
        let object = object.get();
        if let Some(uid) = object.components.get(&TypeId::of::<C>()) {
            return Some(*uid);
        }
        None
    }
}
