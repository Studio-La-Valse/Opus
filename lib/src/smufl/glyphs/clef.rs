use crate::drawable::elements::glyph::Glyph;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::{SmuflGlyph, placed_glyph};

#[derive(Clone)]
pub struct Clef {
    pub codepoint: char,
    pub line: i32,
    pub bbox: BoundingBox,
}

impl SmuflGlyph for Clef {
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        placed_glyph(font, self.codepoint, &self.bbox, color, xy, scale)
    }
}
