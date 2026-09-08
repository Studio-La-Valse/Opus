use crate::drawable::elements::glyph::Glyph;
use crate::drawable::elements::text::FontSpec;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;
use crate::smufl::smufl_font::SmuflFont;

pub trait SmuflGlyph {
    /// The drawable for this glyph, placed at `xy` and drawn at `scale`.
    ///
    /// `xy` is the glyph's own origin for everything except the brace, which
    /// hangs off its right edge instead; see its implementation.
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a>;
}

/// World units per staff space at `scale`. Every figure in the SMuFL metadata
/// -- bounding boxes, anchors, advance widths -- is written in staff spaces,
/// and this is the one conversion out of them.
pub fn staff_space(scale: f32) -> f32 {
    Staff::DEFAULT_SPACE_SIZE * scale
}

/// A glyph with its origin at `origin`, boxed by the staff-space `bbox` the
/// font metadata gives it.
///
/// The font size is the staff height because SMuFL registers its glyphs against
/// an em square equal to a five-line staff: setting the face at that size is
/// what makes a metadata figure of `1.0` come out as one staff space.
pub fn placed_glyph<'a>(
    font: &'a SmuflFont,
    codepoint: char,
    bbox: &BoundingBox,
    color: Color,
    origin: XY,
    scale: f32,
) -> Glyph<'a> {
    let unit = staff_space(scale);

    Glyph {
        glyph: font.glyph_str(codepoint),
        color,
        font_size: unit * Staff::SPACES as f32,
        font: FontSpec::plain(&font.meta.font),
        origin,
        bounds: bbox.placed(origin, unit),
    }
}
