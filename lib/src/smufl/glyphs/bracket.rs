use crate::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::SmuflGlyph;

#[derive(Clone)]
pub struct BracketTop {
    pub codepoint: char,
    pub thickness: f32,
}

impl SmuflGlyph for BracketTop {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a> {
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: FontSpec::plain(&font.meta.font),
            text: font.glyph_str(self.codepoint),
            xy,
            color,
        }
    }
}

#[derive(Clone)]
pub struct BracketBottom {
    pub codepoint: char,
    pub thickness: f32,
}

impl SmuflGlyph for BracketBottom {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a> {
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: FontSpec::plain(&font.meta.font),
            text: font.glyph_str(self.codepoint),
            xy,
            color,
        }
    }
}
