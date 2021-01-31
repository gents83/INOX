use super::platform_impl::platform::handle::*;

pub struct Handle
{
    pub handle_impl: HandleImpl,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}