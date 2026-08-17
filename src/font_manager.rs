use std::fs;
use fontdue::{Font, FontSettings, Metrics};

pub struct FontManager {
    font: Font
}

impl FontManager {
    pub fn new(font_path: &str) -> FontManager {
        let font_data = fs::read(font_path).expect("File not found");

        FontManager {
            font: Font::from_bytes(font_data, FontSettings::default()).expect("Failed to load font. Something is wrong with it ig")
        }
    }

    pub fn prepare_character(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
        self.font.rasterize(c, px)
    }
}
