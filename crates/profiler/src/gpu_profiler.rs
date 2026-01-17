#![cfg(feature = "gpu")]
#![allow(improper_ctypes_definitions)]

use std::{
    collections::VecDeque,
    sync::{Arc, LazyLock, RwLock},
};

use crate::ThreadProfiler;

pub struct GpuProfiler {
    settings: wgpu_profiler::GpuProfilerSettings,
    wgpu_profiler: wgpu_profiler::GpuProfiler,
    cpu_frame_times: VecDeque<f64>,
    base_time_offset: Option<f64>,
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
        cpu_frame_times: VecDeque::new(),
        base_time_offset: None,
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
    pub fn end_frame(
        &mut self,
        profiler: &ThreadProfiler,
        queue: &wgpu::Queue,
        cpu_time: f64,
    ) -> &mut Self {
        self.cpu_frame_times.push_back(cpu_time);
        if self.wgpu_profiler.end_frame().is_ok() {
            let mut wgpu_results = Vec::new();
            while let Some(results) = self
                .wgpu_profiler
                .process_finished_frame(queue.get_timestamp_period())
            {
                let frame_cpu_time = self.cpu_frame_times.pop_front().unwrap_or(cpu_time);
                if self.base_time_offset.is_none() && !results.is_empty() {
                    let first_start = results[0].time.as_ref().map(|t| t.start).unwrap_or(0.);
                    self.base_time_offset = Some(frame_cpu_time - (first_start * 1_000_000.));
                }

                if let Some(offset) = self.base_time_offset {
                    for r in results {
                        if let Some(time) = r.time {
                            let start = (time.start * 1_000_000.) + offset;
                            let end = (time.end * 1_000_000.) + offset;
                            wgpu_results.push((r.label, start, end));
                        }
                    }
                }
            }
            if self.settings.enable_timer_queries && !wgpu_results.is_empty() {
                let gpu_tid = u64::MAX;
                for (label, start, end) in wgpu_results {
                    profiler.push_sample_with_tid(
                        gpu_tid as _,
                        "GPU".to_string(),
                        label,
                        start,
                        end,
                    );
                }
            }
        }
        self
    }
}
