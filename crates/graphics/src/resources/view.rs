use inox_math::{Degrees, MatBase, Matrix4, NewAngle};
use inox_messenger::MessageHubRc;
use inox_resources::{
    DataTypeResource, Handle, ResourceId, ResourceTrait, SharedData, SharedDataRc,
};
use inox_serialize::inox_serializable::SerializableRegistryRc;

pub type ViewId = ResourceId;

#[derive(Clone)]
pub struct View {
    view_index: u32,
    view: Matrix4,
    proj: Matrix4,
}

impl ResourceTrait for View {
    type OnCreateData = ();

    fn on_create(
        &mut self,
        _shared_data_rc: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: &ViewId,
        _on_create_data: Option<&<Self as ResourceTrait>::OnCreateData>,
    ) {
    }
    fn on_destroy(&mut self, _shared_data: &SharedData, _message_hub: &MessageHubRc, _id: &ViewId) {
    }
    fn on_copy(&mut self, other: &Self)
    where
        Self: Sized,
    {
        *self = other.clone();
    }
}

impl DataTypeResource for View {
    type DataType = u32;
    type OnCreateData = <Self as ResourceTrait>::OnCreateData;

    fn new(_id: ResourceId, _shared_data: &SharedDataRc, _message_hub: &MessageHubRc) -> Self {
        Self {
            view_index: 0,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
        }
    }
    fn is_initialized(&self) -> bool {
        true
    }
    fn invalidate(&mut self) -> &mut Self {
        eprintln!("View cannot be invalidated!");
        self
    }
    fn deserialize_data(
        _path: &std::path::Path,
        _registry: &SerializableRegistryRc,
        _f: Box<dyn FnMut(Self::DataType) + 'static>,
    ) {
    }

    fn create_from_data(
        _shared_data: &SharedDataRc,
        _message_hub: &MessageHubRc,
        _id: ResourceId,
        data: Self::DataType,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            view_index: data,
            view: Matrix4::default_identity(),
            proj: inox_math::perspective(Degrees::new(45.), 800. / 600., 0.001, 1000.0),
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
