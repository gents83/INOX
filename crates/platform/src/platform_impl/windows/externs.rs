#![allow(
    bad_style,
    overflowing_literals,
    dead_code,
    improper_ctypes,
    improper_ctypes_definitions,
    clippy::upper_case_acronyms
)]

use super::types::*;
use crate::ctypes::*;
use crate::declare_extern_function;

declare_extern_function! {stdcall WNDPROC(
    HWND,
    UINT,
    WPARAM,
    LPARAM,
) -> LRESULT}

pub type LPOVERLAPPED_COMPLETION_ROUTINE = Option<
    unsafe extern "system" fn(
        dwErrorCode: DWORD,
        dwNumberOfBytesTransfered: DWORD,
        lpOverlapped: LPOVERLAPPED,
    ),
>;

extern "system" {
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM;
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
    pub fn ShowOwnedPopups(hWnd: HWND, fShow: BOOL) -> BOOL;
    pub fn OpenIcon(hWnd: HWND) -> BOOL;
    pub fn CloseWindow(hWnd: HWND) -> BOOL;
    pub fn SetWindowTextA(hWnd: HWND, lpString: LPCSTR) -> BOOL;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn SetActiveWindow(hWnd: HWND) -> HWND;
    pub fn LoadIconA(hInstance: HINSTANCE, lpIconName: LPCSTR) -> HICON;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON;
    pub fn LoadBitmapA(hInstance: HINSTANCE, lpBitmapName: LPCSTR) -> HBITMAP;
    pub fn LoadBitmapW(hInstance: HINSTANCE, lpBitmapName: LPCWSTR) -> HBITMAP;
    pub fn LoadCursorA(hInstance: HINSTANCE, lpCursorName: LPCSTR) -> HCURSOR;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
    pub fn LoadCursorFromFileA(lpFileName: LPCSTR) -> HCURSOR;
    pub fn LoadCursorFromFileW(lpFileName: LPCWSTR) -> HCURSOR;
    pub fn CreateIcon(
        hInstance: HINSTANCE,
        nWidth: c_int,
        nHeight: c_int,
        cPlanes: BYTE,
        cBitsPixel: BYTE,
        lpbANDbits: *const BYTE,
        lpbXORbits: *const BYTE,
    ) -> HICON;
    pub fn DestroyIcon(hIcon: HICON) -> BOOL;
    pub fn LookupIconIdFromDirectory(presbits: PBYTE, fIcon: BOOL) -> c_int;
    pub fn LookupIconIdFromDirectoryEx(
        presbits: PBYTE,
        fIcon: BOOL,
        cxDesired: c_int,
        cyDesired: c_int,
        Flags: UINT,
    ) -> c_int;
    pub fn CreateIconFromResource(
        presbits: PBYTE,
        dwResSize: DWORD,
        fIcon: BOOL,
        dwVer: DWORD,
    ) -> HICON;
    pub fn CreateIconFromResourceEx(
        presbits: PBYTE,
        dwResSize: DWORD,
        fIcon: BOOL,
        dwVer: DWORD,
        cxDesired: c_int,
        cyDesired: c_int,
        Flags: UINT,
    ) -> HICON;
    pub fn LoadImageA(
        hInst: HINSTANCE,
        name: LPCSTR,
        type_: UINT,
        cx: c_int,
        cy: c_int,
        fuLoad: UINT,
    ) -> HANDLE;
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_: UINT,
        cx: c_int,
        cy: c_int,
        fuLoad: UINT,
    ) -> HANDLE;
    pub fn CopyImage(h: HANDLE, type_: UINT, cx: c_int, cy: c_int, flags: UINT) -> HANDLE;
    pub fn GetSysColor(nIndex: c_int) -> DWORD;
    pub fn GetSysColorBrush(nIndex: c_int) -> HBRUSH;
    pub fn SetSysColors(
        cElements: c_int,
        lpaElements: *const INT,
        lpaRgbValues: *const COLORREF,
    ) -> BOOL;
    pub fn DrawFocusRect(hDC: HDC, lprc: *const RECT) -> BOOL;
    pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> c_int;
    pub fn GetStockObject(i: c_int) -> HGDIOBJ;
    pub fn GetWindowDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
    pub fn BeginPaint(hwnd: HWND, lpPaint: LPPAINTSTRUCT) -> HDC;
    pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    pub fn GetUpdateRect(hWnd: HWND, lpRect: LPRECT, bErase: BOOL) -> BOOL;
    pub fn GetUpdateRgn(hWnd: HWND, hRgn: HRGN, bErase: BOOL) -> c_int;
    pub fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> c_int;
    pub fn GetWindowRgn(hWnd: HWND, hRgn: HRGN) -> c_int;
    pub fn GetWindowRgnBox(hWnd: HWND, lprc: LPRECT) -> c_int;
    pub fn ExcludeUpdateRgn(hDC: HDC, hWnd: HWND) -> c_int;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub fn ValidateRect(hWnd: HWND, lpRect: *const RECT) -> BOOL;
    pub fn InvalidateRgn(hWnd: HWND, hRgn: HRGN, bErase: BOOL) -> BOOL;
    pub fn ValidateRgn(hWnd: HWND, hRgn: HRGN) -> BOOL;
    pub fn RedrawWindow(hwnd: HWND, lprcUpdate: *const RECT, hrgnUpdate: HRGN, flags: UINT)
        -> BOOL;
    pub fn DwmDefWindowProc(
        hWnd: HWND,
        msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        plResult: *mut LRESULT,
    ) -> BOOL;
    pub fn DwmSetWindowAttribute(
        hWnd: HWND,
        dwAttribute: DWORD,
        pvAttribute: LPCVOID,
        cbAttribute: DWORD,
    ) -> HRESULT;
    pub fn DwmExtendFrameIntoClientArea(hWnd: HWND, pMarInset: *const MARGINS) -> HRESULT;
    pub fn OpenThemeData(hwnd: HWND, pszClassList: LPCWSTR) -> HTHEME;
    pub fn CloseThemeData(hTheme: HTHEME) -> HRESULT;
    pub fn DrawThemeBackground(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: c_int,
        iStateId: c_int,
        pRect: LPCRECT,
        pClipRect: LPCRECT,
    ) -> HRESULT;
    pub fn DrawThemeBackgroundEx(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: c_int,
        iStateId: c_int,
        pRect: LPCRECT,
        pOptions: *const DTBGOPTS,
    ) -> HRESULT;
    pub fn DrawThemeText(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: c_int,
        iStateId: c_int,
        pszText: LPCWSTR,
        cchText: c_int,
        dwTextFlags: DWORD,
        dwTextFlags2: DWORD,
        pRect: LPCRECT,
    ) -> HRESULT;
    pub fn GetThemeBackgroundContentRect(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: c_int,
        iStateId: c_int,
        pBoundingRect: LPCRECT,
        pContentRect: LPRECT,
    ) -> HRESULT;
    pub fn GetThemeBackgroundExtent(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: c_int,
        iStateId: c_int,
        pContentRect: LPCRECT,
        pExtentRect: LPRECT,
    ) -> HRESULT;
    pub fn GetThemeBackgroundRegion(
        hTheme: HTHEME,
        hdc: HDC,
        iPartId: c_int,
        iStateId: c_int,
        pRect: LPCRECT,
        pRegion: *mut HRGN,
    ) -> HRESULT;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn AddFontResourceA(_: LPCSTR) -> c_int;
    pub fn AddFontResourceW(_: LPCWSTR) -> c_int;
    pub fn AnimatePalette(
        hPal: HPALETTE,
        iStartIndex: UINT,
        cEntries: UINT,
        ppe: *const PALETTEENTRY,
    ) -> BOOL;
    pub fn Arc(
        hdc: HDC,
        x1: c_int,
        y1: c_int,
        x2: c_int,
        y2: c_int,
        x3: c_int,
        y3: c_int,
        x4: c_int,
        y4: c_int,
    ) -> BOOL;
    pub fn BitBlt(
        hdc: HDC,
        x: c_int,
        y: c_int,
        cx: c_int,
        cy: c_int,
        hdcSrc: HDC,
        x1: c_int,
        y1: c_int,
        rop: DWORD,
    ) -> BOOL;
    pub fn CancelDC(hdc: HDC) -> BOOL;
    pub fn Chord(
        hdc: HDC,
        x1: c_int,
        y1: c_int,
        x2: c_int,
        y2: c_int,
        x3: c_int,
        y3: c_int,
        x4: c_int,
        y4: c_int,
    ) -> BOOL;
    pub fn ChoosePixelFormat(hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR) -> c_int;
    pub fn CombineRgn(hrgnDst: HRGN, hrgnSrc1: HRGN, hrgnSrc2: HRGN, iMode: c_int) -> c_int;
    pub fn CreateBitmap(
        nWidth: c_int,
        nHeight: c_int,
        nPlanes: UINT,
        nBitCount: UINT,
        lpBits: *const c_void,
    ) -> HBITMAP;
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: c_int, cy: c_int) -> HBITMAP;
    pub fn CreateDiscardableBitmap(hdc: HDC, cx: c_int, cy: c_int) -> HBITMAP;
    pub fn CreateEllipticRgn(x1: c_int, y1: c_int, x2: c_int, y2: c_int) -> HRGN;
    pub fn CreateEllipticRgnIndirect(lprect: *const RECT) -> HRGN;
    pub fn CreateFontA(
        cHeight: c_int,
        cWidth: c_int,
        cEscapement: c_int,
        cOrientation: c_int,
        cWeight: c_int,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: LPCSTR,
    ) -> HFONT;
    pub fn CreateFontW(
        cHeight: c_int,
        cWidth: c_int,
        cEscapement: c_int,
        cOrientation: c_int,
        cWeight: c_int,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: LPCWSTR,
    ) -> HFONT;
    pub fn CreateHatchBrush(iHatch: c_int, color: COLORREF) -> HBRUSH;
    pub fn CreateMetaFileA(pszFile: LPCSTR) -> HDC;
    pub fn CreateMetaFileW(pszFile: LPCWSTR) -> HDC;
    pub fn CreatePalette(plpal: *const LOGPALETTE) -> HPALETTE;
    pub fn CreatePen(iStyle: c_int, cWidth: c_int, color: COLORREF) -> HPEN;
    pub fn CreatePolyPolygonRgn(
        pptl: *const POINT,
        pc: *const INT,
        cPoly: c_int,
        iMode: c_int,
    ) -> HRGN;
    pub fn CreatePatternBrush(hbm: HBITMAP) -> HBRUSH;
    pub fn CreateRectRgn(x1: c_int, y1: c_int, x2: c_int, y2: c_int) -> HRGN;
    pub fn CreateRectRgnIndirect(lprect: *const RECT) -> HRGN;
    pub fn CreateRoundRectRgn(
        x1: c_int,
        y1: c_int,
        x2: c_int,
        y2: c_int,
        w: c_int,
        h: c_int,
    ) -> HRGN;
    pub fn CreateScalableFontResourceA(
        fdwHidden: DWORD,
        lpszFont: LPCSTR,
        lpszFile: LPCSTR,
        lpszPath: LPCSTR,
    ) -> BOOL;
    pub fn CreateScalableFontResourceW(
        fdwHidden: DWORD,
        lpszFont: LPCWSTR,
        lpszFile: LPCWSTR,
        lpszPath: LPCWSTR,
    ) -> BOOL;
    pub fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn DescribePixelFormat(
        hdc: HDC,
        iPixelFormat: c_int,
        nBytes: UINT,
        ppfd: LPPIXELFORMATDESCRIPTOR,
    ) -> c_int;
    pub fn CreateDIBSection(
        hdc: HDC,
        lpbmi: *const BITMAPINFO,
        usage: UINT,
        ppvBits: *mut *mut c_void,
        hSection: HANDLE,
        offset: DWORD,
    ) -> HBITMAP;
    pub fn GetDIBColorTable(hdc: HDC, iStart: UINT, cEntries: UINT, prgbq: *mut RGBQUAD) -> UINT;
    pub fn SetDIBColorTable(hdc: HDC, iStart: UINT, cEntries: UINT, prgbq: *const RGBQUAD) -> UINT;
    pub fn GetViewportExtEx(hdc: HDC, lpsize: LPSIZE) -> BOOL;
    pub fn GetViewportOrgEx(hdc: HDC, lppoint: LPPOINT) -> BOOL;
    pub fn GetWindowExtEx(hdc: HDC, lpsize: LPSIZE) -> BOOL;
    pub fn GetWindowOrgEx(hdc: HDC, lppoint: LPPOINT) -> BOOL;
    pub fn IntersectClipRect(
        hdc: HDC,
        left: c_int,
        top: c_int,
        right: c_int,
        bottom: c_int,
    ) -> c_int;
    pub fn InvertRgn(hdc: HDC, hrgn: HRGN) -> BOOL;
    pub fn LineTo(hdc: HDC, nXEnd: c_int, nYEnd: c_int) -> BOOL;
    pub fn MaskBlt(
        hdcDest: HDC,
        xDest: c_int,
        yDest: c_int,
        width: c_int,
        height: c_int,
        hdcSrc: HDC,
        xSrc: c_int,
        ySrc: c_int,
        hbmMask: HBITMAP,
        xMask: c_int,
        yMask: c_int,
        rop: DWORD,
    ) -> BOOL;
    pub fn PlgBlt(
        hdcDest: HDC,
        lpPoint: *const POINT,
        hdcSrc: HDC,
        xSrc: c_int,
        ySrc: c_int,
        width: c_int,
        height: c_int,
        hbmMask: HBITMAP,
        xMask: c_int,
        yMask: c_int,
    ) -> BOOL;
    pub fn OffsetClipRgn(hdc: HDC, x: c_int, y: c_int) -> c_int;
    pub fn OffsetRgn(hrgn: HRGN, x: c_int, y: c_int) -> c_int;
    pub fn PatBlt(
        hdc: HDC,
        nXLeft: c_int,
        nYLeft: c_int,
        nWidth: c_int,
        nHeight: c_int,
        dwRop: DWORD,
    ) -> BOOL;
    pub fn Pie(
        hdc: HDC,
        nLeftRect: c_int,
        nTopRect: c_int,
        nRightRect: c_int,
        nBottomRect: c_int,
        nXRadial1: c_int,
        nYRadial1: c_int,
        nXRadial2: c_int,
        nYRadial2: c_int,
    ) -> BOOL;
    pub fn PaintRgn(hdc: HDC, hrgn: HRGN) -> BOOL;
    pub fn PolyPolygon(
        hdc: HDC,
        lpPoints: *const POINT,
        lpPolyCounts: *const INT,
        cCount: DWORD,
    ) -> BOOL;
    pub fn PtInRegion(hrgn: HRGN, x: c_int, y: c_int) -> BOOL;
    pub fn PtVisible(hdc: HDC, x: c_int, y: c_int) -> BOOL;
    pub fn RectInRegion(hrgn: HRGN, lprect: *const RECT) -> BOOL;
    pub fn RectVisible(hdc: HDC, lprect: *const RECT) -> BOOL;
    pub fn Rectangle(hdc: HDC, left: c_int, top: c_int, right: c_int, bottom: c_int) -> BOOL;
    pub fn RestoreDC(hdc: HDC, nSavedDC: c_int) -> BOOL;
    pub fn RealizePalette(hdc: HDC) -> UINT;
    pub fn RemoveFontResourceA(lpFileName: LPCSTR) -> BOOL;
    pub fn RemoveFontResourceW(lpFileName: LPCWSTR) -> BOOL;
    pub fn RoundRect(
        hdc: HDC,
        nLeftRect: c_int,
        nTopRect: c_int,
        nRightRect: c_int,
        nBottomRect: c_int,
        nWidth: c_int,
        nHeight: c_int,
    ) -> BOOL;
    pub fn ResizePalette(hpal: HPALETTE, n: UINT) -> BOOL;
    pub fn SaveDC(hdc: HDC) -> c_int;
    pub fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> c_int;
    pub fn ExtSelectClipRgn(hdc: HDC, hrgn: HRGN, mode: c_int) -> c_int;
    pub fn SetMetaRgn(hdc: HDC) -> c_int;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn SelectPalette(hdc: HDC, hPal: HPALETTE, bForceBkgd: BOOL) -> HPALETTE;
    pub fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SetDCBrushColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SetDCPenColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SetBkMode(hdc: HDC, mode: c_int) -> c_int;
    pub fn SetBitmapBits(hbm: HBITMAP, cb: DWORD, pvBits: *const VOID) -> LONG;
    pub fn SetBoundsRect(hdc: HDC, lprect: *const RECT, flags: UINT) -> UINT;
    pub fn SetDIBits(
        hdc: HDC,
        hbm: HBITMAP,
        start: UINT,
        cLines: UINT,
        lpBits: *const VOID,
        lpbmi: *const BITMAPINFO,
        ColorUse: UINT,
    ) -> c_int;
    pub fn SetDIBitsToDevice(
        hdc: HDC,
        xDest: c_int,
        yDest: c_int,
        w: DWORD,
        h: DWORD,
        xSrc: c_int,
        ySrc: c_int,
        StartScan: UINT,
        cLines: UINT,
        lpvBits: *const VOID,
        lpbmi: *const BITMAPINFO,
        ColorUse: UINT,
    ) -> c_int;
    pub fn SetMapperFlags(hdc: HDC, flags: DWORD) -> DWORD;
    pub fn SetGraphicsMode(hdc: HDC, iMode: c_int) -> c_int;
    pub fn SetMapMode(hdc: HDC, mode: c_int) -> c_int;
    pub fn SetLayout(hdc: HDC, l: DWORD) -> DWORD;
    pub fn GetLayout(hdc: HDC) -> DWORD;
    pub fn SetPaletteEntries(
        hpal: HPALETTE,
        iStart: UINT,
        cEntries: UINT,
        pPalEntries: *const PALETTEENTRY,
    ) -> UINT;
    pub fn SetPixel(hdc: HDC, x: c_int, y: c_int, color: COLORREF) -> COLORREF;
    pub fn SetPixelV(hdc: HDC, x: c_int, y: c_int, color: COLORREF) -> BOOL;
    pub fn SetPixelFormat(
        hdc: HDC,
        iPixelFormat: c_int,
        ppfd: *const PIXELFORMATDESCRIPTOR,
    ) -> BOOL;
    pub fn SetPolyFillMode(hdc: HDC, iPolyFillMode: c_int) -> c_int;
    pub fn StretchBlt(
        hdcDest: HDC,
        xDest: c_int,
        yDest: c_int,
        wDest: c_int,
        hDest: c_int,
        hdcSrc: HDC,
        xSrc: c_int,
        ySrc: c_int,
        wSrc: c_int,
        hSrc: c_int,
        rop: DWORD,
    ) -> BOOL;
    pub fn SetRectRgn(hrgn: HRGN, left: c_int, top: c_int, right: c_int, bottom: c_int) -> BOOL;
    pub fn StretchDIBits(
        hdc: HDC,
        XDest: c_int,
        YDest: c_int,
        nDestWidth: c_int,
        nDestHeight: c_int,
        XSrc: c_int,
        YSrc: c_int,
        nSrcWidth: c_int,
        nSrcHeight: c_int,
        lpBits: *const VOID,
        lpBitsInfo: *const BITMAPINFO,
        iUsage: UINT,
        dwRop: DWORD,
    ) -> c_int;
    pub fn SetROP2(hdc: HDC, rop2: c_int) -> c_int;
    pub fn SetStretchBltMode(hdc: HDC, mode: c_int) -> c_int;
    pub fn SetSystemPaletteUse(hdc: HDC, uuse: UINT) -> UINT;
    pub fn SetTextCharacterExtra(hdc: HDC, extra: c_int) -> c_int;
    pub fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SetTextAlign(hdc: HDC, align: UINT) -> UINT;
    pub fn SetTextJustification(hdc: HDC, extra: c_int, count: c_int) -> BOOL;
    pub fn UpdateColors(hdc: HDC) -> BOOL;
    pub fn MoveWindow(
        hWnd: HWND,
        X: c_int,
        Y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        bRepaint: BOOL,
    ) -> BOOL;
    pub fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: c_int,
        Y: c_int,
        cx: c_int,
        cy: c_int,
        uFlags: UINT,
    ) -> BOOL;
    pub fn AdjustWindowRect(lpRect: LPRECT, dwStyle: DWORD, bMenu: BOOL) -> BOOL;
    pub fn AdjustWindowRectEx(
        lpRect: LPRECT,
        dwStyle: DWORD,
        bMenu: BOOL,
        dwExStyle: DWORD,
    ) -> BOOL;
    pub fn AdjustWindowRectExForDpi(
        lpRect: LPRECT,
        dwStyle: DWORD,
        bMenu: BOOL,
        dwExStyle: DWORD,
        dpi: UINT,
    ) -> BOOL;
    pub fn PeekMessageW(
        lpMsg: LPMSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    pub fn GetMessageW(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    pub fn TranslateMessage(lpmsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpmsg: *const MSG) -> LRESULT;
    pub fn IsIconic(hWnd: HWND) -> BOOL;
    pub fn GetCursorPos(lpPoint: &mut POINT) -> BOOL;
    pub fn GetPhysicalCursorPos(lpPoint: &mut POINT) -> BOOL;
    pub fn PhysicalToLogicalPoint(hWnd: HWND, lpPoint: &mut POINT) -> BOOL;
    pub fn PhysicalToLogicalPointForPerMonitorDPI(hWnd: HWND, lpPoint: &mut POINT) -> BOOL;
    pub fn GetDeviceCaps(hdc: HDC, nIndex: c_int) -> c_int;
    pub fn GetForegroundWindow() -> HWND;
    pub fn GetClientRect(aWnd: HWND, lpRect: &mut RECT);
    pub fn ScreenToClient(hWnd: HWND, lpPoint: &mut POINT);
    pub fn SetProcessDPIAware() -> BOOL;
    pub fn SetProcessDpiAwareness(value: PROCESS_DPI_AWARENESS) -> HRESULT;
    pub fn GetProcessDpiAwareness(hProcess: HANDLE, value: *mut PROCESS_DPI_AWARENESS) -> HRESULT;
    pub fn GetDpiForMonitor(
        hmonitor: HMONITOR,
        dpiType: MONITOR_DPI_TYPE,
        dpiX: *mut UINT,
        dpiY: *mut UINT,
    ) -> HRESULT;
    pub fn MonitorFromPoint(pt: POINT, dwFlags: DWORD) -> HMONITOR;
    pub fn MonitorFromRect(lprc: LPCRECT, dwFlags: DWORD) -> HMONITOR;
    pub fn MonitorFromWindow(hwnd: HWND, dwFlags: DWORD) -> HMONITOR;
    pub fn SetWindowTheme(hwnd: HWND, pszSubAppName: LPCWSTR, pszSubIdList: LPCWSTR) -> HRESULT;
    pub fn PostQuitMessage(nExitCode: INT);
    pub fn LoadLibraryA(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    pub fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    pub fn FreeLibraryAndExitThread(hLibModule: HMODULE, dwExitCode: DWORD);
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
    pub fn CancelIoEx(hFile: HANDLE, lpOverlapped: LPOVERLAPPED) -> BOOL;
    pub fn CancelIo(hFile: HANDLE) -> BOOL;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn ReleaseSemaphore(
        hSemaphore: HANDLE,
        lReleaseCount: LONG,
        lpPreviousCount: LPLONG,
    ) -> BOOL;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn WaitForSingleObjectEx(hHandle: HANDLE, dwMilliseconds: DWORD, bAlertable: BOOL)
        -> DWORD;
    pub fn SetProcessDEPPolicy(dwFlags: DWORD) -> BOOL;
    pub fn ToUnicodeEx(
        wVirtKey: UINT,
        wScanCode: UINT,
        lpKeyState: *const BYTE,
        pwszBuff: LPWSTR,
        cchBuff: c_int,
        wFlags: UINT,
        dwhkl: HKL,
    ) -> c_int;
    pub fn ToAscii(
        uVirtKey: UINT,
        uScanCode: UINT,
        lpKeyState: *const BYTE,
        lpChar: LPWORD,
        uFlags: UINT,
    ) -> c_int;
    pub fn ToAsciiEx(
        uVirtKey: UINT,
        uScanCode: UINT,
        lpKeyState: *const BYTE,
        lpChar: LPWORD,
        uFlags: UINT,
        dwhkl: HKL,
    ) -> c_int;
    pub fn ToUnicode(
        wVirtKey: UINT,
        wScanCode: UINT,
        lpKeyState: *const BYTE,
        lwszBuff: LPWSTR,
        cchBuff: c_int,
        wFlags: UINT,
    ) -> c_int;
}
