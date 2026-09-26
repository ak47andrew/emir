use crate::buffer::Buffer;
use crate::color::Color;
use crate::error::BufferError;
use crate::texture::Texture;

/// A 2D buffer of [`Color`] pixels, backed by a flat [`Buffer<Color>`].
///
/// This is the core drawing surface used throughout the crate: window
/// contents, textures, etc. are all `PixelBuffer`s under the hood.
#[derive(Clone, Debug)]
pub struct PixelBuffer {
    pub buff: Buffer<Color>,
}

impl PixelBuffer {
    /// Creates a new, zero-initialized `PixelBuffer` with the given width and height.
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            buff: Buffer::new(w, h),
        }
    }

    /// Sets a single pixel at `(x, y)` to `color`.
    ///
    /// Silently does nothing if `(x, y)` is out of bounds.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.buff.set(x, y, color);
    }

    /// Fills `length` consecutive pixels starting at `(x, y)` with `color`.
    ///
    /// # Warning
    /// Works with raw bytes so horizontal wraparounds are expected: if
    /// `x + length` exceeds the row width, the fill continues into the
    /// next row(s) of the underlying flat buffer instead of stopping at
    /// the row edge. No-op if `(x, y)` is out of bounds.
    pub fn set_pixel_range_from_value(&mut self, x: usize, y: usize, length: usize, color: Color) {
        if y >= self.buff.h || x >= self.buff.w {
            return;
        }
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        let length = length.min(self.buff.w - 1 - x);
        self.buff.buff[start..(start + length)].fill(color);
    }

    /// Copies `colors` into the buffer starting at `(x, y)`, one color per pixel.
    ///
    /// # Warning
    /// Works with raw bytes so wraparounds are expected: if `colors` is
    /// longer than the remaining space in the row, the write continues
    /// into the next row(s) of the underlying flat buffer. No-op if
    /// `(x, y)` is out of bounds. If `colors` is longer than the
    /// available space, it is truncated to fit.
    pub fn set_pixel_range_from_array(&mut self, x: usize, y: usize, colors: &[Color]) {
        if y >= self.buff.h || x >= self.buff.w {
            return;
        }
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        let length = colors.len().min(self.buff.w - 1 - x);
        self.buff.buff[start..(start + length)].copy_from_slice(&colors[..length]);
    }

    /// Fills every pixel on row `y` between `x1` and `x2` (exclusive of `x2`) with `color`.
    ///
    /// Expects that `x1 <= x2`; the caller is responsible for ensuring this.
    ///
    /// ### Warning
    /// Be careful about `usize` underflows upstream: passing something
    /// like a computed `-10` for `x1` wraps to `18446744073709551606`,
    /// which is greater than `x2` and will simply be ignored as invalid
    /// (this function returns early rather than panicking).
    pub fn set_pixels_between_points(&mut self, y: usize, x1: usize, x2: usize, color: Color) {
        if y >= self.buff.h || x1 > self.buff.w || x1 > x2 {
            return;
        }
        let x2 = x2.min(self.buff.w);
        let start = Buffer::<Color>::idx(x1, y, self.buff.w);
        let end = Buffer::<Color>::idx(x2, y, self.buff.w);
        self.buff.buff[start..end].fill(color);
    }

    /// Stamps a boolean mask onto the buffer at `(x, y)`, setting each pixel
    /// where the mask is `true` to `color` and leaving `false` cells untouched.
    ///
    /// `mask` is indexed `mask[dy][dx]`, so each inner `Vec<bool>` is one row
    /// of the stamp, starting at row `y` and extending downward.
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

    /// Returns the color at `(x, y)`, or a [`BufferError::InvalidAddress`] if
    /// the coordinates are out of bounds.
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, BufferError> {
        if x >= self.buff.w || y >= self.buff.h {
            return Err(BufferError::InvalidAddress {
                x,
                y,
                w: self.buff.w,
                h: self.buff.h,
            });
        }
        unsafe { Ok(self.buff.get_unchecked(x, y)).cloned() }
    }

    /// Returns an immutable slice of `length` pixels starting at `(x, y)`.
    ///
    /// # Panics
    /// Panics if the requested range falls outside the underlying buffer
    /// (no bounds checking is performed here, unlike [`Self::get_pixel`]).
    pub fn get_pixel_range(&self, x: usize, y: usize, length: usize) -> &[Color] {
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        &self.buff.buff[start..(start + length)]
    }

    /// Returns a mutable slice of `length` pixels starting at `(x, y)`.
    ///
    /// # Panics
    /// Panics if the requested range falls outside the underlying buffer
    /// (no bounds checking is performed here, unlike [`Self::get_pixel`]).
    pub fn get_pixel_range_mut(&mut self, x: usize, y: usize, length: usize) -> &mut [Color] {
        let start = Buffer::<Color>::idx(x, y, self.buff.w);
        &mut self.buff.buff[start..(start + length)]
    }

    /// Copies `other` onto `self` with its top-left corner placed at `(x, y)`.
    ///
    /// Rows that would fall outside `self` are clipped via the same
    /// wraparound behavior as [`Self::set_pixel_range_from_array`].
    pub fn blit(&mut self, x: usize, y: usize, other: &PixelBuffer) {
        let (sw, sh) = (self.buff.w, self.buff.h);
        let (ow, oh) = (other.buff.w, other.buff.h);
        if x >= sw || y >= sh { return; }
        let copy_w = ow.min(sw - x);
        let copy_h = oh.min(sh - y);
        for row in 0..copy_h {
            let src = &other.buff.buff[row * ow..row * ow + copy_w];
            let dst = (y + row) * sw + x;
            self.buff.buff[dst..dst + copy_w].copy_from_slice(src);
        }
    }

    /// Copies a [`Texture`]'s pixel data onto `self`, with its top-left
    /// corner placed at `(x, y)`. Convenience wrapper around [`Self::blit`].
    pub fn blit_texture(&mut self, x: usize, y: usize, texture: &Texture) {
        self.blit(x, y, &texture.buff);
    }

    /// Reinterprets the buffer as a slice of packed `u32` pixels (e.g. for
    /// handing off to `minifb`'s `update_with_buffer`).
    pub fn to_array(&self) -> &[u32] {
        bytemuck::cast_slice(&self.buff.buff)
    }

    /// Checks whether `self` and `other` have identical dimensions
    /// (same width and height), i.e. whether one could be swapped in for
    /// the other.
    ///
    /// Yes, it would've been better to change the name to `is_interchangeable` or whatever, but this name is way too good xD
    pub fn will_it_fit(&self, other: &Self) -> bool {
        // Question I ask myself every time I see a girl :D
        self.buff.w == other.buff.w && self.buff.h == other.buff.h
    }

    /// Returns the `(width, height)` of the buffer.
    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.buff.w, self.buff.h)
    }
}
