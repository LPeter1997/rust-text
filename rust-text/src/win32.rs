
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
    font  : GdiObject    ,

    buffer: &'static[COLORREF],
    buff_w: usize      ,
    buff_h: usize      ,
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
            font,

            buffer: unsafe{ std::slice::from_raw_parts(std::ptr::NonNull::dangling().as_ptr(), 0) },
            buff_w: 0,
            buff_h: 0,
        })
    }

    fn ensure_buffer_size(&mut self, width: usize, height: usize) -> Result<()> {
        if self.buff_w >= width && self.buff_h >= height {
            // Already enough
            return Ok(());
        }
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
        self.buffer = unsafe{ std::slice::from_raw_parts(bits as _, width * height) };
        Ok(())
    }

    pub fn rasterize_glyph(&mut self, codepoint: char) -> Result<RasterizedGlyph> {
        // Convert to UTF16
        let utf16str = utf8_to_utf16(&format!("{}", codepoint));
        // Get coordinates
        let mut size = SIZE::new();
        if unsafe{ GetTextExtentPoint32W(self.dc.0, utf16str.as_ptr(), utf16str.len() as _, &mut size) } == 0 {
            return Err(Error::GlyphNotFound(codepoint));
        }
        let width = size.cx;
        let height = size.cy;
        // Ensure buffer size
        self.ensure_buffer_size(width as usize, height as usize)?;
        // Set clear behavior
        if unsafe{ SetBkMode(self.dc.0, TRANSPARENT) } == 0 {
            return Err(Error::SystemError("SetBkMode failed!".into()));
        }
        // Clear the bitmap
        unsafe{ PatBlt(self.dc.0, 0, 0, width, height, BLACKNESS) };
        // Set text color
        if unsafe{ SetTextColor(self.dc.0, 0x00ffffff) } == CLR_INVALID {
            return Err(Error::SystemError("SetTextColor failed!".into()));
        }
        // Render to bitmap
        if unsafe{ TextOutW(self.dc.0, 0, 0, utf16str.as_ptr(), utf16str.len() as _) } == 0 {
            return Err(Error::SystemError("TextOutW failed!".into()));
        }
        // Create the buffer
        let mut data = vec![0u8; (width * height) as usize].into_boxed_slice();
        // Copy the data to the buffer
        for y in 0..height {
            let yoff = ((height - y - 1) * width) as usize;
            for x in 0..width {
                let pixel = self.buffer[(self.buff_w * y as usize) + x as usize];
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
