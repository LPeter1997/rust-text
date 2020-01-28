
mod error;
#[macro_use] mod parse;
mod ttf;
mod font_file;
mod winapi;
mod win32;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

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
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self(itypes::FontImpl::from_bytes(bytes)?))
    }

    /// Returns list of face names contained in this file.
    pub fn get_face_names(&self) -> &[String] {
        self.0.get_face_names()
    }

    /// Returns a face object based on a face name.
    pub fn get_face(&self, name: &str) -> Result<FontFace> {
        Ok(FontFace(self.0.get_face(name)?))
    }
}

/// Represents a single font face selected from a font file.
pub struct FontFace(itypes::FontFaceImpl);

impl FontFace {
    /// Scales the font face to a given size.
    pub fn scale(&self, pts: f64, dpi: f64) -> Result<ScaledFontFace> {
        Ok(ScaledFontFace(self.0.scale(pts, dpi)?))
    }
}

/// Represents a font face that has been scaled to a given size.
pub struct ScaledFontFace(itypes::ScaledFontFaceImpl);

impl ScaledFontFace {
    /// Rasterizes the given character to a grayscale bitmap.
    pub fn rasterize_glyph(&mut self, codepoint: char) -> Result<RasterizedGlyph> {
        self.0.rasterize_glyph(codepoint)
    }

    /// Shapes the passed in text to get laied out in the plane for rendering.
    pub fn shape_text<F: FnMut(GlyphPositioning)>(&self, text: &str, f: F) -> (i32, i32) {
        self.0.shape_text(text, f)
    }
}

/// Represents a glyph that has been rasterized into a byte array.
pub struct RasterizedGlyph {
    /// Horizontal offset to add when rendering.
    pub x_offset: i32,
    /// Vertical offset to add when rendering.
    pub y_offset: i32,
    /// Width of the bitmap in pixels.
    pub width: usize,
    /// Height of the bitmap in pixels.
    pub height: usize,
    /// The bitmap data itself (row-major, grayscale, one byte per pixel).
    pub data: Box<[u8]>,
}

/// Represents the parameter pack passed back to the user for text shaping.
/// Contains information about the actual character's positioning.
pub struct GlyphPositioning {
    /// The character being positioned.
    pub character: char,
    /// The index of the character (0 based, relative to the first one) being
    /// positioned.
    pub index: usize,
    /// The x offset from 0, 0.
    pub x: i32,
    /// The y offset from 0, 0.
    pub y: i32,
    /// The caret's x position before this character.
    pub caret_x: i32,
    /// The caret's y position before this character.
    pub caret_y: i32,
}
