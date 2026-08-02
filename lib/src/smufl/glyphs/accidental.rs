use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::Cutouts;
use crate::visual::staff::Staff;
use crate::xy::XY;

pub struct Accidental {
    pub codepoint: char,
    pub font: String,
    pub bbox: BoundingBox,

    // double sharps and flats don't have cutouts for some reason.
    pub cutouts: Option<Cutouts>,
}

impl SmuflGlyph for Accidental {
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
