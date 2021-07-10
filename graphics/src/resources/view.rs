use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_resources::{
    DataResource, DynamicResource, Resource, ResourceId, ResourceTrait, SharedData, SharedDataRw,
};
use nrg_serialize::{generate_random_uid, Uid, INVALID_UID};

pub type ViewId = Uid;
pub type ViewRc = Resource;

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

impl ResourceTrait for ViewInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        PathBuf::new()
    }
}

impl DynamicResource for ViewInstance {}
impl DataResource for ViewInstance {
    type DataType = u32;
    fn create_from_data(shared_data: &SharedDataRw, view_index: Self::DataType) -> ViewRc {
        let view_id = ViewInstance::find_id_from_view_index(shared_data, view_index);
        if view_id != INVALID_UID {
            return SharedData::get_resource::<Self>(shared_data, view_id);
        }
        let mut data = shared_data.write().unwrap();
        data.add_resource(ViewInstance {
            id: generate_random_uid(),
            view_index,
            view: Matrix4::default_identity(),
            proj: nrg_math::perspective(nrg_math::Deg(45.), 800. / 600., 0.001, 1000.0),
        })
    }
}

impl ViewInstance {
    pub fn view(&self) -> &Matrix4 {
        &self.view
    }
    pub fn proj(&self) -> &Matrix4 {
        &self.proj
    }
    pub fn find_id_from_view_index(shared_data: &SharedDataRw, view_index: u32) -> ViewId {
        SharedData::match_resource(shared_data, |v: &ViewInstance| v.view_index == view_index)
    }

    pub fn update_view(&mut self, mat: Matrix4) {
        self.view = mat;
    }
    pub fn update_proj(&mut self, mat: Matrix4) {
        self.proj = mat;
    }
}
