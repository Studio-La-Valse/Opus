use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

#[derive(Default, Copy, Clone, Serialize)]
pub struct Rect {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<f32>,
}

impl Rect {
    pub fn scale(&self, scale: f32) -> Rect {
        Rect {
            xy: self.xy.scale(scale),
            width: self.width * scale,
            height: self.height * scale,
            stroke_width: self.stroke_width.map(|v| v * scale),
            ..*self
        }
    }
}
