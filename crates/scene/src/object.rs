use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    path::{Path, PathBuf},
};

use sabi_graphics::{Light, Mesh, OnLightCreateData};
use sabi_math::{Mat4Ops, MatBase, Matrix4, Vector3};
use sabi_messenger::MessengerRw;
use sabi_resources::{
    DataTypeResource, GenericResource, Handle, Resource, ResourceCastTo, ResourceId, ResourceTrait,
    SerializableResource, SharedData, SharedDataRc,
};
use sabi_serialize::{generate_random_uid, read_from_file, SerializeFile};
use sabi_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{Camera, ObjectData, OnCameraCreateData, OnScriptCreateData, Script};

pub type ComponentId = ResourceId;
pub type ObjectId = ResourceId;

#[derive(Clone)]
pub struct OnObjectCreateData {
    pub parent_id: ObjectId,
}

#[derive(Clone)]
pub struct Object {
    filepath: PathBuf,
    transform: Matrix4,
    parent: Handle<Object>,
    is_transform_dirty: bool,
    children: Vec<Resource<Object>>,
    components: HashMap<TypeId, GenericResource>,
}

impl Default for Object {
    fn default() -> Self {
        Self {
            filepath: PathBuf::new(),
            transform: Matrix4::default_identity(),
            parent: None,
            is_transform_dirty: true,
            children: Vec::new(),
            components: HashMap::new(),
        }
    }
}

impl UIProperties for Object {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        let mut object_name = format!("Object [{:?}]", id.as_simple().to_string());
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
                            c.get_mut().show(id, ui_registry, ui, collapsed);
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

    fn extension() -> &'static str {
        ObjectData::extension()
    }
}
impl DataTypeResource for Object {
    type DataType = ObjectData;
    type OnCreateData = OnObjectCreateData;

    fn on_create(
        &mut self,
        shared_data_rc: &SharedDataRc,
        _id: &ObjectId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            let parent = shared_data_rc.get_resource::<Object>(&on_create_data.parent_id);
            self.set_parent(parent);
        }
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _id: &ObjectId) {}

    fn is_initialized(&self) -> bool {
        !self.components.is_empty()
    }
    fn invalidate(&mut self) -> &mut Self {
        self.components.clear();
        self.children.clear();
        self
    }
    fn deserialize_data(path: &Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        id: ObjectId,
        object_data: Self::DataType,
    ) -> Self {
        let mut object = Self {
            transform: object_data.transform,
            ..Default::default()
        };

        object_data.components.iter().for_each(|component_path| {
            let path = component_path.as_path();
            if Mesh::is_matching_extension(path) {
                let mesh = Mesh::request_load(shared_data, global_messenger, path, None);
                object.add_component::<Mesh>(mesh);
            } else if Camera::is_matching_extension(path) {
                let camera = Camera::request_load(
                    shared_data,
                    global_messenger,
                    path,
                    Some(OnCameraCreateData { parent_id: id }),
                );
                object.add_component::<Camera>(camera);
            } else if Light::is_matching_extension(path) {
                let light = Light::request_load(
                    shared_data,
                    global_messenger,
                    path,
                    Some(OnLightCreateData {
                        position: object.get_position(),
                    }),
                );
                object.add_component::<Light>(light);
            } else if Script::is_matching_extension(path) {
                let script = Script::request_load(
                    shared_data,
                    global_messenger,
                    path,
                    Some(OnScriptCreateData { parent_id: id }),
                );
                object.add_component::<Script>(script);
            }
        });

        for child in object_data.children.iter() {
            let child = Object::request_load(
                shared_data,
                global_messenger,
                child.as_path(),
                Some(OnObjectCreateData { parent_id: id }),
            );
            object.add_child(child);
        }

        object
    }
}

impl Object {
    #[inline]
    pub fn set_transform(&mut self, transform: Matrix4) -> &mut Self {
        self.transform = transform;
        self.set_dirty();
        self
    }
    #[inline]
    pub fn transform(&self) -> Matrix4 {
        self.transform
    }
    #[inline]
    pub fn set_position(&mut self, position: Vector3) -> &mut Self {
        self.transform.set_translation(position);
        self.set_dirty();
        self
    }
    #[inline]
    pub fn translate(&mut self, translation: Vector3) -> &mut Self {
        self.transform.add_translation(translation);
        self.set_dirty();
        self
    }
    #[inline]
    pub fn rotate(&mut self, roll_yaw_pitch: Vector3) -> &mut Self {
        self.transform.add_rotation(roll_yaw_pitch);
        self.set_dirty();
        self
    }
    #[inline]
    pub fn scale(&mut self, scale: Vector3) -> &mut Self {
        self.transform.add_scale(scale);
        self.set_dirty();
        self
    }
    #[inline]
    pub fn look_at(&mut self, position: Vector3) -> &mut Self {
        self.transform.look_at(position);
        self.set_dirty();
        self
    }
    #[inline]
    pub fn look_towards(&mut self, direction: Vector3) -> &mut Self {
        self.transform.look_towards(direction);
        self.set_dirty();
        self
    }

    pub fn is_dirty(&self) -> bool {
        self.is_transform_dirty
    }

    fn set_dirty(&mut self) {
        self.is_transform_dirty = true;
        self.children.iter().for_each(|c| {
            c.get_mut().set_dirty();
        });
    }

    #[inline]
    pub fn get_position(&self) -> Vector3 {
        self.transform.translation()
    }
    #[inline]
    pub fn get_rotation(&self) -> Vector3 {
        self.transform.rotation()
    }
    #[inline]
    pub fn get_scale(&self) -> Vector3 {
        self.transform.scale()
    }

    #[inline]
    pub fn parent(&self) -> Handle<Object> {
        self.parent.clone()
    }

    #[inline]
    fn set_parent(&mut self, parent: Handle<Object>) {
        self.parent = parent;
        self.set_dirty();
    }

    #[inline]
    pub fn add_child(&mut self, child: Resource<Object>) {
        self.children.push(child);
    }

    #[inline]
    pub fn remove_child(&mut self, child: &Resource<Object>) {
        if let Some(index) = self.children.iter().position(|c| c.id() == child.id()) {
            self.children.remove(index);
        }
    }

    #[inline]
    pub fn is_child(&self, object_id: &ObjectId) -> bool {
        for c in self.children.iter() {
            if c.id() == object_id {
                return true;
            }
        }
        false
    }
    #[inline]
    pub fn is_child_recursive(&self, object_id: &ObjectId) -> bool {
        for c in self.children.iter() {
            if c.id() == object_id || c.get().is_child_recursive(object_id) {
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

    pub fn component<C>(&self) -> Handle<C>
    where
        C: ResourceTrait,
    {
        if let Some(component) = self.components.get(&TypeId::of::<C>()) {
            return Some(component.of_type::<C>());
        }
        None
    }

    pub fn update_transform(&mut self, parent_transform: Option<Matrix4>) {
        if self.is_dirty() {
            self.is_transform_dirty = false;
            if let Some(parent_transform) = parent_transform {
                self.transform = parent_transform * self.transform;
            }
        }
        if let Some(mesh) = self.component::<Mesh>() {
            mesh.get_mut().set_matrix(self.transform);
        }
    }
}
