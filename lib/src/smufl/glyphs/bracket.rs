use crate::drawable::elements::glyph::Glyph;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::{SmuflGlyph, placed_glyph};

#[derive(Clone)]
pub struct BracketTop {
    pub codepoint: char,
    pub bbox: BoundingBox,
    /// Thickness in tenths of the vertical stroke the glyph is drawn against,
    /// from the font's `bracketThickness`.
    pub thickness: f32,
}

impl SmuflGlyph for BracketTop {
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        placed_glyph(font, self.codepoint, &self.bbox, color, xy, scale)
    }
}

#[derive(Clone)]
pub struct BracketBottom {
    pub codepoint: char,
    pub bbox: BoundingBox,
    /// As [`BracketTop::thickness`].
    pub thickness: f32,
}

impl SmuflGlyph for BracketBottom {
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        placed_glyph(font, self.codepoint, &self.bbox, color, xy, scale)
    }
}
