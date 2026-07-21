use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::staff::Staff;
use crate::xy::XY;

#[derive(Clone)]
pub struct BracketTop {
    pub codepoint: char,
    pub thickness: f32,
    pub font: String,
}

impl SmuflGlyph for BracketTop {
    fn as_text(&self, color: Color, xy: XY, scale: f32) -> Text {
        let text = self.codepoint.to_string();
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: self.font.to_string(),
            text,
            xy,
            color,
        }
    }
}

#[derive(Clone)]
pub struct BracketBottom {
    pub codepoint: char,
    pub thickness: f32,
    pub font: String,
}

impl SmuflGlyph for BracketBottom {
    fn as_text(&self, color: Color, xy: XY, scale: f32) -> Text {
        let text = self.codepoint.to_string();
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: self.font.to_string(),
            text,
            xy,
            color,
        }
    }
}
