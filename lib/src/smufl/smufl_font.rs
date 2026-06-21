use crate::bounding_box::BoundingBox;
use crate::smufl::glyphs::notehead_black::NoteheadBlack;
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

    pub fn notehead_black(&self) -> NoteheadBlack {
        let glyph_box = &self.meta.glyph_boxes.notehead_black;
        let bbox: BoundingBox = glyph_box.into();

        let anchors = &self.meta.glyph_anchors.notehead_black;
        let cutouts: Cutouts = anchors.to_boxes(&bbox);

        let stem_anchor_left = anchors.anchor_left();
        let stem_anchor_right = anchors.anchor_right();

        NoteheadBlack {
            bbox,
            cutouts,
            stem_anchor_left,
            stem_anchor_right,
            font: self.name.to_string(),
        }
    }
}
