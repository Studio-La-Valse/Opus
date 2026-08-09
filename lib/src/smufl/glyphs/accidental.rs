use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::Cutouts;

pub struct Accidental {
    pub codepoint: char,
    pub bbox: BoundingBox,

    // double sharps and flats don't have cutouts for some reason.
    pub cutouts: Option<Cutouts>,
}

impl SmuflGlyph for Accidental {
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
