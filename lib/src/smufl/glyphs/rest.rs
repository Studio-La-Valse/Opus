use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::staff::Staff;
use crate::xy::XY;

#[derive(Clone)]
pub struct Rest {
    pub codepoint: char,
    pub bbox: BoundingBox,
}

impl SmuflGlyph for Rest {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a> {
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: Staff::DEFAULT_SPACE_SIZE * Staff::SPACES as f32 * scale,
            font: &font.meta.font,
            text: font.glyph_str(self.codepoint),
            xy,
            color,
        }
    }
}
