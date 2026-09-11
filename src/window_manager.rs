use crate::Error;
use crate::color::Color;
use crate::error::{PixelBufferError, WindowError};
use crate::font_manager::FontManager;
use crate::key::Key;
use crate::mouse_key::MouseKey;
use crate::pixel_buffer::PixelBuffer;
use crate::window_options::WindowManagerOptions;
use minifb::{KeyRepeat, Window, WindowOptions};

pub struct WindowManager {
    w: usize,
    h: usize,

    window: Window,
    buff: PixelBuffer,
}

impl WindowManager {
    pub fn new(w: usize, h: usize, options: WindowManagerOptions) -> Result<Self, WindowError> {
        let buff: PixelBuffer = PixelBuffer::new(w, h);

        let mut window = Window::new(options.title.as_str(), w, h, WindowOptions::default())
            .map_err(|x| WindowError::Create { source: x })?;
        window.set_target_fps(options.fps_cap.map_or(0, |value| value.get() as usize));

        Ok(WindowManager { buff, window, w, h })
    }

    pub fn write_buff(&mut self, buff: PixelBuffer) -> Result<(), PixelBufferError> {
        if !buff.will_it_fit(&self.buff) {
            let (orig_w, orig_h) = self.buff.get_dimensions();
            let (new_w, new_h) = buff.get_dimensions();
            return Err(PixelBufferError::IncorrectBufferSize {
                orig_w,
                orig_h,
                new_w,
                new_h,
            });
        }
        self.buff = buff;
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), WindowError> {
        self.window
            .update_with_buffer(self.buff.to_array(), self.w, self.h)
            .map_err(|x| WindowError::Update { source: x })?;
        Ok(())
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(minifb::Key::Escape)
    }

    fn blend_pixel(&self, x: usize, y: usize, color: Color, alpha: u8) -> Result<Color, Error> {
        Ok(self.buff.get_pixel(x, y)?.lerp(color, alpha as f32 / 255.0))
    }

    pub fn draw_char(
        &mut self,
        font: &FontManager,
        c: char,
        size: f32,
        color: Color,
        x: usize,
        y: usize,
    ) {
        let (metrics, bitmap) = font.prepare_character(c, size);
        if metrics.width == 0 || metrics.height == 0 {
            return;
        }

        let free_x = self.w as i32 - x as i32;
        let free_y = self.h as i32 - y as i32;
        if free_x < 0 || free_y < 0 {
            return;
        }

        for dy in 0..metrics.height.min(free_y as usize) {
            for dx in 0..metrics.width.min(free_x as usize) {
                let coverage = bitmap[dy * metrics.width + dx];
                self.buff.set_pixel(
                    x + dx,
                    y + dy,
                    self.blend_pixel(x + dx, y + dy, color, coverage).unwrap(),
                )
            }
        }
    }

    pub fn draw_string(
        &mut self,
        font: &FontManager,
        s: &str,
        size: f32,
        color: Color,
        x: usize,
        y: usize,
    ) {
        let positions = font.layout(s, x, y, size);

        for pos in positions {
            self.draw_char(
                font,
                pos.parent,
                size,
                color,
                pos.x as usize,
                pos.y as usize,
            );
        }
    }

    /// x, y - center of the circle
    pub fn draw_circle_fill(&mut self, x: usize, y: usize, r: u16, color: Color) {
        let r = r as i32;
        for dy in -r..=r {
            // Range for r^2 - dy^2 is [0; r^2] so we don't give a fuck about checking
            let dx = (r * r - dy * dy).isqrt();

            let x1 = x as i32 - dx;
            let x2 = x as i32 + dx;
            self.buff.set_pixels_between_points(
                (y as i32 + dy) as usize,
                x1.max(0) as usize,
                x2.max(0) as usize,
                color,
            );
        }
    }

    /// x, y - center of the circle
    pub fn draw_circle_stroke(&mut self, x: usize, y: usize, r: usize, color: Color) {
        // Midpoint Circle Algorithm: https://en.wikipedia.org/wiki/Midpoint_circle_algorithm
        let r = r as i32;
        let mut cx = r;
        let mut cy = 0i32;
        let mut p = 1 - r;
        const OFFSETS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        let x0 = x.wrapping_sub(r as usize);
        let y0 = y.wrapping_sub(r as usize);

        while cx >= cy {
            for (j, k) in OFFSETS {
                self.buff.set_pixel(
                    x0.overflowing_add((j * cx + r) as usize).0,
                    y0.overflowing_add((k * cy + r) as usize).0,
                    color,
                );
                self.buff.set_pixel(
                    x0.overflowing_add((k * cy + r) as usize).0,
                    y0.overflowing_add((j * cx + r) as usize).0,
                    color,
                );
            }
            cx -= if p > 0 { 1 } else { 0 };
            cy += 1;
            p += if p > 0 {
                1 - 2 * cx + 2 * cy
            } else {
                1 + 2 * cy
            }
        }
    }

    pub fn draw_line_low(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }
        #[allow(nonstandard_style)] // Shut up, it's math stuff
        let mut D = (2 * dy) - dx;
        let mut y = y0;

        for x in x0..=x1 {
            self.buff.set_pixel(x as usize, y as usize, color);
            if D > 0 {
                y += yi;
                D += 2 * (dy - dx);
            } else {
                D += 2 * dy;
            }
        }
    }

    pub fn draw_line_high(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let mut dx = x1 - x0;
        let dy = y1 - y0;
        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }
        #[allow(nonstandard_style)] // Shut up, it's math stuff
        let mut D = 2 * dx - dy;
        let mut x = x0;

        for y in y0..=y1 {
            self.buff.set_pixel(x as usize, y as usize, color);
            if D > 0 {
                x += xi;
                D += 2 * (dx - dy)
            } else {
                D += 2 * dx;
            }
        }
    }

    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: Color) {
        // Bresenham's line algorithm: https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
        // I think because all of this is usizes, and we're handling x1 > x0 and y1 > y0, casting to and from i32 can't cause any trouble
        let x0 = x0 as i32;
        let y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        if (y1 - y0).abs() < (x1 - x0).abs() {
            if x0 > x1 {
                self.draw_line_low(x1, y1, x0, y0, color);
            } else {
                self.draw_line_low(x0, y0, x1, y1, color);
            }
        } else {
            if y0 > y1 {
                self.draw_line_high(x1, y1, x0, y0, color);
            } else {
                self.draw_line_high(x0, y0, x1, y1, color);
            }
        }
    }

    pub fn draw_rect_fill(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for dy in 0..h {
            self.buff.set_pixel_range_from_value(x, y + dy, w, color);
        }
    }

    pub fn draw_rect_stroke(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        match h {
            0 => {}
            1 => {
                self.buff.set_pixel_range_from_value(x, y, w, color);
            }
            2 => {
                self.buff.set_pixel_range_from_value(x, y, w, color);
                self.buff.set_pixel_range_from_value(x, y + 1, w, color);
            }
            _ => {
                self.buff.set_pixel_range_from_value(x, y, w, color);
                for dy in 1..=h - 2 {
                    self.buff.set_pixel(x, y + dy, color);
                    self.buff.set_pixel(x + w, y + dy, color);
                }
                self.buff.set_pixel_range_from_value(x, y + h - 1, w, color);
            }
        }
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        let minifb_key = minifb::Key::from(key);
        self.window.is_key_down(minifb_key)
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        let minifb_key = minifb::Key::from(key);
        self.window.is_key_released(minifb_key)
    }

    pub fn is_key_pressed(&self, key: Key, is_key_repeat: bool) -> bool {
        let minifb_key = minifb::Key::from(key);
        self.window.is_key_pressed(
            minifb_key,
            if is_key_repeat {
                KeyRepeat::Yes
            } else {
                KeyRepeat::No
            },
        )
    }

    pub fn get_mouse_pos(&self) -> Option<(f32, f32)> {
        self.window.get_mouse_pos(minifb::MouseMode::Discard)
    }

    pub fn get_mouse_down(&self, mouse_key: MouseKey) -> bool {
        self.window
            .get_mouse_down(minifb::MouseButton::from(mouse_key))
    }

    pub fn get_scroll_wheel(&self) -> f32 {
        self.window.get_scroll_wheel().unwrap_or_default().1
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }
    pub fn get_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}
