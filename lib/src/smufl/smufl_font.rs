use crate::bounding_box::BoundingBox;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::{Cutouts, SmuflMetadata};
use std::fs;

pub struct SmuflFont {
    pub name: String,
    pub meta: SmuflMetadata,
}

impl SmuflFont {
    pub fn new(name: String, path: &str) -> SmuflFont {
        let data = fs::read_to_string(path).expect("Cannot read metadata.json");
        let meta = serde_json::from_str(&data).expect("Invalid SMuFL metadata");

        SmuflFont { name, meta }
    }

    pub fn notehead_black(&self) -> SmuflGlyph {
        let codepoint: char = '\u{E0A4}';

        let glyph_box = &self.meta.glyph_boxes.notehead_black;
        let bbox: BoundingBox = glyph_box.into();

        let anchors = &self.meta.glyph_anchors.notehead_black;
        let cutouts: Cutouts = anchors.to_boxes(&bbox);

        SmuflGlyph {
            codepoint,
            bbox,
            cutouts,
            font: self.name.to_string(),
        }
    }
}
