use crate::color::Color;
use crate::error::PixelBufferError;

pub struct PixelBuffer {
    w: usize,
    h: usize,
    pub buff: Vec<Color>,
}

// TODO: optimize
impl PixelBuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self { buff: vec![Color::default(); w * h], w, h }
    }

    #[inline(always)]
    fn idx(x: usize, y: usize, w: usize) -> usize {y * w + x}

    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h or UB (Unleashed butt)
    #[inline]
    pub unsafe fn set_pixel_unsafe(&mut self, x: usize, y: usize, color: Color) {
        unsafe {
            *self.buff.get_unchecked_mut(Self::idx(x, y, self.w)) = color;
        }
    }

    /// # Safety
    /// Caller must ensure that x < self.w and y < self.h or UB (Ultimate banger)
    #[inline]
    pub unsafe fn get_pixel_unsafe(&self, x: usize, y: usize) -> Color {
        unsafe {
            *self.buff.get_unchecked(Self::idx(x, y, self.w))
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.w || y >= self.h {
            return;
        }
        unsafe {
            self.set_pixel_unsafe(x, y, color);
        }
    }

    /// # Warning
    /// Works with raw bytes so wraparounds are expected
    pub fn set_pixel_range_from_value(&mut self, x: usize, y: usize, length: usize, color: Color) {
        let start = PixelBuffer::idx(x, y, self.w);
        self.buff[start..(start + length)].fill(color);
    }

    /// # Warning
    /// Works with raw bytes so wraparounds are expected
    pub fn set_pixel_range_from_array(&mut self, x: usize, y: usize, length: usize, colors: &[Color]) {
        let start = PixelBuffer::idx(x, y, self.w);
        if colors.len() != length {
            panic!("Wrong array size!");
        }
        self.buff[start..(start + length)].copy_from_slice(colors);
    }

    /// Expects that x1 < x2. If not, it depends. If everything is in bounds, it's gonna fix it
    /// but if more than two things are screwed... you're gonna get fucked :D
    pub fn set_pixels_between_points(&mut self, y: usize, x1: usize, x2: usize, color: Color) {
        if y >= self.h {
            return;
        }
        let is_ordered = x1 < x2;
        let is_x1_oob = x1 >= self.w;
        let is_x2_oob = x2 >= self.h;

        // Fucking hack, but it works, fine
        let (x1, x2) = match (is_ordered, is_x1_oob, is_x2_oob) {
            (true, false, false) => {
                (x1, x2)
            }
            (false, false, false) => {
                (x2, x1)
            }
            (false, true, false) => {
                (0, x2)
            }
            (true, false, true) => {
                (x1, self.w - 1)
            }
            (false, true, true) => {
                (0, self.w - 1)
            }
            _ => panic!("Your inputs are fucked. set_pixels_between_points(y={}, x1={}, x2={}, color={:x})",
                            y, x1, x2, color.as_u32())
        };
        let start = PixelBuffer::idx(x1.min(self.w - 1), y, self.w);
        let end = PixelBuffer::idx(x2.min(self.w - 1), y, self.w);
        self.buff[start..end].fill(color);
    }

    pub fn set_pixels_masked(&mut self, x: usize, y: usize, mask: Vec<Vec<bool>>, color: Color) {
        for dy in 0..mask.len() {
            let line = self.get_pixel_range_mut(x, y + dy, mask[dy].len());
            for dx in 0..mask[dy].len() {
                if mask[dy][dx] {
                    line[dx] = color;
                }
            }
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, PixelBufferError> {
        if x >= self.w || y >= self.h {
            return Err(PixelBufferError::InvalidAddress {x, y, w: self.w, h: self.h})
        }
        unsafe {
            Ok(self.get_pixel_unsafe(x, y))
        }
    }

    pub fn get_pixel_range(&self, x: usize, y: usize, length: usize) -> &[Color] {
        let start = PixelBuffer::idx(x, y, self.w);
        &self.buff[start..(start + length)]
    }

    pub fn get_pixel_range_mut(&mut self, x: usize, y: usize, length: usize) -> &mut [Color] {
        let start = PixelBuffer::idx(x, y, self.w);
        &mut self.buff[start..(start + length)]
    }

    pub fn to_array(&self) -> &[u32] {
        bytemuck::cast_slice(&self.buff)
    }

    /// Checks if the size is compatible between two buffers (If widths and heights are the same. Square peg in a square hole way)
    ///
    /// Yes, it would've been better to change the name to `is_interchangeable` or whatever, but this name is way too good xD
    pub fn will_it_fit(&self, other: &Self) -> bool {
        // Question I ask every time I see a girl :D
        self.w == other.w && self.h == other.h
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.w, self.h)
    }
}
