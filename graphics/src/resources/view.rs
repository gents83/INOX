use nrg_math::{MatBase, Matrix4};
use nrg_resources::{
    DataTypeResource, ResourceData, ResourceId, ResourceRef, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, Uid, INVALID_UID};

pub type ViewId = Uid;
pub type ViewRc = ResourceRef<ViewInstance>;

pub struct ViewInstance {
    id: ResourceId,
    view_index: u32,
    view: Matrix4,
    proj: Matrix4,
}

impl Default for ViewInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            view_index: 0,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
        }
    }
}

impl ResourceData for ViewInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn info(&self) -> String {
        format!(
            "View {:?}
            {:?}",
            self.id().to_simple().to_string(),
            self.view_index
        )
    }
}

impl DataTypeResource for ViewInstance {
    type DataType = u32;
    fn create_from_data(shared_data: &SharedDataRw, view_index: Self::DataType) -> ViewRc {
        if let Some(view) = ViewInstance::find_from_view_index(shared_data, view_index) {
            return view;
        }
        SharedData::add_resource(
            shared_data,
            ViewInstance {
                id: generate_random_uid(),
                view_index,
                view: Matrix4::default_identity(),
                proj: nrg_math::perspective(nrg_math::Deg(45.), 800. / 600., 0.001, 1000.0),
            },
        )
    }
}

impl ViewInstance {
    pub fn view(&self) -> &Matrix4 {
        &self.view
    }
    pub fn proj(&self) -> &Matrix4 {
        &self.proj
    }
    pub fn find_from_view_index(shared_data: &SharedDataRw, view_index: u32) -> Option<ViewRc> {
        SharedData::match_resource(shared_data, |v: &ViewInstance| v.view_index == view_index)
    }

    pub fn update_view(&mut self, mat: Matrix4) {
        self.view = mat;
    }
    pub fn update_proj(&mut self, mat: Matrix4) {
        self.proj = mat;
    }
}
