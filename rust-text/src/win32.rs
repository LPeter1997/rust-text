
// Implementation for windows systems, based on the Win32 API.

#![cfg(target_os = "windows")]

use crate::RenderedGlyph;

// Win32 type definitions

type INT      = i32;
type UINT     = u32;
type LONG     = i32;
type BOOL     = INT;
type DWORD    = u32;

type CHAR     = i8;
type WCHAR    = i16;
type LPCWSTR  = *const WCHAR;
type LPCSTR   = *const CHAR;
type LPWSTR   = *mut WCHAR;

type VOID     = std::ffi::c_void;
type PVOID    = *mut VOID;
type LPVOID   = PVOID;

type HANDLE   = PVOID;
type HDC      = HANDLE;
type HBITMAP  = HANDLE;
type HGDIOBJ  = HANDLE;

type COLORREF = DWORD;

/// Kernel32 bindings.
#[link(name = "kernel32")]
extern "system" {
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
const CP_UTF8    : UINT     = 65001;
const BLACKNESS  : DWORD    = 66;
const CLR_INVALID: COLORREF = 4294967295;
const TRANSPARENT: INT      = 1;

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

pub struct Win32Font { }

impl Win32Font {
    pub fn from_bytes() -> Result<Self, ()> {
        Ok(Self{})
    }

    pub fn scale(&self) -> Result<Win32ScaledFont, ()> {
        Win32ScaledFont::create()
    }
}

pub struct Win32ScaledFont {
    dc    : HDC    ,
    bitmap: HBITMAP,

    buff_w: usize  ,
    buff_h: usize  ,
}

impl Win32ScaledFont {
    fn create() -> Result<Win32ScaledFont, ()> {
        // Create Device Context
        let dc = unsafe{ CreateCompatibleDC(std::ptr::null_mut()) };
        if dc == std::ptr::null_mut() {
            return Err(());
        }
        // Create bitmap
        // TODO: Size
        let bitmap = unsafe{ CreateCompatibleBitmap(dc, 0, 0) };
        if bitmap == std::ptr::null_mut() {
            unsafe{ DeleteDC(dc) };
            return Err(());
        }
        // Select the bitmap for the Device Context
        if unsafe{ SelectObject(dc, bitmap) } == std::ptr::null_mut() {
            unsafe{ DeleteObject(bitmap) };
            unsafe{ DeleteDC(dc) };
            return Err(());
        }
        // We succeeded in creating everything
        Ok(Win32ScaledFont{
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

    pub fn render_glyph(&mut self, codepoint: char) -> Result<RenderedGlyph, ()> {
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
        Ok(RenderedGlyph{
            width: width as usize,
            height: height as usize,
            data,
        })
    }
}

impl Drop for Win32ScaledFont {
    fn drop(&mut self) {
        unsafe{ DeleteObject(self.bitmap) };
        unsafe{ DeleteDC(self.dc) };
    }
}
