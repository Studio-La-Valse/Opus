use crate::drawable::elements::glyph::Glyph;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::{SmuflGlyph, placed_glyph};
use crate::smufl::smufl_metadata::Cutouts;

pub struct Accidental {
    pub codepoint: char,
    pub bbox: BoundingBox,

    // double sharps and flats don't have cutouts for some reason.
    pub cutouts: Option<Cutouts>,
}

impl SmuflGlyph for Accidental {
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        placed_glyph(font, self.codepoint, &self.bbox, color, xy, scale)
    }
}
