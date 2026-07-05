use serde::Serialize;

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
