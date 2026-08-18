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

    let colors = [Color::RED, Color::GREEN, Color::DARKBLUE, Color::LIGHTBLUE, Color::MAGENTA, Color::YELLOW, Color::WHITE];
    let text = "HELLO!";
    let px = 100.0;
    let (text_width, text_height) = font_manager.measure_string(text, px);
    println!("{}x{}", text_width, text_height);

    window_wrapper.draw_rect_fill(0, 0, text_width + 15, text_height * colors.len(), Color::BLACK);
    window_wrapper.draw_rect_stroke(0, 0, text_width + 15, text_height * colors.len(), Color::WHITE);
    window_wrapper.draw_circle_fill(1000, 500, 300, Color::WHITE);
    window_wrapper.draw_circle_stroke(1000, 500, 7, Color::RED);
    for (ind, color) in colors.iter().enumerate() {
        window_wrapper.draw_string(&font_manager, text, 100.0, *color, 10, 10 + 100 * ind);
    }

    while !window_wrapper.is_should_close() {
        window_wrapper.update();
    }
}
