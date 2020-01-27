
// Implementation for windows systems, based on the Win32 API.

#![cfg(target_os = "windows")]

use std::io::prelude::*;
use std::fs::File;
use crate::RasterizedGlyph;
use crate::font_file::FontFile;
use crate::winapi::*;
use crate::{Result, Error};

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

/// Writes a file with the given bytes.
fn file_write_bytes(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut buff = File::create(path)?;
    let mut pos = 0;
    while pos < bytes.len() {
        let bytes_written = buff.write(&bytes[pos..])?;
        pos += bytes_written;
    }
    Ok(())
}

/// A wrapper type for a GDI DeviceContext.
struct DeviceContext(HDC);

impl DeviceContext {
    fn is_err(&self) -> bool { self.0.is_null() }

    fn select(&self, obj: &GdiObject) -> bool {
        !unsafe{ SelectObject(self.0, obj.0) }.is_null()
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        if self.0 != std::ptr::null_mut() {
            unsafe{ DeleteDC(self.0) };
        }
    }
}

/// A wrapper type for GDI logical resources that are destroyed using
/// DeleteObject.
struct GdiObject(HGDIOBJ);

impl GdiObject {
    fn is_err(&self) -> bool { self.0.is_null() }
}

impl Drop for GdiObject {
    fn drop(&mut self) {
        if self.0 != std::ptr::null_mut() {
            unsafe{ DeleteObject(self.0) };
        }
    }
}

// Implementation of the font API

// Font

pub struct Win32Font {
    meta   : FontFile    ,
    fname  : String      ,
    fname16: Box<[WCHAR]>,
}

impl Win32Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Get metadata
        let meta = FontFile::from_bytes(bytes)?;
        // Write to file so windows can safely load it as a resource
        // TODO: Some true random name?
        let fname = format!("{}.{}", "_temp", meta.get_extension());
        let fname16 = utf8_to_utf16(&fname);
        // Scope the write so the file gets closed
        file_write_bytes(&fname, bytes).map_err(|e| Error::IoError(e))?;
        // Load resource
        let added_fonts = unsafe{ AddFontResourceExW(fname16.as_ptr(), FR_PRIVATE, std::ptr::null_mut()) };
        if added_fonts == 0 {
            unsafe{ RemoveFontResourceExW(fname16.as_ptr(), FR_PRIVATE, std::ptr::null_mut()) };
            // Remove the file, but don't escalate errors!
            let _ = std::fs::remove_file(&fname);
            return Err(Error::SystemError("AddFontResourceExW failed!".into()));
        }
        // Done
        Ok(Self{
            meta,
            fname,
            fname16,
        })
    }

    pub fn get_face_names(&self) -> &[String] {
        self.meta.get_face_names()
    }

    pub fn get_face(&self, name: &str) -> Result<Win32FontFace> {
        // TODO: Some fuzzy match? Substring match?
        if !self.get_face_names().iter().any(|n| n == name) {
            // No such face
            return Err(Error::UserError(format!("No face named '{}' found in font!", name)));
        }
        // Create the font
        Win32FontFace::create(name)
    }
}

impl Drop for Win32Font {
    fn drop(&mut self) {
        unsafe{ RemoveFontResourceExW(self.fname16.as_ptr(), FR_PRIVATE, std::ptr::null_mut()) };
        let _ = std::fs::remove_file(&self.fname);
    }
}

pub struct Win32FontFace {
    face_name: String,
}

impl Win32FontFace {
    fn create(face_name: &str) -> Result<Self> {
        Ok(Self{
            face_name: face_name.into(),
        })
    }

    pub fn scale(&self) -> Result<Win32ScaledFontFace> {
        Win32ScaledFontFace::create(&self.face_name)
    }
}

// Scaled font face

pub struct Win32ScaledFontFace {
    dc    : DeviceContext,
    bitmap: GdiObject    ,
    _font : GdiObject    ,

    buffer: &'static mut[COLORREF],
    buff_w: usize                 ,
    buff_h: usize                 ,
}

impl Win32ScaledFontFace {
    fn create(face: &str) -> Result<Self> {
        // Create Device Context
        let dc = DeviceContext(unsafe{ CreateCompatibleDC(std::ptr::null_mut()) });
        if dc.is_err() {
            return Err(Error::SystemError("Failed to create Device Context!".into()));
        }
        // Create font
        // TODO: Actual size
        let font = GdiObject(unsafe{ CreateFontW(128, 0, 0, 0, FW_NORMAL, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, ANTIALIASED_QUALITY,
            DEFAULT_PITCH | FF_DONTCARE, utf8_to_utf16(face).as_ptr()) });
        if font.is_err() {
            return Err(Error::SystemError("CreateFontW failed!".into()));
        }
        // Select the font for the Device Context
        if !dc.select(&font) {
            return Err(Error::SystemError("Failed to assign Font to Device Context!".into()));
        }
        // Create bitmap
        // TODO: Size
        let bitmap = GdiObject(unsafe{ CreateCompatibleBitmap(dc.0, 0, 0) });
        if bitmap.is_err() {
            return Err(Error::SystemError("Failed to create Bitmap!".into()));
        }
        // Select the bitmap for the Device Context
        if !dc.select(&bitmap) {
            return Err(Error::SystemError("Failed to assign Bitmap to Device Context!".into()));
        }
        // We succeeded in creating everything
        Ok(Self{
            dc,
            bitmap,
            _font: font,

            buffer: unsafe{ std::slice::from_raw_parts_mut(std::ptr::NonNull::dangling().as_ptr(), 0) },
            buff_w: 0,
            buff_h: 0,
        })
    }

    fn ensure_buffer_size(&mut self, width: usize, height: usize) -> Result<()> {
        if self.buff_w >= width && self.buff_h >= height {
            // Already enough
            return Ok(());
        }
        // Calculate new size
        let width = std::cmp::max(width, self.buff_w);
        let height = std::cmp::max(height, self.buff_h);
        // Need to resize
        let mut info = BITMAPINFO::new();
        info.bmiHeader.biWidth = width as _;
        info.bmiHeader.biHeight = height as _;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB;
        info.bmiHeader.biSizeImage = 0;
        info.bmiHeader.biXPelsPerMeter = 0;
        info.bmiHeader.biYPelsPerMeter = 0;
        info.bmiHeader.biClrUsed = 0;
        info.bmiHeader.biClrImportant = 0;
        let mut bits: PVOID = std::ptr::null_mut();
        let bitmap = GdiObject(unsafe{ CreateDIBSection(self.dc.0, &info, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0) });
        if bitmap.is_err() {
            return Err(Error::SystemError("Failed to create Bitmap!".into()));
        }
        // Select the bitmap for the Device Context
        if !self.dc.select(&bitmap) {
            return Err(Error::SystemError("Failed to assign font to Device Context!".into()));
        }
        // Succeeded, delete old bitmap and swap
        self.bitmap = bitmap;
        self.buff_w = width;
        self.buff_h = height;
        self.buffer = unsafe{ std::slice::from_raw_parts_mut(bits as _, width * height) };
        Ok(())
    }

    fn get_tightest_bounds(&self) -> Bounds {
        let mut result = Bounds::default();

        // Find left bound
        result.left = 0;
        'outer1: for x in 0..self.buff_w {
            for y in 0..self.buff_h {
                if self.buffer[y * self.buff_w + x] != 0 {
                    break 'outer1;
                }
            }
            result.left = x + 1;
        }
        // Find right bound
        result.right = self.buff_w;
        'outer2: for x in (0..self.buff_w).rev() {
            for y in 0..self.buff_h {
                if self.buffer[y * self.buff_w + x] != 0 {
                    break 'outer2;
                }
            }
            result.right = x;
        }
        // Find top bound
        result.top = 0;
        'outer3: for y in 0..self.buff_h {
            for x in 0..self.buff_w {
                if self.buffer[y * self.buff_w + x] != 0 {
                    break 'outer3;
                }
            }
            result.top = y + 1;
        }
        // Find bottom bound
        result.bottom = self.buff_h;
        'outer4: for y in (0..self.buff_h).rev() {
            for x in 0..self.buff_w {
                if self.buffer[y * self.buff_w + x] != 0 {
                    break 'outer4;
                }
            }
            result.bottom = y;
        }

        result
    }

    pub fn rasterize_glyph(&mut self, codepoint: char) -> Result<RasterizedGlyph> {
        // Convert to UTF16
        let utf16str = utf8_to_utf16(&format!("{}", codepoint));
        // Get coordinates
        let mut size = SIZE::new();
        if unsafe{ GetTextExtentPoint32W(self.dc.0, utf16str.as_ptr(), utf16str.len() as _, &mut size) } == 0 {
            return Err(Error::GlyphNotFound(codepoint));
        }
        let required_width = size.cx as usize;
        let required_height = size.cy as usize;
        // Ensure buffer size
        self.ensure_buffer_size(required_width, required_height)?;
        // Set clear behavior
        if unsafe{ SetBkMode(self.dc.0, TRANSPARENT) } == 0 {
            return Err(Error::SystemError("SetBkMode failed!".into()));
        }
        // Clear the bitmap
        unsafe{ PatBlt(self.dc.0, 0, 0, self.buff_w as INT, self.buff_h as INT, BLACKNESS) };
        // Set text color
        if unsafe{ SetTextColor(self.dc.0, 0x00ffffff) } == CLR_INVALID {
            return Err(Error::SystemError("SetTextColor failed!".into()));
        }
        // Render to bitmap
        if unsafe{ TextOutW(self.dc.0, 0, 0, utf16str.as_ptr(), utf16str.len() as _) } == 0 {
            return Err(Error::SystemError("TextOutW failed!".into()));
        }
        // Invert the rows for easier copy (the buffer contents is upside down)
        for y in 0..(self.buff_h / 2) {
            let y_inv = self.buff_h - y - 1;
            for x in 0..self.buff_w {
                self.buffer.swap(
                    y * self.buff_w + x,
                    y_inv * self.buff_w + x);
            }
        }
        // Calculate the tightest bounds
        let bounds = self.get_tightest_bounds();
        if bounds.left > bounds.right {
            // The canvas must be empty, return empty canvas
            return Ok(RasterizedGlyph{
                x_offset: 0,
                y_offset: 0,
                width: 0,
                height: 0,
                data: vec![0u8; 0].into_boxed_slice(),
            });
        }
        let bounds_width = bounds.right - bounds.left;
        let bounds_height = bounds.bottom - bounds.top;
        // Create the resulting buffer
        let mut data = vec![0u8; (bounds_width * bounds_height) as usize].into_boxed_slice();
        // Copy the data to the buffer
        for y in 0..bounds_height {
            let y_buff_offs = (y + bounds.top) * self.buff_w;
            let y_res_offs = y * bounds_width;
            for x in 0..bounds_width {
                let pixel = self.buffer[y_buff_offs + bounds.left + x];
                data[y_res_offs + x] = (pixel & 0xff) as u8;
            }
        }
        // We succeeded
        Ok(RasterizedGlyph{
            x_offset: bounds.left,
            y_offset: bounds.top,
            width: bounds_width,
            height: bounds_height,
            data,
        })
    }

    pub fn shape_text<F: FnMut(usize, usize, char)>(&self, text: &str, mut f: F) -> (usize, usize) {
        // Encode in UTF16
        let text16 = utf8_to_utf16(text);
        // Calculate offsets
        let mut results = GCP_RESULTSW::new();
        let mut glyphs = vec![0i16; text16.len()].into_boxed_slice();
        let mut dx = vec![0i32; text16.len()].into_boxed_slice();
        let mut order = vec![0u32; text16.len()].into_boxed_slice();
        results.lpGlyphs = glyphs.as_mut_ptr();
        results.nGlyphs = text.len() as DWORD;
        results.lpDx = dx.as_mut_ptr();
        results.lpOrder = order.as_mut_ptr();
        let res = unsafe{ GetCharacterPlacementW(self.dc.0,
            text16.as_ptr(), text.len() as INT, 0, &mut results, 0) };
        // The resulting dimensions
        let res_w = (res & 0x0000ffff) as usize;
        let res_h = ((res & 0xffff0000) >> 16) as usize;
        let line_height = res_h;

        // Biggest dimensions
        let mut max_w = 0;
        let mut max_h = 0;

        // Cursor
        let mut xoff = 0;
        let mut yoff = 0;
        // Loop through characters, move cursor along
        let mut chs = text.chars();
        for i in 0..results.nGlyphs {
            let ch = chs.next().unwrap();
            // Get the advance width
            let offs = unsafe{ *results.lpOrder.offset(i as isize) };
            let offs = unsafe{ *results.lpDx.offset(offs as isize) };
            f(xoff, yoff, ch);
            xoff += offs as usize;
            if ch == '\n' {
                xoff = 0;
                yoff += line_height;
            }
            max_w = std::cmp::max(max_w, xoff);
            max_h = std::cmp::max(max_h, yoff + line_height);
        }
        (max_w, max_h)
    }
}

/// Represents bounds for the bitmap.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Bounds {
    left  : usize,
    top   : usize,
    right : usize,
    bottom: usize,
}
