
// Implementation for windows systems, based on the Win32 API.

#![cfg(target_os = "windows")]

use crate::RasterizedGlyph;

// Win32 type definitions

type INT       = i32;
type UINT      = u32;
type LONG      = i32;
type BOOL      = INT;
type DWORD     = u32;

type CHAR      = i8;
type WCHAR     = i16;
type LPCWSTR   = *const WCHAR;
type LPCSTR    = *const CHAR;
type LPWSTR    = *mut WCHAR;

type VOID      = std::ffi::c_void;
type PVOID     = *mut VOID;
type LPVOID    = PVOID;

type HANDLE    = PVOID;
type HDC       = HANDLE;
type HBITMAP   = HANDLE;
type HGDIOBJ   = HANDLE;
type HRSRC     = HANDLE;
type HFONT     = HANDLE;
type HINSTANCE = HANDLE;
type HMODULE   = HINSTANCE;

type COLORREF  = DWORD;

/// Kernel32 bindings.
#[link(name = "kernel32")]
extern "system" {
    // https://docs.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar
    fn MultiByteToWideChar(
        CodePage      : UINT  ,
        dwFlags       : DWORD ,
        lpMultiByteStr: LPCSTR,
        cbMultiByte   : INT   ,
        lpWideCharStr : LPWSTR,
        cchWideChar   : INT
    ) -> INT;
}

/// Gdi32 bindings.
#[link(name = "gdi32")]
extern "system" {
    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createcompatibledc
    fn CreateCompatibleDC(
        hdc: HDC
    ) -> HDC;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deletedc
    fn DeleteDC(
        hdc: HDC
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createcompatiblebitmap
    fn CreateCompatibleBitmap(
        hdc: HDC,
        cx : INT,
        cy : INT
    ) -> HBITMAP;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-selectobject
    fn SelectObject(
        hdc: HDC    ,
        h  : HGDIOBJ
    ) -> HGDIOBJ;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-deleteobject
    fn DeleteObject(
        ho: HGDIOBJ
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-addfontmemresourceex
    fn AddFontMemResourceEx(
        pFileView: PVOID     ,
        cjSize   : DWORD     ,
        pvResrved: PVOID     ,
        pNumFonts: *mut DWORD
    ) -> HANDLE;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-removefontmemresourceex
    fn RemoveFontMemResourceEx(
        h: HANDLE
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createfontw
    fn CreateFontW(
        cHeight        : INT,
        cWidth         : INT,
        cEscapement    : INT,
        cOrientation   : INT,
        cWeight        : INT,
        bItalic        : DWORD,
        bUnderline     : DWORD,
        bStrikeOut     : DWORD,
        iCharSet       : DWORD,
        iOutPrecision  : DWORD,
        iClipPrecision : DWORD,
        iQuality       : DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName    : LPCWSTR
    ) -> HFONT;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-getpixel
    fn GetPixel(
        hdc: HDC,
        x  : INT,
        y  : INT
    ) -> COLORREF;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-gettextextentpoint32w
    fn GetTextExtentPoint32W(
        hdc     : HDC    ,
        lpString: LPCWSTR,
        c       : INT    ,
        psizl   : LPSIZE ,
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-settextcolor
    fn SetTextColor(
        hdc  : HDC     ,
        color: COLORREF
    ) -> COLORREF;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-textoutw
    fn TextOutW(
        hdc     : HDC    ,
        x       : INT    ,
        y       : INT    ,
        lpString: LPCWSTR,
        c       : INT
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-patblt
    fn PatBlt(
        hdc: HDC,
        x  : INT,
        y  : INT,
        w  : INT,
        h  : INT,
        rop: DWORD
    ) -> BOOL;

    // https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setbkmode
    fn SetBkMode(
        hdc : HDC,
        mode: INT
    ) -> INT;
}

// Used constants from Win32
const CP_UTF8            : UINT     = 65001;
const BLACKNESS          : DWORD    = 66;
const CLR_INVALID        : COLORREF = 4294967295;
const TRANSPARENT        : INT      = 1;
const FW_NORMAL          : INT      = 400;
const DEFAULT_CHARSET    : DWORD    = 1;
const OUT_DEFAULT_PRECIS : DWORD    = 0;
const CLIP_DEFAULT_PRECIS: DWORD    = 0;
const ANTIALIASED_QUALITY: DWORD    = 4;
const DEFAULT_PITCH      : DWORD    = 0;
const FF_DONTCARE        : DWORD    = 0;

// https://docs.microsoft.com/en-us/previous-versions/dd145106(v=vs.85)
#[repr(C)]
struct SIZE {
    cx: LONG,
    cy: LONG,
}
type LPSIZE = *mut SIZE;

impl SIZE {
    fn new() -> Self {
        unsafe{ std::mem::zeroed() }
    }
}

/// UTF-8 to UTF-16 conversion.
fn utf8_to_utf16(s: &str) -> Box<[WCHAR]> {
    // Null terminate
    let mut s: String = s.into();
    s.push('\0');
    // Actual conversion
    let len = unsafe{ MultiByteToWideChar(CP_UTF8, 0, s.as_ptr() as _, -1, 0 as _, 0) };
    let mut res = Vec::with_capacity(len as usize);
    unsafe{
        MultiByteToWideChar(CP_UTF8, 0, s.as_ptr() as _, -1, res.as_mut_ptr(), len);
        res.set_len(len as usize);
    }
    res.into_boxed_slice()
}

// Implementation of the font API

pub struct Win32Font {
    resource_handle: HANDLE,
}

impl Win32Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let mut nfonts: DWORD = 0;
        let resource_handle = unsafe{ AddFontMemResourceEx(bytes.as_ptr() as _, bytes.len() as _, std::ptr::null_mut(), &mut nfonts) };
        if resource_handle == std::ptr::null_mut() {
            return Err(());
        }
        Ok(Self{
            resource_handle,
        })
    }

    pub fn scale(&self) -> Result<Win32ScaledFont, ()> {
        Win32ScaledFont::create(self.resource_handle)
    }
}

impl Drop for Win32Font {
    fn drop(&mut self) {
        unsafe{ RemoveFontMemResourceEx(self.resource_handle) };
    }
}

pub struct Win32ScaledFont {
    font_handle: HFONT  ,

    dc         : HDC    ,
    bitmap     : HBITMAP,

    buff_w     : usize  ,
    buff_h     : usize  ,
}

impl Win32ScaledFont {
    fn create(res: HANDLE) -> Result<Win32ScaledFont, ()> {
        // TODO: Use guards?
        // Create font
        // TODO: Actual size
        // TODO: Name
        let font_handle = unsafe{ CreateFontW(128, 0, 0, 0, FW_NORMAL, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, ANTIALIASED_QUALITY,
            DEFAULT_PITCH | FF_DONTCARE, utf8_to_utf16("Austine Demo").as_ptr()) };
        if font_handle == std::ptr::null_mut() {
            return Err(());
        }
        // Create Device Context
        let dc = unsafe{ CreateCompatibleDC(std::ptr::null_mut()) };
        if dc == std::ptr::null_mut() {
            unsafe{ DeleteObject(font_handle) };
            return Err(());
        }
        // Select the font for the Device Context
        if unsafe{ SelectObject(dc, font_handle) } == std::ptr::null_mut() {
            unsafe{ DeleteObject(font_handle) };
            unsafe{ DeleteDC(dc) };
            return Err(());
        }
        // Create bitmap
        // TODO: Size
        let bitmap = unsafe{ CreateCompatibleBitmap(dc, 0, 0) };
        if bitmap == std::ptr::null_mut() {
            unsafe{ DeleteObject(font_handle) };
            unsafe{ DeleteDC(dc) };
            return Err(());
        }
        // Select the bitmap for the Device Context
        if unsafe{ SelectObject(dc, bitmap) } == std::ptr::null_mut() {
            unsafe{ DeleteObject(font_handle) };
            unsafe{ DeleteObject(bitmap) };
            unsafe{ DeleteDC(dc) };
            return Err(());
        }
        // We succeeded in creating everything
        Ok(Win32ScaledFont{
            font_handle,

            dc,
            bitmap,

            buff_w: 0,
            buff_h: 0,
        })
    }

    fn ensure_buffer_size(&mut self, width: usize, height: usize) -> bool {
        if self.buff_w >= width && self.buff_h >= height {
            // Already enough
            return true;
        }
        // Need to resize
        let bitmap = unsafe{ CreateCompatibleBitmap(self.dc, width as INT, height as INT) };
        if bitmap == std::ptr::null_mut() {
            return false;
        }
        // Select the bitmap for the Device Context
        if unsafe{ SelectObject(self.dc, bitmap) } == std::ptr::null_mut() {
            unsafe{ DeleteObject(bitmap) };
            return false;
        }
        // Succeeded, delete old bitmap and swap
        unsafe{ DeleteObject(self.bitmap) };
        self.bitmap = bitmap;
        self.buff_w = width;
        self.buff_h = height;
        true
    }

    pub fn rasterize_glyph(&mut self, codepoint: char) -> Result<RasterizedGlyph, ()> {
        // Convert to UTF16
        let utf16str = utf8_to_utf16(&format!("{}", codepoint));
        // Get coordinates
        let mut size = SIZE::new();
        if unsafe{ GetTextExtentPoint32W(self.dc, utf16str.as_ptr(), utf16str.len() as _, &mut size) } == 0 {
            return Err(());
        }
        let width = size.cx;
        let height = size.cy;
        // Ensure buffer size
        if !self.ensure_buffer_size(width as usize, height as usize) {
            return Err(());
        }
        // Set clear behavior
        if unsafe{ SetBkMode(self.dc, TRANSPARENT) } == 0 {
            return Err(());
        }
        // Clear the bitmap
        unsafe{ PatBlt(self.dc, 0, 0, width, height, BLACKNESS) };
        // Set text color
        if unsafe{ SetTextColor(self.dc, 0x00ffffff) } == CLR_INVALID {
            return Err(());
        }
        // Render to bitmap
        if unsafe{ TextOutW(self.dc, 0, 0, utf16str.as_ptr(), utf16str.len() as _) } == 0 {
            return Err(());
        }
        // Create the buffer
        let mut data = vec![0u8; (width * height) as usize].into_boxed_slice();
        // Copy the data to the buffer
        for y in 0..height {
            let yoff = (y * width) as usize;
            for x in 0..width {
                let pixel = unsafe{ GetPixel(self.dc, x, y) };
                data[yoff + x as usize] = (pixel & 0xff) as u8;
            }
        }
        // We succeeded
        Ok(RasterizedGlyph{
            width: width as usize,
            height: height as usize,
            data,
        })
    }
}

impl Drop for Win32ScaledFont {
    fn drop(&mut self) {
        unsafe{ DeleteObject(self.font_handle) };
        unsafe{ DeleteObject(self.bitmap) };
        unsafe{ DeleteDC(self.dc) };
    }
}
