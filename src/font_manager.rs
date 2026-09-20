use crate::error::FontError;
use crate::window_manager::WindowManager;
use fontdue::layout::{CoordinateSystem, GlyphPosition, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings, Metrics};
use std::{fs, io, slice};

/// Struct responsible for loading and rasterizing text as well as generating layout for text-writing
/// methods at [`WindowManager`]
///
/// # Example
/// ```
/// # use emir::prelude::{WindowManagerOptions, WindowManager};
/// # let mut window_wrapper: WindowManager = WindowManager::new(100, 100, WindowManagerOptions::default()).unwrap();
/// use emir::prelude::{FontManager, Color};
/// let font_manager = FontManager::new("font.ttf").unwrap();  // or ::from_raw(Vec::from(include_bytes!("font.ttf")));
/// let font_id = window_wrapper.load_font(font_manager);
/// window_wrapper.draw_string(font_id, "Hello world!", 32.0, Color::WHITE, 5, 5);
/// ```
pub struct FontManager {
    font: Font,
    pub spacing: usize,  // TODO: I fixed it with `.layout()`. Remove it
}

impl FontManager {
    /// Loads font data from specified file path
    ///
    /// # Errors
    /// - [`FontError::Read`] if can't read file due to file not found errors, wrong permissions, etc.
    /// See `.source` to get source [`io::Error`]
    /// - [`FontError::Parse`] if format of the file is incorrect. See `.reason` to get underlying error
    ///
    /// # Example
    /// ```
    /// use emir::prelude::FontManager;
    /// let font_manager = FontManager::new("font.ttf");
    /// ```
    pub fn new(font_path: &str) -> Result<FontManager, FontError> {
        let font_data = fs::read(font_path).map_err(|x| FontError::Read {
            path: font_path.to_string(),
            source: x,
        })?;
        Self::from_raw(font_data)
    }

    /// Loads font data from raw bytes
    ///
    /// # Errors
    /// - [`FontError::Parse`] if format of the file is incorrect. See `.reason` to get underlying error
    ///
    /// # Example
    /// ```
    /// use emir::prelude::FontManager;
    /// let font_manager = FontManager::from_raw(Vec::from(include_bytes!("../font.ttf")));
    /// ```
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

    pub(crate) fn prepare_character(&self, c: char, px: f32) -> (Metrics, Vec<u8>) {
        self.font.rasterize(c, px)
    }

    /// Measures how much specified text will take up space (in pixels).
    ///
    /// Returns two values: width and height of given text in pixels
    ///
    /// # Example
    /// ```
    /// use emir::prelude::FontManager;
    /// let font = FontManager::new("font.ttf").unwrap();
    /// let (x, y) = font.measure_string("Hello world!", 32.0);
    ///
    /// // Now we can use `x` and `y` values to, for example, draw a bounding box for our text
    /// ```
    pub fn measure_string(&self, s: &str, px: f32) -> (usize, usize) {
        let width = s
            .chars()
            .map(|c| self.prepare_character(c, px).0.width)
            .sum::<usize>()
            + (s.len() - 1) * self.spacing;
        let height = (s.chars().filter(|&x| x == '\n').count() + 1) * px as usize;

        (width, height)
    }

    pub(crate) fn layout(&self, s: &str, x: usize, y: usize, font_size: f32) -> Vec<GlyphPosition> {
        // TODO: think about automatic wrapping
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: x as f32,
            y: y as f32,
            ..LayoutSettings::default()
        });
        // TODO: cloning font here is kinda expensive, but for now I can't find any better solution to it
        layout.append(
            slice::from_ref(&self.font),
            &TextStyle::new(s, font_size, 0),
        );
        // TODO: maybe also do something here?
        layout.glyphs().clone()
    }

    #[deprecated(since = "0.5.0", note = "spacing was a workaround before .layout existed. Spacing is unused and has no effect on code")]
    pub fn with_spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }
}
