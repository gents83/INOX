use std::path::{Path, PathBuf};

use nrg_filesystem::convert_from_local_path;
use nrg_serialize::{deserialize_from_file, Deserialize};

use crate::{ResourceData, ResourceRef, SharedDataRw};

pub const DATA_RAW_FOLDER: &str = "./data_raw/";
pub const DATA_FOLDER: &str = "./data/";

pub trait Data {
    #[inline]
    fn get_data_folder(&self) -> PathBuf {
        PathBuf::from(DATA_FOLDER)
    }
}
pub trait Deserializable: Default + for<'de> Deserialize<'de> {
    fn set_path(&mut self, filepath: &Path);
    fn path(&self) -> &Path;
}

pub trait DataTypeResource: ResourceData {
    type DataType;
    fn create_from_data(shared_data: &SharedDataRw, data: Self::DataType) -> ResourceRef<Self>
    where
        Self: Sized;
}

pub trait SerializableResource: DataTypeResource {
    fn path(&self) -> &Path;

    fn get_name(&self) -> String {
        format!(
            "{:?}",
            if let Some(name) = self.path().file_name() {
                if let Some(name) = name.to_str() {
                    name.to_string()
                } else {
                    self.id().to_simple().to_string()
                }
            } else {
                self.id().to_simple().to_string()
            }
        )
    }
    fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> ResourceRef<Self>
    where
        Self: Sized,
        Self::DataType: Deserializable,
    {
        let data = from_file::<Self::DataType>(filepath);
        Self::create_from_data(shared_data, data)
    }
}

pub trait FileResource: ResourceData {
    fn path(&self) -> &Path;

    fn get_name(&self) -> String {
        format!(
            "{:?}",
            if let Some(name) = self.path().file_name() {
                if let Some(name) = name.to_str() {
                    name.to_string()
                } else {
                    self.id().to_simple().to_string()
                }
            } else {
                self.id().to_simple().to_string()
            }
        )
    }
    fn create_from_file(shared_data: &SharedDataRw, filepath: &Path) -> ResourceRef<Self>
    where
        Self: Sized;
}

#[macro_export]
macro_rules! implement_file_data {
    // input is empty: time to output
    (@munch () -> {pub struct $name:ident $(($id:ident: $ty:ty))*}) => {
        #[repr(C)]
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        #[serde(crate = "nrg_serialize")]
        pub struct $name {
            path: PathBuf,
            $(pub $id: $ty),*
        }
        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}
        impl $crate::Deserializable for $name {
            #[inline]
            fn set_path(&mut self, filepath: &Path) {
                self.path = filepath.to_path_buf();
            }
            #[inline]
            fn path(&self) -> &Path {
                self.path.as_path()
            }
        }
    };

    // branch off to generate an inner struct
    (@munch ($id:ident: struct $name:ident {$($inner:tt)*} $($next:tt)*) -> {pub struct $($output:tt)*}) => {
        implement_file_data!(@munch ($($inner)*) -> {pub struct $name});
        implement_file_data!(@munch ($($next)*) -> {pub struct $($output)* ($id: $name)});
    };

    // throw on the last field
    (@munch ($id:ident: $ty:ty) -> {$($output:tt)*}) => {
        implement_file_data!(@munch () -> {$($output)* ($id: $ty)});
    };

    // throw on another field (not the last one)
    (@munch ($id:ident: $ty:ty, $($next:tt)*) -> {$($output:tt)*}) => {
        implement_file_data!(@munch ($($next)*) -> {$($output)* ($id: $ty)});
    };

    // entry point (this is where a macro call starts)
    (struct $name:ident { $($input:tt)*} ) => {
        implement_file_data!(@munch ($($input)*) -> {pub struct $name});
    };
}

pub fn from_file<T>(filepath: &Path) -> T
where
    T: Deserializable,
{
    let path = convert_from_local_path(PathBuf::from(DATA_FOLDER).as_path(), filepath);
    let mut data = T::default();
    deserialize_from_file(&mut data, path);
    data.set_path(filepath);
    data
}
