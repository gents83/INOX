use std::{borrow::Borrow, cell::RefCell, sync::Arc, sync::Once};

use nrg_math::{Vector2, Vector4};

const DEFAULT_WIDTH: u32 = 1920;
const DEFAULT_HEIGTH: u32 = 1080;

struct ScreenData {
    size: Vector2,
    scale_factor: f32,
}

impl Default for ScreenData {
    #[inline]
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            size: Vector2::new(DEFAULT_WIDTH as _, DEFAULT_HEIGTH as _),
        }
    }
}

static mut SCREEN_DATA: Option<Arc<RefCell<ScreenData>>> = None;
static mut INIT: Once = Once::new();

pub struct Screen {}

impl Screen {
    #[inline]
    fn get_and_init_once() -> &'static Option<Arc<RefCell<ScreenData>>> {
        unsafe {
            INIT.call_once(|| {
                SCREEN_DATA = Some(Arc::new(RefCell::new(ScreenData::default())));
            });
            &SCREEN_DATA
        }
    }

    #[inline]
    fn get() -> &'static RefCell<ScreenData> {
        let screen_data = Screen::get_and_init_once();
        screen_data.as_ref().unwrap().borrow()
    }

    #[inline]
    pub fn create(width: u32, height: u32, scale_factor: f32) {
        let mut screen_data = Screen::get().borrow_mut();
        screen_data.size = Vector2::new(width as _, height as _);
        screen_data.scale_factor = scale_factor;
    }

    #[inline]
    pub fn change_scale_factor(scale_factor: f32) {
        let mut inner = Screen::get().borrow_mut();
        inner.scale_factor = scale_factor;
    }

    #[inline]
    pub fn change_size(width: u32, height: u32) {
        let mut inner = Screen::get().borrow_mut();
        inner.size = Vector2::new(width as _, height as _);
    }

    #[inline]
    pub fn get_draw_area() -> Vector4 {
        let inner = Screen::get().borrow();
        let size = inner.size;
        [0., 0., size.x as _, size.y as _].into()
    }

    #[inline]
    pub fn get_size() -> Vector2 {
        let inner = Screen::get().borrow();
        inner.size
    }

    #[inline]
    pub fn get_center() -> Vector2 {
        Screen::get_size() / 2.
    }

    #[inline]
    pub fn get_scale_factor() -> f32 {
        let inner = Screen::get().borrow();
        inner.scale_factor
    }

    #[inline]
    pub fn from_normalized_into_pixels(normalized_value: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            (normalized_value.x * inner.size.x as f32) as _,
            (normalized_value.y * inner.size.y as f32) as _,
        )
    }

    #[inline]
    pub fn from_pixels_into_normalized(value_in_px: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            value_in_px.x as f32 / inner.size.x as f32,
            value_in_px.y as f32 / inner.size.y as f32,
        )
    }

    #[inline]
    pub fn convert_size_into_pixels(value: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            (value.x * inner.size.x as f32 * 0.5) as _,
            (value.y * inner.size.y as f32 * 0.5) as _,
        )
    }

    #[inline]
    pub fn convert_size_from_pixels(value_in_px: Vector2) -> Vector2 {
        let inner = Screen::get().borrow();
        Vector2::new(
            value_in_px.x as f32 * 2. / inner.size.x as f32,
            value_in_px.y as f32 * 2. / inner.size.y as f32,
        )
    }

    #[inline]
    pub fn from_normalized_into_screen_space(normalized_pos: Vector2) -> Vector2 {
        [normalized_pos.x * 2.0 - 1., normalized_pos.y * 2.0 - 1.].into()
    }

    #[inline]
    pub fn convert_from_pixels_into_screen_space(pos_in_px: Vector2) -> Vector2 {
        let normalized_pos =
            Screen::from_pixels_into_normalized([pos_in_px.x as _, pos_in_px.y as _].into());
        Screen::from_normalized_into_screen_space(normalized_pos)
    }

    #[inline]
    pub fn from_screen_space_into_normalized(pos_in_screen_space: Vector2) -> Vector2 {
        [
            (pos_in_screen_space.x + 1.) * 0.5,
            (pos_in_screen_space.y + 1.) * 0.5,
        ]
        .into()
    }

    #[inline]
    pub fn from_screen_space_into_pixels(pos_in_screen_space: Vector2) -> Vector2 {
        let normalized_pos = Screen::from_screen_space_into_normalized(pos_in_screen_space);
        let pos = Screen::from_normalized_into_pixels(normalized_pos);
        [pos.x as _, pos.y as _].into()
    }
}
