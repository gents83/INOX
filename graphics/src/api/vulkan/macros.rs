#[macro_export]
macro_rules! VK_MAKE_VERSION {
    ($major:expr, $minor:expr, $patch:expr) => {
        ($major as u32) << 22 | ($minor as u32) << 12 | ($patch as u32)
    };
}