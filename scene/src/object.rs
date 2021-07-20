use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
};

use nrg_graphics::{MaterialInstance, MeshInstance};
use nrg_math::Matrix4;
use nrg_resources::{
    DataTypeResource, Deserializable, GenericRef, HandleCastTo, ResourceData, ResourceId,
    ResourceRef, SerializableResource, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, generate_uid_from_string, INVALID_UID};

use crate::{ObjectData, Transform};

pub type ComponentId = ResourceId;
pub type ObjectId = ResourceId;
pub type ObjectRc = ResourceRef<Object>;

pub struct Object {
    id: ResourceId,
    filepath: PathBuf,
    children: Vec<ObjectRc>,
    components: HashMap<TypeId, GenericRef>,
}

impl ResourceData for Object {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl Default for Object {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            filepath: PathBuf::default(),
            components: HashMap::new(),
            children: Vec::new(),
        }
    }
}

impl SerializableResource for Object {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }
}
impl DataTypeResource for Object {
    type DataType = ObjectData;

    fn create_from_data(shared_data: &SharedDataRw, object_data: Self::DataType) -> ObjectRc {
        let object = SharedData::add_resource(
            shared_data,
            Object {
                id: generate_uid_from_string(object_data.path().to_str().unwrap()),
                filepath: object_data.path().to_path_buf(),
                ..Default::default()
            },
        );
        let transform = object
            .resource()
            .get_mut()
            .add_default_component::<Transform>(shared_data);
        transform
            .resource()
            .get_mut()
            .set_matrix(object_data.transform);

        if !object_data.material.clone().into_os_string().is_empty() {
            let material = if let Some(material) =
                MaterialInstance::find_from_path(shared_data, object_data.material.as_path())
            {
                material
            } else {
                MaterialInstance::create_from_file(shared_data, object_data.material.as_path())
            };
            object
                .resource()
                .get_mut()
                .add_component::<MaterialInstance>(material);
        }

        for child in object_data.children.iter() {
            let child = Object::create_from_file(shared_data, child.as_path());
            object.resource().get_mut().add_child(child);
        }

        object
    }
}

impl Object {
    pub fn generate_empty(shared_data: &SharedDataRw) -> ObjectRc {
        SharedData::add_resource::<Object>(
            shared_data,
            Object {
                id: generate_random_uid(),
                ..Default::default()
            },
        )
    }

    pub fn add_child(&mut self, child: ObjectRc) {
        self.children.push(child);
    }

    pub fn children(&self) -> &Vec<ObjectRc> {
        &self.children
    }

    pub fn add_default_component<C>(&mut self, shared_data: &SharedDataRw) -> ResourceRef<C>
    where
        C: ResourceData + Default,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        let component = C::default();
        let resource = SharedData::add_resource(shared_data, component);
        self.components.insert(TypeId::of::<C>(), resource.clone());
        resource
    }
    pub fn add_component<C>(&mut self, component: ResourceRef<C>) -> &mut Self
    where
        C: ResourceData,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        self.components
            .insert(TypeId::of::<C>(), component as GenericRef);
        self
    }

    pub fn get_component<C>(&mut self) -> Option<ResourceRef<C>>
    where
        C: ResourceData,
    {
        if let Some(component) = self.components.get_mut(&TypeId::of::<C>()) {
            return Some(component.clone().of_type::<C>());
        }
        None
    }

    pub fn update_from_parent(&mut self, shared_data: &SharedDataRw, parent_transform: Matrix4) {
        if let Some(transform) = self.get_component::<Transform>() {
            let object_matrix = transform.resource().get().matrix();
            let object_matrix = parent_transform * object_matrix;

            if let Some(mesh) = self.get_component::<MeshInstance>() {
                mesh.resource().get_mut().set_transform(object_matrix);
            }

            let children = self.children();
            for child in children {
                child
                    .resource()
                    .get_mut()
                    .update_from_parent(shared_data, object_matrix);
            }
        }
    }
}
