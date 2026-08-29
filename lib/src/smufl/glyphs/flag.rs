use crate::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::Cutouts;

#[derive(Clone)]
pub struct Flag {
    pub codepoint: char,
    pub bbox: BoundingBox,
    pub cutouts: Cutouts,
    pub stem_anchor: XY,
}

impl SmuflGlyph for Flag {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a> {
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: Staff::DEFAULT_SPACE_SIZE * Staff::SPACES as f32 * scale,
            font: FontSpec::plain(&font.meta.font),
            text: font.glyph_str(self.codepoint),
            xy,
            color,
        }
    }
}
