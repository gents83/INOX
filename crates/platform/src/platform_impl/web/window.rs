use inox_messenger::MessageHubRc;
use std::path::Path;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use super::handle::*;
use crate::handle::*;
use crate::window::*;
use crate::{InputState, Key, MouseButton, MouseState};

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
        let canvas = document.get_element_by_id("canvas").unwrap();
        canvas.set_attribute("tabindex", "0").ok();
        canvas.set_attribute("data-raw-handle", "0").ok();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let device_pixel_ratio = window.device_pixel_ratio();
        *scale_factor = device_pixel_ratio.max(1.) as _;
        let window_width =
            (window.inner_width().unwrap().as_f64().unwrap()).max(*width as f64) as f32;
        let window_height =
            (window.inner_height().unwrap().as_f64().unwrap()).max(*height as f64) as f32;
        *width = window_width as u32;
        *height = window_height as u32;
        canvas.set_width(*width);
        canvas.set_height(*height);
        events_dispatcher.send_event(WindowEvent::PosChanged(0, 0));
        events_dispatcher.send_event(WindowEvent::SizeChanged(*width, *height));
        events_dispatcher.send_event(WindowEvent::ScaleFactorChanged(*scale_factor));

        Self::add_mouse_event_listener(events_dispatcher, &canvas, "mousemove", MouseState::Move);
        Self::add_mouse_event_listener(events_dispatcher, &canvas, "mousedown", MouseState::Down);
        Self::add_mouse_event_listener(events_dispatcher, &canvas, "mouseup", MouseState::Up);

        Self::add_key_event_listener(events_dispatcher, &canvas, "keyup", InputState::Released);
        Self::add_key_event_listener(events_dispatcher, &canvas, "keydown", InputState::Pressed);

        Handle {
            handle_impl: HandleImpl { id: 0, canvas },
        }
    }

    fn add_mouse_event_listener(
        events_dispatcher: &MessageHubRc,
        canvas: &web_sys::HtmlCanvasElement,
        event_name: &str,
        state: MouseState,
    ) {
        let events_dispatcher = events_dispatcher.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document.get_element_by_id("canvas").unwrap();
            let canvas: web_sys::HtmlCanvasElement =
                canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            let rect = canvas.get_bounding_client_rect();
            let width = canvas.width() as f32;
            let height = canvas.height() as f32;
            let x = event.offset_x() as f32 * (width / rect.width() as f32);
            let y = event.offset_y() as f32 * (height / rect.height() as f32);
            let button = match event.button() {
                0 => MouseButton::None,
                1 => MouseButton::Left,
                2 => MouseButton::Right,
                4 => MouseButton::Middle,
                index => MouseButton::Other(index.try_into().unwrap()),
            };
            events_dispatcher.send_event(crate::MouseEvent {
                x: x as _,
                y: y as _,
                normalized_x: x / width,
                normalized_y: y / height,
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
