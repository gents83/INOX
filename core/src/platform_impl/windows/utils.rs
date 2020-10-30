
use super::externs::*;
use super::types::*;
use super::super::window::*;

pub fn handle_message( window : &mut Window ) -> bool {
    unsafe {
        let mut message : MSG = ::std::mem::MaybeUninit::zeroed().assume_init();
        if GetMessageW( &mut message as *mut MSG, window.handle.handle_impl.hwnd, 0, 0 ) > 0 {
            TranslateMessage( &message as *const MSG );
            DispatchMessageW( &message as *const MSG );

            true
        } else {
            false
        }
    }
}