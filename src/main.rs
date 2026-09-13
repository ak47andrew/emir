use std::num::NonZeroU32;

use emir::color::Color;
use emir::font_manager::FontManager;
use emir::pixel_buffer::PixelBuffer;
use emir::texture::Texture;
use emir::window_manager::WindowManager;
use emir::window_options::{ResizeMode, WindowManagerOptions};

pub const WIDTH: usize = 100;
pub const HEIGHT: usize = 100;

fn main() {
    let options = WindowManagerOptions::new("Emir", NonZeroU32::new(999), Some(ResizeMode::Trim));
    let mut window_wrapper: WindowManager = WindowManager::new(WIDTH, HEIGHT, options).unwrap();
    let font_manager = match FontManager::from_raw(Vec::from(include_bytes!("../font.ttf"))) {
        Ok(m) => m.with_spacing(4),
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    window_wrapper.add_render_step(|window_wrapper, width, height, buffer| {
        let _: &mut WindowManager = window_wrapper;
        let _: &mut PixelBuffer = buffer;
        for y in 0..height {
            for x in 0..width {
                let red = (255 * y / height) as u8;
                let green = (255 * x / width) as u8;
                let blue = !red.min(!green);

                buffer.set_pixel(x, y, Color::new(red, green, blue));
            }
        }
        buffer.set_pixel_range_from_value(1900, 100, 1000, Color::RED);
    });

    let img = Texture::from_file("output.jpg").unwrap();

    window_wrapper.add_draw_step(|window_wrapper, _, _| {
        window_wrapper.draw_circle_fill(0, 0, 50, Color::RED);
        window_wrapper.draw_circle_stroke(0, 0, 50, Color::WHITE);
    });

    let font_id = window_wrapper.load_font(font_manager);
    window_wrapper.add_draw_step(move |window_wrapper, _, _| {
        _ = window_wrapper.draw_string(
            font_id,
            "Really long text to try out the thing. Really, it should go out of the box. Why the fuck it's so ununiform btw? WTF is going on man?",
            72.0,
            Color::BLACK,
            10,
            10,
        );
    });

    let mut x_pos = 10.0f32;
    let mut y_pos = 100.0f32;
    let mut x_direction = 1.0;
    let mut y_direction = 1.0;

    while !window_wrapper.should_close() {
        let delta = window_wrapper.delta_time();
        let size = window_wrapper.get_window_size();
        let size = (size.0 as f32, size.1 as f32);
        x_pos += (1000.0 * delta) * x_direction;
        if x_pos >= size.0 {
            x_direction *= -1.0;
            x_pos = size.0;
        } else if x_pos < 0.0 {
            x_direction *= -1.0;
            x_pos = 0.0;
        }
        y_pos += (1000.0 * delta) * y_direction;
        if y_pos >= size.1 {
            y_direction *= -1.0;
            y_pos = size.1;
        } else if y_pos < 0.0 {
            y_direction *= -1.0;
            y_pos = 0.0;
        }
        window_wrapper
            .with_buffer_mut(|buff| buff.blit_texture(x_pos as usize, y_pos as usize, &img));
        window_wrapper.update().unwrap();
    }
}
