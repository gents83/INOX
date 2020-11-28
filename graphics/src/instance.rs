pub struct Instance {
    pub inner: super::api::backend::instance::Instance,
}

impl Instance {
    pub fn create() -> Instance {
        Instance {
            inner: super::api::backend::instance::Instance::new(),
        } 
    }
}