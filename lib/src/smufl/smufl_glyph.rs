use serde::Serialize;
use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::xy::XY;

#[derive(Debug, Clone, Serialize)]
pub struct SmuflGlyph {
    pub codepoint: char,
    pub font: String,
    pub bbox: Option<BoundingBox>,
}

impl SmuflGlyph {

    pub fn as_text(&self, color: Color, xy: XY) -> Text {
        Text {
            text: self.codepoint.to_string(),
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            xy: xy.mv(0., 5.),
            font_size: 40.,
            font: self.font.to_string(),
            color,
        }
    }
}