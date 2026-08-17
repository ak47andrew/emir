use bytemuck::{Pod, Zeroable};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Pod, Zeroable)]
pub struct Color(u32);

impl Color {
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255);

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    #[inline]
    pub const fn r(self) -> u8 {(self.0 >> 16) as u8}
    #[inline]
    pub const fn g(self) -> u8 {(self.0 >> 8) as u8}
    #[inline]
    pub const fn b(self) -> u8 {self.0 as u8}

    #[inline]
    pub fn set_r(&mut self, r: u8) {self.0 = (self.0 & 0xFF00_FFFF) | ((r as u32) << 16)}
    #[inline]
    pub fn set_g(&mut self, g: u8) {self.0 = (self.0 & 0xFFFF_00FF) | ((g as u32) << 8)}
    #[inline]
    pub fn set_b(&mut self, b: u8) {self.0 = (self.0 & 0xFFFF_FF00) | (b as u32)}

    #[inline]
    pub const fn as_u32(self) -> u32 {self.0}
    #[inline]
    pub const fn from_u32(v: u32) -> Self {Self(v)}

    pub fn lerp(&self, other: Color, t: f32) -> Color {
        #[inline]
        fn c(a: u8, b: u8, t: f32) -> u8 {
            let a = a as f32;
            let b = b as f32;
            (a + (b - a) * t).round() as u8
        }

        Color::new(
            c(self.r(), other.r(), t),
            c(self.g(), other.g(), t),
            c(self.b(), other.b(), t),
        )
    }
}
