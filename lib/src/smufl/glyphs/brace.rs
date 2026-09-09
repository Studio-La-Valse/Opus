use crate::drawable::elements::glyph::Glyph;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::{SmuflGlyph, placed_glyph, staff_space};

#[derive(Clone)]
pub struct Brace {
    pub codepoint: char,
    pub bbox: BoundingBox,
    /// Advance width in staff spaces. The brace is the one glyph placed by its
    /// right edge rather than its origin, so this is what that placement steps
    /// back by.
    pub advance: f32,
}

impl SmuflGlyph for Brace {
    /// Unlike every other glyph, `xy` is the brace's *right* edge: a brace hangs
    /// to the left of the system it braces, and the system's left edge is what
    /// the layout knows. Stepping back one advance width turns that into the
    /// origin -- which is what the element used to ask the sink to do by
    /// right-aligning the text, back when a glyph was drawn as text.
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        let origin = xy.mv(-self.advance * staff_space(scale), 0.);

        placed_glyph(font, self.codepoint, &self.bbox, color, origin, scale)
    }
}
