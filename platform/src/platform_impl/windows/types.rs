#![allow(bad_style, overflowing_literals, dead_code)]

use std::os::raw::c_ulonglong;

use crate::declare_handle;
use crate::ctypes::*;
use super::externs::*;

declare_handle!{HWND, HWND__}
declare_handle!{HINSTANCE, HINSTANCE__}
declare_handle!{HICON, HICON__}
declare_handle!{HBRUSH, HBRUSH__}
declare_handle!{HMENU, HMENU__}

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
pub const WS_OVERLAPPEDWINDOW: DWORD = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME
    | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
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
pub const FILE_GENERIC_READ: DWORD = STANDARD_RIGHTS_READ | FILE_READ_DATA
    | FILE_READ_ATTRIBUTES | FILE_READ_EA | SYNCHRONIZE;
pub const FILE_GENERIC_WRITE: DWORD = STANDARD_RIGHTS_WRITE | FILE_WRITE_DATA
    | FILE_WRITE_ATTRIBUTES | FILE_WRITE_EA | FILE_APPEND_DATA | SYNCHRONIZE;
pub const FILE_GENERIC_EXECUTE: DWORD = STANDARD_RIGHTS_EXECUTE | FILE_READ_ATTRIBUTES
    | FILE_EXECUTE | SYNCHRONIZE;
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
    pub lpszClassName: LPCWSTR
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