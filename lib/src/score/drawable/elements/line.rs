use crate::core::color::Color;
use crate::core::xy::XY;
use serde::Serialize;

#[derive(Serialize)]
pub struct Line {
    pub start: XY,
    pub end: XY,
    pub stroke_color: Color,
    pub stroke_width: f32,
}
