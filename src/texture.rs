use std::path::Path;

use image::{GenericImageView, ImageReader};

use crate::{buffer::Buffer, color::Color, error::TextureError, pixel_buffer::PixelBuffer};

pub struct Texture {
    pub buff: PixelBuffer,
}

impl Texture {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TextureError> {
        let path = path.as_ref();
        let img = ImageReader::open(path).map_err(|e| TextureError::OpenError { path: path.to_str().map(|value| value.to_owned()), source: e })?
                            .decode().map_err(|e| TextureError::DecodeError { path: path.to_str().map(|value| value.to_owned()), source: e })?;
        let s = img.to_rgba8().into_raw();
        let w = img.dimensions().0;

        let iterator = s
            .chunks(4)
            .map(|x| Color::new(x[0], x[1], x[2]));

        Ok(Texture { 
            buff: PixelBuffer {
                buff: Buffer::try_from_iterator(w as usize, iterator).unwrap()
            } 
        })
    }
}
