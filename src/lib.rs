//!  Decoding and Encoding of BMP Images
//!
//!  A decoder and encoder for BMP (Windows Bitmap) images
//!
//!  # Related Links
//!  * <https://msdn.microsoft.com/en-us/library/windows/desktop/dd183375%28v=vs.85%29.aspx>
//!  * <https://en.wikipedia.org/wiki/BMP_file_format>
//!

pub use crate::decoder::BMPDecoder;
pub use crate::encoder::BMPEncoder;

#[derive(Debug)]
pub enum ImageError {
    FormatError(String),
    UnsupportedError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ImageError {
    fn from(err: std::io::Error) -> ImageError {
        ImageError::IoError(err)
    }
}

mod decoder;
mod encoder;
