use std::{cell::RefCell, sync::Arc};

use nrg_math::*;
use nrg_platform::*;

const DEFAULT_WIDTH: f32 = 1920.0;
const DEFAULT_HEIGTH: f32 = 1080.0;

struct ScreenData {
    size: Vector2f,
    window_events: EventsRw,
}

impl Default for ScreenData {
    fn default() -> Self {
        Self {
            size: Vector2f {
                x: DEFAULT_WIDTH,
                y: DEFAULT_HEIGTH,
            },
            window_events: EventsRw::default(),
        }
    }
}

#[derive(Clone)]
pub struct Screen {
    inner: Arc<RefCell<ScreenData>>,
}
unsafe impl Send for Screen {}
unsafe impl Sync for Screen {}

impl Default for Screen {
    fn default() -> Self {
        Self {
            inner: Arc::new(RefCell::new(ScreenData::default())),
        }
    }
}

impl Screen {
    pub fn init(&mut self, window: &Window) {
        self.inner.borrow_mut().window_events = window.get_events();
        self.inner.borrow_mut().size = Vector2f {
            x: window.get_width() as _,
            y: window.get_heigth() as _,
        }
    }

    pub fn update(&mut self) {
        let mut inner = self.inner.borrow_mut();
        let mut size = inner.size;
        {
            let events = inner.window_events.read().unwrap();
            let window_events = events.read_events::<WindowEvent>();
            for event in window_events.iter() {
                if let WindowEvent::SizeChanged(width, height) = event {
                    size.x = *width as _;
                    size.y = *height as _;
                }
            }
        }
        inner.size = size;
    }

    pub fn convert_into_pixels(&self, value: Vector2f) -> Vector2f {
        value * self.inner.borrow().size
    }
    pub fn convert_from_pixels(&self, value: Vector2f) -> Vector2f {
        value / self.inner.borrow().size
    }
    pub fn convert_into_screen_space(&self, position: Vector2f) -> Vector2f {
        position * 2.0 - [1.0, 1.0].into()
    }
    pub fn convert_from_pixels_into_screen_space(&self, position: Vector2f) -> Vector2f {
        self.convert_from_pixels(position) * 2.0 - [1.0, 1.0].into()
    }
}
