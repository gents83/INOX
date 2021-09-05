use nrg_math::{MatBase, Matrix4};
use nrg_resources::{
    DataTypeResource, Handle, Resource, ResourceData, ResourceId, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, Uid, INVALID_UID};

pub type ViewId = Uid;

pub struct View {
    id: ResourceId,
    view_index: u32,
    view: Matrix4,
    proj: Matrix4,
}

impl Default for View {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            view_index: 0,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
        }
    }
}

impl ResourceData for View {
    fn id(&self) -> ResourceId {
        self.id
    }
}

impl DataTypeResource for View {
    type DataType = u32;
    fn create_from_data(shared_data: &SharedDataRw, view_index: Self::DataType) -> Resource<Self> {
        if let Some(view) = View::find_from_view_index(shared_data, view_index) {
            return view;
        }
        SharedData::add_resource(
            shared_data,
            View {
                id: generate_random_uid(),
                view_index,
                view: Matrix4::default_identity(),
                proj: nrg_math::perspective(nrg_math::Deg(45.), 800. / 600., 0.001, 1000.0),
            },
        )
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
    pub fn find_from_view_index(shared_data: &SharedDataRw, view_index: u32) -> Handle<View> {
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
