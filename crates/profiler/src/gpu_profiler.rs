#![cfg(feature = "gpu")]
#![allow(improper_ctypes_definitions)]

use std::sync::{Arc, RwLock};

use crate::ThreadProfiler;

pub struct GpuProfiler {
    settings: wgpu_profiler::GpuProfilerSettings,
    wgpu_profiler: wgpu_profiler::GpuProfiler,
}

pub type GlobalGpuProfiler = Arc<RwLock<GpuProfiler>>;

pub const GET_GPU_PROFILER_FUNCTION_NAME: &str = "get_gpu_profiler";
pub type PfnGetGpuProfiler = ::std::option::Option<unsafe extern "C" fn() -> GlobalGpuProfiler>;

pub const CREATE_GPU_PROFILER_FUNCTION_NAME: &str = "create_gpu_profiler";
pub type PfnCreateGpuProfiler =
    ::std::option::Option<unsafe extern "C" fn()>;

pub static mut GLOBAL_GPU_PROFILER: Option<GlobalGpuProfiler> = None;

#[no_mangle]
pub extern "C" fn create_gpu_profiler() {
    unsafe {
        if GLOBAL_GPU_PROFILER.is_none() {            
            let settings = wgpu_profiler::GpuProfilerSettings {
                enable_timer_scopes: false,
                ..Default::default()
            };
            if let Ok(wgpu_profiler) = wgpu_profiler::GpuProfiler::new(settings.clone()) {
                GLOBAL_GPU_PROFILER.replace(Arc::new(RwLock::new(GpuProfiler {
                    wgpu_profiler,
                    settings,
                })));
            }
        }
    }
}
#[no_mangle]
pub extern "C" fn get_gpu_profiler() -> GlobalGpuProfiler {
    unsafe { GLOBAL_GPU_PROFILER.as_ref().unwrap().clone() }
}

impl GpuProfiler {
    pub fn enable(&mut self, enabled: bool) -> &mut Self {
        self.settings.enable_timer_scopes = enabled;
        self.wgpu_profiler.change_settings(self.settings.clone()).ok();
        self
    }
    pub fn profile<'a, P>(
        &'a mut self,
        label: &str,
        recorder: &'a mut P,
        device: &wgpu::Device,
    ) -> wgpu_profiler::scope::Scope<P>
    where
        P: wgpu_profiler::ProfilerCommandRecorder,
    {
        wgpu_profiler::scope::Scope::start(label, &mut self.wgpu_profiler, recorder, device)
    }
    pub fn resolve_queries(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.wgpu_profiler.resolve_queries(encoder);
    }
    pub fn end_frame(&mut self, profiler: &ThreadProfiler, queue: &wgpu::Queue) -> &mut Self {
        if self.wgpu_profiler.end_frame().is_ok() {
            let mut wgpu_results = Vec::new();
            while let Some(mut results) = self.wgpu_profiler.process_finished_frame(queue.get_timestamp_period()) {
                wgpu_results.append(&mut results);
            }
            if self.settings.enable_timer_scopes && !wgpu_results.is_empty() {
                wgpu_results.iter().for_each(|r| {
                    profiler.push_sample(
                        "GPU".to_string(),
                        r.label.to_string(),
                        r.time.start * 1000.,
                        r.time.end * 1000.,
                    );
                });
            }
        }
        self
    }
}
