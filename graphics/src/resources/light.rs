use std::path::{Path, PathBuf};

use nrg_math::Vector3;
use nrg_messenger::MessengerRw;
use nrg_resources::{
    DataTypeResource, Resource, ResourceId, SerializableResource, SharedData, SharedDataRc,
};
use nrg_serialize::read_from_file;

use crate::LightData;

pub type LightId = ResourceId;

#[derive(Clone)]
pub struct Light {
    filepath: PathBuf,
    data: LightData,
    is_active: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            filepath: PathBuf::new(),
            data: LightData::default(),
            is_active: true,
        }
    }
}

impl SerializableResource for Light {
    fn path(&self) -> &Path {
        self.filepath.as_path()
    }

    fn set_path(&mut self, path: &Path) {
        self.filepath = path.to_path_buf();
    }

    fn is_matching_extension(path: &Path) -> bool {
        const LIGHT_EXTENSION: &str = "light_data";

        if let Some(ext) = path.extension().unwrap().to_str() {
            return ext == LIGHT_EXTENSION;
        }
        false
    }
}
impl DataTypeResource for Light {
    type DataType = LightData;
    fn is_initialized(&self) -> bool {
        true
    }

    fn invalidate(&mut self) {
        panic!("Light cannot be invalidated!");
    }

    fn deserialize_data(path: &std::path::Path) -> Self::DataType {
        read_from_file::<Self::DataType>(path)
    }
    fn create_from_data(
        shared_data: &SharedDataRc,
        _global_messenger: &MessengerRw,
        id: LightId,
        data: Self::DataType,
    ) -> Resource<Self> {
        let light = Self {
            data,
            ..Default::default()
        };
        SharedData::add_resource(shared_data, id, light)
    }
}

impl Light {
    #[inline]
    pub fn set_position(&mut self, position: Vector3) -> &mut Self {
        self.data.position = position.into();
        self
    }

    #[inline]
    pub fn data(&self) -> &LightData {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut LightData {
        &mut self.data
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
}
