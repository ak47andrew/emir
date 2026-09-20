use bytemuck::{Pod, Zeroable};

/// Struct representing color in an RGB format. It's stored as u32 in a 0x00RRGGBB format as required
/// by underlying `minifb` lib.
///
/// # Example
/// ```
/// use emir::prelude::Color;
/// let red = Color::new(255, 0, 0);
/// assert_eq!(red, Color::RED);
/// assert_eq!(red.as_u32(), 0x00FF0000);
///
/// let green = Color::from_u32(0x0000FF00);
/// assert_eq!(green, Color::GREEN);
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Pod, Zeroable, Debug)]
pub struct Color(u32);

impl Color {
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const DARKBLUE: Color = Color::new(0, 0, 255);
    pub const LIGHTBLUE: Color = Color::new(0, 255, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);

    /// Creates a color from individual `r`, `g` and `b` channels
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// assert_eq!(Color::new(0, 0, 255), Color::DARKBLUE);
    /// ```
    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    /// Returns red channel of the color
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// assert_eq!(Color::RED.r(), 255);
    /// ```
    #[inline(always)]
    pub const fn r(self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// Returns green channel of the color
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// assert_eq!(Color::GREEN.g(), 255);
    /// ```
    #[inline(always)]
    pub const fn g(self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// Returns blue channel of the color
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// assert_eq!(Color::DARKBLUE.b(), 255);
    /// ```
    #[inline(always)]
    pub const fn b(self) -> u8 {
        self.0 as u8
    }

    /// Sets red channel of the color to the specified value
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// let mut color = Color::RED;
    /// color.set_r(0);
    /// assert_eq!(color, Color::BLACK);
    /// ```
    #[inline(always)]
    pub fn set_r(&mut self, r: u8) {
        self.0 = (self.0 & 0xFF00_FFFF) | ((r as u32) << 16)
    }

    /// Sets green channel of the color to the specified value
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// let mut color = Color::GREEN;
    /// color.set_g(0);
    /// assert_eq!(color, Color::BLACK);
    /// ```
    #[inline(always)]
    pub fn set_g(&mut self, g: u8) {
        self.0 = (self.0 & 0xFFFF_00FF) | ((g as u32) << 8)
    }

    /// Sets blue channel of the color to the specified value
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// let mut color = Color::DARKBLUE;
    /// color.set_b(0);
    /// assert_eq!(color, Color::BLACK);
    /// ```
    #[inline(always)]
    pub fn set_b(&mut self, b: u8) {
        self.0 = (self.0 & 0xFFFF_FF00) | (b as u32)
    }

    /// Transforms color object into a [`u32`] in a 0x00RRGGBB format
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// assert_eq!(Color::WHITE.as_u32(), 0x00FFFFFF)
    /// ```
    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Transforms [`u32`] in a 0x00RRGGBB format into color object
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// let value = 0x00FFFFFF;  // White color
    /// assert_eq!(Color::from_u32(value), Color::WHITE)
    /// ```
    #[inline(always)]
    pub const fn from_u32(v: u32) -> Self {
        Self(v)
    }

    /// Performs [linear interpolation](https://en.wikipedia.org/wiki/Linear_interpolation) between two colors at point `t`
    ///
    /// # Requirements
    /// `0 <= t <= 1`. If specified value are outside this range, returned value will follow the
    /// specified linear slope when corrected for over- and overflows
    ///
    /// # Example
    /// ```
    /// use emir::prelude::Color;
    /// let a = Color::BLACK;
    /// let b = Color::WHITE;
    /// assert_eq!(a.lerp(b, 0.0), Color::BLACK);
    /// assert_eq!(a.lerp(b, 0.5), Color::from_u32(0x00808080));
    /// assert_eq!(a.lerp(b, 1.0), Color::WHITE);
    /// ```
    pub fn lerp(&self, other: Color, t: f32) -> Color {
        macro_rules! lerp_channel {
            ($self:expr, $other:expr) => {{
                let a = $self as f32;
                let b = $other as f32;
                (a + (b - a) * t).round() as u8
            }};
        }

        Color::new(
            lerp_channel!(self.r(), other.r()),
            lerp_channel!(self.g(), other.g()),
            lerp_channel!(self.b(), other.b()),
        )
    }
}
