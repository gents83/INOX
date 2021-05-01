use std::{
    borrow::Borrow,
    cell::{RefCell, RefMut},
    sync::Arc,
    sync::Once,
};

use nrg_events::EventsRw;
use nrg_platform::{WindowEvent, DEFAULT_DPI};
use nrg_math::{Vector2, Vector4};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGTH: u32 = 1080;

struct ScreenData {
    size: Vector2,
    scale_factor: f32,
    window_events: EventsRw,
}

impl Default for ScreenData {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            size: Vector2::new(DEFAULT_WIDTH as _, DEFAULT_HEIGTH as _),
            window_events: EventsRw::default(),
        }
    }
}

static mut SCREEN_DATA: Option<Arc<RefCell<ScreenData>>> = None;
static mut INIT: Once = Once::new();

pub struct Screen {}

impl Screen {
    fn get_and_init_once() -> &'static Option<Arc<RefCell<ScreenData>>> {
        unsafe {
            INIT.call_once(|| {
                SCREEN_DATA = Some(Arc::new(RefCell::new(ScreenData::default())));
            });
            &SCREEN_DATA
        }
    }

    fn get() -> &'static RefCell<ScreenData> {
        let screen_data = Screen::get_and_init_once();
        screen_data.as_ref().unwrap().borrow()
    }

    fn get_mut<'a>() -> RefMut<'a, ScreenData> {
        let screen_data = Screen::get_and_init_once();
        screen_data.as_ref().unwrap().borrow_mut()
    }

    pub fn create(width: u32, height: u32, scale_factor: f32, events_rw: EventsRw) {
        let mut screen_data = Screen::get_mut();
        screen_data.window_events = events_rw;
        screen_data.size = Vector2::new(width as _, height as _);
        screen_data.scale_factor = scale_factor;
    }

    pub fn update() {
        let mut inner = Screen::get().borrow_mut();
        let mut size = inner.size;
        let mut scale_factor = inner.scale_factor;
        {
            let events = inner.window_events.read().unwrap();
            if let Some(window_events) = events.read_all_events::<WindowEvent>() {
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

    pub fn get_draw_area() -> Vector4 {
        let inner = Screen::get().borrow();
        let size = inner.size;
        [0., 0., size.x as _, size.y as _].into()
    }
    pub fn get_size() -> Vector2 {
        let inner = Screen::get().borrow();
        inner.size
    }
    pub fn get_center() -> Vector2 {
        Screen::get_size() / 2.
    }
    pub fn get_scale_factor() -> f32 {
        let inner = Screen::get().borrow();
        inner.scale_factor
    }

    pub fn from_normalized_into_pixels(normalized_value: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            (normalized_value.x * inner.size.x as f32) as _,
            (normalized_value.y * inner.size.y as f32) as _,
        )
    }
    pub fn from_pixels_into_normalized(value_in_px: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            value_in_px.x as f32 / inner.size.x as f32,
            value_in_px.y as f32 / inner.size.y as f32,
        )
    }
    pub fn convert_size_into_pixels(value: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            (value.x * inner.size.x as f32 * 0.5) as _,
            (value.y * inner.size.y as f32 * 0.5) as _,
        )
    }
    pub fn convert_size_from_pixels(value_in_px: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            value_in_px.x as f32 * 2. / inner.size.x as f32,
            value_in_px.y as f32 * 2. / inner.size.y as f32,
        )
    }
    pub fn from_normalized_into_screen_space(normalized_pos: Vector2) -> Vector2 {
        [normalized_pos.x * 2.0 - 1., normalized_pos.y * 2.0 - 1.].into()
    }
    pub fn convert_from_pixels_into_screen_space(pos_in_px: Vector2) -> Vector2 {
        let normalized_pos =
            Screen::from_pixels_into_normalized([pos_in_px.x as _, pos_in_px.y as _].into());
        Screen::from_normalized_into_screen_space(normalized_pos)
    }

    pub fn from_screen_space_into_normalized(pos_in_screen_space: Vector2) -> Vector2 {
        [
            (pos_in_screen_space.x + 1.) * 0.5,
            (pos_in_screen_space.y + 1.) * 0.5,
        ]
        .into()
    }

    pub fn from_screen_space_into_pixels(pos_in_screen_space: Vector2) -> Vector2 {
        let normalized_pos = Screen::from_screen_space_into_normalized(pos_in_screen_space);
        let pos = Screen::from_normalized_into_pixels(normalized_pos);
        [pos.x as _, pos.y as _].into()
    }
}
