use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
};

use nrg_graphics::MaterialInstance;
use nrg_math::Matrix4;
use nrg_resources::{
    DataTypeResource, Deserializable, GenericRef, HandleCastFrom, HandleCastTo, ResourceData,
    ResourceId, ResourceRef, SerializableResource, SharedData, SharedDataRw,
};
use nrg_serialize::generate_random_uid;

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
            id: generate_random_uid(),
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
                filepath: object_data.path().to_path_buf(),
                ..Default::default()
            },
        );
        let transform = object
            .get_mut()
            .add_default_component::<Transform>(shared_data);
        transform.get_mut().set_matrix(object_data.transform);

        if !object_data.material.clone().into_os_string().is_empty() {
            let material =
                MaterialInstance::create_from_file(shared_data, object_data.material.as_path());
            object.get_mut().add_component::<MaterialInstance>(material);
        }

        for child in object_data.children.iter() {
            let child = Object::create_from_file(shared_data, child.as_path());
            object.get_mut().add_child(child);
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
        let mut resource = SharedData::add_resource(shared_data, component);
        self.components
            .insert(TypeId::of::<C>(), resource.as_generic().clone());
        resource
    }
    pub fn add_component<C>(&mut self, mut component: ResourceRef<C>)
    where
        C: ResourceData,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        self.components
            .insert(TypeId::of::<C>(), component.as_generic());
    }

    pub fn get_component<C>(&mut self) -> Option<ResourceRef<C>>
    where
        C: ResourceData,
    {
        if let Some(component) = self.components.get_mut(&TypeId::of::<C>()) {
            return Some(component.from_generic());
        }
        None
    }

    pub fn update_from_parent(&mut self, shared_data: &SharedDataRw, parent_transform: Matrix4) {
        if let Some(transform) = self.get_component::<Transform>() {
            let object_matrix = transform.get().matrix();
            let object_matrix = parent_transform * object_matrix;
            transform.get_mut().set_matrix(object_matrix);

            if let Some(material) = self.get_component::<MaterialInstance>() {
                for mesh in material.get().meshes() {
                    let matrix = object_matrix * *mesh.get().transform();
                    mesh.get_mut().set_transform(matrix);
                }
            }

            let children = self.children();
            for child in children {
                child
                    .get_mut()
                    .update_from_parent(shared_data, object_matrix);
            }
        }
    }
}
