#![cfg(feature = "gpu")]
#![allow(improper_ctypes_definitions)]

use std::sync::{Arc, LazyLock, RwLock};

use crate::ThreadProfiler;

pub struct GpuProfiler {
    settings: wgpu_profiler::GpuProfilerSettings,
    wgpu_profiler: wgpu_profiler::GpuProfiler,
}

pub type GlobalGpuProfiler = Arc<RwLock<GpuProfiler>>;

pub static GLOBAL_GPU_PROFILER: LazyLock<GlobalGpuProfiler> = LazyLock::new(|| {
    let settings = wgpu_profiler::GpuProfilerSettings {
        enable_timer_queries: false,
        ..Default::default()
    };
    let wgpu_profiler = wgpu_profiler::GpuProfiler::new(settings.clone()).unwrap();
    Arc::new(RwLock::new(GpuProfiler {
        wgpu_profiler,
        settings,
    }))
});

impl GpuProfiler {
    pub fn enable(&mut self, enabled: bool) -> &mut Self {
        self.settings.enable_timer_queries = enabled;
        self.wgpu_profiler
            .change_settings(self.settings.clone())
            .ok();
        self
    }
    pub fn profile<'a, P>(
        &'a self,
        label: &str,
        encoder_or_pass: &'a mut P,
        device: &wgpu::Device,
    ) -> wgpu_profiler::Scope<'a, P>
    where
        P: wgpu_profiler::ProfilerCommandRecorder,
    {
        self.wgpu_profiler.scope(label, encoder_or_pass, device)
    }
    pub fn resolve_queries(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.wgpu_profiler.resolve_queries(encoder);
    }
    pub fn end_frame(&mut self, profiler: &ThreadProfiler, queue: &wgpu::Queue) -> &mut Self {
        if self.wgpu_profiler.end_frame().is_ok() {
            let mut wgpu_results = Vec::new();
            while let Some(mut results) = self
                .wgpu_profiler
                .process_finished_frame(queue.get_timestamp_period())
            {
                wgpu_results.append(&mut results);
            }
            if self.settings.enable_timer_queries && !wgpu_results.is_empty() {
                wgpu_results.iter().for_each(|r| {
                    if let Some(time) = &r.time {
                        profiler.push_sample(
                            "GPU".to_string(),
                            r.label.to_string(),
                            time.start * 1000.,
                            time.end * 1000.,
                        );
                    }
                });
            }
        }
        self
    }
}
