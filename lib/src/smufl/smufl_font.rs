use crate::bounding_box::BoundingBox;
use crate::smufl::glyph_name::{GlyphName, load_glyph_names};
use crate::smufl::glyphs::notehead::Notehead;
use crate::smufl::smufl_metadata::{Cutouts, SmuflMetadata};
use std::collections::HashMap;
use std::fs;

pub struct SmuflFont {
    pub font: String,
    pub meta: SmuflMetadata,
    pub glyph_names: HashMap<String, GlyphName>,
}

impl SmuflFont {
    pub fn load(font: String, path_meta_json: &str, path_glyphs_json: &str) -> SmuflFont {
        let data = fs::read_to_string(path_meta_json).expect("Cannot read metadata.json");
        let meta = serde_json::from_str(&data).expect("Invalid SMuFL metadata");
        let glyph_names = load_glyph_names(path_glyphs_json);

        SmuflFont {
            font,
            meta,
            glyph_names,
        }
    }

    pub fn notehead(&self, name: &str) -> Notehead {
        let codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        let glyph_box = self.meta.glyph_boxes.get(name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        let anchors = self.meta.glyph_anchors.get(name).unwrap();
        let cutouts: Cutouts = anchors.to_cutouts(&bbox);

        let stem_anchor_left = anchors.anchor_left();
        let stem_anchor_right = anchors.anchor_right();

        Notehead {
            codepoint,
            bbox,
            cutouts,
            stem_anchor_left,
            stem_anchor_right,
            font: self.font.to_string(),
        }
    }
}
