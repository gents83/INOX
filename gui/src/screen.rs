use std::{cell::RefCell, sync::Arc};

use nrg_math::*;
use nrg_platform::*;

const DEFAULT_WIDTH: f32 = 1920.0;
const DEFAULT_HEIGTH: f32 = 1080.0;

struct ScreenData {
    size: Vector2f,
    scale_factor: f32,
    window_events: EventsRw,
}

impl Default for ScreenData {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
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
        };
        self.inner.borrow_mut().scale_factor = window.get_scale_factor();
    }

    pub fn update(&mut self) {
        let mut inner = self.inner.borrow_mut();
        let mut size = inner.size;
        let mut scale_factor = inner.scale_factor;
        {
            let events = inner.window_events.read().unwrap();
            if let Some(window_events) = events.read_events::<WindowEvent>() {
                for event in window_events.iter() {
                    match event {
                        WindowEvent::SizeChanged(width, height) => {
                            size.x = *width as _;
                            size.y = *height as _;
                        }
                        WindowEvent::DpiChanged(x, _y) => {
                            scale_factor = x / DEFAULT_DPI;
                        }
                        _ => {}
                    }
                }
            }
        }
        inner.size = size;
        inner.scale_factor = scale_factor;
    }

    pub fn get_size(&self) -> Vector2f {
        self.inner.borrow().size
    }
    pub fn get_center(&self) -> Vector2f {
        self.get_size() * 0.5
    }
    pub fn get_scale_factor(&self) -> f32 {
        self.inner.borrow().scale_factor
    }
    pub fn convert_position_into_pixels(&self, value: Vector2f) -> Vector2f {
        value * self.inner.borrow().size
    }
    pub fn convert_size_into_pixels(&self, value: Vector2f) -> Vector2f {
        value * self.inner.borrow().size * 0.5
    }
    pub fn convert_position_from_pixels(&self, value: Vector2f) -> Vector2f {
        value / self.inner.borrow().size
    }
    pub fn convert_size_from_pixels(&self, value: Vector2f) -> Vector2f {
        value * 2. / self.inner.borrow().size
    }
    pub fn convert_into_screen_space(&self, position: Vector2f) -> Vector2f {
        position * 2.0 - [1.0, 1.0].into()
    }
    pub fn convert_from_pixels_into_screen_space(&self, position: Vector2f) -> Vector2f {
        self.convert_into_screen_space(self.convert_position_from_pixels(position))
    }

    pub fn convert_from_screen_space(&self, position: Vector2f) -> Vector2f {
        (position + [1.0, 1.0].into()) * 0.5
    }

    pub fn convert_from_screen_space_into_pixels(&self, position: Vector2f) -> Vector2f {
        self.convert_position_into_pixels(self.convert_from_screen_space(position))
    }
}
