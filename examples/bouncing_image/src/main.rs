use std::time::Instant;

use emir::Vector2D;
use emir::color::Color;
use emir::font_manager::FontManager;
use emir::pixel_buffer::PixelBuffer;
use emir::texture::Texture;
use emir::window_manager::WindowManager;
use emir::window_options::{ResizeMode, WindowManagerOptions};

pub const WINDOW_SIZE: Vector2D<usize> = Vector2D {x: 1280, y: 720};

fn main() {
    env_logger::init();

    let options = WindowManagerOptions::new("Emir", None, Some(ResizeMode::Trim));
    let mut window_wrapper: WindowManager = WindowManager::new(WINDOW_SIZE, options).unwrap();
    let font_manager = match FontManager::from_raw(Vec::from(include_bytes!("../font.ttf"))) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    window_wrapper.add_render_step(|window_wrapper, new_size, buffer| {
        let _: &mut WindowManager = window_wrapper;
        let _: &mut PixelBuffer = buffer;
        for y in 0..new_size.y {
            for x in 0..new_size.x {
                let red = (255 * y / new_size.y) as u8;
                let green = (255 * x / new_size.x) as u8;
                let blue = !red.min(!green);

                buffer.set_pixel(Vector2D::new(x, y), Color::new(red, green, blue));
            }
        }
    });

    let img = Texture::from_file("output.jpg").unwrap();

    window_wrapper.add_draw_step(|window_wrapper, size| {
        window_wrapper
            .draw_circle_fill(Vector2D::new(0, 0), 50, Color::RED)
            .unwrap();
        window_wrapper
            .draw_circle_stroke(Vector2D::new(0, 0), 50, Color::WHITE)
            .unwrap();
        window_wrapper
            .draw_rect_fill(Vector2D::new(50, 50), Vector2D::new(size.x + 20, 50), Color::WHITE)
            .unwrap();
        window_wrapper
            .draw_rect_stroke(Vector2D::new(50, 50), Vector2D::new(size.x + 20, 50), Color::BLACK)
            .unwrap();
    });

    let font_id = window_wrapper.load_font(font_manager);
    window_wrapper.add_draw_step(move |window_wrapper, _| {
        window_wrapper.draw_string(
            font_id,
            "Really long text to try out the thing. Really, it should go out of the box. Why the fuck it's so ununiform btw? WTF is going on man?",
            72.0,
            Color::BLACK,
            Vector2D::new(10, 10)
        ).unwrap();
    });

    let mut pos = Vector2D::new(10.0f32, 100.0f32);
    let mut direction = Vector2D::new(1.0f32, 1.0f32);
    const SPEED: f32 = 1000.0;

    let mut last_time = Instant::now();
    let mut frame_count = 0;

    while !window_wrapper.should_close() {
        let delta = window_wrapper.delta_time();
        let size = window_wrapper.get_window_size_vector().as_f32s();

        pos += direction * (SPEED * delta);

        if pos.x >= size.x {
            direction.x *= -1.0;
            pos.x = size.x;
        } else if pos.x < 0.0 {
            direction.x *= -1.0;
            pos.x = 0.0;
        }
        if pos.y >= size.y {
            direction.y *= -1.0;
            pos.y = size.y;
        } else if pos.y < 0.0 {
            direction.y *= -1.0;
            pos.y = 0.0;
        }
        window_wrapper
            .with_buffer_mut(|buff| buff.blit_texture(pos.as_usizes(), &img))
            .unwrap();
        window_wrapper.update().unwrap();

        frame_count += 1;
        let elapsed = last_time.elapsed().as_secs_f32();

        if elapsed >= 1.0 {
            let fps = frame_count as f32 / elapsed;
            window_wrapper.get_window_mut().set_title(&format!("Emir - FPS: {:.2}", fps));
            frame_count = 0;
            last_time = Instant::now();
        }
    }
}
