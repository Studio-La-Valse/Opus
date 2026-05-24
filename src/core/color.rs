#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub a: f32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
}

impl Color {
    pub const WHITE: Color = Color {a: 1.0, r: 255, g: 255, b: 255};
    pub const BLACK: Color = Color {a: 1.0, r: 0, g: 0, b: 0};
}