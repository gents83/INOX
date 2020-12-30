#![allow(bad_style, overflowing_literals, dead_code)]

use crate::declare_extern_function;
use crate::ctypes::*;
use super::types::*;

declare_extern_function!{stdcall WNDPROC(
    HWND,
    UINT,
    WPARAM,
    LPARAM,
) -> LRESULT}

extern "system" {
    pub fn GetModuleHandleW(
        lpModuleName: LPCWSTR,
    ) -> HMODULE;
    pub fn DefWindowProcW(
        hWnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;
    pub fn RegisterClassW(
        lpWndClass: *const WNDCLASSW,
    ) -> ATOM;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        x: c_int,
        y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    pub fn PeekMessageW(
        lpMsg: LPMSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    pub fn GetMessageW(
        lpMsg: LPMSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
    ) -> BOOL;
    pub fn TranslateMessage(
        lpmsg: *const MSG,
    ) -> BOOL;
    pub fn DispatchMessageW(
        lpmsg: *const MSG,
    ) -> LRESULT;   
    pub fn IsIconic(
        hWnd: HWND,
    ) -> BOOL;
    pub fn LoadLibraryA(
        lpLibFileName: LPCWSTR,
    ) -> HMODULE;
    pub fn LoadLibraryW(
        lpLibFileName: LPCWSTR,
    ) -> HMODULE;
    pub fn LoadLibraryExW(
        lpLibFileName: LPCWSTR,
        hFile: HANDLE,
        dwFlags: DWORD,
    ) -> HMODULE;
    pub fn GetProcAddress(
        hModule: HMODULE,
        lpProcName: LPCSTR,
    ) -> FARPROC;
    pub fn FreeLibrary(
        hLibModule: HMODULE,
    ) -> BOOL;
    pub fn GetLastError() -> DWORD;
}