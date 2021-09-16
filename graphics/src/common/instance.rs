use nrg_platform::Handle;

use crate::api::backend::BackendInstance;

pub struct Instance {
    inner: BackendInstance,
}

impl std::ops::Deref for Instance {
    type Target = BackendInstance;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Instance {
    pub fn create(handle: &Handle, debug_enabled: bool) -> Instance {
        Self {
            inner: BackendInstance::new(handle, debug_enabled),
        }
    }

    pub fn destroy(&self) {
        self.inner.delete();
    }
}
