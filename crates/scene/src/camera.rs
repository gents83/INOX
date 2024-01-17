use std::path::{Path, PathBuf};

use inox_graphics::{DEFAULT_ASPECT_RATIO, DEFAULT_FAR, DEFAULT_FOV, DEFAULT_NEAR};
use inox_math::{convert_in_3d, Degrees, MatBase, Matrix4, NewAngle, Radians, Vector2, Vector3};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, Resource, ResourceId, ResourceTrait, SerializableResource,
    SharedDataRc,
};
use inox_serialize::{inox_serializable::SerializableRegistryRc, read_from_file, SerializeFile};
use inox_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{CameraData, Object};

pub type CameraId = ResourceId;

#[derive(Clone)]
pub struct Camera {
    filepath: PathBuf,
    parent: Handle<Object>,
    proj: Matrix4,
    is_active: bool,
    fov_in_degrees: Degrees,
    aspect_ratio: f32,
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

    fn deserialize_data(
        path: &std::path::Path,
        registry: &SerializableRegistryRc,
        f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
        read_from_file::<Self::DataType>(path, registry, f);
    }
}

impl ResourceTrait for Camera {
    fn is_initialized(&self) -> bool {
        true
    }
    fn invalidate(&mut self) -> &mut Self {
        self
    }
}

impl DataTypeResource for Camera {
    type DataType = CameraData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            filepath: PathBuf::new(),
            parent: None,
            proj: Matrix4::default_identity(),
            is_active: true,
            fov_in_degrees: Degrees::new(DEFAULT_FOV),
            near_plane: DEFAULT_NEAR,
            far_plane: DEFAULT_FAR,
            aspect_ratio: DEFAULT_ASPECT_RATIO,
        }
    }

    fn create_from_data(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        id: CameraId,
        data: &Self::DataType,
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
        fov_in_degrees: Degrees,
        screen_width: f32,
        screen_height: f32,
        near: f32,
        far: f32,
    ) -> &mut Self {
        let proj = inox_math::perspective(fov_in_degrees, screen_width / screen_height, near, far);

        self.proj = proj;

        self.fov_in_degrees = fov_in_degrees;
        self.aspect_ratio = screen_width / screen_height;
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
    pub fn fov_in_degrees(&self) -> Degrees {
        self.fov_in_degrees
    }

    #[inline]
    pub fn fov_in_radians(&self) -> Radians {
        self.fov_in_degrees.into()
    }

    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
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
        convert_in_3d(normalized_pos, &self.transform(), &self.proj_matrix())
    }
}
