use std::collections::HashMap;
use std::hint::cold_path;
use std::time::{Duration, Instant};

use crate::Error;
use crate::color::Color;
use crate::error::{DrawError, PixelBufferError, WindowError};
use crate::font_manager::FontManager;
use crate::key::Key;
use crate::mouse_key::MouseKey;
use crate::pixel_buffer::PixelBuffer;
use crate::window_options::{ResizeMode, WindowManagerOptions};
use minifb::{KeyRepeat, ScaleMode, Window, WindowOptions};

pub trait UserRenderStep {
    fn call(
        &mut self,
        window_manger: &mut WindowManager,
        new_size: (usize, usize),
        buffer: &mut PixelBuffer,
    );
    fn clone(&self) -> Box<dyn UserRenderStep>;
}

pub trait UserUpdateStep {
    fn call(&mut self, window_manger: &mut WindowManager, new_size: (usize, usize));
    fn clone(&self) -> Box<dyn UserUpdateStep>;
}

impl<F: FnMut(&mut WindowManager, usize, usize, &mut PixelBuffer) + Clone + 'static> UserRenderStep
    for F
{
    fn call(
        &mut self,
        window_manager: &mut WindowManager,
        new_size: (usize, usize),
        buffer: &mut PixelBuffer,
    ) {
        self(window_manager, new_size.0, new_size.1, buffer)
    }

    fn clone(&self) -> Box<dyn UserRenderStep> {
        Box::new(Clone::clone(self))
    }
}

impl<F: FnMut(&mut WindowManager, usize, usize) + Clone + 'static> UserUpdateStep for F {
    fn call(&mut self, window_manager: &mut WindowManager, new_size: (usize, usize)) {
        self(window_manager, new_size.0, new_size.1)
    }

    fn clone(&self) -> Box<dyn UserUpdateStep> {
        Box::new(Clone::clone(self))
    }
}

struct UserUpdateStepBox(Box<dyn UserUpdateStep>);
struct UserRenderStepBox(Box<dyn UserRenderStep>);

impl Clone for UserUpdateStepBox {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Clone for UserRenderStepBox {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct FontId(u64);

pub struct WindowManager {
    w: usize,
    h: usize,

    window: Window,
    buff: PixelBuffer,

    before_last_render: Instant,
    last_render: Instant,

    render_steps: Vec<UserRenderStepBox>,
    draw_steps: Vec<UserUpdateStepBox>,

    loaded_fonts: HashMap<FontId, FontManager>,
    next_font_id: FontId,

    during_render_step: bool,
}

impl WindowManager {
    pub fn new(w: usize, h: usize, options: WindowManagerOptions) -> Result<Self, WindowError> {
        let buff: PixelBuffer = PixelBuffer::new(w, h);

        let mut window_options = WindowOptions::default();
        if let Some(resize_mode) = options.resize_mode.as_ref() {
            window_options.resize = true;
            window_options.scale_mode = match resize_mode {
                ResizeMode::Fit => ScaleMode::Stretch,
                ResizeMode::FitPreserveAspectRatio => ScaleMode::AspectRatioStretch,
                ResizeMode::Trim => ScaleMode::UpperLeft,
            };
        }
        let mut window = Window::new(options.title.as_str(), w, h, window_options)
            .map_err(|x| WindowError::Create { source: x })?;
        window.set_target_fps(options.fps_cap.map_or(0, |value| value.get() as usize));

        let before_last_render = Instant::now();
        let last_render = Instant::now();
        Ok(WindowManager {
            buff,
            window,
            w,
            h,
            before_last_render,
            last_render,
            render_steps: vec![],
            draw_steps: vec![],
            loaded_fonts: HashMap::new(),
            next_font_id: FontId(0),
            during_render_step: false,
        })
    }

    #[must_use = "The whole reason this function exists is to easily propagate an error in case we are in a render step"]
    #[inline(always)]
    fn ensure_not_in_render_step(&self) -> Result<(), DrawError> {
        if self.during_render_step {
            cold_path();
            return Err(DrawError::DrawDuringRenderStep);
        }
        Ok(())
    }

    pub fn write_buff(&mut self, buff: PixelBuffer) -> Result<(), Error> {
        self.ensure_not_in_render_step()?;

        if !buff.will_it_fit(&self.buff) {
            let (orig_w, orig_h) = self.buff.get_dimensions();
            let (new_w, new_h) = buff.get_dimensions();
            return Err(Error::PixelBuffer(PixelBufferError::IncorrectBufferSize {
                orig_w,
                orig_h,
                new_w,
                new_h,
            }));
        }
        self.buff = buff;
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Error> {
        self.ensure_not_in_render_step()?;
        // TODO: Would probably be a good idea to make sure we aren't inside a draw step either.

        let size = self.get_window_size();
        if self.w != size.0 || self.h != size.1 {
            let mut buffer = PixelBuffer::new(size.0, size.1);
            self.w = size.0;
            self.h = size.1;
            self.during_render_step = true;
            self.call_user_render_steps(&mut buffer);
            self.during_render_step = false;
            self.buff = buffer;
            self.call_user_draw_steps();
        }

        self.window
            .update_with_buffer(self.buff.to_array(), self.w, self.h)
            .map_err(|x| WindowError::Update { source: x })?;
        self.before_last_render = self.last_render;
        self.last_render = Instant::now();

        Ok(())
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(minifb::Key::Escape)
    }

    fn blend_pixel_from(
        buff: &PixelBuffer,
        x: usize,
        y: usize,
        color: Color,
        alpha: u8,
    ) -> Result<Color, Error> {
        Ok(buff.get_pixel(x, y)?.lerp(color, alpha as f32 / 255.0))
    }

    // fn blend_pixel(&self, x: usize, y: usize, color: Color, alpha: u8) -> Result<Color, Error> {
    //     Self::blend_pixel_from(&self.buff, x, y, color, alpha)
    // }

    fn draw_char_font_into(
        buff: &mut PixelBuffer,
        font: &FontManager,
        c: char,
        size: f32,
        color: Color,
        position: (usize, usize),
        buffer_size: (usize, usize),
    ) -> bool {
        let (x, y) = position;
        let (w, h) = buffer_size;

        let (metrics, bitmap) = font.prepare_character(c, size);
        if metrics.width == 0 || metrics.height == 0 {
            return false;
        }

        let Some(free_x) = w.checked_sub(x) else {
            return false;
        };
        let Some(free_y) = h.checked_sub(y) else {
            return false;
        };
        println!("{free_x} {free_y}");

        for dy in 0..metrics.height.min(free_y) {
            for dx in 0..metrics.width.min(free_x) {
                let coverage = bitmap[dy * metrics.width + dx];
                buff.set_pixel(
                    x + dx,
                    y + dy,
                    Self::blend_pixel_from(buff, x + dx, y + dy, color, coverage).unwrap(),
                )
            }
        }

        true
    }

    pub fn draw_char(
        &mut self,
        font_id: FontId,
        c: char,
        size: f32,
        color: Color,
        x: usize,
        y: usize,
    ) -> Result<bool, DrawError> {
        self.ensure_not_in_render_step()?;

        let Some(font) = self.loaded_fonts.get(&font_id) else {
            return Err(DrawError::FontNotLoaded { font_id });
        };

        Ok(Self::draw_char_font_into(
            &mut self.buff,
            font,
            c,
            size,
            color,
            (x, y),
            (self.w, self.h),
        ))
    }

    pub fn draw_string(
        &mut self,
        font_id: FontId,
        s: &str,
        size: f32,
        color: Color,
        x: usize,
        y: usize,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

        let Some(font) = self.loaded_fonts.get(&font_id) else {
            return Err(DrawError::FontNotLoaded { font_id });
        };

        let positions = font.layout(s, x, y, size);

        for pos in positions {
            Self::draw_char_font_into(
                &mut self.buff,
                font,
                pos.parent,
                size,
                color,
                (pos.x as usize, pos.y as usize),
                (self.w, self.h),
            );
        }

        Ok(())
    }

    /// x, y - center of the circle
    pub fn draw_circle_fill(
        &mut self,
        x: usize,
        y: usize,
        r: u16,
        color: Color,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

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

        Ok(())
    }

    /// x, y - center of the circle
    pub fn draw_circle_stroke(
        &mut self,
        x: usize,
        y: usize,
        r: usize,
        color: Color,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

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

        Ok(())
    }

    pub fn draw_line_low(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: Color,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

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

        Ok(())
    }

    pub fn draw_line_high(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: Color,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

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

        Ok(())
    }

    pub fn draw_line(
        &mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color: Color,
    ) -> Result<(), DrawError> {
        // Bresenham's line algorithm: https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
        // I think because all of this is usizes, and we're handling x1 > x0 and y1 > y0, casting to and from i32 can't cause any trouble
        let x0 = x0 as i32;
        let y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        if (y1 - y0).abs() < (x1 - x0).abs() {
            if x0 > x1 {
                self.draw_line_low(x1, y1, x0, y0, color)
            } else {
                self.draw_line_low(x0, y0, x1, y1, color)
            }
        } else {
            if y0 > y1 {
                self.draw_line_high(x1, y1, x0, y0, color)
            } else {
                self.draw_line_high(x0, y0, x1, y1, color)
            }
        }
    }

    pub fn draw_rect_fill(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: Color,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

        for dy in 0..h {
            self.buff.set_pixel_range_from_value(x, y + dy, w, color);
        }

        Ok(())
    }

    pub fn draw_rect_stroke(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: Color,
    ) -> Result<(), DrawError> {
        self.ensure_not_in_render_step()?;

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

        Ok(())
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

    #[inline]
    pub fn since_last_frame(&self) -> Duration {
        self.last_render.duration_since(self.before_last_render)
    }

    #[inline(always)]
    pub fn delta_time(&self) -> f32 {
        self.since_last_frame().as_secs_f32()
    }

    #[inline(always)]
    pub fn delta_time_f64(&self) -> f64 {
        self.since_last_frame().as_secs_f64()
    }

    #[inline(always)]
    pub fn get_window_size(&self) -> (usize, usize) {
        self.window.get_size()
    }

    pub fn add_render_step(
        &mut self,
        render_step: impl FnMut(&mut WindowManager, usize, usize, &mut PixelBuffer) + Clone + 'static,
    ) {
        self.render_steps
            .push(UserRenderStepBox(Box::new(render_step)));
    }

    pub fn add_draw_step(
        &mut self,
        draw_step: impl FnMut(&mut WindowManager, usize, usize) + Clone + 'static,
    ) {
        self.draw_steps.push(UserUpdateStepBox(Box::new(draw_step)));
    }

    fn call_user_render_steps(&mut self, buffer: &mut PixelBuffer) {
        let size = self.get_window_size();

        for mut step in self.render_steps.clone() {
            step.0.call(self, size, buffer);
        }
    }

    fn call_user_draw_steps(&mut self) {
        let size = self.get_window_size();

        for mut step in self.draw_steps.clone() {
            step.0.call(self, size);
        }
    }

    pub fn load_font(&mut self, font: FontManager) -> FontId {
        let id = self.next_font_id;
        self.next_font_id.0 += 1;
        self.loaded_fonts.insert(id, font);
        id
    }

    pub fn unload_font(&mut self, font_id: FontId) -> Option<FontManager> {
        self.loaded_fonts.remove(&font_id)
    }

    #[inline(always)]
    pub fn with_buffer<T>(&self, func: impl FnOnce(&PixelBuffer) -> T) -> Result<T, DrawError> {
        self.ensure_not_in_render_step()?;

        Ok(func(&self.buff))
    }

    #[inline(always)]
    pub fn with_buffer_mut<T>(
        &mut self,
        func: impl FnOnce(&mut PixelBuffer) -> T,
    ) -> Result<T, DrawError> {
        self.ensure_not_in_render_step()?;

        Ok(func(&mut self.buff))
    }
}
