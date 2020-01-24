
mod win32;

// Import underlying types.
#[cfg(target_os = "windows")]
mod itypes {
    use crate::win32;

    pub type FontImpl       = win32::Win32Font;
    pub type ScaledFontImpl = win32::Win32ScaledFont;
}

// Here we lay out a platform-independent wrapper-type just to make sure all
// interfaces match.

/// Represents a font resource, that contains glyph descriptions and other
/// metadata like kerning.
pub struct Font(itypes::FontImpl);

impl Font {
    pub fn from_bytes() -> Result<Self, ()> {
        Ok(Self(itypes::FontImpl::from_bytes()?))
    }

    pub fn scale(&self) -> Result<ScaledFont, ()> {
        Ok(ScaledFont(self.0.scale()?))
    }
}

/// Represents a font that has been scaled to a given size.
pub struct ScaledFont(itypes::ScaledFontImpl);

impl ScaledFont {
    pub fn render_glyph(&mut self, codepoint: char) -> Result<RenderedGlyph, ()> {
        self.0.render_glyph(codepoint)
    }
}

/// Represents a glyph that has been rendered into a byte array.
pub struct RenderedGlyph {
    /// Width of the bitmap in pixels.
    pub width: usize,
    /// Height of the bitmap in pixels.
    pub height: usize,
    /// The bitmap data itself (row-major, grayscale, one byte per pixel).
    pub data: Box<[u8]>,
}
