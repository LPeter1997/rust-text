
mod error;
#[macro_use] mod parse;
mod ttf;
mod font_file;
mod winapi;
mod win32;
mod pack;
use std::ops::{BitOr, BitOrAssign, BitAnd, BitAndAssign, BitXor, BitXorAssign, Not};
use pack::PackResult;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub use pack::Rect;
pub type GlyphPack = PackResult<char>;

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
    pub fn face_names(&self) -> &[String] {
        self.0.face_names()
    }

    /// Returns a face object based on a face name.
    pub fn face(&self, name: &str) -> Result<FontFace> {
        Ok(FontFace(self.0.face(name)?))
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
    pub fn shape_text<F: FnMut(GlyphPositioning)>(&self, text: &str, options: ShapeOptions, f: F) -> (i32, i32) {
        self.0.shape_text(text, options, f)
    }
}

/// Represents a glyph that has been rasterized into a byte array.
pub struct RasterizedGlyph {
    /// The character that got rasterized.
    pub character: char,
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

/// Contains options for shaping text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ShapeOptions(u8);

impl ShapeOptions {
    /// Use kerning when calculating coordienates, meaning that spacing is
    /// adjusted between characters for more natural reading.
    pub const USE_KERNING: ShapeOptions = ShapeOptions(0b00000001);

    /// Returns true if a given option (or options) is present in the options.
    pub fn contains(&self, option: ShapeOptions) -> bool {
        (*self & option) == option
    }
}

impl BitOr for ShapeOptions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output { Self(self.0 | rhs.0) }
}

impl BitOrAssign for ShapeOptions {
    fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
}

impl BitAnd for ShapeOptions {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output { Self(self.0 & rhs.0) }
}

impl BitAndAssign for ShapeOptions {
    fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0; }
}

impl BitXor for ShapeOptions {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output { Self(self.0 ^ rhs.0) }
}

impl BitXorAssign for ShapeOptions {
    fn bitxor_assign(&mut self, rhs: Self) { self.0 ^= rhs.0; }
}

impl Not for ShapeOptions {
    type Output = Self;
    fn not(self) -> Self::Output { Self(!self.0) }
}

/// Packs the glyphs with a best-effort algorithm to occupy the least amount of
/// space possible.
pub fn pack_glyphs<'a>(glyphs: impl IntoIterator<Item = &'a RasterizedGlyph>) -> GlyphPack {
    use std::cmp::max;
    pack::bin_pack(glyphs.into_iter(),
        |e| (e.width, e.height), |(w1, h1), (w2, h2)| max(w1, h1).cmp(max(w2, h2)), |e| e.character)
}
