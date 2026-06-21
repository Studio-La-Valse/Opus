use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::Cutouts;
use crate::xy::XY;

#[derive(Clone)]
pub struct NoteheadBlack {
    pub font: String,
    pub bbox: BoundingBox,
    pub cutouts: Cutouts,
    pub stem_anchor_left: XY,
    pub stem_anchor_right: XY,
}

impl NoteheadBlack {
    pub const CODEPOINT: char = '\u{E0A4}';
}

impl SmuflGlyph for NoteheadBlack {
    fn as_text(&self, color: Color, xy: XY) -> Text {
        Text {
            text: NoteheadBlack::CODEPOINT.to_string(),
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: 40.,
            font: self.font.to_string(),
            xy,
            color,
        }
    }
}
