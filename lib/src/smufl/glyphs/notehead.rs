use crate::drawable::elements::glyph::Glyph;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::{SmuflGlyph, placed_glyph};
use crate::smufl::smufl_metadata::Cutouts;

#[derive(Clone)]
pub struct Notehead {
    pub codepoint: char,
    pub bbox: BoundingBox,
    pub cutouts: Cutouts,
    pub stem_anchor_left: Option<XY>,
    pub stem_anchor_right: Option<XY>,
}

impl SmuflGlyph for Notehead {
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        placed_glyph(font, self.codepoint, &self.bbox, color, xy, scale)
    }
}
