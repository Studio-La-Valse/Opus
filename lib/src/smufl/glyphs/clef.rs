use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::SmuflGlyph;

#[derive(Clone)]
pub struct Clef {
    pub codepoint: char,
    pub line: i32,
}

impl SmuflGlyph for Clef {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a> {
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: &font.meta.font,
            text: font.glyph_str(self.codepoint),
            xy,
            color,
        }
    }
}
