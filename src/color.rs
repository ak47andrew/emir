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

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let s_norm = s / 100.0;
        let l_norm = l / 100.0;

        let c = (1.0 - (2.0 * l_norm - 1.0).abs()) * s_norm;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        
        let (r_1, g_1, b_1) = match h {
            0.0..60.0 => {
                (c, x, 0.0)
            }
            60.0..120.0 => {
                (x, c, 0.0)
            }
            120.0..180.0 => {
                (0.0, c, x)
            }
            180.0..240.0 => {
                (0.0, x, c)
            }
            240.0..300.0 => {
                (x, 0.0, c)
            }
            _ => {
                (c, 0.0, x)
            }
        };

        let m = l_norm - c / 2.0;

        let r_norm = r_1 + m;
        let g_norm = g_1 + m;
        let b_norm = b_1 + m;

        Self::new(
            (r_norm * 255.0) as u8,
            (g_norm * 255.0) as u8,
            (b_norm * 255.0) as u8,
        )
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

    fn to_hsl(&self) -> (f32, f32, f32) {
        let r_norm = self.r() as f32 / 255.0;
        let g_norm = self.g() as f32 / 255.0;
        let b_norm = self.b() as f32 / 255.0;

        let c_max = r_norm.max(g_norm).max(b_norm);
        let c_min = r_norm.min(g_norm).min(b_norm);

        let delta = c_max - c_min;
        let l = (c_max + c_min) / 2.0;

        let s = if delta == 0.0 { 0.0 } else {
            delta / (1.0 - (2.0 * l - 1.0).abs())
        };

        let h = if delta == 0.0 {0.0} else {
            (
                if c_max == r_norm {
                    60.0 * ((g_norm - b_norm) / delta)
                } else if c_max == g_norm {
                    60.0 * ((b_norm - r_norm) / delta) + 2.0
                } else {
                    60.0 * ((r_norm - g_norm) / delta) + 4.0
                }
            ) % 360.0
        };

        (h, s, l)
    }

    pub fn complementary(&self) -> Self {
        let (h, s, l) = self.to_hsl();

        Self::from_hsl((h + 180.0) % 360.0, s, l)
    }

    pub fn analogous(&self) -> (Self, Self) {
        let (h, s, l) = self.to_hsl();

        (
            Self::from_hsl((h + 30.0) % 360.0, s, l),
            Self::from_hsl((h - 30.0) % 360.0, s, l)
        )
    }

    pub fn triadic(&self) -> (Self, Self) {
        let (h, s, l) = self.to_hsl();

        (
            Self::from_hsl((h + 120.0) % 360.0, s, l),
            Self::from_hsl((h - 120.0) % 360.0, s, l)
        )
    }

    pub fn split_complementary(&self) -> (Self, Self) {
        let (h, s, l) = self.to_hsl();

        (
            Self::from_hsl((h + 150.0) % 360.0, s, l),
            Self::from_hsl((h - 210.0) % 360.0, s, l)
        )
    }
}
