use crate::error::FontError;
use fontdue::layout::{CoordinateSystem, GlyphPosition, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings, Metrics};
use std::fs;

pub struct FontManager {
    font: Font,
    pub spacing: usize,
}

impl FontManager {
    pub fn new(font_path: &str) -> Result<FontManager, FontError> {
        let font_data = fs::read(font_path).map_err(|x| FontError::Read {
            path: font_path.to_string(),
            source: x,
        })?;
        Self::from_raw(font_data)
    }

    pub fn from_raw(font_data: Vec<u8>) -> Result<FontManager, FontError> {
        Ok(FontManager {
            font: Font::from_bytes(font_data, FontSettings::default()).map_err(|x| {
                FontError::Parse {
                    reason: x.to_string(),
                }
            })?,
            spacing: 0,
        })
    }

    pub fn prepare_character(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
        self.font.rasterize(c, px)
    }

    pub fn measure_string(&self, s: &str, px: f32) -> (usize, usize) {
        let width = s
            .chars()
            .map(|c| self.prepare_character(c, px).0.width)
            .sum::<usize>()
            + (s.len() - 1) * self.spacing;
        let height = (s.chars().filter(|&x| x == '\n').count() + 1) * px as usize;

        (width, height)
    }

    pub fn layout(&self, s: &str, x: usize, y: usize, font_size: f32) -> Vec<GlyphPosition> {
        // TODO: think about automatic wrapping
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: x as f32,
            y: y as f32,
            ..LayoutSettings::default()
        });
        // TODO: cloning font here is kinda expensive, but for now I can't find any better solution to it
        layout.append(&[self.font.clone()], &TextStyle::new(s, font_size, 0));
        // TODO: maybe also do something here?
        layout.glyphs().clone()
    }

    pub fn with_spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }
}
