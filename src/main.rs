use emir::color::Color;
use emir::font_manager::FontManager;
use emir::window_wrapper::{WindowWrapper};

pub const WIDTH: usize = 1920;
pub const HEIGHT: usize = 1080;

fn main() {
    let mut window_wrapper = WindowWrapper::new(WIDTH, HEIGHT);
    let font_manager = FontManager::new("font.ttf")
        .with_spacing(3);
    let mut buffer: Vec<Color> = vec![Color::default(); WIDTH * HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let index = y * WIDTH + x;

            let red = (255 * y / HEIGHT) as u8;
            let green = (255 * x / WIDTH) as u8;
            let blue = !red.min(!green);

            buffer[index] = Color::new(red, green, blue);
        }
    }
    window_wrapper.write_buff(buffer.clone());
    for (ind, color) in [Color::BLACK, Color::RED, Color::GREEN, Color::DARKBLUE, Color::LIGHTBLUE, Color::MAGENTA, Color::YELLOW, Color::WHITE].iter().enumerate() {
        window_wrapper.draw_string(&font_manager, "HELLO!", 100.0, *color, 10, 10 + 100 * ind);
    }

    while !window_wrapper.is_should_close() {
        window_wrapper.update();
    }
}
