use std::ops::Add;

// RGBA
#[derive(Debug, Copy, Clone)]
pub struct Color(u32);

impl Color {
    pub const WHITE: Color = Color::new(0xff, 0xff, 0xff);
    pub const SILVER: Color = Color::new(0xc0, 0xc0, 0xc0);
    pub const GRAY: Color = Color::new(0x80, 0x80, 0x80);
    pub const BLACK: Color = Color::new(0x00, 0x00, 0x00);
    pub const RED: Color = Color::new(0xff, 0x00, 0x00);
    pub const MAROON: Color = Color::new(0x80, 0x00, 0x00);
    pub const YELLOW: Color = Color::new(0xff, 0xff, 0x00);
    pub const OLIVE: Color = Color::new(0x80, 0x80, 0x00);
    pub const LIME: Color = Color::new(0x00, 0xff, 0x00);
    pub const GREEN: Color = Color::new(0x00, 0x80, 0x00);
    pub const AQUA: Color = Color::new(0x00, 0xff, 0xff);
    pub const TEAL: Color = Color::new(0x00, 0x80, 0x80);
    pub const BLUE: Color = Color::new(0x00, 0x00, 0x0ff);
    pub const NAVY: Color = Color::new(0x00, 0x00, 0x80);
    pub const FUCHSIA: Color = Color::new(0xff, 0x00, 0xff);
    pub const PURPLE: Color = Color::new(0x80, 0x00, 0x80);

    pub const TRANSPARENT: Color = Color::with_alpha(0x00, 0x00, 0x00, 0x00);

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self::with_alpha(r, g, b, 0xff)
    }
    #[inline]
    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32))
    }
    #[inline]
    pub const fn r(&self) -> u8 {
        (self.0 >> 24) as u8
    }
    #[inline]
    pub const fn g(&self) -> u8 {
        (self.0 >> 16) as u8
    }
    #[inline]
    pub const fn b(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    #[inline]
    pub const fn a(&self) -> u8 {
        self.0 as u8
    }
    #[inline]
    pub const fn set_r(&mut self, r: u8) {
        self.0 = (self.0 & !(0xff << 24)) | ((r as u32) << 24)
    }
    #[inline]
    pub const fn set_g(&mut self, g: u8) {
        self.0 = (self.0 & !(0xff << 16)) | ((g as u32) << 16)
    }
    #[inline]
    pub const fn set_b(&mut self, b: u8) {
        self.0 = (self.0 & !(0xff << 8)) | ((b as u32) << 8)
    }
    #[inline]
    pub const fn set_a(&mut self, a: u8) {
        self.0 = (self.0 & !0xff) | (a as u32)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct XRGB8888(u32);

impl XRGB8888 {
    pub const WHITE: XRGB8888 = XRGB8888::new(0xff, 0xff, 0xff);
    pub const SILVER: XRGB8888 = XRGB8888::new(0xc0, 0xc0, 0xc0);
    pub const GRAY: XRGB8888 = XRGB8888::new(0x80, 0x80, 0x80);
    pub const BLACK: XRGB8888 = XRGB8888::new(0x00, 0x00, 0x00);
    pub const RED: XRGB8888 = XRGB8888::new(0xff, 0x00, 0x00);
    pub const MAROON: XRGB8888 = XRGB8888::new(0x80, 0x00, 0x00);
    pub const YELLOW: XRGB8888 = XRGB8888::new(0xff, 0xff, 0x00);
    pub const OLIVE: XRGB8888 = XRGB8888::new(0x80, 0x80, 0x00);
    pub const LIME: XRGB8888 = XRGB8888::new(0x00, 0xff, 0x00);
    pub const GREEN: XRGB8888 = XRGB8888::new(0x00, 0x80, 0x00);
    pub const AQUA: XRGB8888 = XRGB8888::new(0x00, 0xff, 0xff);
    pub const TEAL: XRGB8888 = XRGB8888::new(0x00, 0x80, 0x80);
    pub const BLUE: XRGB8888 = XRGB8888::new(0x00, 0x00, 0x0ff);
    pub const NAVY: XRGB8888 = XRGB8888::new(0x00, 0x00, 0x80);
    pub const FUCHSIA: XRGB8888 = XRGB8888::new(0xff, 0x00, 0xff);
    pub const PURPLE: XRGB8888 = XRGB8888::new(0x80, 0x00, 0x80);

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }
    #[inline]
    pub const fn r(&self) -> u8 {
        (self.0 >> 16) as u8
    }
    #[inline]
    pub const fn g(&self) -> u8 {
        (self.0 >> 8) as u8
    }
    #[inline]
    pub const fn b(&self) -> u8 {
        self.0 as u8
    }
    #[inline]
    pub const fn set_r(&mut self, r: u8) {
        self.0 = (self.0 & !(0xff << 16)) | ((r as u32) << 16)
    }
    #[inline]
    pub const fn set_g(&mut self, g: u8) {
        self.0 = (self.0 & !(0xff << 8)) | ((g as u32) << 8)
    }
    #[inline]
    pub const fn set_b(&mut self, b: u8) {
        self.0 = (self.0 & !0xff) | (b as u32)
    }
}
