use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::Cutouts;
use crate::xy::XY;

#[derive(Clone)]
pub struct Flag {
    pub codepoint: char,
    pub font: String,
    pub bbox: BoundingBox,
    pub cutouts: Cutouts,
    pub stem_anchor: XY,
}

impl SmuflGlyph for Flag {
    fn as_text(&self, color: Color, xy: XY) -> Text {
        let text = self.codepoint.to_string();
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: 40.,
            font: self.font.to_string(),
            text,
            xy,
            color,
        }
    }
}
