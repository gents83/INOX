use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
};

use nrg_graphics::Mesh;
use nrg_math::Matrix4;
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, GenericResource, Handle, Resource, ResourceCastTo, ResourceId, ResourceTrait,
    SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::{generate_random_uid, read_from_file};
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{ObjectData, Transform};

pub type ComponentId = ResourceId;
pub type ObjectId = ResourceId;

#[derive(Default, Clone)]
pub struct Object {
    filepath: PathBuf,
    children: Vec<Resource<Object>>,
    components: HashMap<TypeId, GenericResource>,
}

impl UIProperties for Object {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        let mut object_name = format!("Object [{:?}]", id.to_simple().to_string());
        if let Some(name) = self.path().file_stem() {
            if let Some(name) = name.to_str() {
                object_name = name.to_string();
            }
        }
        CollapsingHeader::new(object_name.as_str())
            .selected(true)
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                CollapsingHeader::new(format!("Components [{}]", self.components.len()))
                    .default_open(!collapsed)
                    .show(ui, |ui| {
                        for (typeid, c) in self.components.iter() {
                            ui_registry.show(*typeid, c, ui);
                        }
                    });
                CollapsingHeader::new(format!("Children [{}]", self.children.len()))
                    .default_open(false)
                    .show(ui, |ui| {
                        for c in self.children.iter() {
                            let id = c.id();
                            c.get_mut(|c| {
                                c.show(id, ui_registry, ui, collapsed);
                            })
                        }
                    });
            });
    }
}

impl SerializableResource for Object {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    fn is_matching_extension(path: &Path) -> bool {
        const OBJECT_EXTENSION: &str = "object_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == OBJECT_EXTENSION;
        }
        false
    }
}
impl DataTypeResource for Object {
    type DataType = ObjectData;

    fn is_initialized(&self) -> bool {
        !self.components.is_empty()
    }

    fn invalidate(&mut self) {
        self.components.clear();
        self.children.clear();
    }

    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: ObjectId,
        object_data: Self::DataType,
    ) -> Resource<Self> {
        let mut object = Self::default();
        let transform = object.add_default_component::<Transform>(shared_data);
        transform.get_mut(|t| {
            t.set_matrix(object_data.transform);
        });

        if !object_data.mesh.to_str().unwrap_or_default().is_empty() {
            let mesh =
                Mesh::load_from_file(shared_data, global_messenger, object_data.mesh.as_path());
            object.add_component::<Mesh>(mesh);
        }

        for child in object_data.children.iter() {
            let child = Object::load_from_file(shared_data, global_messenger, child.as_path());
            object.add_child(child);
        }

        SharedData::add_resource(shared_data, id, object)
    }
}

impl Object {
    pub fn generate_empty(shared_data: &SharedDataRc) -> Resource<Self> {
        SharedData::add_resource::<Object>(
            shared_data,
            generate_random_uid(),
            Object {
                ..Default::default()
            },
        )
    }

    pub fn add_child(&mut self, child: Resource<Object>) {
        self.children.push(child);
    }

    pub fn is_child(&self, object_id: &ObjectId) -> bool {
        for c in self.children.iter() {
            if c.id() == object_id {
                return true;
            }
        }
        false
    }

    pub fn is_child_recursive(&self, object_id: &ObjectId) -> bool {
        for c in self.children.iter() {
            if c.id() == object_id || c.get(|o| o.is_child_recursive(object_id)) {
                return true;
            }
        }
        false
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn children(&self) -> &Vec<Resource<Object>> {
        &self.children
    }

    pub fn components(&self) -> &HashMap<TypeId, GenericResource> {
        &self.components
    }

    pub fn add_default_component<C>(&mut self, shared_data: &SharedDataRc) -> Resource<C>
    where
        C: ResourceTrait + Default,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        let resource = SharedData::add_resource(shared_data, generate_random_uid(), C::default());
        self.components.insert(TypeId::of::<C>(), resource.clone());
        resource
    }
    pub fn add_component<C>(&mut self, component: Resource<C>) -> &mut Self
    where
        C: ResourceTrait,
    {
        debug_assert!(
            !self.components.contains_key(&TypeId::of::<C>()),
            "Object already contains a component of type {:?}",
            type_name::<C>()
        );
        self.components
            .insert(TypeId::of::<C>(), component as GenericResource);
        self
    }

    pub fn get_component<C>(&self) -> Handle<C>
    where
        C: ResourceTrait,
    {
        if let Some(component) = self.components.get(&TypeId::of::<C>()) {
            return Some(component.of_type::<C>());
        }
        None
    }

    pub fn update_from_parent<F>(
        &mut self,
        shared_data: &SharedDataRc,
        parent_transform: Matrix4,
        f: F,
    ) where
        F: Fn(&mut Self, Matrix4) + Copy,
    {
        if let Some(transform) = self.get_component::<Transform>() {
            let object_matrix = transform.get(|t| t.matrix());
            let object_matrix = parent_transform * object_matrix;

            f(self, object_matrix);

            let children = self.children();
            for child in children {
                child.get_mut(|o| {
                    o.update_from_parent(shared_data, object_matrix, f);
                });
            }
        }
    }
}
