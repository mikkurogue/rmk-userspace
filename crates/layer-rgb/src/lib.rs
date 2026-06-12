#![no_std]

/// HSV color representation.
#[derive(Debug, Clone, Copy)]
pub struct Hsv {
    pub h: u8,
    pub s: u8,
    pub v: u8,
}

/// RGB color representation.
#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// Common color constants (HSV)
pub const HSV_BLUE: Hsv = Hsv { h: 170, s: 255, v: 255 };
pub const HSV_PURPLE: Hsv = Hsv { h: 191, s: 255, v: 255 };
pub const HSV_GREEN: Hsv = Hsv { h: 85, s: 255, v: 255 };
pub const HSV_RED: Hsv = Hsv { h: 0, s: 255, v: 255 };
pub const HSV_OFF: Hsv = Hsv { h: 0, s: 0, v: 0 };

/// A mapping from layer index to HSV color.
///
/// Use `None` for layers that should keep the default animation.
pub struct LayerColors<const N: usize> {
    pub colors: [Option<Hsv>; N],
}

impl<const N: usize> LayerColors<N> {
    pub const fn new(colors: [Option<Hsv>; N]) -> Self {
        Self { colors }
    }

    /// Get the color for a given layer, if one is defined.
    pub fn get(&self, layer: usize) -> Option<Hsv> {
        if layer < N {
            self.colors[layer]
        } else {
            None
        }
    }
}

/// Convert HSV to RGB.
///
/// Standard HSV to RGB conversion for LED control.
pub fn hsv_to_rgb(hsv: Hsv) -> Rgb {
    if hsv.s == 0 {
        return Rgb { r: hsv.v, g: hsv.v, b: hsv.v };
    }

    let region = hsv.h as u16 / 43;
    let remainder = (hsv.h as u16 - (region * 43)) * 6;

    let p = ((hsv.v as u16) * (255 - hsv.s as u16)) >> 8;
    let q = ((hsv.v as u16) * (255 - ((hsv.s as u16 * remainder) >> 8))) >> 8;
    let t = ((hsv.v as u16) * (255 - ((hsv.s as u16 * (255 - remainder)) >> 8))) >> 8;

    match region {
        0 => Rgb { r: hsv.v, g: t as u8, b: p as u8 },
        1 => Rgb { r: q as u8, g: hsv.v, b: p as u8 },
        2 => Rgb { r: p as u8, g: hsv.v, b: t as u8 },
        3 => Rgb { r: p as u8, g: q as u8, b: hsv.v },
        4 => Rgb { r: t as u8, g: p as u8, b: hsv.v },
        _ => Rgb { r: hsv.v, g: p as u8, b: q as u8 },
    }
}

/// Scale the brightness of an HSV color.
pub fn scale_brightness(mut hsv: Hsv, brightness: u8) -> Hsv {
    hsv.v = ((hsv.v as u16 * brightness as u16) / 255) as u8;
    hsv
}

pub mod controller;
