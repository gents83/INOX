pub struct ScopedProfile;

impl ScopedProfile {
    pub fn new(category: &str, name: &str) -> Self {
        superluminal_perf::begin_event(format!("[{}]{}", category, name).as_str());
        ScopedProfile
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        superluminal_perf::end_event();
    }
}
