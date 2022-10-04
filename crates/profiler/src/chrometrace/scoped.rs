use crate::{get_cpu_profiler, GlobalCpuProfiler, THREAD_PROFILER};

pub struct ScopedProfile {
    profiler: GlobalCpuProfiler,
    category: String,
    name: String,
    time_start: f64,
}

impl ScopedProfile {
    pub fn new(profiler: GlobalCpuProfiler, category: &str, name: &str) -> Self {
        let time_start = profiler.get_elapsed_time();
        Self {
            profiler,
            category: category.to_string(),
            name: name.to_string(),
            time_start,
        }
    }
}

impl Drop for ScopedProfile {
    fn drop(&mut self) {
        let time_end = self.profiler.get_elapsed_time();
        THREAD_PROFILER.with(|profiler| {
            if profiler.borrow().is_none() {
                let thread_profiler = get_cpu_profiler().current_thread_profiler();
                *profiler.borrow_mut() = Some(thread_profiler);
            }
            profiler.borrow().as_ref().unwrap().push_sample(
                self.category.clone(),
                self.name.clone(),
                self.time_start,
                time_end,
            );
        });
    }
}
