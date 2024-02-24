use inox_math::{Degrees, MatBase, Matrix4, NewAngle, Radians};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, ResourceId, ResourceTrait, SharedData, SharedDataRc,
};

use crate::{DEFAULT_FAR, DEFAULT_FOV, DEFAULT_HEIGHT, DEFAULT_NEAR, DEFAULT_WIDTH};

pub type ViewId = ResourceId;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Clone)]
pub struct View {
    view_index: u32,
    view: Matrix4,
    proj: Matrix4,
    fov_in_degrees: Degrees,
}

impl ResourceTrait for View {
    fn is_initialized(&self) -> bool {
        true
    }
    fn invalidate(&mut self) -> &mut Self {
        self
    }
}

impl DataTypeResource for View {
    type DataType = u32;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            view_index: 0,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
            fov_in_degrees: Degrees::new(DEFAULT_FOV),
        }
    }

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: ResourceId,
        data: &Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        let fov_in_degrees = Degrees::new(DEFAULT_FOV);
        Self {
            view_index: *data,
            view: Matrix4::default_identity(),
            proj: inox_math::perspective(
                fov_in_degrees,
                DEFAULT_WIDTH as f32 / DEFAULT_HEIGHT as f32,
                DEFAULT_NEAR,
                DEFAULT_FAR,
            ),
            fov_in_degrees,
        }
    }
}

impl View {
    pub fn view_index(&self) -> u32 {
        self.view_index
    }
    pub fn view(&self) -> Matrix4 {
        self.view
    }
    pub fn proj(&self) -> Matrix4 {
        self.proj
    }
    pub fn near(&self) -> f32 {
        DEFAULT_NEAR
    }
    pub fn far(&self) -> f32 {
        DEFAULT_FAR
    }
    pub fn fov_in_radians(&self) -> Radians {
        self.fov_in_degrees.into()
    }
    pub fn fov_in_degrees(&self) -> Degrees {
        self.fov_in_degrees
    }
    pub fn find_from_view_index(shared_data: &SharedDataRc, view_index: u32) -> Handle<View> {
        SharedData::match_resource(shared_data, |v: &View| v.view_index == view_index)
    }

    pub fn update_fov(&mut self, fov_in_degrees: Degrees) -> &mut Self {
        self.fov_in_degrees = fov_in_degrees;
        self
    }
    pub fn update_view(&mut self, mat: Matrix4) -> &mut Self {
        self.view = mat;
        self
    }
    pub fn update_proj(&mut self, mat: Matrix4) -> &mut Self {
        self.proj = mat;
        self
    }
}
