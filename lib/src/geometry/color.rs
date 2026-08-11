use serde::Serialize;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct Color {
    pub a: f32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
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
        let component = |slice: &str| -> Result<i32, ColorParseError> {
            i32::from_str_radix(slice, 16).map_err(|_| ColorParseError(s.to_string()))
        };

        match hex.len() {
            6 => Ok(Color {
                a: 1.0,
                r: component(&hex[0..2])?,
                g: component(&hex[2..4])?,
                b: component(&hex[4..6])?,
            }),
            8 => Ok(Color {
                r: component(&hex[0..2])?,
                g: component(&hex[2..4])?,
                b: component(&hex[4..6])?,
                a: component(&hex[6..8])? as f32 / 255.0,
            }),
            _ => Err(ColorParseError(s.to_string())),
        }
    }
}
