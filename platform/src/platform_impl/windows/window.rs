use super::externs::*;
use super::handle::*;
use super::types::*;
use crate::ctypes::*;
use crate::events::*;
use crate::handle::*;
use crate::input::*;
use crate::window::*;

static mut EVENTS: *mut EventsRw = std::ptr::null_mut();

impl Window {
    pub fn create_handle(
        title: String,
        x: u32,
        y: u32,
        width: &mut u32,
        height: &mut u32,
        scale_factor: &mut f32,
        events: &mut EventsRw,
    ) -> Handle {
        unsafe {
            EVENTS = events as *mut EventsRw;
        };

        let mut title: Vec<u16> = title.encode_utf16().collect();
        title.push(0);

        unsafe {
            // Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
            let win_hinstance = GetModuleHandleW(std::ptr::null_mut());

            // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
            let wnd_class = WNDCLASSW {
                style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW, // Style
                lpfnWndProc: Some(Window::window_process), // The callbackfunction for any window event that can occur in our window!!! Here you could react to events like WM_SIZE or WM_QUIT.
                hInstance: win_hinstance, // The instance handle for our application which we can retrieve by calling GetModuleHandleW.
                lpszClassName: title.as_ptr(), // Our class name which needs to be a UTF-16 string (defined earlier before unsafe). as_ptr() (Rust's own function) returns a raw pointer to the slice's buffer
                cbClsExtra: 0,
                cbWndExtra: 0,
                hIcon: ::std::ptr::null_mut(),
                hCursor: ::std::ptr::null_mut(),
                hbrBackground: ::std::ptr::null_mut(),
                lpszMenuName: ::std::ptr::null_mut(),
            };

            // We have to register this class for Windows to use
            RegisterClassW(&wnd_class);

            SetProcessDpiAwareness(PROCESS_DPI_AWARENESS::PROCESS_PER_MONITOR_DPI_AWARE);
            let (dpi_x, _dpi_y) = Self::compute_dpi();
            *scale_factor = dpi_x as f32 / DEFAULT_DPI as f32;

            // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
            // Create a window based on registered class
            let win_handle = CreateWindowExW(
                0,                                // dwExStyle
                title.as_ptr(), // lpClassName, name of the class that we want to use for this window, which will be the same that we have registered before.
                title.as_ptr(), // lpWindowName
                WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
                x as i32,       // Int x
                y as i32,       // Int y
                *width as i32,  // Int nWidth
                *height as i32, // Int nHeight
                ::std::ptr::null_mut(), // hWndParent
                ::std::ptr::null_mut(), // hMenu
                win_hinstance,  // hInstance
                ::std::ptr::null_mut(),
            ); // lpParam

            Handle {
                handle_impl: HandleImpl {
                    hwnd: win_handle,
                    hinstance: win_hinstance,
                },
            }
        }
    }

    pub fn internal_update(handle: &Handle, events: &mut EventsRw) -> bool {
        unsafe {
            EVENTS = events as *mut EventsRw;

            let mut can_continue = true;
            let mut message: MSG = ::std::mem::MaybeUninit::zeroed().assume_init();
            while PeekMessageW(
                &mut message as *mut MSG,
                handle.handle_impl.hwnd,
                0,
                0,
                PM_REMOVE,
            ) > 0
            {
                TranslateMessage(&message as *const MSG);
                DispatchMessageW(&message as *const MSG);

                if message.message == WM_MOUSEMOVE
                    || message.message == WM_LBUTTONDOWN
                    || message.message == WM_LBUTTONUP
                    || message.message == WM_LBUTTONDBLCLK
                    || message.message == WM_RBUTTONDOWN
                    || message.message == WM_RBUTTONUP
                    || message.message == WM_RBUTTONDBLCLK
                    || message.message == WM_MBUTTONDOWN
                    || message.message == WM_MBUTTONUP
                    || message.message == WM_MBUTTONDBLCLK
                {
                    let mut mouse_pos = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut mouse_pos);
                    ScreenToClient(handle.handle_impl.hwnd, &mut mouse_pos);
                    let mut events = events.write().unwrap();
                    events.send_event(MouseEvent {
                        x: mouse_pos.x as f64,
                        y: mouse_pos.y as f64,
                        button: match message.message {
                            WM_LBUTTONDOWN | WM_LBUTTONUP | WM_LBUTTONDBLCLK => MouseButton::Left,
                            WM_RBUTTONDOWN | WM_RBUTTONUP | WM_RBUTTONDBLCLK => MouseButton::Right,
                            WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MBUTTONDBLCLK => MouseButton::Middle,
                            _ => MouseButton::None,
                        },
                        state: match message.message {
                            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => MouseState::Down,
                            WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP => MouseState::Up,
                            WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDBLCLK => {
                                MouseState::DoubleClick
                            }
                            _ => MouseState::Move,
                        },
                    });
                } else if message.message == WM_CHAR {
                    let char = message.wParam as INT;
                    let mut events = events.write().unwrap();
                    events.send_event(KeyTextEvent {
                        char: char as u8 as _,
                    });
                    can_continue = true;
                } else if message.message == WM_KEYDOWN
                    || message.message == WM_KEYUP
                    || message.message == WM_SYSCHAR
                    || message.message == WM_SYSDEADCHAR
                    || message.message == WM_SYSKEYDOWN
                    || message.message == WM_SYSKEYUP
                {
                    let char = message.wParam as INT;
                    let mut key = convert_key(char);
                    if key == Key::Unidentified {
                        key = convert_command(char);
                    }
                    let is_repeat = (message.lParam >> 30) & 1 == 1;
                    let mut events = events.write().unwrap();
                    events.send_event(KeyEvent {
                        code: key,
                        state: match message.message {
                            WM_KEYDOWN | WM_SYSCHAR | WM_SYSKEYDOWN => {
                                if is_repeat {
                                    InputState::Pressed
                                } else {
                                    InputState::JustPressed
                                }
                            }
                            WM_KEYUP | WM_SYSDEADCHAR | WM_SYSKEYUP => {
                                if is_repeat {
                                    InputState::Released
                                } else {
                                    InputState::JustReleased
                                }
                            }
                            _ => InputState::Invalid,
                        },
                    });
                    can_continue = true;
                }
            }
            can_continue
        }
    }

    fn compute_dpi() -> (UINT, UINT) {
        unsafe {
            let window = GetForegroundWindow();
            let monitor = MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST);
            let mut x: UINT = 0;
            let mut y: UINT = 0;

            GetDpiForMonitor(monitor, MONITOR_DPI_TYPE::MDT_EFFECTIVE_DPI, &mut x, &mut y);

            (x, y)
        }
    }

    unsafe extern "system" fn window_process(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DPICHANGED => {
                let rect: RECT = *(lparam as *const c_void as *const RECT);
                let dpi_x = LOWORD(wparam as _);
                let dpi_y = HIWORD(wparam as _);
                SetWindowPos(
                    hwnd,
                    0 as _, // No relative window
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
                if EVENTS != ::std::ptr::null_mut() {
                    let mut events = (*EVENTS).write().unwrap();
                    events.send_event(WindowEvent::DpiChanged(dpi_x as _, dpi_y as _));
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_SIZE => {
                let width = LOWORD(lparam as _);
                let height = HIWORD(lparam as _);
                if EVENTS != ::std::ptr::null_mut() {
                    let mut events = (*EVENTS).write().unwrap();
                    events.send_event(WindowEvent::SizeChanged(width as _, height as _));
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_MOVE => {
                let x = LOWORD(lparam as _);
                let y = HIWORD(lparam as _);
                if EVENTS != ::std::ptr::null_mut() {
                    let mut events = (*EVENTS).write().unwrap();
                    events.send_event(WindowEvent::PosChanged(x as _, y as _));
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            WM_DESTROY | WM_CLOSE | WM_QUIT | WM_NCDESTROY => {
                if EVENTS != ::std::ptr::null_mut() {
                    let mut events = (*EVENTS).write().unwrap();
                    events.send_event(WindowEvent::Close);
                }
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub fn convert_key(key: INT) -> Key {
    // VK_* codes are documented here https://msdn.microsoft.com/en-us/library/windows/desktop/dd375731(v=vs.85).aspx
    match key {
        VK_KEY_1 => Key::Key1,
        VK_KEY_2 => Key::Key2,
        VK_KEY_3 => Key::Key3,
        VK_KEY_4 => Key::Key4,
        VK_KEY_5 => Key::Key5,
        VK_KEY_6 => Key::Key6,
        VK_KEY_7 => Key::Key7,
        VK_KEY_8 => Key::Key8,
        VK_KEY_9 => Key::Key9,
        VK_KEY_0 => Key::Key0,
        VK_KEY_A => Key::A,
        VK_KEY_B => Key::B,
        VK_KEY_C => Key::C,
        VK_KEY_D => Key::D,
        VK_KEY_E => Key::E,
        VK_KEY_F => Key::F,
        VK_KEY_G => Key::G,
        VK_KEY_H => Key::H,
        VK_KEY_I => Key::I,
        VK_KEY_J => Key::J,
        VK_KEY_K => Key::K,
        VK_KEY_L => Key::L,
        VK_KEY_M => Key::M,
        VK_KEY_N => Key::N,
        VK_KEY_O => Key::O,
        VK_KEY_P => Key::P,
        VK_KEY_Q => Key::Q,
        VK_KEY_R => Key::R,
        VK_KEY_S => Key::S,
        VK_KEY_T => Key::T,
        VK_KEY_U => Key::U,
        VK_KEY_V => Key::V,
        VK_KEY_W => Key::W,
        VK_KEY_X => Key::X,
        VK_KEY_Y => Key::Y,
        VK_KEY_Z => Key::Z,
        VK_NUMPAD0 => Key::Numpad0,
        VK_NUMPAD1 => Key::Numpad1,
        VK_NUMPAD2 => Key::Numpad2,
        VK_NUMPAD3 => Key::Numpad3,
        VK_NUMPAD4 => Key::Numpad4,
        VK_NUMPAD5 => Key::Numpad5,
        VK_NUMPAD6 => Key::Numpad6,
        VK_NUMPAD7 => Key::Numpad7,
        VK_NUMPAD8 => Key::Numpad8,
        VK_NUMPAD9 => Key::Numpad9,
        VK_MENU => Key::Alt,
        VK_RMENU => Key::AltGraph,
        VK_CAPITAL => Key::CapsLock,
        VK_CONTROL => Key::Control,
        VK_LWIN => Key::Meta,
        VK_RWIN => Key::Meta,
        VK_NUMLOCK => Key::NumLock,
        VK_SCROLL => Key::ScrollLock,
        VK_SHIFT => Key::Shift,
        VK_LSHIFT => Key::Shift,
        VK_RSHIFT => Key::Shift,
        VK_RETURN => Key::Enter,
        VK_TAB => Key::Tab,
        VK_SPACE => Key::Space,
        VK_DOWN => Key::ArrowDown,
        VK_LEFT => Key::ArrowLeft,
        VK_RIGHT => Key::ArrowRight,
        VK_UP => Key::ArrowUp,
        VK_END => Key::End,
        VK_HOME => Key::Home,
        VK_NEXT => Key::PageDown,
        VK_PRIOR => Key::PageUp,
        VK_BACK => Key::Backspace,
        VK_CLEAR => Key::Clear,
        VK_OEM_CLEAR => Key::Clear,
        VK_CRSEL => Key::CrSel,
        VK_DELETE => Key::Delete,
        VK_EREOF => Key::EraseEof,
        VK_EXSEL => Key::ExSel,
        VK_INSERT => Key::Insert,
        VK_ACCEPT => Key::Accept,
        VK_APPS => Key::ContextMenu,
        VK_ESCAPE => Key::Escape,
        VK_EXECUTE => Key::Execute,
        VK_OEM_FINISH => Key::Finish,
        VK_HELP => Key::Help,
        VK_PAUSE => Key::Pause,
        VK_PLAY => Key::Play,
        VK_SELECT => Key::Select,
        VK_SNAPSHOT => Key::PrintScreen,
        VK_SLEEP => Key::Standby,
        VK_OEM_ATTN => Key::Alphanumeric,
        VK_CONVERT => Key::Convert,
        VK_FINAL => Key::FinalMode,
        VK_MODECHANGE => Key::ModeChange,
        VK_NONCONVERT => Key::NonConvert,
        VK_PROCESSKEY => Key::Process,
        VK_F1 => Key::F1,
        VK_F2 => Key::F2,
        VK_F3 => Key::F3,
        VK_F4 => Key::F4,
        VK_F5 => Key::F5,
        VK_F6 => Key::F6,
        VK_F7 => Key::F7,
        VK_F8 => Key::F8,
        VK_F9 => Key::F9,
        VK_F10 => Key::F10,
        VK_F11 => Key::F11,
        VK_F12 => Key::F12,
        VK_F13 => Key::F13,
        VK_F14 => Key::F14,
        VK_F15 => Key::F15,
        VK_F16 => Key::F16,
        VK_F17 => Key::F17,
        VK_F18 => Key::F18,
        VK_F19 => Key::F19,
        VK_F20 => Key::F20,
        VK_MEDIA_PLAY_PAUSE => Key::MediaPlayPause,
        VK_MEDIA_STOP => Key::MediaStop,
        VK_MEDIA_NEXT_TRACK => Key::MediaTrackNext,
        VK_MEDIA_PREV_TRACK => Key::MediaTrackPrevious,
        VK_VOLUME_DOWN => Key::AudioVolumeDown,
        VK_VOLUME_MUTE => Key::AudioVolumeMute,
        VK_VOLUME_UP => Key::AudioVolumeUp,
        VK_ZOOM => Key::ZoomToggle,
        VK_LAUNCH_MAIL => Key::LaunchMail,
        VK_LAUNCH_MEDIA_SELECT => Key::LaunchMediaPlayer,
        VK_LAUNCH_APP1 => Key::LaunchApplication1,
        VK_LAUNCH_APP2 => Key::LaunchApplication2,
        VK_BROWSER_BACK => Key::BrowserBack,
        VK_BROWSER_FAVORITES => Key::BrowserFavorites,
        VK_BROWSER_FORWARD => Key::BrowserForward,
        VK_BROWSER_HOME => Key::BrowserHome,
        VK_BROWSER_REFRESH => Key::BrowserRefresh,
        VK_BROWSER_SEARCH => Key::BrowserSearch,
        VK_BROWSER_STOP => Key::BrowserStop,
        VK_DECIMAL => Key::Decimal,
        VK_MULTIPLY => Key::Multiply,
        VK_ADD => Key::Add,
        VK_DIVIDE => Key::Divide,
        VK_SUBTRACT => Key::Subtract,
        VK_SEPARATOR => Key::Separator,
        _ => Key::Unidentified,
    }
}

pub fn convert_command(key: INT) -> Key {
    let app_command = GET_APPCOMMAND_LPARAM(key as _);
    match app_command {
        APPCOMMAND_COPY => Key::Copy,
        APPCOMMAND_CUT => Key::Cut,
        APPCOMMAND_PASTE => Key::Paste,
        APPCOMMAND_REDO => Key::Redo,
        APPCOMMAND_UNDO => Key::Undo,
        APPCOMMAND_FIND => Key::Find,
        APPCOMMAND_HELP => Key::Help,
        APPCOMMAND_MEDIA_CHANNEL_DOWN => Key::ChannelDown,
        APPCOMMAND_MEDIA_CHANNEL_UP => Key::ChannelUp,
        APPCOMMAND_MEDIA_FAST_FORWARD => Key::MediaFastForward,
        APPCOMMAND_MEDIA_PAUSE => Key::MediaPause,
        APPCOMMAND_MEDIA_PLAY => Key::MediaPlay,
        APPCOMMAND_MEDIA_PLAY_PAUSE => Key::MediaPlayPause,
        APPCOMMAND_MEDIA_RECORD => Key::MediaRecord,
        APPCOMMAND_MEDIA_REWIND => Key::MediaRewind,
        APPCOMMAND_MEDIA_STOP => Key::MediaStop,
        APPCOMMAND_MEDIA_NEXTTRACK => Key::MediaTrackNext,
        APPCOMMAND_MEDIA_PREVIOUSTRACK => Key::MediaTrackPrevious,
        APPCOMMAND_BASS_DOWN => Key::AudioBassDown,
        APPCOMMAND_BASS_BOOST => Key::AudioBassBoostToggle,
        APPCOMMAND_BASS_UP => Key::AudioBassUp,
        APPCOMMAND_TREBLE_DOWN => Key::AudioTrebleDown,
        APPCOMMAND_TREBLE_UP => Key::AudioTrebleUp,
        APPCOMMAND_VOLUME_DOWN => Key::AudioVolumeDown,
        APPCOMMAND_VOLUME_MUTE => Key::AudioVolumeMute,
        APPCOMMAND_VOLUME_UP => Key::AudioVolumeUp,
        APPCOMMAND_MIC_ON_OFF_TOGGLE => Key::MicrophoneToggle,
        APPCOMMAND_MICROPHONE_VOLUME_DOWN => Key::MicrophoneVolumeDown,
        APPCOMMAND_MICROPHONE_VOLUME_MUTE => Key::MicrophoneVolumeMute,
        APPCOMMAND_MICROPHONE_VOLUME_UP => Key::MicrophoneVolumeUp,
        APPCOMMAND_CORRECTION_LIST => Key::SpeechCorrectionList,
        APPCOMMAND_DICTATE_OR_COMMAND_CONTROL_TOGGLE => Key::SpeechInputToggle,
        APPCOMMAND_CLOSE => Key::Close,
        APPCOMMAND_NEW => Key::New,
        APPCOMMAND_OPEN => Key::Open,
        APPCOMMAND_PRINT => Key::Print,
        APPCOMMAND_SAVE => Key::Save,
        APPCOMMAND_SPELL_CHECK => Key::SpellCheck,
        APPCOMMAND_FORWARD_MAIL => Key::MailForward,
        APPCOMMAND_REPLY_TO_MAIL => Key::MailReply,
        APPCOMMAND_SEND_MAIL => Key::MailSend,
        APPCOMMAND_LAUNCH_MAIL => Key::LaunchMail,
        APPCOMMAND_LAUNCH_MEDIA_SELECT => Key::LaunchMediaPlayer,
        APPCOMMAND_LAUNCH_APP1 => Key::LaunchApplication1,
        APPCOMMAND_LAUNCH_APP2 => Key::LaunchApplication2,
        APPCOMMAND_BROWSER_BACKWARD => Key::BrowserBack,
        APPCOMMAND_BROWSER_FAVORITES => Key::BrowserFavorites,
        APPCOMMAND_BROWSER_FORWARD => Key::BrowserForward,
        APPCOMMAND_BROWSER_HOME => Key::BrowserHome,
        APPCOMMAND_BROWSER_REFRESH => Key::BrowserRefresh,
        APPCOMMAND_BROWSER_SEARCH => Key::BrowserSearch,
        APPCOMMAND_BROWSER_STOP => Key::BrowserStop,
        _ => Key::Unidentified,
    }
}
