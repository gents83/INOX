use inox_messenger::MessageHubRc;
use std::path::Path;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use super::handle::*;
use crate::handle::*;
use crate::window::*;
use crate::{InputState, Key, MouseState};

const CANVAS_HANDLE_ID: u32 = 1;

fn positive_dimension(value: Option<f64>, fallback: u32, scale_factor: f32) -> u32 {
    let css_dimension = value
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(fallback.max(1) as f64);
    let pixels = css_dimension * f64::from(scale_factor);
    pixels.ceil().clamp(1.0, f64::from(u32::MAX)) as u32
}

fn canvas_dimensions(
    window: &web_sys::Window,
    requested_width: u32,
    requested_height: u32,
) -> (u32, u32, f32) {
    let device_pixel_ratio = window.device_pixel_ratio();
    let scale_factor = if device_pixel_ratio.is_finite() && device_pixel_ratio > 0.0 {
        device_pixel_ratio
    } else {
        1.0
    }
    .max(1.0) as f32;
    let width = positive_dimension(
        window.inner_width().ok().and_then(|value| value.as_f64()),
        requested_width,
        scale_factor,
    );
    let height = positive_dimension(
        window.inner_height().ok().and_then(|value| value.as_f64()),
        requested_height,
        scale_factor,
    );
    (width, height, scale_factor)
}

impl Window {
    pub fn create_handle(
        _title: String,
        _x: u32,
        _y: u32,
        width: &mut u32,
        height: &mut u32,
        scale_factor: &mut f32,
        _icon_path: &Path,
        events_dispatcher: &MessageHubRc,
    ) -> Handle {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let element = document.get_element_by_id("canvas").unwrap();
        element.set_attribute("tabindex", "0").ok();
        element
            .set_attribute("data-raw-handle", &CANVAS_HANDLE_ID.to_string())
            .ok();
        let canvas: web_sys::HtmlCanvasElement = element.dyn_into().unwrap();
        let (canvas_width, canvas_height, canvas_scale_factor) =
            canvas_dimensions(&window, *width, *height);
        *width = canvas_width;
        *height = canvas_height;
        *scale_factor = canvas_scale_factor;
        canvas.set_width(*width);
        canvas.set_height(*height);
        // Keep ownership and configured device-pixel dimensions observable to
        // browser smoke without requiring synchronous GPU inspection.
        canvas.set_attribute("data-inox-smoke-canvas", "owned").ok();
        canvas
            .set_attribute("data-inox-smoke-width", &width.to_string())
            .ok();
        canvas
            .set_attribute("data-inox-smoke-height", &height.to_string())
            .ok();
        events_dispatcher.send_event(WindowEvent::PosChanged(0, 0));
        events_dispatcher.send_event(WindowEvent::SizeChanged(*width, *height));
        events_dispatcher.send_event(WindowEvent::ScaleFactorChanged(*scale_factor));

        Self::add_mouse_event_listener(events_dispatcher, &canvas, "mousemove", MouseState::Move);
        Self::add_mouse_event_listener(events_dispatcher, &canvas, "mousedown", MouseState::Down);
        Self::add_mouse_event_listener(events_dispatcher, &canvas, "mouseup", MouseState::Up);
        Self::add_key_event_listener(events_dispatcher, &canvas, "keyup", InputState::Released);
        Self::add_key_event_listener(events_dispatcher, &canvas, "keydown", InputState::Pressed);

        let resize_events = events_dispatcher.clone();
        let resize_canvas = canvas.clone();
        let resize_window = window.clone();
        let resize_closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let (width, height, scale_factor) = canvas_dimensions(&resize_window, 1, 1);
            resize_canvas.set_width(width);
            resize_canvas.set_height(height);
            resize_canvas
                .set_attribute("data-inox-smoke-width", &width.to_string())
                .ok();
            resize_canvas
                .set_attribute("data-inox-smoke-height", &height.to_string())
                .ok();
            resize_events.send_event(WindowEvent::SizeChanged(width, height));
            resize_events.send_event(WindowEvent::ScaleFactorChanged(scale_factor));
        }) as Box<dyn FnMut(_)>);
        window
            .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
            .ok();
        resize_closure.forget();

        Handle {
            handle_impl: HandleImpl {
                id: CANVAS_HANDLE_ID,
                canvas,
            },
        }
    }

    fn add_mouse_event_listener(
        events_dispatcher: &MessageHubRc,
        canvas: &web_sys::HtmlCanvasElement,
        event_name: &str,
        state: MouseState,
    ) {
        let events_dispatcher = events_dispatcher.clone();
        let callback_canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let rect = callback_canvas.get_bounding_client_rect();
            let (x, y, normalized_x, normalized_y) = map_canvas_mouse_coordinates(
                event.client_x() as f64,
                event.client_y() as f64,
                rect.left(),
                rect.top(),
                rect.width(),
                rect.height(),
                callback_canvas.width(),
                callback_canvas.height(),
            );
            let button = canvas_mouse_button(state, event.button());
            events_dispatcher.send_event(crate::MouseEvent {
                x,
                y,
                normalized_x,
                normalized_y,
                button,
                state,
            });
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())
            .ok();
        closure.forget();
    }

    fn add_key_event_listener(
        events_dispatcher: &MessageHubRc,
        canvas: &web_sys::HtmlCanvasElement,
        event_name: &str,
        state: InputState,
    ) {
        let events_dispatcher = events_dispatcher.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            events_dispatcher.send_event(crate::KeyEvent {
                code: convert_key(&event.code()),
                state,
            });
        }) as Box<dyn FnMut(_)>);
        canvas
            .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())
            .ok();
        closure.forget();
    }

    pub fn change_title(_handle: &Handle, _title: &str) {}
    pub fn change_visibility(_handle: &Handle, _is_visible: bool) {}

    pub fn change_position(_handle: &Handle, _x: u32, _y: u32) {}

    pub fn change_size(_handle: &Handle, _width: u32, _height: u32) {}

    #[inline]
    pub fn internal_update(_handle: &Handle) -> bool {
        true
    }
}

#[inline]
pub fn convert_key(key: &str) -> Key {
    match key {
        "Digit1" => Key::Key1,
        "Digit2" => Key::Key2,
        "Digit3" => Key::Key3,
        "Digit4" => Key::Key4,
        "Digit5" => Key::Key5,
        "Digit6" => Key::Key6,
        "Digit7" => Key::Key7,
        "Digit8" => Key::Key8,
        "Digit9" => Key::Key9,
        "Digit0" => Key::Key0,
        "KeyA" => Key::A,
        "KeyB" => Key::B,
        "KeyC" => Key::C,
        "KeyD" => Key::D,
        "KeyE" => Key::E,
        "KeyF" => Key::F,
        "KeyG" => Key::G,
        "KeyH" => Key::H,
        "KeyI" => Key::I,
        "KeyJ" => Key::J,
        "KeyK" => Key::K,
        "KeyL" => Key::L,
        "KeyM" => Key::M,
        "KeyN" => Key::N,
        "KeyO" => Key::O,
        "KeyP" => Key::P,
        "KeyQ" => Key::Q,
        "KeyR" => Key::R,
        "KeyS" => Key::S,
        "KeyT" => Key::T,
        "KeyU" => Key::U,
        "KeyV" => Key::V,
        "KeyW" => Key::W,
        "KeyX" => Key::X,
        "KeyY" => Key::Y,
        "KeyZ" => Key::Z,
        "Escape" => Key::Escape,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "F13" => Key::F13,
        "F14" => Key::F14,
        "F15" => Key::F15,
        "F16" => Key::F16,
        "F17" => Key::F17,
        "F18" => Key::F18,
        "F19" => Key::F19,
        "F20" => Key::F20,
        "F21" => Key::Soft1,
        "F22" => Key::Soft2,
        "F23" => Key::Soft3,
        "F24" => Key::Soft4,
        "PrintScreen" => Key::PrintScreen,
        "ScrollLock" => Key::ScrollLock,
        "Pause" => Key::Pause,
        "Insert" => Key::Insert,
        "Home" => Key::Home,
        "Delete" => Key::Delete,
        "End" => Key::End,
        "PageDown" => Key::PageDown,
        "PageUp" => Key::PageUp,
        "ArrowLeft" => Key::ArrowLeft,
        "ArrowUp" => Key::ArrowUp,
        "ArrowRight" => Key::ArrowRight,
        "ArrowDown" => Key::ArrowDown,
        "Backspace" => Key::Backspace,
        "Enter" => Key::Enter,
        "Space" => Key::Space,
        "Compose" => Key::Compose,
        "Caret" => Key::Unidentified,
        "NumLock" => Key::NumLock,
        "Numpad0" => Key::Numpad0,
        "Numpad1" => Key::Numpad1,
        "Numpad2" => Key::Numpad2,
        "Numpad3" => Key::Numpad3,
        "Numpad4" => Key::Numpad4,
        "Numpad5" => Key::Numpad5,
        "Numpad6" => Key::Numpad6,
        "Numpad7" => Key::Numpad7,
        "Numpad8" => Key::Numpad8,
        "Numpad9" => Key::Numpad9,
        "Apps" => Key::AppSwitch,
        "Backslash" => Key::Divide,
        "Calculator" => Key::LaunchCalculator,
        "Capital" => Key::CapsLock,
        "Convert" => Key::Convert,
        "NumpadDecimal" => Key::Decimal,
        "NumpadDivide" => Key::Divide,
        "Kana" => Key::KanaMode,
        "Kanji" => Key::KanjiMode,
        "AltLeft" => Key::Alt,
        "ControlLeft" => Key::Control,
        "ShiftLeft" => Key::Shift,
        "MetaLeft" => Key::Meta,
        "Mail" => Key::LaunchMail,
        "MediaSelect" => Key::MediaPlay,
        "MediaStop" => Key::MediaStop,
        "Minus" => Key::Subtract,
        "NumpadMultiply" => Key::Multiply,
        "Mute" => Key::AudioVolumeMute,
        "LaunchMyComputer" => Key::LaunchMyComputer,
        "NavigateForward" => Key::BrowserForward,
        "NavigateBackward" => Key::BrowserBack,
        "NextTrack" => Key::MediaTrackNext,
        "NoConvert" => Key::NonConvert,
        "Comma" => Key::Decimal,
        "NumpadComma" => Key::Separator,
        "NumpadEnter" => Key::Key11,
        "NumpadEquals" => Key::Key12,
        "Period" => Key::Separator,
        "PlayPause" => Key::MediaPlayPause,
        "Power" => Key::Power,
        "PrevTrack" => Key::MediaTrackPrevious,
        "AltRight" => Key::AltGraph,
        "ControlRight" => Key::ContextMenu,
        "ShiftRight" => Key::Shift,
        "MetaRight" => Key::Meta,
        "Sleep" => Key::Standby,
        "Stop" => Key::MediaStop,
        "NumpadSubtract" => Key::Subtract,
        "Tab" => Key::Tab,
        "AudioVolumeDown" => Key::AudioVolumeDown,
        "AudioVolumeUp" => Key::AudioVolumeUp,
        "Wake" => Key::WakeUp,
        "WebBack" => Key::BrowserBack,
        "WebFavorites" => Key::BrowserFavorites,
        "WebForward" => Key::BrowserForward,
        "WebHome" => Key::BrowserHome,
        "WebRefresh" => Key::BrowserRefresh,
        "WebSearch" => Key::BrowserSearch,
        "WebStop" => Key::BrowserStop,
        _ => Key::Unidentified,
    }
}
