use std::path::{Path, PathBuf};

use nrg_math::{
    direction_to_euler_angles, Degrees, InnerSpace, Mat4Ops, MatBase, Matrix4, NewAngle,
    SquareMatrix, Vector2, Vector3, Vector4, Zero,
};
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::read_from_file;

use crate::CameraData;

pub const DEFAULT_CAMERA_FOV: f32 = 45.;
pub const DEFAULT_CAMERA_ASPECT_RATIO: f32 = 1920. / 1080.;
pub const DEFAULT_CAMERA_NEAR: f32 = 0.001;
pub const DEFAULT_CAMERA_FAR: f32 = 1000.;

pub type CameraId = ResourceId;

#[derive(Clone)]
pub struct Camera {
    filepath: PathBuf,
    position: Vector3,
    rotation: Vector3, //pitch, yaw, roll
    direction: Vector3,
    proj_matrix: Matrix4,
    is_active: bool,
    fov_in_degrees: f32,
    near_plane: f32,
    far_plane: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            filepath: PathBuf::new(),
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            direction: Vector3::unit_z(),
            proj_matrix: Matrix4::default_identity(),
            is_active: true,
            fov_in_degrees: DEFAULT_CAMERA_FOV,
            near_plane: DEFAULT_CAMERA_NEAR,
            far_plane: DEFAULT_CAMERA_FAR,
        }
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
        let (position, rotation, _) = data.transform.get_translation_rotation_scale();
        let mut camera = Self {
            position,
            rotation,
            ..Default::default()
        };
        camera.set_projection(data.fov, data.aspect_ratio, 1., data.near, data.far);
        SharedData::add_resource(shared_data, id, camera)
    }
}

impl Camera {
    pub fn new(position: Vector3, target: Vector3) -> Self {
        let mut camera = Self {
            position,
            ..Default::default()
        };
        camera.look_at(target);
        camera.update();
        camera
    }

    #[inline]
    pub fn set_projection(
        &mut self,
        fov: f32,
        screen_width: f32,
        screen_height: f32,
        near: f32,
        far: f32,
    ) -> &mut Self {
        let proj =
            nrg_math::perspective(Degrees::new(fov), screen_width / screen_height, near, far);

        self.proj_matrix = proj;

        self.fov_in_degrees = fov;
        self.near_plane = near;
        self.far_plane = far;

        self
    }

    #[inline]
    pub fn translate(&mut self, movement: Vector3) -> &mut Self {
        self.position += self.direction * movement.z;
        let up: Vector3 = [0., 1., 0.].into();
        let right = self.direction.cross(up).normalize();
        let up = right.cross(self.direction).normalize();
        self.position += right * movement.x;
        self.position += up * movement.y;
        self.update();
        self
    }

    #[inline]
    pub fn rotate(&mut self, rotation_angle: Vector3) -> &mut Self {
        self.rotation += rotation_angle;
        self.update();
        self
    }

    #[inline]
    pub fn look_at(&mut self, position: Vector3) -> &mut Self {
        self.direction = (position - self.position).normalize();
        self.rotation = direction_to_euler_angles(self.direction);
        self.update();
        self
    }

    #[inline]
    fn update(&mut self) -> &mut Self {
        let mut forward = Vector3::zero();
        /*
        x = cos(yaw)*cos(pitch)
        y = sin(yaw)*cos(pitch)
        z = sin(pitch)
        */
        forward.x = self.rotation.y.cos() * self.rotation.x.cos();
        forward.y = self.rotation.y.sin() * self.rotation.x.cos();
        forward.z = self.rotation.x.sin();

        self.direction = forward.normalize();

        self
    }

    #[inline]
    pub fn view_matrix(&self) -> Matrix4 {
        let up: Vector3 = [0., 1., 0.].into();
        let right = self.direction.cross(up).normalize();
        let up = right.cross(self.direction).normalize();

        nrg_math::create_look_at(self.position, self.position + self.direction, up)
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
        self.proj_matrix
    }

    #[inline]
    pub fn position(&self) -> Vector3 {
        self.position
    }

    #[inline]
    pub fn direction(&self) -> Vector3 {
        self.direction
    }

    #[inline]
    pub fn fov_in_degrees(&self) -> f32 {
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

        let ray_start_world = self.position;

        let mut ray_end_camera = inv_proj * ray_end;
        ray_end_camera /= ray_end_camera.w;
        let mut ray_end_world = inv_view * ray_end_camera;
        ray_end_world /= ray_end_world.w;

        (ray_start_world.xyz(), ray_end_world.xyz())
    }
}
