
// Common font abstraction between font file types.

use crate::ttf::TtfFile;

/// Represents font file metadata in a platform-independent way.
pub(crate) struct FontFile {
    pub(crate) extension : String     ,
    pub(crate) face_names: Vec<String>,
}

impl FontFile {
    /// Creates the metadata by parsing a slice of bytes. The parser tries to
    /// guess the correct format.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        // Try TTF
        if let Ok(ttf) = TtfFile::parse(bytes) {
            if let Some(name) = ttf.get_name(4) {
                return Ok(Self{
                    extension: "ttf".into(),
                    face_names: vec![name.into()],
                });
            }
        }
        Err(())
    }

    /// Returns the appropriate extension name for this font type.
    pub(crate) fn get_extension(&self) -> &str {
        &self.extension
    }

    /// Returns the font face names stored in this font.
    pub(crate) fn get_face_names(&self) -> &[String] {
        &self.face_names
    }
}
