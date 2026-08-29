use serde::Serialize;
use std::fmt;
use std::str::FromStr;

/// An sRGB colour with a separate alpha. The `r`/`g`/`b` channels are 8-bit;
/// `a` is a `0.0..=1.0` fraction and is clamped into that range on construction.
#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct Color {
    a: f32,
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        a: 1.0,
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Color = Color {
        a: 1.0,
        r: 0,
        g: 0,
        b: 0,
    };
    pub const TRANSPARENT: Color = Color {
        a: 0.,
        r: 0,
        g: 0,
        b: 0,
    };
    pub const RED: Color = Color {
        a: 1.,
        r: 255,
        g: 0,
        b: 0,
    };
    pub const GREEN: Color = Color {
        a: 1.,
        r: 0,
        g: 255,
        b: 0,
    };
    pub const BLUE: Color = Color {
        a: 1.,
        r: 0,
        g: 0,
        b: 255,
    };

    /// An opaque colour from 8-bit channels.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { a: 1.0, r, g, b }
    }

    /// A colour from 8-bit channels and an alpha fraction, clamped to `0.0..=1.0`.
    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color {
        Color {
            a: a.clamp(0.0, 1.0),
            r,
            g,
            b,
        }
    }

    pub const fn r(&self) -> u8 {
        self.r
    }

    pub const fn g(&self) -> u8 {
        self.g
    }

    pub const fn b(&self) -> u8 {
        self.b
    }

    pub const fn a(&self) -> f32 {
        self.a
    }

    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.r,
            self.g,
            self.b,
            (self.a * 255.) as i32
        )
    }
}

#[derive(Debug)]
pub struct ColorParseError(String);

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid color '{}': expected hex format #RRGGBB or #RRGGBBAA",
            self.0
        )
    }
}

impl std::error::Error for ColorParseError {}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').unwrap_or(s);
        let component = |slice: &str| -> Result<u8, ColorParseError> {
            u8::from_str_radix(slice, 16).map_err(|_| ColorParseError(s.to_string()))
        };

        match hex.len() {
            6 => Ok(Color::rgb(
                component(&hex[0..2])?,
                component(&hex[2..4])?,
                component(&hex[4..6])?,
            )),
            8 => Ok(Color::rgba(
                component(&hex[0..2])?,
                component(&hex[2..4])?,
                component(&hex[4..6])?,
                component(&hex[6..8])? as f32 / 255.0,
            )),
            _ => Err(ColorParseError(s.to_string())),
        }
    }
}
