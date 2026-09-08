use bytemuck::{Pod, Zeroable};

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

    #[inline(always)]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    #[inline(always)]
    pub const fn r(self) -> u8 {
        (self.0 >> 16) as u8
    }
    #[inline(always)]
    pub const fn g(self) -> u8 {
        (self.0 >> 8) as u8
    }
    #[inline(always)]
    pub const fn b(self) -> u8 {
        self.0 as u8
    }

    #[inline(always)]
    pub fn set_r(&mut self, r: u8) {
        self.0 = (self.0 & 0xFF00_FFFF) | ((r as u32) << 16)
    }
    #[inline(always)]
    pub fn set_g(&mut self, g: u8) {
        self.0 = (self.0 & 0xFFFF_00FF) | ((g as u32) << 8)
    }
    #[inline(always)]
    pub fn set_b(&mut self, b: u8) {
        self.0 = (self.0 & 0xFFFF_FF00) | (b as u32)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
    #[inline(always)]
    pub const fn from_u32(v: u32) -> Self {
        Self(v)
    }

    pub fn lerp(&self, other: Color, t: f32) -> Color {
        macro_rules! lerp_channel {
            ($self:expr, $other:expr) => {
                {
                    let a = $self as f32;
                    let b = $other as f32;
                    (a + (b - a) * t).round() as u8
                }
            };
        }

        Color::new(
            lerp_channel!(self.r(), other.r()),
            lerp_channel!(self.g(), other.g()),
            lerp_channel!(self.b(), other.b()),
        )
    }
}
