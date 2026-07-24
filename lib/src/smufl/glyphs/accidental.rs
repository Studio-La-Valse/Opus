use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::staff::Staff;
use crate::xy::XY;

pub struct Accidental {
    pub codepoint: char,
    pub font: String,
}

impl SmuflGlyph for Accidental {
    fn as_text(&self, color: Color, xy: XY, scale: f32) -> Text {
        let text = self.codepoint.to_string();
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Right,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: self.font.to_string(),
            text,
            xy,
            color,
        }
    }
}
