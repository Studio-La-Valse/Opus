use crate::drawable::drawable_element::Scale;
use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

#[derive(Serialize, Copy, Clone)]
pub struct Line {
    pub start: XY,
    pub end: XY,
    pub stroke_color: Color,
    pub stroke_width: f32,
}

impl Scale for Line {
    fn scale(&self, factor: f32, pivot: XY) -> Line {
        Line {
            start: self.start.scale_about(factor, pivot),
            end: self.end.scale_about(factor, pivot),
            stroke_width: self.stroke_width * factor,
            ..*self
        }
    }
}

impl Line {
    pub fn extrude(&self, dir: XY) -> Polygon {
        Polygon {
            pts: vec![
                self.start,
                self.end,
                self.end.mv(dir.x, dir.y),
                self.start.mv(dir.x, dir.y),
            ],
            color: self.stroke_color,
            stroke_color: None,
            stroke_width: None,
        }
    }
}
