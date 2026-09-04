use emir::color::Color;
use emir::font_manager::FontManager;
use emir::pixel_buffer::PixelBuffer;
use emir::window_options::WindowManagerOptions;
use emir::window_manager::{WindowManager};

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;

fn main() {
    let options = WindowManagerOptions::new("Emir", Some(999));
    let mut window_wrapper: WindowManager = WindowManager::new(WIDTH, HEIGHT, options).unwrap();
    let font_manager = match FontManager::from_raw(Vec::from(include_bytes!("../font.ttf"))) {
        Ok(m) => m.with_spacing(4),
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    let mut buffer: PixelBuffer = PixelBuffer::new(WIDTH, HEIGHT);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let red = (255 * y / HEIGHT) as u8;
            let green = (255 * x / WIDTH) as u8;
            let blue = !red.min(!green);

            buffer.set_pixel(x, y, Color::new(red, green, blue));
        }
    }
    buffer.set_pixel_range_from_value(1900, 100, 1000, Color::RED);
    window_wrapper.write_buff(buffer).unwrap();

    window_wrapper.draw_circle_fill(0, 0, 50, Color::RED);
    // window_wrapper.draw_circle_stroke(0, 0, 50, Color::WHITE);

    window_wrapper.draw_string(
        &font_manager,
        "Really long text to try out the thing. Really, it should go out of the box. Why the fuck it's so ununiform btw? WTF is going on man?",
        72.0,
        Color::BLACK,
        10,
        10
    );

    while !window_wrapper.is_should_close() {
        window_wrapper.update().unwrap();
    }
}
