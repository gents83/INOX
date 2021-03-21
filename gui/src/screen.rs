use std::{cell::RefCell, sync::Arc};

use nrg_math::*;
use nrg_platform::*;

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGTH: u32 = 1080;

struct ScreenData {
    size: Vector2u,
    scale_factor: f32,
    window_events: EventsRw,
}

impl Default for ScreenData {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            size: Vector2u {
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
        self.inner.borrow_mut().size = Vector2u {
            x: window.get_width(),
            y: window.get_heigth(),
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
                            size.x = *width;
                            size.y = *height;
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

    pub fn get_size(&self) -> Vector2u {
        self.inner.borrow().size
    }
    pub fn get_center(&self) -> Vector2u {
        self.get_size() / 2
    }
    pub fn get_scale_factor(&self) -> f32 {
        self.inner.borrow().scale_factor
    }

    pub fn from_normalized_into_pixels(&self, normalized_value: Vector2f) -> Vector2i {
        Vector2i {
            x: (normalized_value.x * self.inner.borrow().size.x as f32) as _,
            y: (normalized_value.y * self.inner.borrow().size.y as f32) as _,
        }
    }
    pub fn from_pixels_into_normalized(&self, value_in_px: Vector2i) -> Vector2f {
        Vector2f {
            x: value_in_px.x as f32 / self.inner.borrow().size.x as f32,
            y: value_in_px.y as f32 / self.inner.borrow().size.y as f32,
        }
    }
    pub fn convert_size_into_pixels(&self, value: Vector2f) -> Vector2u {
        Vector2u {
            x: (value.x * self.inner.borrow().size.x as f32 * 0.5) as _,
            y: (value.y * self.inner.borrow().size.y as f32 * 0.5) as _,
        }
    }
    pub fn convert_size_from_pixels(&self, value_in_px: Vector2u) -> Vector2f {
        Vector2f {
            x: value_in_px.x as f32 * 2. / self.inner.borrow().size.x as f32,
            y: value_in_px.y as f32 * 2. / self.inner.borrow().size.y as f32,
        }
    }
    pub fn from_normalized_into_screen_space(&self, normalized_pos: Vector2f) -> Vector2f {
        normalized_pos * 2.0 - [1.0, 1.0].into()
    }
    pub fn convert_from_pixels_into_screen_space(&self, pos_in_px: Vector2u) -> Vector2f {
        let normalized_pos = self.from_pixels_into_normalized(pos_in_px.convert());
        self.from_normalized_into_screen_space(normalized_pos)
    }

    pub fn from_screen_space_into_normalized(&self, pos_in_screen_space: Vector2f) -> Vector2f {
        (pos_in_screen_space + [1.0, 1.0].into()) * 0.5
    }

    pub fn from_screen_space_into_pixels(&self, pos_in_screen_space: Vector2f) -> Vector2u {
        let normalized_pos = self.from_screen_space_into_normalized(pos_in_screen_space);
        self.from_normalized_into_pixels(normalized_pos).convert()
    }
}
