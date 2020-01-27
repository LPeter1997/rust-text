
// Minimalistic WinAPI bindings.

#![cfg(target_os = "windows")]

// Win32 type definitions

pub type BYTE      = u8;
pub type INT       = i32;
pub type UINT      = u32;
pub type LONG      = i32;
pub type BOOL      = INT;
pub type WORD      = u16;
pub type DWORD     = u32;

pub type HRESULT   = LONG;

pub type CHAR      = i8;
pub type WCHAR     = i16;
pub type LPCWSTR   = *const WCHAR;
pub type LPCSTR    = *const CHAR;
pub type LPSTR     = *mut CHAR;
pub type LPWSTR    = *mut WCHAR;

pub type VOID      = std::ffi::c_void;
pub type PVOID     = *mut VOID;

pub type HANDLE    = PVOID;
pub type HDC       = HANDLE;
pub type HBITMAP   = HANDLE;
pub type HGDIOBJ   = HANDLE;
pub type HFONT     = HANDLE;

pub type COLORREF  = DWORD;

/// Kernel32 bindings.
#[link(name = "kernel32")]
extern "system" {
    // https://docs.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar
    pub fn MultiByteToWideChar(
        CodePage      : UINT  ,
        dwFlags       : DWORD ,
        lpMultiByteStr: LPCSTR,
        cbMultiByte   : INT   ,
        lpWideCharStr : LPWSTR,
        cchWideChar   : INT   ,
    ) -> INT;
}

/// Gdi32 bindings.
#[link(name = "gdi32")]
extern "system" {
    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createcompatibledc
    pub fn CreateCompatibleDC(
        hdc: HDC
    ) -> HDC;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deletedc
    pub fn DeleteDC(
        hdc: HDC
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createcompatiblebitmap
    pub fn CreateCompatibleBitmap(
        hdc: HDC,
        cx : INT,
        cy : INT,
    ) -> HBITMAP;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createdibsection
    pub fn CreateDIBSection(
        hdc     : HDC              ,
        pbmi    : *const BITMAPINFO,
        usage   : UINT             ,
        ppvBits : *mut *mut VOID   ,
        hSection: HANDLE           ,
        offset  : DWORD            ,
    ) -> HBITMAP;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-selectobject
    pub fn SelectObject(
        hdc: HDC    ,
        h  : HGDIOBJ,
    ) -> HGDIOBJ;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deleteobject
    pub fn DeleteObject(
        ho: HGDIOBJ
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-addfontresourcew
    pub fn AddFontResourceExW(
        name: LPCWSTR,
        fl  : DWORD  ,
        res : PVOID  ,
    ) -> INT;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-removefontresourceexw
    pub fn RemoveFontResourceExW(
        name: LPCWSTR,
        fl  : DWORD  ,
        pdv : PVOID  ,
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createfontw
    pub fn CreateFontW(
        cHeight        : INT    ,
        cWidth         : INT    ,
        cEscapement    : INT    ,
        cOrientation   : INT    ,
        cWeight        : INT    ,
        bItalic        : DWORD  ,
        bUnderline     : DWORD  ,
        bStrikeOut     : DWORD  ,
        iCharSet       : DWORD  ,
        iOutPrecision  : DWORD  ,
        iClipPrecision : DWORD  ,
        iQuality       : DWORD  ,
        iPitchAndFamily: DWORD  ,
        pszFaceName    : LPCWSTR,
    ) -> HFONT;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-gettextextentpoint32w
    pub fn GetTextExtentPoint32W(
        hdc     : HDC    ,
        lpString: LPCWSTR,
        c       : INT    ,
        psizl   : LPSIZE ,
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-settextcolor
    pub fn SetTextColor(
        hdc  : HDC     ,
        color: COLORREF,
    ) -> COLORREF;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-textoutw
    pub fn TextOutW(
        hdc     : HDC    ,
        x       : INT    ,
        y       : INT    ,
        lpString: LPCWSTR,
        c       : INT    ,
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-patblt
    pub fn PatBlt(
        hdc: HDC  ,
        x  : INT  ,
        y  : INT  ,
        w  : INT  ,
        h  : INT  ,
        rop: DWORD,
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setbkmode
    pub fn SetBkMode(
        hdc : HDC,
        mode: INT,
    ) -> INT;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getcharacterplacementw
    pub fn GetCharacterPlacementW(
        hdc       : HDC,
        lpString  : LPCWSTR,
        nCount    : INT,
        nMexExtent: INT,
        lpResults : LPGCP_RESULTSW,
        dwFlags   : DWORD,
    ) -> DWORD;
}

// Used constants from Win32
pub const CP_UTF8            : UINT     = 65001;
pub const BLACKNESS          : DWORD    = 66;
pub const CLR_INVALID        : COLORREF = 4294967295;
pub const TRANSPARENT        : INT      = 1;
pub const FW_NORMAL          : INT      = 400;
pub const DEFAULT_CHARSET    : DWORD    = 1;
pub const OUT_DEFAULT_PRECIS : DWORD    = 0;
pub const CLIP_DEFAULT_PRECIS: DWORD    = 0;
pub const ANTIALIASED_QUALITY: DWORD    = 4;
pub const DEFAULT_PITCH      : DWORD    = 0;
pub const FF_DONTCARE        : DWORD    = 0;
pub const DIB_RGB_COLORS     : UINT     = 0;
pub const BI_RGB             : DWORD    = 0;
pub const FR_PRIVATE         : DWORD    = 0x10;

// https://docs.microsoft.com/en-us/previous-versions/dd145106(v=vs.85)
#[repr(C)]
pub struct SIZE {
    pub cx: LONG,
    pub cy: LONG,
}
pub type LPSIZE = *mut SIZE;

impl SIZE {
    pub fn new() -> Self {
        unsafe{ std::mem::zeroed() }
    }
}

// https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapinfo
#[allow(non_snake_case)]
#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1]    ,
}

impl BITMAPINFO {
    pub fn new() -> Self {
        Self{
            bmiHeader: BITMAPINFOHEADER::new(),
            bmiColors: [RGBQUAD::new()],
        }
    }
}

// https://docs.microsoft.com/en-us/previous-versions/dd183376(v=vs.85)
#[allow(non_snake_case)]
#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize         : DWORD,
    pub biWidth        : LONG ,
    pub biHeight       : LONG ,
    pub biPlanes       : WORD ,
    pub biBitCount     : WORD ,
    pub biCompression  : DWORD,
    pub biSizeImage    : DWORD,
    pub biXPelsPerMeter: LONG ,
    pub biYPelsPerMeter: LONG ,
    pub biClrUsed      : DWORD,
    pub biClrImportant : DWORD,
}

impl BITMAPINFOHEADER {
    pub fn new() -> Self {
        let mut result: Self = unsafe{ std::mem::zeroed() };
        result.biSize = std::mem::size_of::<Self>() as _;
        result
    }
}

// https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-rgbquad
#[allow(non_snake_case)]
#[repr(C)]
pub struct RGBQUAD {
    pub rgbBlue    : BYTE,
    pub rgbGreen   : BYTE,
    pub rgbRed     : BYTE,
    pub rgbReserved: BYTE,
}

impl RGBQUAD {
    pub fn new() -> Self {
        unsafe{ std::mem::zeroed() }
    }
}

// https://docs.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-gcp_resultsw
#[allow(non_snake_case)]
#[repr(C)]
pub struct GCP_RESULTSW {
    pub lStructSize: DWORD    ,
    pub lpOutString: LPWSTR   ,
    pub lpOrder    : *mut UINT,
    pub lpDx       : *mut INT ,
    pub lpCaretPos : *mut INT ,
    pub lpClass    : LPSTR    ,
    pub lpGlyphs   : LPWSTR   ,
    pub nGlyphs    : UINT     ,
    pub nMaxFit    : INT      ,
}
#[allow(non_snake_case)]
pub type LPGCP_RESULTSW = *mut GCP_RESULTSW;

impl GCP_RESULTSW {
    pub fn new() -> Self {
        let mut result: Self = unsafe{ std::mem::zeroed() };
        result.lStructSize = std::mem::size_of::<Self>() as DWORD;
        result
    }
}
