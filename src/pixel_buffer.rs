use crate::buffer::Buffer;
use crate::color::Color;
use crate::error::BufferError;
use crate::texture::Texture;

#[derive(Clone, Debug)]
pub struct PixelBuffer {
    pub buff: Buffer<Color>,
}

impl PixelBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buff: Buffer::new(w, h),
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.buff.set(x, y, color);
    }

    /// # Warning
    /// Works with raw bytes so wraparounds are expected
    pub fn set_pixel_range_from_value(&mut self, x: usize, y: usize, length: usize, color: Color) {
        if y >= self.buff.h || x >= self.buff.w {
            return;
        }
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        let length = length.min(self.buff.w - 1 - x);
        self.buff.buff[start..(start + length)].fill(color);
    }

    /// # Warning
    /// Works with raw bytes so wraparounds are expected
    pub fn set_pixel_range_from_array(&mut self, x: usize, y: usize, colors: &[Color]) {
        if y >= self.buff.h || x >= self.buff.w {
            return;
        }
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        let length = colors.len().min(self.buff.w - 1 - x);
        self.buff.buff[start..(start + length)].copy_from_slice(&colors[..length]);
    }

    /// Expects that x1 <= x2. Caller is responsible for ensuring that
    ///
    /// ### Warning
    /// Be careful about usize underflows. Something like -10 to 30 turns 18446744073709551606 to 30 and such will be ignored as invalid
    pub fn set_pixels_between_points(&mut self, y: usize, x1: usize, x2: usize, color: Color) {
        if y >= self.buff.h || x1 > self.buff.w || x1 > x2 {
            return;
        }
        let x2 = x2.min(self.buff.w);
        let start = Buffer::<Color>::idx(x1, y, self.buff.w);
        let end = Buffer::<Color>::idx(x2, y, self.buff.w);
        self.buff.buff[start..end].fill(color);
    }

    pub fn set_pixels_masked(&mut self, x: usize, y: usize, mask: Vec<Vec<bool>>, color: Color) {
        for (dy, element) in mask.iter().enumerate() {
            let line = self.get_pixel_range_mut(x, y + dy, element.len());
            for dx in 0..element.len() {
                if element[dx] {
                    line[dx] = color;
                }
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, BufferError> {
        if x >= self.buff.w || y >= self.buff.h {
            return Err(BufferError::InvalidAddress {
                x,
                y,
                w: self.buff.w,
                h: self.buff.h,
            });
        }
        unsafe { Ok(self.buff.get_unchecked(x, y)) }
    }

    pub fn get_pixel_range(&self, x: usize, y: usize, length: usize) -> &[Color] {
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        &self.buff.buff[start..(start + length)]
    }

    pub fn get_pixel_range_mut(&mut self, x: usize, y: usize, length: usize) -> &mut [Color] {
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        &mut self.buff.buff[start..(start + length)]
    }

    pub fn blit(&mut self, x: usize, y: usize, other: &PixelBuffer) {
        for (ind, line) in other.buff.clone().lines().enumerate() {
            self.set_pixel_range_from_array(x, y + ind, &line);
        }
    }

    pub fn blit_texture(&mut self, x: usize, y: usize, texture: &Texture) {
        self.blit(x, y, &texture.buff);
    }

    pub fn to_array(&self) -> &[u32] {
        bytemuck::cast_slice(&self.buff.buff)
    }

    /// Checks if the size is compatible between two buffers (If widths and heights are the same. Square peg in a square hole way)
    ///
    /// Yes, it would've been better to change the name to `is_interchangeable` or whatever, but this name is way too good xD
    pub fn will_it_fit(&self, other: &Self) -> bool {
        // Question I ask myself every time I see a girl :D
        self.buff.w == other.buff.w && self.buff.h == other.buff.h
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.buff.w, self.buff.h)
    }
}
