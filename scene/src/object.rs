use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::PathBuf,
};

use nrg_graphics::{MaterialInstance, MeshInstance};
use nrg_math::Matrix4;
use nrg_resources::{
    DataResource, Deserializable, DynamicResource, Resource, ResourceBase, ResourceId,
    ResourceTrait, SerializableResource, SharedDataRw,
};
use nrg_serialize::generate_random_uid;

use crate::{ObjectData, Transform};

pub type ComponentId = ResourceId;
pub type ObjectId = ResourceId;
pub type ObjectRc = Resource;

pub struct Object {
    id: ResourceId,
    filepath: PathBuf,
    children: Vec<ObjectRc>,
    components: HashMap<TypeId, Resource>,
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
            children: Vec::new(),
        }
    }
}

impl DynamicResource for Object {}
impl SerializableResource for Object {}
impl DataResource for Object {
    type DataType = ObjectData;

    fn create_from_data(shared_data: &SharedDataRw, object_data: Self::DataType) -> ObjectRc {
        let object = {
            let mut data = shared_data.write().unwrap();
            let object = Object {
                filepath: object_data.path().to_path_buf(),
                ..Default::default()
            };
            data.add_resource(object)
        };

        let transform = object
            .get_mut::<Object>()
            .add_default_component::<Transform>(shared_data);
        transform
            .get_mut::<Transform>()
            .set_matrix(object_data.transform);

        if !object_data.material.clone().into_os_string().is_empty() {
            let material =
                MaterialInstance::create_from_file(shared_data, object_data.material.as_path());
            object
                .get_mut::<Object>()
                .add_component::<MaterialInstance>(material);
        }

        for child in object_data.children.iter() {
            let child = Object::create_from_file(shared_data, child.as_path());
            object.get_mut::<Object>().add_child(child);
        }

        object
    }
}

impl Object {
    pub fn add_child(&mut self, child: ObjectRc) {
        self.children.push(child);
    }

    pub fn children(&self) -> &Vec<ObjectRc> {
        &self.children
    }

    pub fn add_default_component<C>(&mut self, shared_data: &SharedDataRw) -> Resource
    where
        C: DynamicResource + Default,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        let resource = {
            let mut data = shared_data.write().unwrap();
            let component = C::default();
            data.add_resource(component)
        };
        self.components.insert(TypeId::of::<C>(), resource.clone());
        resource
    }
    pub fn add_component<C>(&mut self, component: Resource)
    where
        C: DynamicResource,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        self.components.insert(TypeId::of::<C>(), component);
    }

    pub fn get_component<C>(&self) -> Option<Resource>
    where
        C: ResourceTrait + 'static,
    {
        if let Some(component) = self.components.get(&TypeId::of::<C>()) {
            return Some(component.clone());
        }
        None
    }

    pub fn update_from_parent(&mut self, shared_data: &SharedDataRw, parent_transform: Matrix4) {
        if let Some(transform) = self.get_component::<Transform>() {
            let object_matrix = transform.get::<Transform>().matrix();
            let object_matrix = parent_transform * object_matrix;
            transform.get_mut::<Transform>().set_matrix(object_matrix);

            if let Some(material) = self.get_component::<MaterialInstance>() {
                for mesh in material.get::<MaterialInstance>().meshes() {
                    let matrix = object_matrix * *mesh.get::<MeshInstance>().transform();
                    mesh.get_mut::<MeshInstance>().set_transform(matrix);
                }
            }

            let children = self.children();
            for child in children {
                child
                    .get_mut::<Object>()
                    .update_from_parent(shared_data, object_matrix);
            }
        }
    }
}
