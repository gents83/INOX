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
    width: u32,
    height: u32,
    view: Matrix4,
    proj: Matrix4,
}

impl Default for ViewInstance {
    fn default() -> Self {
        Self {
            id: INVALID_UID,
            view_index: 0,
            width: 800,
            height: 600,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
        }
    }
}

impl ResourceData for ViewInstance {
    fn id(&self) -> ResourceId {
        self.id
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
                width: 800,
                height: 600,
                view: Matrix4::default_identity(),
                proj: nrg_math::perspective(nrg_math::Deg(45.), 800. / 600., 0.001, 1000.0),
            },
        )
    }
}

impl ViewInstance {
    pub fn view_index(&self) -> u32 {
        self.view_index
    }
    pub fn view(&self) -> Matrix4 {
        self.view
    }
    pub fn proj(&self) -> Matrix4 {
        self.proj
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn find_from_view_index(shared_data: &SharedDataRw, view_index: u32) -> Option<ViewRc> {
        SharedData::match_resource(shared_data, |v: &ViewInstance| v.view_index == view_index)
    }

    pub fn update_view(&mut self, mat: Matrix4) -> &mut Self {
        self.view = mat;
        self
    }
    pub fn update_proj(&mut self, mat: Matrix4) -> &mut Self {
        self.proj = mat;
        self
    }
    pub fn update_size(&mut self, width: u32, height: u32) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }
}
