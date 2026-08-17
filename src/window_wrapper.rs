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

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        self.buff[WindowWrapper::idx(x, y, self.width)]
    }

    fn blend_pixel(&self, x: usize, y: usize, color: Color, alpha: u8) -> Color {
        self.get_pixel(x, y).lerp(color, alpha as f32 / 255.0)
    }

    pub fn draw_char(&mut self, font: &FontManager, c: char, size: f32, color: Color, x: usize, y: usize) {
        let (metrics, bitmap) = font.prepare_character(c, size);
        // println!("{:?}", bitmap);

        for dy in 0..metrics.height {
            for dx in 0..metrics.width {
                let coverage = bitmap[dy * metrics.width + dx];
                if coverage > 0 {
                    self.set_pixel(x + dx, y + dy, color);
                }
                // self.set_pixel(x + dx, y + dy, self.blend_pixel(x, y, color, coverage))
            }
        }
    }

    pub fn draw_string(&mut self, font: &FontManager, s: &str, size: f32, color: Color, x: usize, y: usize) {
        let spacing = 5;
        let chars = s.chars().collect::<Vec<char>>();

        let mut x = x;
        for c in chars {
            self.draw_char(&font, c, size, color, x, y);
            x += spacing;
            x += font.prepare_character(c, size).0.width;
        }
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }
}
