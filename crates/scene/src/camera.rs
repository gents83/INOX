use std::path::{Path, PathBuf};

use inox_graphics::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use inox_math::{convert_in_3d, Degrees, Mat4Ops, MatBase, Matrix4, NewAngle, Vector2, Vector3};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedData, SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};
use inox_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{CameraData, Object, ObjectId};

pub const DEFAULT_CAMERA_FOV: f32 = 45.;
pub const DEFAULT_CAMERA_ASPECT_RATIO: f32 = DEFAULT_WIDTH as f32 / DEFAULT_HEIGHT as f32;
pub const DEFAULT_CAMERA_NEAR: f32 = 0.1;
pub const DEFAULT_CAMERA_FAR: f32 = 10000.;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub type CameraId = ResourceId;

#[derive(Clone)]
pub struct OnCameraCreateData {
    pub parent_id: ObjectId,
}

#[derive(Clone)]
pub struct Camera {
    filepath: PathBuf,
    parent: Handle<Object>,
    proj: Matrix4,
    is_active: bool,
    fov_in_degrees: Degrees,
    near_plane: f32,
    far_plane: f32,
}

impl UIProperties for Camera {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Camera [{:?}]", id.as_simple().to_string()))
            .show_background(true)
            .default_open(!collapsed)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("FOV: ");
                    self.fov_in_degrees.show(id, ui_registry, ui, collapsed);
                });
                ui.horizontal(|ui| {
                    ui.label("Near plane: ");
                    self.near_plane.show(id, ui_registry, ui, collapsed);
                });
                ui.horizontal(|ui| {
                    ui.label("Far plane: ");
                    self.far_plane.show(id, ui_registry, ui, collapsed);
                });
            });
    }
}

pub struct CameraInput {
    pub movement: Vector3,
    pub rotation: Vector3,
    pub speed: f32,
}

impl SerializableResource for Camera {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) -> &mut Self {
        self.filepath = path.to_path_buf();
        self
    }

    fn extension() -> &'static str {
        CameraData::extension()
    }
}

impl ResourceTrait for Camera {
    type OnCreateData = OnCameraCreateData;

    fn on_create(
        &mut self,
        shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &CameraId,
        on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
        if let Some(on_create_data) = on_create_data {
            if let Some(parent) = shared_data_rc.get_resource::<Object>(&on_create_data.parent_id) {
                self.set_parent(&parent);
            }
        }
    }
    fn on_destroy(
        &mut self,
        _shared_data: &SharedData,
        _message_hub: &MessageHubRc,
        _id: &CameraId,
    ) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for Camera {
    type DataType = CameraData;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            filepath: PathBuf::new(),
            parent: None,
            proj: Matrix4::default_identity(),
            is_active: true,
            fov_in_degrees: Degrees::new(DEFAULT_CAMERA_FOV),
            near_plane: DEFAULT_CAMERA_NEAR,
            far_plane: DEFAULT_CAMERA_FAR,
        }
    }
    fn is_initialized(&self) -> bool {
        true
    }
    fn invalidate(&mut self) -> &mut Self {
        eprintln!("Camera cannot be invalidated!");
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
        id: CameraId,
        data: Self::DataType,
    ) -> Self {
        let mut camera = Self::new(id, shared_data, message_hub);
        camera.set_projection(data.fov, data.aspect_ratio, 1., data.near, data.far);
        camera
    }
}

impl Camera {
    #[inline]
    pub fn set_projection(
        &mut self,
        fov: Degrees,
        screen_width: f32,
        screen_height: f32,
        near: f32,
        far: f32,
    ) -> &mut Self {
        let proj = inox_math::perspective(fov, screen_width / screen_height, near, far);

        self.proj = /*OPENGL_TO_WGPU_MATRIX * */proj;

        self.fov_in_degrees = fov;
        self.near_plane = near;
        self.far_plane = far;

        self
    }
    #[inline]
    pub fn set_transform(&mut self, transform: Matrix4) -> &mut Self {
        if let Some(parent) = &self.parent {
            parent.get_mut().set_transform(transform);
        }
        self
    }
    #[inline]
    pub fn transform(&self) -> Matrix4 {
        if let Some(parent) = &self.parent {
            let transform = parent.get().transform();
            return transform;
        }
        Matrix4::default_identity()
    }
    #[inline]
    pub fn translate(&self, translation: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut().translate(translation);
        }
    }
    #[inline]
    pub fn rotate(&self, roll_yaw_pitch: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut().rotate(roll_yaw_pitch);
        }
    }
    #[inline]
    pub fn look_at(&self, target: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut().look_at(target);
        }
    }
    #[inline]
    pub fn look_toward(&self, direction: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut().look_towards(direction);
        }
    }

    #[inline]
    pub fn view_matrix(&self) -> Matrix4 {
        Matrix4::from_nonuniform_scale(1., 1., -1.) * self.transform().inverse()
    }

    #[inline]
    pub fn parent(&self) -> &Handle<Object> {
        &self.parent
    }

    #[inline]
    pub fn set_parent(&mut self, parent: &Resource<Object>) -> &mut Self {
        self.parent = Some(parent.clone());
        self
    }

    #[inline]
    pub fn set_active(&mut self, is_active: bool) -> &mut Self {
        self.is_active = is_active;
        self
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    #[inline]
    pub fn proj_matrix(&self) -> Matrix4 {
        self.proj
    }

    #[inline]
    pub fn position(&self) -> Vector3 {
        self.view_matrix().translation()
    }

    #[inline]
    pub fn fov_in_degrees(&self) -> Degrees {
        self.fov_in_degrees
    }

    #[inline]
    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }

    #[inline]
    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }

    pub fn convert_in_3d(&self, normalized_pos: Vector2) -> (Vector3, Vector3) {
        convert_in_3d(normalized_pos, &self.view_matrix(), &self.proj_matrix())
    }
}
