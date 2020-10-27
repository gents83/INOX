
use super::types::*;

pub fn create_window( name : &str, title : &str ) -> Window {
    
    let mut name: Vec<u16> = name.encode_utf16().collect();
    let mut title: Vec<u16> = title.encode_utf16().collect();
    name.push(0);
    title.push(0);

    unsafe {
    	// Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
        let hinstance = GetModuleHandleW( std::ptr::null_mut() );

        // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
        let wnd_class = WNDCLASSW {
            style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,		// Style
            lpfnWndProc : Some( DefWindowProcW ),			// The callbackfunction for any window event that can occur in our window!!! Here you could react to events like WM_SIZE or WM_QUIT.
            hInstance : hinstance,							// The instance handle for our application which we can retrieve by calling GetModuleHandleW.
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
            CW_USEDEFAULT,						// Int x
            CW_USEDEFAULT,						// Int y
            CW_USEDEFAULT,						// Int nWidth
            CW_USEDEFAULT,						// Int nHeight
            ::std::ptr::null_mut(),							// hWndParent
            ::std::ptr::null_mut(),							// hMenu
            hinstance,							// hInstance
            ::std::ptr::null_mut() );						// lpParam

        Window{ handle : win_handle }
    }
}

pub fn handle_message( window : &mut Window ) -> bool {
    unsafe {
        let mut message : MSG = ::std::mem::uninitialized();
        if GetMessageW( &mut message as *mut MSG, window.handle, 0, 0 ) > 0 {
            TranslateMessage( &message as *const MSG );
            DispatchMessageW( &message as *const MSG );

            true
        } else {
            false
        }
    }
}