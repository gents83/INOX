use nrg_math::{Degrees, MatBase, Matrix4, NewAngle};
use nrg_messenger::MessengerRw;
use nrg_resources::{DataTypeResource, Handle, ResourceId, SharedData, SharedDataRc};

pub type ViewId = ResourceId;

#[derive(Clone)]
pub struct View {
    view_index: u32,
    view: Matrix4,
    proj: Matrix4,
}

impl Default for View {
    fn default() -> Self {
        Self {
            view_index: 0,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
        }
    }
}

impl DataTypeResource for View {
    type DataType = u32;

    fn is_initialized(&self) -> bool {
        true
    }
    fn invalidate(&mut self) {
        panic!("View cannot be invalidated!");
    }
    fn deserialize_data(_path: &std::path::Path) -> Self::DataType {
        0
    }

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            view_index: data,
            view: Matrix4::default_identity(),
            proj: nrg_math::perspective(Degrees::new(45.), 800. / 600., 0.001, 1000.0),
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
    pub fn find_from_view_index(shared_data: &SharedDataRc, view_index: u32) -> Handle<View> {
        SharedData::match_resource(shared_data, |v: &View| v.view_index == view_index)
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
