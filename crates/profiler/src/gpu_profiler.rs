#![cfg(feature = "gpu")]
#![allow(improper_ctypes_definitions)]

use std::sync::{Arc, RwLock};

use wgpu_profiler::GpuProfiler;

pub type GlobalGpuProfiler = Arc<RwLock<GpuProfiler>>;

pub const GET_GPU_PROFILER_FUNCTION_NAME: &str = "get_gpu_profiler";
pub type PfnGetGpuProfiler = ::std::option::Option<unsafe extern "C" fn() -> GlobalGpuProfiler>;

pub const CREATE_GPU_PROFILER_FUNCTION_NAME: &str = "create_gpu_profiler";
pub type PfnCreateGpuProfiler =
    ::std::option::Option<unsafe extern "C" fn(&wgpu::Device, &wgpu::Queue, bool)>;

pub static mut GLOBAL_GPU_PROFILER: Option<GlobalGpuProfiler> = None;

#[no_mangle]
pub extern "C" fn create_gpu_profiler(device: &wgpu::Device, queue: &wgpu::Queue, start: bool) {
    unsafe {
        if GLOBAL_GPU_PROFILER.is_none() {
            let gpu_profiler = GpuProfiler::new(device, queue, start);
            GLOBAL_GPU_PROFILER.replace(Arc::new(RwLock::new(gpu_profiler)));
        }
    }
}
#[no_mangle]
pub extern "C" fn get_gpu_profiler() -> GlobalGpuProfiler {
    unsafe { GLOBAL_GPU_PROFILER.as_ref().unwrap().clone() }
}
