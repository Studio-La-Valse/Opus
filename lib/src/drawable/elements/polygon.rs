use crate::drawable::drawable_element::Scale;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

#[derive(Default, Serialize, Clone)]
pub struct Polygon {
    pub pts: Vec<XY>,
    pub color: Color,
    pub stroke_color: Option<Color>,
    pub stroke_width: Option<f32>,
}

impl Polygon {
    pub fn mv(&self, x: f32, y: f32) -> Polygon {
        Polygon {
            pts: self.pts.iter().map(|pt| pt.mv(x, y)).collect(),
            ..self.clone()
        }
    }
}

impl Scale for Polygon {
    fn scale(&self, factor: f32, pivot: XY) -> Polygon {
        Polygon {
            pts: self
                .pts
                .iter()
                .map(|pt| pt.scale_about(factor, pivot))
                .collect(),
            stroke_width: self.stroke_width.map(|v| v * factor),
            ..self.clone()
        }
    }
}
