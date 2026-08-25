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

    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        self.buff[WindowWrapper::idx(x, y, self.width)]
    }

    pub fn get_pixel_range(&self, x: usize, y: usize, length: usize) -> &[Color] {
        let start = WindowWrapper::idx(x, y, self.width);
        &self.buff[start..(start + length)]
    }

    pub fn get_pixel_range_mut(&mut self, x: usize, y: usize, length: usize) -> &mut [Color] {
        let start = WindowWrapper::idx(x, y, self.width);
        &mut self.buff[start..(start + length)]
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
        }
    }

    /// x, y - center of the circle
    pub fn draw_circle_stroke(&mut self, x: usize, y: usize, r: usize, color: Color) {
        // Midpoint Circle Algorithm: https://en.wikipedia.org/wiki/Midpoint_circle_algorithm
        // Yes, probably it's better to do straight to the source, but I'll do it by filling a matrix
        // because it showed this way on Wikipedia, and I'm a baby and don't want to spend time translating more than needed
        // (And this actually might be somewhat more performant tbh, but I'm not sure)
        let mut array = vec![vec![false; 2 * r + 1]; 2 * r + 1];
        let r = r as i32;
        let mut cx = r;
        let mut cy = 0i32;
        let mut p = 1 - r;
        const OFFSETS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        while cx >= cy {
            for (j, k) in OFFSETS {
                array[(j * cx + r) as usize][(k * cy + r) as usize] = true;
                array[(k * cy + r) as usize][(j * cx + r) as usize] = true;
            }
            cx -= if p > 0 {1} else {0};
            cy += 1;
            p += if p > 0 {
                1 - 2 * cx + 2 * cy
            } else {
                1 + 2 * cy
            }
        }

        self.set_pixels_masked(x - r as usize, y - r as usize, array, color);
    }

    pub fn draw_line_low(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }
        #[allow(nonstandard_style)]  // Shut up, it's math stuff
        let mut D = (2 * dy) - dx;
        let mut y = y0;

        for x in x0..=x1 {
            self.set_pixel(x as usize, y as usize, color);
            if D > 0 {
                y = y + yi;
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
        #[allow(nonstandard_style)]  // Shut up, it's math stuff
        let mut D = 2 * dx - dy;
        let mut x = x0;

        for y in y0..=y1 {
            self.set_pixel(x as usize, y as usize, color);
            if D > 0 {
                x = x + xi;
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
            self.set_pixel_range_from_value(x, y + dy, w, color);
        }
    }

    pub fn draw_rect_stroke(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
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
