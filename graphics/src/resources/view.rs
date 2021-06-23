use std::path::PathBuf;

use nrg_math::{MatBase, Matrix4};
use nrg_resources::{ResourceId, ResourceTrait, SharedData, SharedDataRw};
use nrg_serialize::{generate_random_uid, Uid, INVALID_UID};

pub type ViewId = Uid;

pub struct ViewInstance {
    id: ResourceId,
    view_index: u32,
    view: Matrix4,
    proj: Matrix4,
}

impl ResourceTrait for ViewInstance {
    fn id(&self) -> ResourceId {
        self.id
    }
    fn path(&self) -> PathBuf {
        PathBuf::new()
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

    pub fn update_view(shared_data: &SharedDataRw, view_id: ViewId, mat: Matrix4) {
        let view = SharedData::get_resource::<Self>(shared_data, view_id);
        let view = &mut view.get_mut();
        view.view = mat;
    }

    pub fn update_proj(shared_data: &SharedDataRw, view_id: ViewId, mat: Matrix4) {
        let view = SharedData::get_resource::<Self>(shared_data, view_id);
        let view = &mut view.get_mut();
        view.proj = mat;
    }

    pub fn create(shared_data: &SharedDataRw, view_index: u32) -> ViewId {
        let view_id = ViewInstance::find_id_from_view_index(shared_data, view_index);
        if view_id != INVALID_UID {
            return view_id;
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
