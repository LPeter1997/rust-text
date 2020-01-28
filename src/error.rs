
// Errors thrown by the library.

/// All the different possible errors.
#[derive(Debug)]
pub enum Error {
    /// Standard Rust IO error.
    IoError(std::io::Error),
    /// Something's wrong with the passed in format.
    FormatError(String),
    /// Something went wrong in the system layer.
    SystemError(String),
    /// The user did something wrong.
    UserError(String),
    /// The glyph could not be found.
    GlyphNotFound(char),
}
