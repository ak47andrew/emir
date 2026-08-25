use std::fs;
use fontdue::{Font, FontSettings, Metrics};

pub struct FontManager {
    font: Font,
    pub spacing: usize
}

impl FontManager {
    pub fn new(font_path: &str) -> FontManager {
        let font_data = fs::read(font_path).expect("File not found");
        Self::from_raw(font_data)
    }
    
    pub fn from_raw(font_data: Vec<u8>) -> FontManager {
        FontManager {
            font: Font::from_bytes(font_data, FontSettings::default()).expect("Failed to load font. Something is wrong with it ig"),
            spacing: 0
        }        
    }

    pub fn prepare_character(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
        self.font.rasterize(c, px)
    }

    pub fn measure_string(&self, s: &str, px: f32) -> (usize, usize) {
        let width = s
            .chars()
            .map(|c| self.prepare_character(c, px).0.width)
            .sum::<usize>() + (s.len() - 1) * self.spacing;
        let height = (s.chars().filter(|&x| x == '\n').count() + 1) * px as usize;

        (width, height)
    }

    pub fn with_spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }
}
