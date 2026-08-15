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

impl Line {
    pub fn scale(&self, scale: f32) -> Line {
        Line {
            start: self.start.scale(scale),
            end: self.end.scale(scale),
            stroke_width: self.stroke_width * scale,
            ..*self
        }
    }

    pub fn extrude(&self, dir: &XY) -> Polygon {
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
