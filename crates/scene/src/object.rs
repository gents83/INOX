use std::{
    any::TypeId,
    collections::HashMap,
    path::{Path, PathBuf},
};

use inox_graphics::{Light, Mesh, OnLightCreateData, OnMeshCreateData};
use inox_math::{Mat4Ops, MatBase, Matrix4, Vector3};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, GenericResource, Handle, Resource, ResourceCastTo, ResourceId, ResourceTrait,
    SerializableResource, SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};
use inox_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};
use inox_uid::generate_random_uid;

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
    components: HashMap<TypeId, Vec<GenericResource>>,
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
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                CollapsingHeader::new(format!("Components [{}]", self.components.len()))
                    .default_open(!collapsed)
                    .show(ui, |ui| {
                        for (typeid, components) in self.components.iter() {
                            components.iter().for_each(|c| {
                                ui_registry.show(*typeid, c, ui);
                            });
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

    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.filepath = path.to_path_buf();
        self
    }

    fn extension() -> &'static str {
        ObjectData::extension()
    }
}

impl ResourceTrait for Object {
    type OnCreateData = OnObjectCreateData;

    fn on_create(
        &mut self,
        shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ObjectId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            let parent = shared_data_rc.get_resource::<Object>(&on_create_data.parent_id);
            self.set_parent(parent);
        }
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &ObjectId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for Object {
    type DataType = ObjectData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            filepath: PathBuf::new(),
            transform: Matrix4::default_identity(),
            parent: None,
            is_transform_dirty: true,
            children: Vec::new(),
            components: HashMap::new(),
        }
    }

    fn is_initialized(&self) -> bool {
        !self.components.is_empty()
    }
    fn invalidate(&mut self) -> &mut Self {
        self.components.clear();
        self.children.clear();
        self
    }
    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: ObjectId,
        object_data: Self::DataType,
    ) -> Self {
        let mut object = Self::new(id, shared_data, message_hub);
        object.transform = object_data.transform;

        object_data.components.iter().for_each(|component_path| {
            let path = component_path.as_path();
            if <Mesh as SerializableResource>::is_matching_extension(path) {
                let mesh = Mesh::request_load(
                    shared_data,
                    message_hub,
                    path,
                    Some(OnMeshCreateData {
                        parent_matrix: object_data.transform,
                    }),
                );
                object.add_component::<Mesh>(mesh);
            } else if <Camera as SerializableResource>::is_matching_extension(path) {
                let camera = Camera::request_load(
                    shared_data,
                    message_hub,
                    path,
                    Some(OnCameraCreateData { parent_id: id }),
                );
                object.add_component::<Camera>(camera);
            } else if <Light as SerializableResource>::is_matching_extension(path) {
                let light = Light::request_load(
                    shared_data,
                    message_hub,
                    path,
                    Some(OnLightCreateData {
                        position: object.position(),
                    }),
                );
                object.add_component::<Light>(light);
            } else if <Script as SerializableResource>::is_matching_extension(path) {
                let script = Script::request_load(
                    shared_data,
                    message_hub,
                    path,
                    Some(OnScriptCreateData { parent_id: id }),
                );
                object.add_component::<Script>(script);
            }
        });

        for child in object_data.children.iter() {
            let child = Object::request_load(
                shared_data,
                message_hub,
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
    pub fn position(&self) -> Vector3 {
        self.transform.translation()
    }
    #[inline]
    pub fn rotation(&self) -> Vector3 {
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

    pub fn components(&self) -> &HashMap<TypeId, Vec<GenericResource>> {
        &self.components
    }

    pub fn add_default_component<C>(
        &mut self,
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
    ) -> Resource<C>
    where
        C: DataTypeResource + 'static,
    {
        let id = generate_random_uid();
        let resource =
            shared_data.add_resource(message_hub, id, C::new(id, shared_data, message_hub));
        let components = self.components.entry(TypeId::of::<C>()).or_default();
        components.push(resource.clone());
        resource
    }
    pub fn add_component<C>(&mut self, component: Resource<C>) -> &mut Self
    where
        C: ResourceTrait + 'static,
    {
        let components = self.components.entry(TypeId::of::<C>()).or_default();
        components.push(component as GenericResource);
        self
    }

    pub fn components_of_type<C>(&self) -> Vec<Resource<C>>
    where
        C: ResourceTrait + 'static,
    {
        let mut result = Vec::new();
        if let Some(components) = self.components.get(&TypeId::of::<C>()) {
            components.iter().for_each(|c| {
                result.push(c.of_type::<C>());
            });
        }
        result
    }

    pub fn update_transform(&mut self, parent_transform: Option<Matrix4>) {
        if self.is_dirty() {
            self.is_transform_dirty = false;
            if let Some(parent_transform) = parent_transform {
                self.transform = parent_transform * self.transform;
            }
        }
        self.components_of_type::<Mesh>().iter().for_each(|mesh| {
            mesh.get_mut().set_matrix(self.transform);
        });
        self.components_of_type::<Light>().iter().for_each(|light| {
            light.get_mut().set_position(self.position());
        });
    }
}
