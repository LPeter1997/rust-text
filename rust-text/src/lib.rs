
#[macro_use]
mod parse;
mod ttf;
mod font_file;
mod win32;

// Import underlying types.
#[cfg(target_os = "windows")]
mod itypes {
    use crate::win32;

    pub type FontImpl           = win32::Win32Font;
    pub type FontFaceImpl       = win32::Win32FontFace;
    pub type ScaledFontFaceImpl = win32::Win32ScaledFontFace;
}

// Here we lay out a platform-independent wrapper-type just to make sure all
// interfaces match.

/// Represents a loaded font file resource that contains one or more font faces.
pub struct Font(itypes::FontImpl);

impl Font {
    /// Parses the binary contents of a font file.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        Ok(Self(itypes::FontImpl::from_bytes(bytes)?))
    }

    /// Returns list of face names contained in this file.
    pub fn get_face_names(&self) -> &[String] {
        self.0.get_face_names()
    }

    /// Returns a face object based on a face name.
    pub fn get_face(&self, name: &str) -> Result<FontFace, ()> {
        Ok(FontFace(self.0.get_face(name)?))
    }
}

/// Represents a single font face selected from a font file.
pub struct FontFace(itypes::FontFaceImpl);

impl FontFace {
    /// Scales the font face to a given size.
    pub fn scale(&self) -> Result<ScaledFontFace, ()> {
        Ok(ScaledFontFace(self.0.scale()?))
    }
}

/// Represents a font face that has been scaled to a given size.
pub struct ScaledFontFace(itypes::ScaledFontFaceImpl);

impl ScaledFontFace {
    /// Rasterizes the given character to a grayscale bitmap.
    pub fn rasterize_glyph(&mut self, codepoint: char) -> Result<RasterizedGlyph, ()> {
        self.0.rasterize_glyph(codepoint)
    }
}

/// Represents a glyph that has been rasterized into a byte array.
pub struct RasterizedGlyph {
    /// Width of the bitmap in pixels.
    pub width: usize,
    /// Height of the bitmap in pixels.
    pub height: usize,
    /// The bitmap data itself (row-major, grayscale, one byte per pixel).
    pub data: Box<[u8]>,
}
