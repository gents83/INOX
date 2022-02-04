#[macro_export]
macro_rules! load_serializable_registry_lib {
    () => {
        use std::path::PathBuf;
        use $crate::*;
        unsafe {
            if SABI_SERIALIZABLE_REGISTRY_LIB.is_none() {
                let library_name = inox_filesystem::library_filename("inox_serializable");
                let (path, filename) = inox_filesystem::library::compute_folder_and_filename(
                    PathBuf::from(library_name),
                );
                let fullpath = path.join(filename);
                let library = inox_filesystem::Library::new(fullpath);
                SABI_SERIALIZABLE_REGISTRY_LIB = Some(library);
            }
        }
    };
}

#[macro_export]
macro_rules! deserialize_serializable {
    ($T: ty, $D: ty, $Deserializer: expr, $DeserializeType: expr) => {
        unsafe {
            use $crate::*;
            use inox_serializable::serde::de::Error;

            $crate::load_serializable_registry_lib!();

            if let Some(get_registry_fn) = SABI_SERIALIZABLE_REGISTRY_LIB
                .as_ref()
                .unwrap()
                .get::<PfnGetSerializableRegistry>(GET_SERIALIZABLE_REGISTRY_FUNCTION_NAME)
            {
                let serializable_registry = get_registry_fn.unwrap()();
                serializable_registry.deserialize::<$T, $D>($Deserializer, $DeserializeType)
            } else {
                panic!("get_registry_fn is None - probably unable to load serializable_registry library");
            }
        }
    };
}

#[macro_export]
macro_rules! get_serializable_registry {
    () => {
        unsafe {
            use $crate::*;

            $crate::load_serializable_registry_lib!();

            if let Some(get_registry_fn) = SABI_SERIALIZABLE_REGISTRY_LIB
                .as_ref()
                .unwrap()
                .get::<PfnGetSerializableRegistry>(GET_SERIALIZABLE_REGISTRY_FUNCTION_NAME)
            {
                get_registry_fn.unwrap()()
            } else {
                panic!("get_registry_fn is None - probably unable to load serializable_registry library");
            }
        }
    };
}

#[macro_export]
macro_rules! create_serializable_registry {
    () => {
        use $crate::*;

        $crate::load_serializable_registry_lib!();

        unsafe {
            if let Some(create_fn) = SABI_SERIALIZABLE_REGISTRY_LIB
                .as_ref()
                .unwrap()
                .get::<PfnCreateSerializableRegistry>(
                CREATE_SERIALIZABLE_REGISTRY_FUNCTION_NAME,
            ) {
                create_fn.unwrap()();
            }
        }
    };
}
