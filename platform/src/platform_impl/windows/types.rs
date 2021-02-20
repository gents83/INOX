#![allow(
    bad_style,
    overflowing_literals,
    dead_code,
    improper_ctypes,
    improper_ctypes_definitions
)]

use std::os::raw::c_ulonglong;

use super::externs::*;
use crate::ctypes::*;
use crate::declare_handle;

declare_handle! {HWND, HWND__}
declare_handle! {HINSTANCE, HINSTANCE__}
declare_handle! {HICON, HICON__}
declare_handle! {HBRUSH, HBRUSH__}
declare_handle! {HMENU, HMENU__}

pub type HANDLE = *mut c_void;
pub type PHANDLE = *mut HANDLE;
pub type HMODULE = HINSTANCE;
pub type HCURSOR = HICON;
pub type HRESULT = c_long;
pub type wchar_t = u16;
pub type BOOL = c_int;
pub type CHAR = c_char;
pub type WCHAR = wchar_t;
pub type WORD = c_ushort;
pub type DWORD = c_ulong;
pub type LPDWORD = *mut DWORD;
pub type INT = c_int;
pub type UINT = c_uint;
pub type LONG = c_long;
pub type UINT_PTR = usize;
pub type LONG_PTR = isize;
pub type ULONG_PTR = c_ulonglong;
pub type ATOM = WORD;
pub type LPCSTR = *const CHAR;
pub type LPCWSTR = *const WCHAR;
pub type WPARAM = UINT_PTR;
pub type LPARAM = LONG_PTR;
pub type LRESULT = LONG_PTR;
pub type LPVOID = *mut ::std::ffi::c_void;
pub type LPLONG = *mut c_long;
pub type LPMSG = *mut MSG;

pub const KF_EXTENDED: WORD = 0x0100;
pub const KF_DLGMODE: WORD = 0x0800;
pub const KF_MENUMODE: WORD = 0x1000;
pub const KF_ALTDOWN: WORD = 0x2000;
pub const KF_REPEAT: WORD = 0x4000;
pub const KF_UP: WORD = 0x8000;
pub const VK_KEY_0: c_int = 0x0030;
pub const VK_KEY_1: c_int = 0x0031;
pub const VK_KEY_2: c_int = 0x0032;
pub const VK_KEY_3: c_int = 0x0033;
pub const VK_KEY_4: c_int = 0x0034;
pub const VK_KEY_5: c_int = 0x0035;
pub const VK_KEY_6: c_int = 0x0036;
pub const VK_KEY_7: c_int = 0x0037;
pub const VK_KEY_8: c_int = 0x0038;
pub const VK_KEY_9: c_int = 0x0039;
pub const VK_KEY_A: c_int = 0x0041;
pub const VK_KEY_B: c_int = 0x0042;
pub const VK_KEY_C: c_int = 0x0043;
pub const VK_KEY_D: c_int = 0x0044;
pub const VK_KEY_E: c_int = 0x0045;
pub const VK_KEY_F: c_int = 0x0046;
pub const VK_KEY_G: c_int = 0x0047;
pub const VK_KEY_H: c_int = 0x0048;
pub const VK_KEY_I: c_int = 0x0049;
pub const VK_KEY_J: c_int = 0x004A;
pub const VK_KEY_K: c_int = 0x004B;
pub const VK_KEY_L: c_int = 0x004C;
pub const VK_KEY_M: c_int = 0x004D;
pub const VK_KEY_N: c_int = 0x004E;
pub const VK_KEY_O: c_int = 0x004F;
pub const VK_KEY_P: c_int = 0x0050;
pub const VK_KEY_Q: c_int = 0x0051;
pub const VK_KEY_R: c_int = 0x0052;
pub const VK_KEY_S: c_int = 0x0053;
pub const VK_KEY_T: c_int = 0x0054;
pub const VK_KEY_U: c_int = 0x0055;
pub const VK_KEY_V: c_int = 0x0056;
pub const VK_KEY_W: c_int = 0x0057;
pub const VK_KEY_X: c_int = 0x0058;
pub const VK_KEY_Y: c_int = 0x0059;
pub const VK_KEY_Z: c_int = 0x005A;

pub const VK_LBUTTON: c_int = 0x01;
pub const VK_RBUTTON: c_int = 0x02;
pub const VK_CANCEL: c_int = 0x03;
pub const VK_MBUTTON: c_int = 0x04;
pub const VK_XBUTTON1: c_int = 0x05;
pub const VK_XBUTTON2: c_int = 0x06;
pub const VK_BACK: c_int = 0x08;
pub const VK_TAB: c_int = 0x09;
pub const VK_CLEAR: c_int = 0x0C;
pub const VK_RETURN: c_int = 0x0D;
pub const VK_SHIFT: c_int = 0x10;
pub const VK_CONTROL: c_int = 0x11;
pub const VK_MENU: c_int = 0x12;
pub const VK_PAUSE: c_int = 0x13;
pub const VK_CAPITAL: c_int = 0x14;
pub const VK_KANA: c_int = 0x15;
pub const VK_HANGEUL: c_int = 0x15;
pub const VK_HANGUL: c_int = 0x15;
pub const VK_JUNJA: c_int = 0x17;
pub const VK_FINAL: c_int = 0x18;
pub const VK_HANJA: c_int = 0x19;
pub const VK_KANJI: c_int = 0x19;
pub const VK_ESCAPE: c_int = 0x1B;
pub const VK_CONVERT: c_int = 0x1C;
pub const VK_NONCONVERT: c_int = 0x1D;
pub const VK_ACCEPT: c_int = 0x1E;
pub const VK_MODECHANGE: c_int = 0x1F;
pub const VK_SPACE: c_int = 0x20;
pub const VK_PRIOR: c_int = 0x21;
pub const VK_NEXT: c_int = 0x22;
pub const VK_END: c_int = 0x23;
pub const VK_HOME: c_int = 0x24;
pub const VK_LEFT: c_int = 0x25;
pub const VK_UP: c_int = 0x26;
pub const VK_RIGHT: c_int = 0x27;
pub const VK_DOWN: c_int = 0x28;
pub const VK_SELECT: c_int = 0x29;
pub const VK_PRINT: c_int = 0x2A;
pub const VK_EXECUTE: c_int = 0x2B;
pub const VK_SNAPSHOT: c_int = 0x2C;
pub const VK_INSERT: c_int = 0x2D;
pub const VK_DELETE: c_int = 0x2E;
pub const VK_HELP: c_int = 0x2F;
pub const VK_LWIN: c_int = 0x5B;
pub const VK_RWIN: c_int = 0x5C;
pub const VK_APPS: c_int = 0x5D;
pub const VK_SLEEP: c_int = 0x5F;
pub const VK_NUMPAD0: c_int = 0x60;
pub const VK_NUMPAD1: c_int = 0x61;
pub const VK_NUMPAD2: c_int = 0x62;
pub const VK_NUMPAD3: c_int = 0x63;
pub const VK_NUMPAD4: c_int = 0x64;
pub const VK_NUMPAD5: c_int = 0x65;
pub const VK_NUMPAD6: c_int = 0x66;
pub const VK_NUMPAD7: c_int = 0x67;
pub const VK_NUMPAD8: c_int = 0x68;
pub const VK_NUMPAD9: c_int = 0x69;
pub const VK_MULTIPLY: c_int = 0x6A;
pub const VK_ADD: c_int = 0x6B;
pub const VK_SEPARATOR: c_int = 0x6C;
pub const VK_SUBTRACT: c_int = 0x6D;
pub const VK_DECIMAL: c_int = 0x6E;
pub const VK_DIVIDE: c_int = 0x6F;
pub const VK_F1: c_int = 0x70;
pub const VK_F2: c_int = 0x71;
pub const VK_F3: c_int = 0x72;
pub const VK_F4: c_int = 0x73;
pub const VK_F5: c_int = 0x74;
pub const VK_F6: c_int = 0x75;
pub const VK_F7: c_int = 0x76;
pub const VK_F8: c_int = 0x77;
pub const VK_F9: c_int = 0x78;
pub const VK_F10: c_int = 0x79;
pub const VK_F11: c_int = 0x7A;
pub const VK_F12: c_int = 0x7B;
pub const VK_F13: c_int = 0x7C;
pub const VK_F14: c_int = 0x7D;
pub const VK_F15: c_int = 0x7E;
pub const VK_F16: c_int = 0x7F;
pub const VK_F17: c_int = 0x80;
pub const VK_F18: c_int = 0x81;
pub const VK_F19: c_int = 0x82;
pub const VK_F20: c_int = 0x83;
pub const VK_F21: c_int = 0x84;
pub const VK_F22: c_int = 0x85;
pub const VK_F23: c_int = 0x86;
pub const VK_F24: c_int = 0x87;
pub const VK_NAVIGATION_VIEW: c_int = 0x88;
pub const VK_NAVIGATION_MENU: c_int = 0x89;
pub const VK_NAVIGATION_UP: c_int = 0x8A;
pub const VK_NAVIGATION_DOWN: c_int = 0x8B;
pub const VK_NAVIGATION_LEFT: c_int = 0x8C;
pub const VK_NAVIGATION_RIGHT: c_int = 0x8D;
pub const VK_NAVIGATION_ACCEPT: c_int = 0x8E;
pub const VK_NAVIGATION_CANCEL: c_int = 0x8F;
pub const VK_NUMLOCK: c_int = 0x90;
pub const VK_SCROLL: c_int = 0x91;
pub const VK_OEM_NEC_EQUAL: c_int = 0x92;
pub const VK_OEM_FJ_JISHO: c_int = 0x92;
pub const VK_OEM_FJ_MASSHOU: c_int = 0x93;
pub const VK_OEM_FJ_TOUROKU: c_int = 0x94;
pub const VK_OEM_FJ_LOYA: c_int = 0x95;
pub const VK_OEM_FJ_ROYA: c_int = 0x96;
pub const VK_LSHIFT: c_int = 0xA0;
pub const VK_RSHIFT: c_int = 0xA1;
pub const VK_LCONTROL: c_int = 0xA2;
pub const VK_RCONTROL: c_int = 0xA3;
pub const VK_LMENU: c_int = 0xA4;
pub const VK_RMENU: c_int = 0xA5;
pub const VK_BROWSER_BACK: c_int = 0xA6;
pub const VK_BROWSER_FORWARD: c_int = 0xA7;
pub const VK_BROWSER_REFRESH: c_int = 0xA8;
pub const VK_BROWSER_STOP: c_int = 0xA9;
pub const VK_BROWSER_SEARCH: c_int = 0xAA;
pub const VK_BROWSER_FAVORITES: c_int = 0xAB;
pub const VK_BROWSER_HOME: c_int = 0xAC;
pub const VK_VOLUME_MUTE: c_int = 0xAD;
pub const VK_VOLUME_DOWN: c_int = 0xAE;
pub const VK_VOLUME_UP: c_int = 0xAF;
pub const VK_MEDIA_NEXT_TRACK: c_int = 0xB0;
pub const VK_MEDIA_PREV_TRACK: c_int = 0xB1;
pub const VK_MEDIA_STOP: c_int = 0xB2;
pub const VK_MEDIA_PLAY_PAUSE: c_int = 0xB3;
pub const VK_LAUNCH_MAIL: c_int = 0xB4;
pub const VK_LAUNCH_MEDIA_SELECT: c_int = 0xB5;
pub const VK_LAUNCH_APP1: c_int = 0xB6;
pub const VK_LAUNCH_APP2: c_int = 0xB7;
pub const VK_OEM_1: c_int = 0xBA;
pub const VK_OEM_PLUS: c_int = 0xBB;
pub const VK_OEM_COMMA: c_int = 0xBC;
pub const VK_OEM_MINUS: c_int = 0xBD;
pub const VK_OEM_PERIOD: c_int = 0xBE;
pub const VK_OEM_2: c_int = 0xBF;
pub const VK_OEM_3: c_int = 0xC0;
pub const VK_GAMEPAD_A: c_int = 0xC3;
pub const VK_GAMEPAD_B: c_int = 0xC4;
pub const VK_GAMEPAD_X: c_int = 0xC5;
pub const VK_GAMEPAD_Y: c_int = 0xC6;
pub const VK_GAMEPAD_RIGHT_SHOULDER: c_int = 0xC7;
pub const VK_GAMEPAD_LEFT_SHOULDER: c_int = 0xC8;
pub const VK_GAMEPAD_LEFT_TRIGGER: c_int = 0xC9;
pub const VK_GAMEPAD_RIGHT_TRIGGER: c_int = 0xCA;
pub const VK_GAMEPAD_DPAD_UP: c_int = 0xCB;
pub const VK_GAMEPAD_DPAD_DOWN: c_int = 0xCC;
pub const VK_GAMEPAD_DPAD_LEFT: c_int = 0xCD;
pub const VK_GAMEPAD_DPAD_RIGHT: c_int = 0xCE;
pub const VK_GAMEPAD_MENU: c_int = 0xCF;
pub const VK_GAMEPAD_VIEW: c_int = 0xD0;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_BUTTON: c_int = 0xD1;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_BUTTON: c_int = 0xD2;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_UP: c_int = 0xD3;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_DOWN: c_int = 0xD4;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_RIGHT: c_int = 0xD5;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_LEFT: c_int = 0xD6;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_UP: c_int = 0xD7;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_DOWN: c_int = 0xD8;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_RIGHT: c_int = 0xD9;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_LEFT: c_int = 0xDA;
pub const VK_OEM_4: c_int = 0xDB;
pub const VK_OEM_5: c_int = 0xDC;
pub const VK_OEM_6: c_int = 0xDD;
pub const VK_OEM_7: c_int = 0xDE;
pub const VK_OEM_8: c_int = 0xDF;
pub const VK_OEM_AX: c_int = 0xE1;
pub const VK_OEM_102: c_int = 0xE2;
pub const VK_ICO_HELP: c_int = 0xE3;
pub const VK_ICO_00: c_int = 0xE4;
pub const VK_PROCESSKEY: c_int = 0xE5;
pub const VK_ICO_CLEAR: c_int = 0xE6;
pub const VK_PACKET: c_int = 0xE7;
pub const VK_OEM_RESET: c_int = 0xE9;
pub const VK_OEM_JUMP: c_int = 0xEA;
pub const VK_OEM_PA1: c_int = 0xEB;
pub const VK_OEM_PA2: c_int = 0xEC;
pub const VK_OEM_PA3: c_int = 0xED;
pub const VK_OEM_WSCTRL: c_int = 0xEE;
pub const VK_OEM_CUSEL: c_int = 0xEF;
pub const VK_OEM_ATTN: c_int = 0xF0;
pub const VK_OEM_FINISH: c_int = 0xF1;
pub const VK_OEM_COPY: c_int = 0xF2;
pub const VK_OEM_AUTO: c_int = 0xF3;
pub const VK_OEM_ENLW: c_int = 0xF4;
pub const VK_OEM_BACKTAB: c_int = 0xF5;
pub const VK_ATTN: c_int = 0xF6;
pub const VK_CRSEL: c_int = 0xF7;
pub const VK_EXSEL: c_int = 0xF8;
pub const VK_EREOF: c_int = 0xF9;
pub const VK_PLAY: c_int = 0xFA;
pub const VK_ZOOM: c_int = 0xFB;
pub const VK_NONAME: c_int = 0xFC;
pub const VK_PA1: c_int = 0xFD;
pub const VK_OEM_CLEAR: c_int = 0xFE;
pub const WH_MIN: c_int = -1;
pub const WH_MSGFILTER: c_int = -1;
pub const WH_JOURNALRECORD: c_int = 0;
pub const WH_JOURNALPLAYBACK: c_int = 1;
pub const WH_KEYBOARD: c_int = 2;
pub const WH_GETMESSAGE: c_int = 3;
pub const WH_CALLWNDPROC: c_int = 4;
pub const WH_CBT: c_int = 5;
pub const WH_SYSMSGFILTER: c_int = 6;
pub const WH_MOUSE: c_int = 7;
pub const WH_HARDWARE: c_int = 8;
pub const WH_DEBUG: c_int = 9;
pub const WH_SHELL: c_int = 10;
pub const WH_FOREGROUNDIDLE: c_int = 11;
pub const WH_CALLWNDPROCRET: c_int = 12;
pub const WH_KEYBOARD_LL: c_int = 13;
pub const WH_MOUSE_LL: c_int = 14;
pub const WH_MAX: c_int = 14;
pub const WH_MINHOOK: c_int = WH_MIN;
pub const WH_MAXHOOK: c_int = WH_MAX;

pub const APPCOMMAND_BROWSER_BACKWARD: c_short = 1;
pub const APPCOMMAND_BROWSER_FORWARD: c_short = 2;
pub const APPCOMMAND_BROWSER_REFRESH: c_short = 3;
pub const APPCOMMAND_BROWSER_STOP: c_short = 4;
pub const APPCOMMAND_BROWSER_SEARCH: c_short = 5;
pub const APPCOMMAND_BROWSER_FAVORITES: c_short = 6;
pub const APPCOMMAND_BROWSER_HOME: c_short = 7;
pub const APPCOMMAND_VOLUME_MUTE: c_short = 8;
pub const APPCOMMAND_VOLUME_DOWN: c_short = 9;
pub const APPCOMMAND_VOLUME_UP: c_short = 10;
pub const APPCOMMAND_MEDIA_NEXTTRACK: c_short = 11;
pub const APPCOMMAND_MEDIA_PREVIOUSTRACK: c_short = 12;
pub const APPCOMMAND_MEDIA_STOP: c_short = 13;
pub const APPCOMMAND_MEDIA_PLAY_PAUSE: c_short = 14;
pub const APPCOMMAND_LAUNCH_MAIL: c_short = 15;
pub const APPCOMMAND_LAUNCH_MEDIA_SELECT: c_short = 16;
pub const APPCOMMAND_LAUNCH_APP1: c_short = 17;
pub const APPCOMMAND_LAUNCH_APP2: c_short = 18;
pub const APPCOMMAND_BASS_DOWN: c_short = 19;
pub const APPCOMMAND_BASS_BOOST: c_short = 20;
pub const APPCOMMAND_BASS_UP: c_short = 21;
pub const APPCOMMAND_TREBLE_DOWN: c_short = 22;
pub const APPCOMMAND_TREBLE_UP: c_short = 23;
pub const APPCOMMAND_MICROPHONE_VOLUME_MUTE: c_short = 24;
pub const APPCOMMAND_MICROPHONE_VOLUME_DOWN: c_short = 25;
pub const APPCOMMAND_MICROPHONE_VOLUME_UP: c_short = 26;
pub const APPCOMMAND_HELP: c_short = 27;
pub const APPCOMMAND_FIND: c_short = 28;
pub const APPCOMMAND_NEW: c_short = 29;
pub const APPCOMMAND_OPEN: c_short = 30;
pub const APPCOMMAND_CLOSE: c_short = 31;
pub const APPCOMMAND_SAVE: c_short = 32;
pub const APPCOMMAND_PRINT: c_short = 33;
pub const APPCOMMAND_UNDO: c_short = 34;
pub const APPCOMMAND_REDO: c_short = 35;
pub const APPCOMMAND_COPY: c_short = 36;
pub const APPCOMMAND_CUT: c_short = 37;
pub const APPCOMMAND_PASTE: c_short = 38;
pub const APPCOMMAND_REPLY_TO_MAIL: c_short = 39;
pub const APPCOMMAND_FORWARD_MAIL: c_short = 40;
pub const APPCOMMAND_SEND_MAIL: c_short = 41;
pub const APPCOMMAND_SPELL_CHECK: c_short = 42;
pub const APPCOMMAND_DICTATE_OR_COMMAND_CONTROL_TOGGLE: c_short = 43;
pub const APPCOMMAND_MIC_ON_OFF_TOGGLE: c_short = 44;
pub const APPCOMMAND_CORRECTION_LIST: c_short = 45;
pub const APPCOMMAND_MEDIA_PLAY: c_short = 46;
pub const APPCOMMAND_MEDIA_PAUSE: c_short = 47;
pub const APPCOMMAND_MEDIA_RECORD: c_short = 48;
pub const APPCOMMAND_MEDIA_FAST_FORWARD: c_short = 49;
pub const APPCOMMAND_MEDIA_REWIND: c_short = 50;
pub const APPCOMMAND_MEDIA_CHANNEL_UP: c_short = 51;
pub const APPCOMMAND_MEDIA_CHANNEL_DOWN: c_short = 52;
pub const APPCOMMAND_DELETE: c_short = 53;
pub const APPCOMMAND_DWM_FLIP3D: c_short = 54;
pub const FAPPCOMMAND_MOUSE: WORD = 0x8000;
pub const FAPPCOMMAND_KEY: WORD = 0;
pub const FAPPCOMMAND_OEM: WORD = 0x1000;
pub const FAPPCOMMAND_MASK: WORD = 0xF000;

pub enum __some_function {}
/// Pointer to a function with unknown type signature.
pub type FARPROC = *mut __some_function;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;
pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
pub const INFINITE: DWORD = 0xFFFFFFFF;

pub const CS_VREDRAW: UINT = 0x0001;
pub const CS_HREDRAW: UINT = 0x0002;
pub const CS_OWNDC: UINT = 0x0020;

pub const WS_OVERLAPPED: DWORD = 0x00000000;
pub const WS_CAPTION: DWORD = 0x00C00000;
pub const WS_SYSMENU: DWORD = 0x00080000;
pub const WS_THICKFRAME: DWORD = 0x00040000;
pub const WS_MINIMIZEBOX: DWORD = 0x00020000;
pub const WS_MAXIMIZEBOX: DWORD = 0x00010000;
pub const WS_OVERLAPPEDWINDOW: DWORD =
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
pub const WS_VISIBLE: DWORD = 0x10000000;

pub const WM_DESTROY: UINT = 0x0002;
pub const WM_MOVE: UINT = 0x0003;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_ACTIVATE: UINT = 0x0006;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_QUIT: UINT = 0x0012;
pub const WM_NCDESTROY: UINT = 0x0082;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_KEYUP: UINT = 0x0101;
pub const WM_SIZING: UINT = 0x0214;
pub const WM_MOVING: UINT = 0x0216;
pub const WM_MOUSEFIRST: UINT = 0x0200;
pub const WM_MOUSEMOVE: UINT = 0x0200;
pub const WM_LBUTTONDOWN: UINT = 0x0201;
pub const WM_LBUTTONUP: UINT = 0x0202;
pub const WM_LBUTTONDBLCLK: UINT = 0x0203;
pub const WM_RBUTTONDOWN: UINT = 0x0204;
pub const WM_RBUTTONUP: UINT = 0x0205;
pub const WM_RBUTTONDBLCLK: UINT = 0x0206;
pub const WM_MBUTTONDOWN: UINT = 0x0207;
pub const WM_MBUTTONUP: UINT = 0x0208;
pub const WM_MBUTTONDBLCLK: UINT = 0x0209;
pub const WM_MOUSEWHEEL: UINT = 0x020A;
pub const WM_XBUTTONDOWN: UINT = 0x020B;
pub const WM_XBUTTONUP: UINT = 0x020C;
pub const WM_XBUTTONDBLCLK: UINT = 0x020D;
pub const WM_MOUSEHWHEEL: UINT = 0x020E;
pub const WM_MOUSELAST: UINT = 0x020E;
pub const WHEEL_DELTA: c_short = 120;

#[inline]
pub fn LOWORD(l: DWORD) -> WORD {
    (l & 0xffff) as WORD
}
#[inline]
pub fn HIWORD(l: DWORD) -> WORD {
    ((l >> 16) & 0xffff) as WORD
}
#[inline]
pub fn GET_X_LPARAM(lp: LPARAM) -> c_int {
    LOWORD(lp as DWORD) as c_short as c_int
}
#[inline]
pub fn GET_Y_LPARAM(lp: LPARAM) -> c_int {
    HIWORD(lp as DWORD) as c_short as c_int
}
#[inline]
pub fn GET_APPCOMMAND_LPARAM(lParam: LPARAM) -> c_short {
    (HIWORD(lParam as DWORD) & !FAPPCOMMAND_MASK) as c_short
}
#[inline]
pub fn GET_DEVICE_LPARAM(lParam: LPARAM) -> WORD {
    HIWORD(lParam as DWORD) & FAPPCOMMAND_MASK
}

pub const PM_NOREMOVE: UINT = 0x0000;
pub const PM_REMOVE: UINT = 0x0001;
pub const PM_NOYIELD: UINT = 0x0002;

pub const LOAD_LIBRARY_SEARCH_SYSTEM32: DWORD = 0x00000800;

pub const OPEN_EXISTING: DWORD = 3;
pub const OPEN_ALWAYS: DWORD = 4;

pub const STATUS_WAIT_0: DWORD = 0x00000000;
pub const STATUS_ABANDONED_WAIT_0: DWORD = 0x00000080;
pub const STATUS_USER_APC: DWORD = 0x000000C0;
pub const STATUS_TIMEOUT: DWORD = 0x00000102;
pub const STATUS_PENDING: DWORD = 0x00000103;

pub const FILE_BEGIN: DWORD = 0;
pub const FILE_CURRENT: DWORD = 1;
pub const FILE_END: DWORD = 2;
pub const WAIT_FAILED: DWORD = 0xFFFFFFFF;
pub const WAIT_OBJECT_0: DWORD = STATUS_WAIT_0 as u32;
pub const WAIT_ABANDONED: DWORD = STATUS_ABANDONED_WAIT_0 as u32;
pub const WAIT_ABANDONED_0: DWORD = STATUS_ABANDONED_WAIT_0 as u32;
pub const WAIT_IO_COMPLETION: DWORD = STATUS_USER_APC as u32;

pub const DELETE: DWORD = 0x00010000;
pub const READ_CONTROL: DWORD = 0x00020000;
pub const WRITE_DAC: DWORD = 0x00040000;
pub const WRITE_OWNER: DWORD = 0x00080000;
pub const SYNCHRONIZE: DWORD = 0x00100000;
pub const STANDARD_RIGHTS_REQUIRED: DWORD = 0x000F0000;
pub const STANDARD_RIGHTS_READ: DWORD = READ_CONTROL;
pub const STANDARD_RIGHTS_WRITE: DWORD = READ_CONTROL;
pub const STANDARD_RIGHTS_EXECUTE: DWORD = READ_CONTROL;
pub const STANDARD_RIGHTS_ALL: DWORD = 0x001F0000;
pub const SPECIFIC_RIGHTS_ALL: DWORD = 0x0000FFFF;
pub const ACCESS_SYSTEM_SECURITY: DWORD = 0x01000000;
pub const MAXIMUM_ALLOWED: DWORD = 0x02000000;
pub const GENERIC_READ: DWORD = 0x80000000;
pub const GENERIC_WRITE: DWORD = 0x40000000;
pub const GENERIC_EXECUTE: DWORD = 0x20000000;
pub const GENERIC_ALL: DWORD = 0x10000000;
pub const FILE_READ_DATA: DWORD = 0x0001;
pub const FILE_LIST_DIRECTORY: DWORD = 0x0001;
pub const FILE_WRITE_DATA: DWORD = 0x0002;
pub const FILE_ADD_FILE: DWORD = 0x0002;
pub const FILE_APPEND_DATA: DWORD = 0x0004;
pub const FILE_ADD_SUBDIRECTORY: DWORD = 0x0004;
pub const FILE_CREATE_PIPE_INSTANCE: DWORD = 0x0004;
pub const FILE_READ_EA: DWORD = 0x0008;
pub const FILE_WRITE_EA: DWORD = 0x0010;
pub const FILE_EXECUTE: DWORD = 0x0020;
pub const FILE_TRAVERSE: DWORD = 0x0020;
pub const FILE_DELETE_CHILD: DWORD = 0x0040;
pub const FILE_READ_ATTRIBUTES: DWORD = 0x0080;
pub const FILE_WRITE_ATTRIBUTES: DWORD = 0x0100;
pub const FILE_ALL_ACCESS: DWORD = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x1FF;
pub const FILE_GENERIC_READ: DWORD =
    STANDARD_RIGHTS_READ | FILE_READ_DATA | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE;
pub const FILE_GENERIC_WRITE: DWORD = STANDARD_RIGHTS_WRITE
    | FILE_WRITE_DATA
    | FILE_WRITE_ATTRIBUTES
    | FILE_WRITE_EA
    | FILE_APPEND_DATA
    | SYNCHRONIZE;
pub const FILE_GENERIC_EXECUTE: DWORD =
    STANDARD_RIGHTS_EXECUTE | FILE_READ_ATTRIBUTES | FILE_EXECUTE | SYNCHRONIZE;
pub const FILE_SHARE_READ: DWORD = 0x00000001;
pub const FILE_SHARE_WRITE: DWORD = 0x00000002;
pub const FILE_SHARE_DELETE: DWORD = 0x00000004;
pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;
pub const FILE_ATTRIBUTE_HIDDEN: DWORD = 0x00000002;
pub const FILE_ATTRIBUTE_SYSTEM: DWORD = 0x00000004;
pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;
pub const FILE_ATTRIBUTE_ARCHIVE: DWORD = 0x00000020;
pub const FILE_ATTRIBUTE_DEVICE: DWORD = 0x00000040;
pub const FILE_ATTRIBUTE_NORMAL: DWORD = 0x00000080;
pub const FILE_ATTRIBUTE_TEMPORARY: DWORD = 0x00000100;
pub const FILE_ATTRIBUTE_SPARSE_FILE: DWORD = 0x00000200;
pub const FILE_ATTRIBUTE_REPARSE_POINT: DWORD = 0x00000400;
pub const FILE_ATTRIBUTE_COMPRESSED: DWORD = 0x00000800;
pub const FILE_ATTRIBUTE_OFFLINE: DWORD = 0x00001000;
pub const FILE_ATTRIBUTE_NOT_CONTENT_INDEXED: DWORD = 0x00002000;
pub const FILE_ATTRIBUTE_ENCRYPTED: DWORD = 0x00004000;
pub const FILE_ATTRIBUTE_INTEGRITY_STREAM: DWORD = 0x00008000;
pub const FILE_ATTRIBUTE_VIRTUAL: DWORD = 0x00010000;
pub const FILE_ATTRIBUTE_NO_SCRUB_DATA: DWORD = 0x00020000;
pub const FILE_ATTRIBUTE_EA: DWORD = 0x00040000;
pub const FILE_ATTRIBUTE_PINNED: DWORD = 0x00080000;
pub const FILE_ATTRIBUTE_UNPINNED: DWORD = 0x00100000;
pub const FILE_ATTRIBUTE_RECALL_ON_OPEN: DWORD = 0x00040000;
pub const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: DWORD = 0x00400000;

pub const FILE_FLAG_WRITE_THROUGH: DWORD = 0x80000000;
pub const FILE_FLAG_OVERLAPPED: DWORD = 0x40000000;
pub const FILE_FLAG_NO_BUFFERING: DWORD = 0x20000000;
pub const FILE_FLAG_RANDOM_ACCESS: DWORD = 0x10000000;
pub const FILE_FLAG_SEQUENTIAL_SCAN: DWORD = 0x08000000;
pub const FILE_FLAG_DELETE_ON_CLOSE: DWORD = 0x04000000;
pub const FILE_FLAG_BACKUP_SEMANTICS: DWORD = 0x02000000;
pub const FILE_FLAG_POSIX_SEMANTICS: DWORD = 0x01000000;
pub const FILE_FLAG_SESSION_AWARE: DWORD = 0x00800000;
pub const FILE_FLAG_OPEN_REPARSE_POINT: DWORD = 0x00200000;
pub const FILE_FLAG_OPEN_NO_RECALL: DWORD = 0x00100000;
pub const FILE_FLAG_FIRST_PIPE_INSTANCE: DWORD = 0x00080000;
pub const FILE_FLAG_OPEN_REQUIRING_OPLOCK: DWORD = 0x00040000;

pub const FILE_NOTIFY_CHANGE_FILE_NAME: DWORD = 1;
pub const FILE_NOTIFY_CHANGE_DIR_NAME: DWORD = 2;
pub const FILE_NOTIFY_CHANGE_ATTRIBUTES: DWORD = 4;
pub const FILE_NOTIFY_CHANGE_SIZE: DWORD = 8;
pub const FILE_NOTIFY_CHANGE_LAST_WRITE: DWORD = 16;
pub const FILE_NOTIFY_CHANGE_LAST_ACCESS: DWORD = 32;
pub const FILE_NOTIFY_CHANGE_CREATION: DWORD = 64;
pub const FILE_NOTIFY_CHANGE_SECURITY: DWORD = 256;
pub const FILE_ACTION_ADDED: DWORD = 0x00000001;
pub const FILE_ACTION_REMOVED: DWORD = 0x00000002;
pub const FILE_ACTION_MODIFIED: DWORD = 0x00000003;
pub const FILE_ACTION_RENAMED_OLD_NAME: DWORD = 0x00000004;
pub const FILE_ACTION_RENAMED_NEW_NAME: DWORD = 0x00000005;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WNDCLASSW {
    pub style: UINT,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: c_int,
    pub cbWndExtra: c_int,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: LPCWSTR,
    pub lpszClassName: LPCWSTR,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: DWORD,
    pub lpSecurityDescriptor: LPVOID,
    pub bInheritHandle: BOOL,
}

pub type LPSECURITY_ATTRIBUTES = *mut SECURITY_ATTRIBUTES;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OVERLAPPED {
    pub Internal: ULONG_PTR,
    pub InternalHigh: ULONG_PTR,
    pub Offset: DWORD,
    pub OffsetHigh: DWORD,
    pub hEvent: HANDLE,
}

pub type LPOVERLAPPED = *mut OVERLAPPED;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILE_NOTIFY_INFORMATION {
    pub NextEntryOffset: DWORD,
    pub Action: DWORD,
    pub FileNameLength: DWORD,
    pub FileName: [WCHAR; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FILE_DISPOSITION_INFO {
    pub DeleteFile: BOOL,
}

#[repr(C)]
pub enum FILE_INFO_BY_HANDLE_CLASS {
    FileBasicInfo,
    FileStandardInfo,
    FileNameInfo,
    FileRenameInfo,
    FileDispositionInfo,
    FileAllocationInfo,
    FileEndOfFileInfo,
    FileStreamInfo,
    FileCompressionInfo,
    FileAttributeTagInfo,
    FileIdBothDirectoryInfo,
    FileIdBothDirectoryRestartInfo,
    FileIoPriorityHintInfo,
    FileRemoteProtocolInfo,
    FileFullDirectoryInfo,
    FileFullDirectoryRestartInfo,
    FileStorageInfo,
    FileAlignmentInfo,
    FileIdInfo,
    FileIdExtdDirectoryInfo,
    FileIdExtdDirectoryRestartInfo,
    FileDispositionInfoEx,
    FileRenameInfoEx,
    MaximumFileInfoByHandleClass,
}
pub type PFILE_INFO_BY_HANDLE_CLASS = *mut FILE_INFO_BY_HANDLE_CLASS;
