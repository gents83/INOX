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

pub type LPOVERLAPPED_COMPLETION_ROUTINE = Option<unsafe extern "system" fn(dwErrorCode: DWORD, dwNumberOfBytesTransfered: DWORD, lpOverlapped: LPOVERLAPPED)>;

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
    pub fn PostQuitMessage(
        nExitCode: INT,
    );
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
    pub fn FreeLibraryAndExitThread(
        hLibModule: HMODULE,
        dwExitCode: DWORD,
    );
    pub fn GetLastError() -> DWORD;
    pub fn CreateFileW(
        lpFileName: LPCWSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    pub fn SetFileInformationByHandle(
        hFile: HANDLE,
        FileInformationClass: FILE_INFO_BY_HANDLE_CLASS,
        lpFileInformation: LPVOID,
        dwBufferSize: DWORD,
    ) -> BOOL;
    pub fn ReadDirectoryChangesW(
        hDirectory: HANDLE,
        lpBuffer: LPVOID,
        nBufferLength: DWORD,
        bWatchSubtree: BOOL,
        dwNotifyFilter: DWORD,
        lpBytesReturned: LPDWORD,
        lpOverlapped: LPOVERLAPPED,
        lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE,
    ) -> BOOL;
    pub fn CreateSemaphoreW(
        lpSemaphoreAttributes: LPSECURITY_ATTRIBUTES,
        lInitialCount: LONG,
        lMaximumCount: LONG,
        lpName: LPCWSTR,
    ) -> HANDLE;
    pub fn CancelIoEx(
        hFile: HANDLE,
        lpOverlapped: LPOVERLAPPED,
    ) -> BOOL;
    pub fn CancelIo(
        hFile: HANDLE,
    ) -> BOOL;
    pub fn CloseHandle(
        hObject: HANDLE,
    ) -> BOOL;
    pub fn ReleaseSemaphore(
        hSemaphore: HANDLE,
        lReleaseCount: LONG,
        lpPreviousCount: LPLONG,
    ) -> BOOL;
    pub fn WaitForSingleObject(
        hHandle: HANDLE,
        dwMilliseconds: DWORD,
    ) -> DWORD;
    pub fn WaitForSingleObjectEx(
        hHandle: HANDLE,
        dwMilliseconds: DWORD,
        bAlertable: BOOL,
    ) -> DWORD;
    pub fn SetProcessDEPPolicy(
        dwFlags: DWORD,
    ) -> BOOL;
}