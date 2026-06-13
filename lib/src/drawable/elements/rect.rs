use crate::core::color::Color;
use crate::core::xy::XY;
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Rect {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<f32>,
}
