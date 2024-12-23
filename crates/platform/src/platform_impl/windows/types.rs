#![allow(
    bad_style,
    overflowing_literals,
    dead_code,
    improper_ctypes,
    improper_ctypes_definitions,
    clippy::upper_case_acronyms
)]

use std::os::raw::c_ulonglong;

use super::externs::*;
use crate::ctypes::*;
use crate::declare_extern_function;
use crate::declare_handle;

declare_handle! {HWND, HWND__}
declare_handle! {HINSTANCE, HINSTANCE__}
declare_handle! {HICON, HICON__}
declare_handle! {HBITMAP, HBITMAP__}
declare_handle! {HBRUSH, HBRUSH__}
declare_handle! {HCOLORSPACE, HCOLORSPACE__}
declare_handle! {HDC, HDC__}
declare_handle! {HMENU, HMENU__}
declare_handle! {HFONT, HFONT__}
declare_handle! {HPALETTE, HPALETTE__}
declare_handle! {HPEN, HPEN__}
declare_handle! {HWINEVENTHOOK, HWINEVENTHOOK__}
declare_handle! {HUMPD, HUMPD__}
declare_handle! {HMONITOR, HMONITOR__}
declare_handle! {HKL, HKL__}
declare_handle! {HRGN, HRGN__}

#[inline]
pub fn MAKEWORD(a: BYTE, b: BYTE) -> WORD {
    (a as WORD) | ((b as WORD) << 8)
}
#[inline]
pub fn MAKELONG(a: WORD, b: WORD) -> LONG {
    ((a as DWORD) | ((b as DWORD) << 16)) as LONG
}
#[inline]
pub fn LOWORD(l: DWORD) -> WORD {
    (l & 0xffff) as WORD
}
#[inline]
pub fn HIWORD(l: DWORD) -> WORD {
    ((l >> 16) & 0xffff) as WORD
}
#[inline]
pub fn LOBYTE(l: WORD) -> BYTE {
    (l & 0xff) as BYTE
}
#[inline]
pub fn HIBYTE(l: WORD) -> BYTE {
    ((l >> 8) & 0xff) as BYTE
}

pub type HANDLE = *mut c_void;
pub type PHANDLE = *mut HANDLE;
pub type HMODULE = HINSTANCE;
pub type HCURSOR = HICON;
pub type COLORREF = DWORD;
pub type LPCOLORREF = *mut DWORD;
pub type HRESULT = c_long;
pub type wchar_t = u16;
pub type BOOL = c_int;
pub type BYTE = c_uchar;
pub type FLOAT = c_float;
pub type CHAR = c_char;
pub type WCHAR = wchar_t;
pub type WORD = c_ushort;
pub type DWORD = c_ulong;
pub type LPWSTR = *mut WCHAR;
pub type LPWORD = *mut WORD;
pub type LPDWORD = *mut DWORD;
pub type INT = c_int;
pub type UINT = c_uint;
pub type LONG = c_long;
pub type UINT_PTR = usize;
pub type LONG_PTR = isize;
pub type ULONG_PTR = c_ulonglong;
pub type OLECHAR = WCHAR;
pub type LPOLESTR = *mut OLECHAR;
pub type LPCOLESTR = *const OLECHAR;
pub type UCHAR = c_uchar;
pub type SHORT = c_short;
pub type USHORT = c_ushort;
pub type ULONG = DWORD;
pub type DOUBLE = c_double;
pub type ATOM = WORD;
pub type LPCSTR = *const CHAR;
pub type LPCWSTR = *const WCHAR;
pub type WPARAM = UINT_PTR;
pub type LPARAM = LONG_PTR;
pub type LRESULT = LONG_PTR;
pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type LPLONG = *mut c_long;
pub type LPMSG = *mut MSG;
pub type PVOID = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type INT_PTR = isize;
pub type PINT_PTR = *mut isize;
pub type HTHEME = HANDLE;
pub type PVOID64 = u64; // This is a 64-bit pointer, even when in 32-bit
pub type VOID = c_void;
pub type PBYTE = *mut BYTE;
pub type LPBYTE = *mut BYTE;

pub const IMAGE_BITMAP: UINT = 0;
pub const IMAGE_ICON: UINT = 1;
pub const IMAGE_CURSOR: UINT = 2;
pub const IMAGE_ENHMETAFILE: UINT = 3;
pub const LR_DEFAULTCOLOR: UINT = 0x00000000;
pub const LR_MONOCHROME: UINT = 0x00000001;
pub const LR_COLOR: UINT = 0x00000002;
pub const LR_COPYRETURNORG: UINT = 0x00000004;
pub const LR_COPYDELETEORG: UINT = 0x00000008;
pub const LR_LOADFROMFILE: UINT = 0x00000010;
pub const LR_LOADTRANSPARENT: UINT = 0x00000020;
pub const LR_DEFAULTSIZE: UINT = 0x00000040;
pub const LR_VGACOLOR: UINT = 0x00000080;
pub const LR_LOADMAP3DCOLORS: UINT = 0x00001000;
pub const LR_CREATEDIBSECTION: UINT = 0x00002000;
pub const LR_COPYFROMRESOURCE: UINT = 0x00004000;
pub const LR_SHARED: UINT = 0x00008000;

pub const DTBG_CLIPRECT: DWORD = 0x00000001;
pub const DTBG_DRAWSOLID: DWORD = 0x00000002;
pub const DTBG_OMITBORDER: DWORD = 0x00000004;
pub const DTBG_OMITCONTENT: DWORD = 0x00000008;
pub const DTBG_COMPUTINGREGION: DWORD = 0x00000010;
pub const DTBG_MIRRORDC: DWORD = 0x00000020;
pub const DTBG_NOMIRROR: DWORD = 0x00000040;
pub const DTBG_VALIDBITS: DWORD = DTBG_CLIPRECT
    | DTBG_DRAWSOLID
    | DTBG_OMITBORDER
    | DTBG_OMITCONTENT
    | DTBG_COMPUTINGREGION
    | DTBG_MIRRORDC
    | DTBG_NOMIRROR;

#[repr(C)]
pub struct DTBGOPTS {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub rcClip: RECT,
}
pub type PDTBGOPTS = *mut DTBGOPTS;

#[repr(C)]
pub enum DWMNCRENDERINGPOLICY {
    DWMNCRP_USEWINDOWSTYLE = 0,
    DWMNCRP_DISABLED = 1,
    DWMNCRP_ENABLED = 2,
    DWMNCRP_LAST = 3,
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: DWORD,
    pub biWidth: LONG,
    pub biHeight: LONG,
    pub biPlanes: WORD,
    pub biBitCount: WORD,
    pub biCompression: DWORD,
    pub biSizeImage: DWORD,
    pub biXPelsPerMeter: LONG,
    pub biYPelsPerMeter: LONG,
    pub biClrUsed: DWORD,
    pub biClrImportant: DWORD,
}
pub type LPBITMAPINFOHEADER = *mut BITMAPINFOHEADER;
pub type PBITMAPINFOHEADER = *mut BITMAPINFOHEADER;

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}
pub type LPBITMAPINFO = *mut BITMAPINFO;
pub type PBITMAPINFO = *mut BITMAPINFO;

#[repr(C)]
pub struct PALETTEENTRY {
    pub peRed: BYTE,
    pub peGreen: BYTE,
    pub peBlue: BYTE,
    pub peFlags: BYTE,
}
pub type PPALETTEENTRY = *mut PALETTEENTRY;
pub type LPPALETTEENTRY = *mut PALETTEENTRY;
pub const TRANSPARENT: DWORD = 1;
pub const OPAQUE: DWORD = 2;

#[repr(C)]
pub struct LOGPALETTE {
    pub palVersion: WORD,
    pub palNumEntries: WORD,
    pub palPalEntry: [PALETTEENTRY; 1],
}
pub type PLOGPALETTE = *mut LOGPALETTE;
pub type NPLOGPALETTE = *mut LOGPALETTE;
pub type LPLOGPALETTE = *mut LOGPALETTE;

#[repr(C)]
pub struct RGBQUAD {
    pub rgbBlue: BYTE,
    pub rgbGreen: BYTE,
    pub rgbRed: BYTE,
    pub rgbReserved: BYTE,
}
pub type LPRGBQUAD = *mut RGBQUAD;

pub const BI_RGB: DWORD = 0;
pub const BI_RLE8: DWORD = 1;
pub const BI_RLE4: DWORD = 2;
pub const BI_BITFIELDS: DWORD = 3;
pub const BI_JPEG: DWORD = 4;
pub const BI_PNG: DWORD = 5;

#[inline]
pub fn GDI_WIDTHBYTES(bits: DWORD) -> DWORD {
    ((bits + 31) & !31) / 8
}
#[inline]
pub fn GDI_DIBWIDTHBYTES(bi: &BITMAPINFOHEADER) -> DWORD {
    GDI_WIDTHBYTES((bi.biWidth as DWORD) * (bi.biBitCount as DWORD))
}
#[inline]
pub fn GDI__DIBSIZE(bi: &BITMAPINFOHEADER) -> DWORD {
    GDI_DIBWIDTHBYTES(bi) * bi.biHeight as DWORD
}
#[inline]
pub fn GDI_DIBSIZE(bi: &BITMAPINFOHEADER) -> DWORD {
    if bi.biHeight < 0 {
        GDI__DIBSIZE(bi) * -1i32 as u32
    } else {
        GDI__DIBSIZE(bi)
    }
}

#[repr(C)]
pub enum DWMWINDOWATTRIBUTE {
    DWMWA_NCRENDERING_ENABLED = 1,
    DWMWA_NCRENDERING_POLICY = 2,
    DWMWA_TRANSITIONS_FORCEDISABLED = 3,
    DWMWA_ALLOW_NCPAINT = 4,
    DWMWA_CAPTION_BUTTON_BOUNDS = 5,
    DWMWA_NONCLIENT_RTL_LAYOUT = 6,
    DWMWA_FORCE_ICONIC_REPRESENTATION = 7,
    DWMWA_FLIP3D_POLICY = 8,
    DWMWA_EXTENDED_FRAME_BOUNDS = 9,
    DWMWA_HAS_ICONIC_BITMAP = 10,
    DWMWA_DISALLOW_PEEK = 11,
    DWMWA_EXCLUDED_FROM_PEEK = 12,
    DWMWA_CLOAK = 13,
    DWMWA_CLOAKED = 14,
    DWMWA_FREEZE_REPRESENTATION = 15,
    DWMWA_LAST = 16,
}

#[repr(C)]
pub struct WINDOWPOS {
    pub hwnd: HWND,
    pub hwndInsertAfter: HWND,
    pub x: c_int,
    pub y: c_int,
    pub cx: c_int,
    pub cy: c_int,
    pub flags: UINT,
}
pub type LPWINDOWPOS = *mut WINDOWPOS;
pub type PWINDOWPOS = *mut WINDOWPOS;

#[repr(C)]
pub struct NCCALCSIZE_PARAMS {
    pub rgrc: [RECT; 3],
    pub lppos: PWINDOWPOS,
}

pub type LPNCCALCSIZE_PARAMS = *mut NCCALCSIZE_PARAMS;

#[repr(C)]
pub struct MARGINS {
    pub cxLeftWidth: c_int,
    pub cxRightWidth: c_int,
    pub cyTopHeight: c_int,
    pub cyBottomHeight: c_int,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [BYTE; 32],
}
pub type PPAINTSTRUCT = *mut PAINTSTRUCT;
pub type NPPAINTSTRUCT = *mut PAINTSTRUCT;
pub type LPPAINTSTRUCT = *mut PAINTSTRUCT;

#[repr(C)]
pub struct PIXELFORMATDESCRIPTOR {
    pub nSize: WORD,
    pub nVersion: WORD,
    pub dwFlags: DWORD,
    pub iPixelType: BYTE,
    pub cColorBits: BYTE,
    pub cRedBits: BYTE,
    pub cRedShift: BYTE,
    pub cGreenBits: BYTE,
    pub cGreenShift: BYTE,
    pub cBlueBits: BYTE,
    pub cBlueShift: BYTE,
    pub cAlphaBits: BYTE,
    pub cAlphaShift: BYTE,
    pub cAccumBits: BYTE,
    pub cAccumRedBits: BYTE,
    pub cAccumGreenBits: BYTE,
    pub cAccumBlueBits: BYTE,
    pub cAccumAlphaBits: BYTE,
    pub cDepthBits: BYTE,
    pub cStencilBits: BYTE,
    pub cAuxBuffers: BYTE,
    pub iLayerType: BYTE,
    pub bReserved: BYTE,
    pub dwLayerMask: DWORD,
    pub dwVisibleMask: DWORD,
    pub dwDamageMask: DWORD,
}
pub type PPIXELFORMATDESCRIPTOR = *mut PIXELFORMATDESCRIPTOR;
pub type LPPIXELFORMATDESCRIPTOR = *mut PIXELFORMATDESCRIPTOR;
pub const PFD_TYPE_RGBA: BYTE = 0;
pub const PFD_TYPE_COLORINDEX: BYTE = 1;
pub const PFD_MAIN_PLANE: BYTE = 0;
pub const PFD_OVERLAY_PLANE: BYTE = 1;
pub const PFD_UNDERLAY_PLANE: BYTE = -1i8 as u8;
pub const PFD_DOUBLEBUFFER: DWORD = 0x00000001;
pub const PFD_STEREO: DWORD = 0x00000002;
pub const PFD_DRAW_TO_WINDOW: DWORD = 0x00000004;
pub const PFD_DRAW_TO_BITMAP: DWORD = 0x00000008;
pub const PFD_SUPPORT_GDI: DWORD = 0x00000010;
pub const PFD_SUPPORT_OPENGL: DWORD = 0x00000020;
pub const PFD_GENERIC_FORMAT: DWORD = 0x00000040;
pub const PFD_NEED_PALETTE: DWORD = 0x00000080;
pub const PFD_NEED_SYSTEM_PALETTE: DWORD = 0x00000100;
pub const PFD_SWAP_EXCHANGE: DWORD = 0x00000200;
pub const PFD_SWAP_COPY: DWORD = 0x00000400;
pub const PFD_SWAP_LAYER_BUFFERS: DWORD = 0x00000800;
pub const PFD_GENERIC_ACCELERATED: DWORD = 0x00001000;
pub const PFD_SUPPORT_DIRECTDRAW: DWORD = 0x00002000;
pub const PFD_DIRECT3D_ACCELERATED: DWORD = 0x00004000;
pub const PFD_SUPPORT_COMPOSITION: DWORD = 0x00008000;
pub const PFD_DEPTH_DONTCARE: DWORD = 0x20000000;
pub const PFD_DOUBLEBUFFER_DONTCARE: DWORD = 0x40000000;
pub const PFD_STEREO_DONTCARE: DWORD = 0x80000000;

pub const DTT_TEXTCOLOR: DWORD = 1 << 0;
pub const DTT_BORDERCOLOR: DWORD = 1 << 1;
pub const DTT_SHADOWCOLOR: DWORD = 1 << 2;
pub const DTT_SHADOWTYPE: DWORD = 1 << 3;
pub const DTT_SHADOWOFFSET: DWORD = 1 << 4;
pub const DTT_BORDERSIZE: DWORD = 1 << 5;
pub const DTT_FONTPROP: DWORD = 1 << 6;
pub const DTT_COLORPROP: DWORD = 1 << 7;
pub const DTT_STATEID: DWORD = 1 << 8;
pub const DTT_CALCRECT: DWORD = 1 << 9;
pub const DTT_APPLYOVERLAY: DWORD = 1 << 10;
pub const DTT_GLOWSIZE: DWORD = 1 << 11;
pub const DTT_CALLBACK: DWORD = 1 << 12;
pub const DTT_COMPOSITED: DWORD = 1 << 13;
pub const DTT_VALIDBITS: DWORD = DTT_TEXTCOLOR
    | DTT_BORDERCOLOR
    | DTT_SHADOWCOLOR
    | DTT_SHADOWTYPE
    | DTT_SHADOWOFFSET
    | DTT_BORDERSIZE
    | DTT_FONTPROP
    | DTT_COLORPROP
    | DTT_STATEID
    | DTT_CALCRECT
    | DTT_APPLYOVERLAY
    | DTT_GLOWSIZE
    | DTT_COMPOSITED;

declare_extern_function!(stdcall DTT_CALLBACK_PROC(
    hdc: HDC,
    pszText: LPWSTR,
    cchText: c_int,
    prc: LPRECT,
    dwFlags: UINT,
    lParam: LPARAM,
) -> c_int);

#[repr(C)]
pub struct DTTOPTS {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub crText: COLORREF,
    pub crBorder: COLORREF,
    pub crShadow: COLORREF,
    pub iTextShadowType: c_int,
    pub ptShadowOffset: POINT,
    pub iBorderSize: c_int,
    pub iFontPropId: c_int,
    pub iColorPropId: c_int,
    pub iStateId: c_int,
    pub fApplyOverlay: BOOL,
    pub iGlowSize: c_int,
    pub pfnDrawTextCallback: DTT_CALLBACK_PROC,
    pub lParam: LPARAM,
}
pub type PDTTOPTS = *mut DTTOPTS;

pub const UINT_MAX: c_uint = 0xffffffff;
pub const SW_HIDE: c_int = 0;
pub const SW_SHOWNORMAL: c_int = 1;
pub const SW_NORMAL: c_int = 1;
pub const SW_SHOWMINIMIZED: c_int = 2;
pub const SW_SHOWMAXIMIZED: c_int = 3;
pub const SW_MAXIMIZE: c_int = 3;
pub const SW_SHOWNOACTIVATE: c_int = 4;
pub const SW_SHOW: c_int = 5;
pub const SW_MINIMIZE: c_int = 6;
pub const SW_SHOWMINNOACTIVE: c_int = 7;
pub const SW_SHOWNA: c_int = 8;
pub const SW_RESTORE: c_int = 9;
pub const SW_SHOWDEFAULT: c_int = 10;
pub const SW_FORCEMINIMIZE: c_int = 11;
pub const SW_MAX: c_int = 11;

pub const CTLCOLOR_MSGBOX: c_int = 0;
pub const CTLCOLOR_EDIT: c_int = 1;
pub const CTLCOLOR_LISTBOX: c_int = 2;
pub const CTLCOLOR_BTN: c_int = 3;
pub const CTLCOLOR_DLG: c_int = 4;
pub const CTLCOLOR_SCROLLBAR: c_int = 5;
pub const CTLCOLOR_STATIC: c_int = 6;
pub const CTLCOLOR_MAX: c_int = 7;
pub const COLOR_SCROLLBAR: c_int = 0;
pub const COLOR_BACKGROUND: c_int = 1;
pub const COLOR_ACTIVECAPTION: c_int = 2;
pub const COLOR_INACTIVECAPTION: c_int = 3;
pub const COLOR_MENU: c_int = 4;
pub const COLOR_WINDOW: c_int = 5;
pub const COLOR_WINDOWFRAME: c_int = 6;
pub const COLOR_MENUTEXT: c_int = 7;
pub const COLOR_WINDOWTEXT: c_int = 8;
pub const COLOR_CAPTIONTEXT: c_int = 9;
pub const COLOR_ACTIVEBORDER: c_int = 10;
pub const COLOR_INACTIVEBORDER: c_int = 11;
pub const COLOR_APPWORKSPACE: c_int = 12;
pub const COLOR_HIGHLIGHT: c_int = 13;
pub const COLOR_HIGHLIGHTTEXT: c_int = 14;
pub const COLOR_BTNFACE: c_int = 15;
pub const COLOR_BTNSHADOW: c_int = 16;
pub const COLOR_GRAYTEXT: c_int = 17;
pub const COLOR_BTNTEXT: c_int = 18;
pub const COLOR_INACTIVECAPTIONTEXT: c_int = 19;
pub const COLOR_BTNHIGHLIGHT: c_int = 20;
pub const COLOR_3DDKSHADOW: c_int = 21;
pub const COLOR_3DLIGHT: c_int = 22;
pub const COLOR_INFOTEXT: c_int = 23;
pub const COLOR_INFOBK: c_int = 24;
pub const COLOR_HOTLIGHT: c_int = 26;
pub const COLOR_GRADIENTACTIVECAPTION: c_int = 27;
pub const COLOR_GRADIENTINACTIVECAPTION: c_int = 28;
pub const COLOR_MENUHILIGHT: c_int = 29;
pub const COLOR_MENUBAR: c_int = 30;
pub const COLOR_DESKTOP: c_int = COLOR_BACKGROUND;
pub const COLOR_3DFACE: c_int = COLOR_BTNFACE;
pub const COLOR_3DSHADOW: c_int = COLOR_BTNSHADOW;
pub const COLOR_3DHIGHLIGHT: c_int = COLOR_BTNHIGHLIGHT;
pub const COLOR_3DHILIGHT: c_int = COLOR_BTNHIGHLIGHT;
pub const COLOR_BTNHILIGHT: c_int = COLOR_BTNHIGHLIGHT;
pub const WHITE_BRUSH: DWORD = 0;
pub const LTGRAY_BRUSH: DWORD = 1;
pub const GRAY_BRUSH: DWORD = 2;
pub const DKGRAY_BRUSH: DWORD = 3;
pub const BLACK_BRUSH: DWORD = 4;
pub const NULL_BRUSH: DWORD = 5;
pub const HOLLOW_BRUSH: DWORD = NULL_BRUSH;
pub const WHITE_PEN: DWORD = 6;
pub const BLACK_PEN: DWORD = 7;
pub const NULL_PEN: DWORD = 8;

#[inline]
pub fn RGB(r: BYTE, g: BYTE, b: BYTE) -> COLORREF {
    r as COLORREF | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}
#[inline]
pub fn GetRValue(rgb: COLORREF) -> BYTE {
    LOBYTE(rgb as WORD)
}
#[inline]
pub fn GetGValue(rgb: COLORREF) -> BYTE {
    LOBYTE((rgb as WORD) >> 8)
}
#[inline]
pub fn GetBValue(rgb: COLORREF) -> BYTE {
    LOBYTE((rgb >> 16) as WORD)
}

pub const IDI_APPLICATION: LPCWSTR = 32512 as LPCWSTR;
pub const IDI_HAND: LPCWSTR = 32513 as LPCWSTR;
pub const IDI_QUESTION: LPCWSTR = 32514 as LPCWSTR;
pub const IDI_EXCLAMATION: LPCWSTR = 32515 as LPCWSTR;
pub const IDI_ASTERISK: LPCWSTR = 32516 as LPCWSTR;
pub const IDI_WINLOGO: LPCWSTR = 32517 as LPCWSTR;
pub const IDI_SHIELD: LPCWSTR = 32518 as LPCWSTR;
pub const IDI_WARNING: LPCWSTR = IDI_EXCLAMATION;
pub const IDI_ERROR: LPCWSTR = IDI_HAND;
pub const IDI_INFORMATION: LPCWSTR = IDI_ASTERISK;
//10853
pub const IDOK: c_int = 1;
pub const IDCANCEL: c_int = 2;
pub const IDABORT: c_int = 3;
pub const IDRETRY: c_int = 4;
pub const IDIGNORE: c_int = 5;
pub const IDYES: c_int = 6;
pub const IDNO: c_int = 7;
pub const IDCLOSE: c_int = 8;
pub const IDHELP: c_int = 9;
pub const IDTRYAGAIN: c_int = 10;
pub const IDCONTINUE: c_int = 11;
pub const IDTIMEOUT: c_int = 32000;

pub const RDW_INVALIDATE: UINT = 0x0001;
pub const RDW_INTERNALPAINT: UINT = 0x0002;
pub const RDW_ERASE: UINT = 0x0004;
pub const RDW_VALIDATE: UINT = 0x0008;
pub const RDW_NOINTERNALPAINT: UINT = 0x0010;
pub const RDW_NOERASE: UINT = 0x0020;
pub const RDW_NOCHILDREN: UINT = 0x0040;
pub const RDW_ALLCHILDREN: UINT = 0x0080;
pub const RDW_UPDATENOW: UINT = 0x0100;
pub const RDW_ERASENOW: UINT = 0x0200;
pub const RDW_FRAME: UINT = 0x0400;
pub const RDW_NOFRAME: UINT = 0x0800;

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
pub const WS_POPUP: DWORD = 0x80000000;
pub const WS_CHILD: DWORD = 0x40000000;
pub const WS_MINIMIZE: DWORD = 0x20000000;
pub const WS_VISIBLE: DWORD = 0x10000000;
pub const WS_DISABLED: DWORD = 0x08000000;
pub const WS_CLIPSIBLINGS: DWORD = 0x04000000;
pub const WS_CLIPCHILDREN: DWORD = 0x02000000;
pub const WS_MAXIMIZE: DWORD = 0x01000000;
pub const WS_CAPTION: DWORD = 0x00C00000;
pub const WS_BORDER: DWORD = 0x00800000;
pub const WS_DLGFRAME: DWORD = 0x00400000;
pub const WS_VSCROLL: DWORD = 0x00200000;
pub const WS_HSCROLL: DWORD = 0x00100000;
pub const WS_SYSMENU: DWORD = 0x00080000;
pub const WS_THICKFRAME: DWORD = 0x00040000;
pub const WS_GROUP: DWORD = 0x00020000;
pub const WS_TABSTOP: DWORD = 0x00010000;
pub const WS_MINIMIZEBOX: DWORD = 0x00020000;
pub const WS_MAXIMIZEBOX: DWORD = 0x00010000;
pub const WS_TILED: DWORD = WS_OVERLAPPED;
pub const WS_ICONIC: DWORD = WS_MINIMIZE;
pub const WS_SIZEBOX: DWORD = WS_THICKFRAME;
pub const WS_TILEDWINDOW: DWORD = WS_OVERLAPPEDWINDOW;
pub const WS_OVERLAPPEDWINDOW: DWORD =
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
pub const WS_POPUPWINDOW: DWORD = WS_POPUP | WS_BORDER | WS_SYSMENU;
pub const WS_CHILDWINDOW: DWORD = WS_CHILD;
pub const WS_EX_DLGMODALFRAME: DWORD = 0x00000001;
pub const WS_EX_NOPARENTNOTIFY: DWORD = 0x00000004;
pub const WS_EX_TOPMOST: DWORD = 0x00000008;
pub const WS_EX_ACCEPTFILES: DWORD = 0x00000010;
pub const WS_EX_TRANSPARENT: DWORD = 0x00000020;
pub const WS_EX_MDICHILD: DWORD = 0x00000040;
pub const WS_EX_TOOLWINDOW: DWORD = 0x00000080;
pub const WS_EX_WINDOWEDGE: DWORD = 0x00000100;
pub const WS_EX_CLIENTEDGE: DWORD = 0x00000200;
pub const WS_EX_CONTEXTHELP: DWORD = 0x00000400;
pub const WS_EX_RIGHT: DWORD = 0x00001000;
pub const WS_EX_LEFT: DWORD = 0x00000000;
pub const WS_EX_RTLREADING: DWORD = 0x00002000;
pub const WS_EX_LTRREADING: DWORD = 0x00000000;
pub const WS_EX_LEFTSCROLLBAR: DWORD = 0x00004000;
pub const WS_EX_RIGHTSCROLLBAR: DWORD = 0x00000000;
pub const WS_EX_CONTROLPARENT: DWORD = 0x00010000;
pub const WS_EX_STATICEDGE: DWORD = 0x00020000;
pub const WS_EX_APPWINDOW: DWORD = 0x00040000;
pub const WS_EX_OVERLAPPEDWINDOW: DWORD = WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE;
pub const WS_EX_PALETTEWINDOW: DWORD = WS_EX_WINDOWEDGE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST;
pub const WS_EX_LAYERED: DWORD = 0x00080000;
pub const WS_EX_NOINHERITLAYOUT: DWORD = 0x00100000;
pub const WS_EX_NOREDIRECTIONBITMAP: DWORD = 0x00200000;
pub const WS_EX_LAYOUTRTL: DWORD = 0x00400000;
pub const WS_EX_COMPOSITED: DWORD = 0x02000000;
pub const WS_EX_NOACTIVATE: DWORD = 0x08000000;
pub const WM_NULL: UINT = 0x0000;
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_MOVE: UINT = 0x0003;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_ACTIVATE: UINT = 0x0006;
pub const WA_INACTIVE: WORD = 0;
pub const WA_ACTIVE: WORD = 1;
pub const WA_CLICKACTIVE: WORD = 2;
pub const WM_SETFOCUS: UINT = 0x0007;
pub const WM_KILLFOCUS: UINT = 0x0008;
pub const WM_ENABLE: UINT = 0x000A;
pub const WM_SETREDRAW: UINT = 0x000B;
pub const WM_SETTEXT: UINT = 0x000C;
pub const WM_GETTEXT: UINT = 0x000D;
pub const WM_GETTEXTLENGTH: UINT = 0x000E;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_QUERYENDSESSION: UINT = 0x0011;
pub const WM_QUERYOPEN: UINT = 0x0013;
pub const WM_ENDSESSION: UINT = 0x0016;
pub const WM_QUIT: UINT = 0x0012;
pub const WM_ERASEBKGND: UINT = 0x0014;
pub const WM_SYSCOLORCHANGE: UINT = 0x0015;
pub const WM_SHOWWINDOW: UINT = 0x0018;
pub const WM_WININICHANGE: UINT = 0x001A;
pub const WM_SETTINGCHANGE: UINT = WM_WININICHANGE;
pub const WM_DEVMODECHANGE: UINT = 0x001B;
pub const WM_ACTIVATEAPP: UINT = 0x001C;
pub const WM_FONTCHANGE: UINT = 0x001D;
pub const WM_TIMECHANGE: UINT = 0x001E;
pub const WM_CANCELMODE: UINT = 0x001F;
pub const WM_SETCURSOR: UINT = 0x0020;
pub const WM_MOUSEACTIVATE: UINT = 0x0021;
pub const WM_CHILDACTIVATE: UINT = 0x0022;
pub const WM_QUEUESYNC: UINT = 0x0023;
pub const WM_GETMINMAXINFO: UINT = 0x0024;
#[repr(C)]
pub struct MINMAXINFO {
    ptReserved: POINT,
    ptMaxSize: POINT,
    ptMaxPosition: POINT,
    ptMinTrackSize: POINT,
    ptMaxTrackSize: POINT,
}
pub type PMINMAXINFO = *mut MINMAXINFO;
pub type LPMINMAXINFO = *mut MINMAXINFO;
pub const WM_PAINTICON: UINT = 0x0026;
pub const WM_ICONERASEBKGND: UINT = 0x0027;
pub const WM_NEXTDLGCTL: UINT = 0x0028;
pub const WM_SPOOLERSTATUS: UINT = 0x002A;
pub const WM_DRAWITEM: UINT = 0x002B;
pub const WM_MEASUREITEM: UINT = 0x002C;
pub const WM_DELETEITEM: UINT = 0x002D;
pub const WM_VKEYTOITEM: UINT = 0x002E;
pub const WM_CHARTOITEM: UINT = 0x002F;
pub const WM_SETFONT: UINT = 0x0030;
pub const WM_GETFONT: UINT = 0x0031;
pub const WM_SETHOTKEY: UINT = 0x0032;
pub const WM_GETHOTKEY: UINT = 0x0033;
pub const WM_QUERYDRAGICON: UINT = 0x0037;
pub const WM_COMPAREITEM: UINT = 0x0039;
pub const WM_GETOBJECT: UINT = 0x003D;
pub const WM_COMPACTING: UINT = 0x0041;
pub const WM_COMMNOTIFY: UINT = 0x0044;
pub const WM_WINDOWPOSCHANGING: UINT = 0x0046;
pub const WM_WINDOWPOSCHANGED: UINT = 0x0047;
pub const WM_POWER: UINT = 0x0048;
pub const PWR_OK: WPARAM = 1;
pub const PWR_FAIL: WPARAM = -1isize as usize;
pub const PWR_SUSPENDREQUEST: WPARAM = 1;
pub const PWR_SUSPENDRESUME: WPARAM = 2;
pub const PWR_CRITICALRESUME: WPARAM = 3;
pub const WM_COPYDATA: UINT = 0x004A;
pub const WM_CANCELJOURNAL: UINT = 0x004B;
#[repr(C)]
pub struct COPYDATASTRUCT {
    dwData: ULONG_PTR,
    cbData: DWORD,
    lpData: PVOID,
}
pub type PCOPYDATASTRUCT = *mut COPYDATASTRUCT;
#[repr(C)]
pub struct MDINEXTMENU {
    hmenuIn: HMENU,
    hmenuNext: HMENU,
    hwndNext: HWND,
}
pub type PMDINEXTMENU = *mut MDINEXTMENU;
pub type LPMDINEXTMENU = *mut MDINEXTMENU;
pub const WM_NOTIFY: UINT = 0x004E;
pub const WM_INPUTLANGCHANGEREQUEST: UINT = 0x0050;
pub const WM_INPUTLANGCHANGE: UINT = 0x0051;
pub const WM_TCARD: UINT = 0x0052;
pub const WM_HELP: UINT = 0x0053;
pub const WM_USERCHANGED: UINT = 0x0054;
pub const WM_NOTIFYFORMAT: UINT = 0x0055;
pub const NFR_ANSI: LRESULT = 1;
pub const NFR_UNICODE: LRESULT = 2;
pub const NF_QUERY: LPARAM = 3;
pub const NF_REQUERY: LPARAM = 4;
pub const WM_CONTEXTMENU: UINT = 0x007B;
pub const WM_STYLECHANGING: UINT = 0x007C;
pub const WM_STYLECHANGED: UINT = 0x007D;
pub const WM_DISPLAYCHANGE: UINT = 0x007E;
pub const WM_GETICON: UINT = 0x007F;
pub const WM_SETICON: UINT = 0x0080;
pub const WM_NCCREATE: UINT = 0x0081;
pub const WM_NCDESTROY: UINT = 0x0082;
pub const WM_NCCALCSIZE: UINT = 0x0083;
pub const WM_NCHITTEST: UINT = 0x0084;
pub const WM_NCPAINT: UINT = 0x0085;
pub const WM_NCACTIVATE: UINT = 0x0086;
pub const WM_GETDLGCODE: UINT = 0x0087;
pub const WM_SYNCPAINT: UINT = 0x0088;
pub const WM_NCMOUSEMOVE: UINT = 0x00A0;
pub const WM_NCLBUTTONDOWN: UINT = 0x00A1;
pub const WM_NCLBUTTONUP: UINT = 0x00A2;
pub const WM_NCLBUTTONDBLCLK: UINT = 0x00A3;
pub const WM_NCRBUTTONDOWN: UINT = 0x00A4;
pub const WM_NCRBUTTONUP: UINT = 0x00A5;
pub const WM_NCRBUTTONDBLCLK: UINT = 0x00A6;
pub const WM_NCMBUTTONDOWN: UINT = 0x00A7;
pub const WM_NCMBUTTONUP: UINT = 0x00A8;
pub const WM_NCMBUTTONDBLCLK: UINT = 0x00A9;
pub const WM_NCXBUTTONDOWN: UINT = 0x00AB;
pub const WM_NCXBUTTONUP: UINT = 0x00AC;
pub const WM_NCXBUTTONDBLCLK: UINT = 0x00AD;
pub const WM_INPUT_DEVICE_CHANGE: UINT = 0x00FE;
pub const WM_INPUT: UINT = 0x00FF;
pub const WM_KEYFIRST: UINT = 0x0100;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_KEYUP: UINT = 0x0101;
pub const WM_CHAR: UINT = 0x0102;
pub const WM_DEADCHAR: UINT = 0x0103;
pub const WM_SYSKEYDOWN: UINT = 0x0104;
pub const WM_SYSKEYUP: UINT = 0x0105;
pub const WM_SYSCHAR: UINT = 0x0106;
pub const WM_SYSDEADCHAR: UINT = 0x0107;
pub const WM_UNICHAR: UINT = 0x0109;
pub const WM_KEYLAST: UINT = 0x0109;
pub const UNICODE_NOCHAR: WPARAM = 0xFFFF;
pub const WM_IME_STARTCOMPOSITION: UINT = 0x010D;
pub const WM_IME_ENDCOMPOSITION: UINT = 0x010E;
pub const WM_IME_COMPOSITION: UINT = 0x010F;
pub const WM_IME_KEYLAST: UINT = 0x010F;
pub const WM_INITDIALOG: UINT = 0x0110;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_SYSCOMMAND: UINT = 0x0112;
pub const WM_TIMER: UINT = 0x0113;
pub const WM_HSCROLL: UINT = 0x0114;
pub const WM_VSCROLL: UINT = 0x0115;
pub const WM_INITMENU: UINT = 0x0116;
pub const WM_INITMENUPOPUP: UINT = 0x0117;
pub const WM_GESTURE: UINT = 0x0119;
pub const WM_GESTURENOTIFY: UINT = 0x011A;
pub const WM_MENUSELECT: UINT = 0x011F;
pub const WM_MENUCHAR: UINT = 0x0120;
pub const WM_ENTERIDLE: UINT = 0x0121;
pub const WM_MENURBUTTONUP: UINT = 0x0122;
pub const WM_MENUDRAG: UINT = 0x0123;
pub const WM_MENUGETOBJECT: UINT = 0x0124;
pub const WM_UNINITMENUPOPUP: UINT = 0x0125;
pub const WM_MENUCOMMAND: UINT = 0x0126;
pub const WM_CHANGEUISTATE: UINT = 0x0127;
pub const WM_UPDATEUISTATE: UINT = 0x0128;
pub const WM_QUERYUISTATE: UINT = 0x0129;
pub const UIS_SET: WORD = 1;
pub const UIS_CLEAR: WORD = 2;
pub const UIS_INITIALIZE: WORD = 3;
pub const UISF_HIDEFOCUS: WORD = 0x1;
pub const UISF_HIDEACCEL: WORD = 0x2;
pub const UISF_ACTIVE: WORD = 0x4;
pub const WM_CTLCOLORMSGBOX: UINT = 0x0132;
pub const WM_CTLCOLOREDIT: UINT = 0x0133;
pub const WM_CTLCOLORLISTBOX: UINT = 0x0134;
pub const WM_CTLCOLORBTN: UINT = 0x0135;
pub const WM_CTLCOLORDLG: UINT = 0x0136;
pub const WM_CTLCOLORSCROLLBAR: UINT = 0x0137;
pub const WM_CTLCOLORSTATIC: UINT = 0x0138;
pub const MN_GETHMENU: UINT = 0x01E1;
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
pub fn GET_WHEEL_DELTA_WPARAM(wParam: WPARAM) -> c_short {
    HIWORD(wParam as DWORD) as c_short
}
pub const WHEEL_PAGESCROLL: UINT = UINT_MAX;
#[inline]
pub fn GET_KEYSTATE_WPARAM(wParam: WPARAM) -> WORD {
    LOWORD(wParam as DWORD)
}
#[inline]
pub fn GET_NCHITTEST_WPARAM(wParam: WPARAM) -> c_short {
    LOWORD(wParam as DWORD) as c_short
}
#[inline]
pub fn GET_XBUTTON_WPARAM(wParam: WPARAM) -> WORD {
    HIWORD(wParam as DWORD)
}
pub const XBUTTON1: WORD = 0x0001;
pub const XBUTTON2: WORD = 0x0002;
pub const WM_PARENTNOTIFY: UINT = 0x0210;
pub const WM_ENTERMENULOOP: UINT = 0x0211;
pub const WM_EXITMENULOOP: UINT = 0x0212;
pub const WM_NEXTMENU: UINT = 0x0213;
pub const WM_SIZING: UINT = 0x0214;
pub const WM_CAPTURECHANGED: UINT = 0x0215;
pub const WM_MOVING: UINT = 0x0216;
pub const WM_POWERBROADCAST: UINT = 0x0218;
pub const WM_DEVICECHANGE: UINT = 0x0219;
pub const WM_MDICREATE: UINT = 0x0220;
pub const WM_MDIDESTROY: UINT = 0x0221;
pub const WM_MDIACTIVATE: UINT = 0x0222;
pub const WM_MDIRESTORE: UINT = 0x0223;
pub const WM_MDINEXT: UINT = 0x0224;
pub const WM_MDIMAXIMIZE: UINT = 0x0225;
pub const WM_MDITILE: UINT = 0x0226;
pub const WM_MDICASCADE: UINT = 0x0227;
pub const WM_MDIICONARRANGE: UINT = 0x0228;
pub const WM_MDIGETACTIVE: UINT = 0x0229;
pub const WM_MDISETMENU: UINT = 0x0230;
pub const WM_ENTERSIZEMOVE: UINT = 0x0231;
pub const WM_EXITSIZEMOVE: UINT = 0x0232;
pub const WM_DROPFILES: UINT = 0x0233;
pub const WM_MDIREFRESHMENU: UINT = 0x0234;
pub const WM_POINTERDEVICECHANGE: UINT = 0x238;
pub const WM_POINTERDEVICEINRANGE: UINT = 0x239;
pub const WM_POINTERDEVICEOUTOFRANGE: UINT = 0x23A;
pub const WM_TOUCH: UINT = 0x0240;
pub const WM_NCPOINTERUPDATE: UINT = 0x0241;
pub const WM_NCPOINTERDOWN: UINT = 0x0242;
pub const WM_NCPOINTERUP: UINT = 0x0243;
pub const WM_POINTERUPDATE: UINT = 0x0245;
pub const WM_POINTERDOWN: UINT = 0x0246;
pub const WM_POINTERUP: UINT = 0x0247;
pub const WM_POINTERENTER: UINT = 0x0249;
pub const WM_POINTERLEAVE: UINT = 0x024A;
pub const WM_POINTERACTIVATE: UINT = 0x024B;
pub const WM_POINTERCAPTURECHANGED: UINT = 0x024C;
pub const WM_TOUCHHITTESTING: UINT = 0x024D;
pub const WM_POINTERWHEEL: UINT = 0x024E;
pub const WM_POINTERHWHEEL: UINT = 0x024F;
pub const DM_POINTERHITTEST: UINT = 0x0250;
pub const WM_POINTERROUTEDTO: UINT = 0x0251;
pub const WM_POINTERROUTEDAWAY: UINT = 0x0252;
pub const WM_POINTERROUTEDRELEASED: UINT = 0x0253;
pub const WM_IME_SETCONTEXT: UINT = 0x0281;
pub const WM_IME_NOTIFY: UINT = 0x0282;
pub const WM_IME_CONTROL: UINT = 0x0283;
pub const WM_IME_COMPOSITIONFULL: UINT = 0x0284;
pub const WM_IME_SELECT: UINT = 0x0285;
pub const WM_IME_CHAR: UINT = 0x0286;
pub const WM_IME_REQUEST: UINT = 0x0288;
pub const WM_IME_KEYDOWN: UINT = 0x0290;
pub const WM_IME_KEYUP: UINT = 0x0291;
pub const WM_MOUSEHOVER: UINT = 0x02A1;
pub const WM_MOUSELEAVE: UINT = 0x02A3;
pub const WM_NCMOUSEHOVER: UINT = 0x02A0;
pub const WM_NCMOUSELEAVE: UINT = 0x02A2;
pub const WM_WTSSESSION_CHANGE: UINT = 0x02B1;
pub const WM_TABLET_FIRST: UINT = 0x02c0;
pub const WM_TABLET_LAST: UINT = 0x02df;
pub const WM_DPICHANGED: UINT = 0x02E0;
pub const WM_DPICHANGED_BEFOREPARENT: UINT = 0x02E2;
pub const WM_DPICHANGED_AFTERPARENT: UINT = 0x02E3;
pub const WM_GETDPISCALEDSIZE: UINT = 0x02E4;
pub const WM_CUT: UINT = 0x0300;
pub const WM_COPY: UINT = 0x0301;
pub const WM_PASTE: UINT = 0x0302;
pub const WM_CLEAR: UINT = 0x0303;
pub const WM_UNDO: UINT = 0x0304;
pub const WM_RENDERFORMAT: UINT = 0x0305;
pub const WM_RENDERALLFORMATS: UINT = 0x0306;
pub const WM_DESTROYCLIPBOARD: UINT = 0x0307;
pub const WM_DRAWCLIPBOARD: UINT = 0x0308;
pub const WM_PAINTCLIPBOARD: UINT = 0x0309;
pub const WM_VSCROLLCLIPBOARD: UINT = 0x030A;
pub const WM_SIZECLIPBOARD: UINT = 0x030B;
pub const WM_ASKCBFORMATNAME: UINT = 0x030C;
pub const WM_CHANGECBCHAIN: UINT = 0x030D;
pub const WM_HSCROLLCLIPBOARD: UINT = 0x030E;
pub const WM_QUERYNEWPALETTE: UINT = 0x030F;
pub const WM_PALETTEISCHANGING: UINT = 0x0310;
pub const WM_PALETTECHANGED: UINT = 0x0311;
pub const WM_HOTKEY: UINT = 0x0312;
pub const WM_PRINT: UINT = 0x0317;
pub const WM_PRINTCLIENT: UINT = 0x0318;
pub const WM_APPCOMMAND: UINT = 0x0319;
pub const WM_THEMECHANGED: UINT = 0x031A;
pub const WM_CLIPBOARDUPDATE: UINT = 0x031D;
pub const WM_DWMCOMPOSITIONCHANGED: UINT = 0x031E;
pub const WM_DWMNCRENDERINGCHANGED: UINT = 0x031F;
pub const WM_DWMCOLORIZATIONCOLORCHANGED: UINT = 0x0320;
pub const WM_DWMWINDOWMAXIMIZEDCHANGE: UINT = 0x0321;
pub const WM_DWMSENDICONICTHUMBNAIL: UINT = 0x0323;
pub const WM_DWMSENDICONICLIVEPREVIEWBITMAP: UINT = 0x0326;
pub const WM_GETTITLEBARINFOEX: UINT = 0x033F;
pub const WM_HANDHELDFIRST: UINT = 0x0358;
pub const WM_HANDHELDLAST: UINT = 0x035F;
pub const WM_AFXFIRST: UINT = 0x0360;
pub const WM_AFXLAST: UINT = 0x037F;
pub const WM_PENWINFIRST: UINT = 0x0380;
pub const WM_PENWINLAST: UINT = 0x038F;
pub const WM_APP: UINT = 0x8000;
pub const WM_USER: UINT = 0x0400;
pub const MONITOR_DEFAULTTONULL: DWORD = 0x00000000;
pub const MONITOR_DEFAULTTOPRIMARY: DWORD = 0x00000001;
pub const MONITOR_DEFAULTTONEAREST: DWORD = 0x00000002;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}
pub type PRECT = *mut RECT;
pub type NPRECT = *mut RECT;
pub type LPRECT = *mut RECT;
pub type LPCRECT = *const RECT;

#[repr(C)]
pub struct SIZE {
    pub cx: LONG,
    pub cy: LONG,
}
pub type PSIZE = *mut SIZE;
pub type LPSIZE = *mut SIZE;
pub type SIZEL = SIZE;
pub type PSIZEL = *mut SIZE;
pub type LPSIZEL = *mut SIZE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct POINTS {
    pub x: SHORT,
    pub y: SHORT,
}

pub type PPOINTS = *mut POINTS;
pub type LPPOINTS = *mut POINTS;

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
#[inline]
pub fn MAKEPOINTS(l: DWORD) -> POINTS {
    unsafe { ::core::mem::transmute::<DWORD, POINTS>(l) }
}
pub const SWP_NOSIZE: UINT = 0x0001;
pub const SWP_NOMOVE: UINT = 0x0002;
pub const SWP_NOZORDER: UINT = 0x0004;
pub const SWP_NOREDRAW: UINT = 0x0008;
pub const SWP_NOACTIVATE: UINT = 0x0010;
pub const SWP_FRAMECHANGED: UINT = 0x0020;
pub const SWP_SHOWWINDOW: UINT = 0x0040;
pub const SWP_HIDEWINDOW: UINT = 0x0080;
pub const SWP_NOCOPYBITS: UINT = 0x0100;
pub const SWP_NOOWNERZORDER: UINT = 0x0200;
pub const SWP_NOSENDCHANGING: UINT = 0x0400;
pub const SWP_DRAWFRAME: UINT = SWP_FRAMECHANGED;
pub const SWP_NOREPOSITION: UINT = SWP_NOOWNERZORDER;
pub const SWP_DEFERERASE: UINT = 0x2000;
pub const SWP_ASYNCWINDOWPOS: UINT = 0x4000;

pub const USER_DEFAULT_SCREEN_DPI: LONG = 96;
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
pub const WAIT_OBJECT_0: DWORD = STATUS_WAIT_0;
pub const WAIT_ABANDONED: DWORD = STATUS_ABANDONED_WAIT_0;
pub const WAIT_ABANDONED_0: DWORD = STATUS_ABANDONED_WAIT_0;
pub const WAIT_IO_COMPLETION: DWORD = STATUS_USER_APC;

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
pub type PPOINT = *mut POINT;
pub type NPPOINT = *mut POINT;
pub type LPPOINT = *mut POINT;

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

#[repr(C)]
pub enum PROCESS_DPI_AWARENESS {
    PROCESS_DPI_UNAWARE = 0,
    PROCESS_SYSTEM_DPI_AWARE = 1,
    PROCESS_PER_MONITOR_DPI_AWARE = 2,
}

#[repr(C)]
pub enum MONITOR_DPI_TYPE {
    MDT_EFFECTIVE_DPI = 0,
    MDT_ANGULAR_DPI = 1,
    MDT_RAW_DPI = 2,
}
pub const MDT_DEFAULT: MONITOR_DPI_TYPE = MONITOR_DPI_TYPE::MDT_EFFECTIVE_DPI;
