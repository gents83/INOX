use std::path::{Path, PathBuf};

use nrg_math::{
    Degrees, Mat4Ops, MatBase, Matrix4, NewAngle, SquareMatrix, Vector2, Vector3, Vector4,
};
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::read_from_file;
use nrg_ui::{CollapsingHeader, UIProperties, UIPropertiesRegistry, Ui};

use crate::{CameraData, Object};

pub const DEFAULT_CAMERA_FOV: f32 = 45.;
pub const DEFAULT_CAMERA_ASPECT_RATIO: f32 = 1920. / 1080.;
pub const DEFAULT_CAMERA_NEAR: f32 = 0.001;
pub const DEFAULT_CAMERA_FAR: f32 = 1000.;

pub type CameraId = ResourceId;

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

impl Default for Camera {
    fn default() -> Self {
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
}

impl UIProperties for Camera {
    fn show(
        &mut self,
        id: &ResourceId,
        ui_registry: &UIPropertiesRegistry,
        ui: &mut Ui,
        collapsed: bool,
    ) {
        CollapsingHeader::new(format!("Camera [{:?}]", id.to_simple().to_string()))
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

    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    fn is_matching_extension(path: &Path) -> bool {
        const CAMERA_EXTENSION: &str = "camera_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == CAMERA_EXTENSION;
        }
        false
    }
}
impl DataTypeResource for Camera {
    type DataType = CameraData;
    fn is_initialized(&self) -> bool {
        true
    }

    fn invalidate(&mut self) {
        panic!("Camera cannot be invalidated!");
    }

    fn deserialize_data(path: &std::path::Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        id: CameraId,
        data: Self::DataType,
    ) -> Resource<Self> {
        let mut camera = Self {
            ..Default::default()
        };
        camera.set_projection(data.fov, data.aspect_ratio, 1., data.near, data.far);
        SharedData::add_resource(shared_data, id, camera)
    }
}

impl Camera {
    pub fn new(parent: &Resource<Object>) -> Self {
        Self {
            parent: Some(parent.clone()),
            ..Default::default()
        }
    }

    #[inline]
    pub fn set_projection(
        &mut self,
        fov: Degrees,
        screen_width: f32,
        screen_height: f32,
        near: f32,
        far: f32,
    ) -> &mut Self {
        let proj = nrg_math::perspective(fov, screen_width / screen_height, near, far);

        self.proj = proj;

        self.fov_in_degrees = fov;
        self.near_plane = near;
        self.far_plane = far;

        self
    }
    #[inline]
    pub fn set_transform(&mut self, transform: Matrix4) -> &mut Self {
        if let Some(parent) = &self.parent {
            parent.get_mut(|o| {
                o.set_transform(transform);
            });
        }
        self
    }
    #[inline]
    pub fn transform(&self) -> Matrix4 {
        if let Some(parent) = &self.parent {
            let transform = parent.get(|o| o.transform());
            return transform;
        }
        Matrix4::default_identity()
    }
    #[inline]
    pub fn translate(&self, translation: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut(|o| {
                o.translate(translation);
            });
        }
    }
    #[inline]
    pub fn rotate(&self, roll_yaw_pitch: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut(|o| {
                o.rotate(roll_yaw_pitch);
            });
        }
    }
    #[inline]
    pub fn look_at(&self, target: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut(|o| {
                o.look_at(target);
            });
        }
    }
    #[inline]
    pub fn look_toward(&self, direction: Vector3) {
        if let Some(parent) = &self.parent {
            parent.get_mut(|o| {
                o.look_towards(direction);
            });
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
        let view = self.view_matrix();
        let proj = self.proj_matrix();

        // The ray Start and End positions, in Normalized Device Coordinates (Have you read Tutorial 4 ?)
        let ray_end = Vector4::new(
            normalized_pos.x * 2. - 1.,
            normalized_pos.y * 2. - 1.,
            1.,
            1.,
        );

        let inv_proj = proj.invert().unwrap();
        let inv_view = view.invert().unwrap();

        let ray_start_world = self.view_matrix().translation();

        let mut ray_end_camera = inv_proj * ray_end;
        ray_end_camera /= ray_end_camera.w;
        let mut ray_end_world = inv_view * ray_end_camera;
        ray_end_world /= ray_end_world.w;

        (ray_start_world.xyz(), ray_end_world.xyz())
    }
}
