#[macro_export]
macro_rules! scoped_profile {
    ($string:expr) => {
        #[cfg(debug_assertions)]
        let _profile_scope = $crate::ScopedProfile::new(format!("{}: {}", module_path!(), $string));
    };
}

#[macro_export]
macro_rules! register_thread_into_profiler {
    () => {
        #[cfg(debug_assertions)]
        $crate::Profiler::get_mut().register_thread();
    };
}

#[macro_export]
macro_rules! write_profile_file {
    () => {
        #[cfg(debug_assertions)]
        {
            let profile_file = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "app.nrg_profile");
            $crate::Profiler::get_mut().write_profile_file("app.nrg_profile");
        }
    };
}
