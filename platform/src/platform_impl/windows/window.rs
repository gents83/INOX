

use super::externs::*;
use super::types::*;
use super::handle::*;
use crate::handle::*;
use crate::window::*;

impl Window {
    pub fn new(
        _name: String,
        _title: String,
        _x: u32,
        _y: u32,
        _width: u32,
        _height: u32) -> Window {
            
        let mut name: Vec<u16> = _name.encode_utf16().collect();
        let mut title: Vec<u16> = _title.encode_utf16().collect();
        name.push(0);
        title.push(0);

        unsafe {
            // Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
            let win_hinstance = GetModuleHandleW( std::ptr::null_mut() );

            // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
            let wnd_class = WNDCLASSW {
                style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,		// Style
                lpfnWndProc : Some( Window::window_process ),			// The callbackfunction for any window event that can occur in our window!!! Here you could react to events like WM_SIZE or WM_QUIT.
                hInstance : win_hinstance,							// The instance handle for our application which we can retrieve by calling GetModuleHandleW.
                lpszClassName : name.as_ptr(),					// Our class name which needs to be a UTF-16 string (defined earlier before unsafe). as_ptr() (Rust's own function) returns a raw pointer to the slice's buffer
                cbClsExtra : 0,									
                cbWndExtra : 0,
                hIcon: ::std::ptr::null_mut(),
                hCursor: ::std::ptr::null_mut(),
                hbrBackground: ::std::ptr::null_mut(),
                lpszMenuName: ::std::ptr::null_mut(),
            };

            // We have to register this class for Windows to use
            RegisterClassW( &wnd_class );

            // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
            // Create a window based on registered class
            let win_handle = CreateWindowExW(
                0,									// dwExStyle 
                name.as_ptr(),						// lpClassName, name of the class that we want to use for this window, which will be the same that we have registered before.
                title.as_ptr(),						// lpWindowName
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,	// dwStyle
                _x as i32,						// Int x
                _y as i32,						// Int y
                _width as i32,						// Int nWidth
                _height as i32,						// Int nHeight
                ::std::ptr::null_mut(),							// hWndParent
                ::std::ptr::null_mut(),							// hMenu
                win_hinstance,							// hInstance
                ::std::ptr::null_mut() );						// lpParam

            Window {
                handle : Handle { 
                    handle_impl: HandleImpl { 
                        hwnd : win_handle, 
                        hinstance : win_hinstance 
                    }, 
                },
                x: _x,
                y: _y,
                width: _width,
                height: _height,
                name: _name,
                title: _title
            }        
        }
    }

    pub fn internal_update(&self) -> bool {
        unsafe {
            let mut is_ended = false;
            let mut message : MSG = ::std::mem::MaybeUninit::zeroed().assume_init();
            while PeekMessageW(&mut message as *mut MSG, ::std::ptr::null_mut(), 0, 0, PM_REMOVE) > 0 {
                TranslateMessage( &message as *const MSG );
                DispatchMessageW( &message as *const MSG );
                
                if message.message == WM_DESTROY ||
                   message.message == WM_CLOSE ||
                   message.message == WM_QUIT ||
                   message.message == WM_NCDESTROY {
                    is_ended = true;
                }
            }
            is_ended
        } 
    }

    unsafe extern "system"
    fn window_process(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0); 
                0
            },
            _ => {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
    }
}