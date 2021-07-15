use std::path::{Path, PathBuf};

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

#[inline]
pub fn convert_from_local_path(parent_folder: &Path, relative_path: &Path) -> PathBuf {
    let mut pathbuf = parent_folder.to_path_buf();
    let data_folder = pathbuf
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let string = relative_path.to_str().unwrap().to_string();
    if string.contains(parent_folder.to_str().unwrap()) {
        pathbuf = relative_path.canonicalize().unwrap()
    } else if string.contains(data_folder.as_str()) {
        pathbuf = relative_path.to_path_buf()
    } else if let Ok(result_path) = pathbuf.join(relative_path).canonicalize() {
        pathbuf = result_path;
    } else {
        eprintln!("Unable to join {:?} with {:?}", pathbuf, relative_path);
    }
    pathbuf
}

pub fn convert_in_local_path(original_path: &Path, base_path: &Path) -> PathBuf {
    let path = original_path.to_str().unwrap().to_string();
    let path = path.replace(
        PathBuf::from(base_path)
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap(),
        "",
    );
    let mut path = path.replace("\\", "/");
    if path.starts_with('/') {
        path.remove(0);
    }
    PathBuf::from(path)
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
