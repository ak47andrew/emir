use minifb::{Key, Window, WindowOptions};
use crate::color::Color;
use crate::font_manager::FontManager;

pub struct WindowWrapper {
    width: usize,
    height: usize,

    window: Window,
    buff: Vec<Color>
}

impl WindowWrapper {
    pub fn new(width: usize, height: usize) -> WindowWrapper {
        let buff: Vec<Color> = vec![Color::default(); width * height];

        let mut window = Window::new(
            "Interactive Pixel Buffer",
            width,
            height,
            WindowOptions::default(),
        ).unwrap();
        window.set_target_fps(60);

        WindowWrapper {buff, window, width, height}
    }

    pub fn write_buff(&mut self, buff: Vec<Color>) {
        if buff.len() != self.buff.len() {
            panic!("Wrong buffer size!");
        }
        self.buff = buff;
    }

    pub fn update(&mut self) {
        self.window.update_with_buffer(bytemuck::cast_slice(&self.buff), self.width, self.height).unwrap();
    }

    pub fn is_should_close(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(Key::Escape)
    }

    #[inline(always)]
    fn idx(x: usize, y: usize, width: usize) -> usize {y * width + x}

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.buff[WindowWrapper::idx(x, y, self.width)] = color;
    }

    pub fn set_pixel_range_from_value(&mut self, x: usize, y: usize, length: usize, color: Color) {
        let start = WindowWrapper::idx(x, y, self.width);
        self.buff[start..(start + length)].fill(color);
    }

    pub fn set_pixel_range_from_array(&mut self, x: usize, y: usize, length: usize, colors: &[Color]) {
        let start = WindowWrapper::idx(x, y, self.width);
        if colors.len() != length {
            panic!("Wrong buffer size!");
        }
        self.buff[start..(start + length)].copy_from_slice(colors);
    }

    pub fn set_pixels_between_points(&mut self, y: usize, x1: usize, x2: usize, color: Color) {
        let start = WindowWrapper::idx(x1, y, self.width);
        let end = WindowWrapper::idx(x2, y, self.width);
        self.buff[start..end].fill(color);
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        self.buff[WindowWrapper::idx(x, y, self.width)]
    }

    pub fn get_pixel_range(&self, x: usize, y: usize) -> &[Color] {
        let start = WindowWrapper::idx(x, y, self.width);
        &self.buff[start..(start + self.width)]
    }

    pub fn get_pixel_range_mut(&mut self, x: usize, y: usize) -> &mut [Color] {
        let start = WindowWrapper::idx(x, y, self.width);
        &mut self.buff[start..(start + self.width)]
    }

    fn blend_pixel(&self, x: usize, y: usize, color: Color, alpha: u8) -> Color {
        self.get_pixel(x, y).lerp(color, alpha as f32 / 255.0)
    }

    pub fn draw_char(&mut self, font: &FontManager, c: char, size: f32, color: Color, x: usize, y: usize) {
        let (metrics, bitmap) = font.prepare_character(c, size);

        for dy in 0..metrics.height {
            for dx in 0..metrics.width {
                let coverage = bitmap[dy * metrics.width + dx];
                self.set_pixel(x + dx, y + dy, self.blend_pixel(x + dx, y + dy, color, coverage))
            }
        }
    }

    pub fn draw_string(&mut self, font: &FontManager, s: &str, size: f32, color: Color, x: usize, y: usize) {
        let chars = s.chars().collect::<Vec<char>>();

        let mut x = x;
        for c in chars {
            self.draw_char(&font, c, size, color, x, y);
            x += font.spacing;
            x += font.prepare_character(c, size).0.width;
        }
    }

    /// x, y - center of the circle
    pub fn draw_circle_fill(&mut self, x: usize, y: usize, r: u16, color: Color) {
        let r = r as i32;
        for dy in -r..=r {
            // Range for r^2 - dy^2 is [0; r^2] so we don't give a fuck about checking
            let dx = (r * r - dy * dy).isqrt();
            let dx_1 = -dx; let dx_2 = dx;
            self.set_pixels_between_points((y as i32 + dy) as usize, (x as i32 + dx_1) as usize, (x as i32 + dx_2) as usize, color);
            self.update();
        }
    }

    /// x, y - center of the circle
    pub fn draw_circle_stroke(&mut self, x: usize, y: usize, r: usize, color: Color) {
        // TODO: add thickness bc one pixel is basically invisible
        // Yeah, it's duplicated code. But at the moment, I don't give a fuck
        let r = r as i32;
        let r_sq = r * r;
        for dx in -r..=r {
            for dy in -r..=r {
                let d_sq = dx * dx + dy * dy;
                if d_sq == r_sq {
                    self.set_pixel((x as i32 + dx) as usize, (y as i32 + dy) as usize, color);
                }
            }
        }
    }

    pub fn draw_rect_fill(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        for dy in 0..h {
            self.set_pixel_range_from_value(x, y + dy, w, color);
        }
    }

    pub fn draw_rect_stroke(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        // TODO: add thickness (even tho one pixel here is much more visible)
        match h {
            0 => {},
            1 => {
                self.set_pixel_range_from_value(x, y, w, color);
            },
            2 => {
                self.set_pixel_range_from_value(x, y, w, color);
                self.set_pixel_range_from_value(x, y + 1, w, color);
            }
            _ => {
                self.set_pixel_range_from_value(x, y, w, color);
                for dy in 0..=h-2 {
                    self.set_pixel(x, y + dy, color);
                    self.set_pixel(x + w, y + dy, color);
                }
                self.set_pixel_range_from_value(x, y + h - 1, w, color);
            }
        }
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }
    pub fn get_window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}
