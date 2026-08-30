use crate::drawable::drawable_element::Scale;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

#[derive(Default, Copy, Clone, Serialize)]
pub struct Circle {
    /// Center of the circle.
    pub xy: XY,
    pub radius: f32,
    pub color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<f32>,
}

impl Scale for Circle {
    fn scale(&self, factor: f32, pivot: XY) -> Circle {
        Circle {
            xy: self.xy.scale_about(factor, pivot),
            radius: self.radius * factor,
            stroke_width: self.stroke_width.map(|v| v * factor),
            ..*self
        }
    }
}
