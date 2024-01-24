pub struct ScopedProfile;

impl ScopedProfile {
    pub fn new(_category: &str, name: &'static str) -> Self {
        superluminal_perf::begin_event(name);
        ScopedProfile
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        superluminal_perf::end_event();
    }
}
