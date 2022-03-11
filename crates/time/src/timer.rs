use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::Duration,
};

#[cfg(target_arch = "wasm32")]
use crate::platform::wasm::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;

pub struct Timer {
    current_time: SystemTime,
    dt: Duration,
    fps: VecDeque<SystemTime>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            fps: VecDeque::new(),
            current_time: SystemTime::now(),
            dt: Duration::default(),
        }
    }
}

impl Timer {
    pub fn update(&mut self) -> &mut Self {
        let lastframe_time = self.current_time;
        self.current_time = self.instant_time();
        self.dt = self
            .current_time
            .duration_since(lastframe_time)
            .unwrap_or_default();

        let one_sec_before = self.current_time - Duration::from_secs(1);
        self.fps.push_back(self.current_time);
        self.fps.retain(|t| *t >= one_sec_before);

        self
    }

    pub fn frame_time(&self) -> &SystemTime {
        &self.current_time
    }

    pub fn instant_time(&self) -> SystemTime {
        SystemTime::now()
    }

    pub fn dt(&self) -> &Duration {
        &self.dt
    }

    pub fn dt_from_frame_time(&self) -> Duration {
        self.current_time
            .duration_since(self.instant_time())
            .unwrap_or_default()
    }

    pub fn fps(&self) -> u32 {
        self.fps.len() as _
    }
}

pub type TimerRw = Arc<RwLock<Timer>>;
