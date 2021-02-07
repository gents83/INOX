use super::platform_impl::platform::handle::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle
{
    pub handle_impl: HandleImpl,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}