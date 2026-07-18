use crate::bounding_box::BoundingBox;
use crate::smufl::glyph_name::{GlyphName, load_glyph_names};
use crate::smufl::glyphs::bracket::{BracketBottom, BracketTop};
use crate::smufl::glyphs::clef::Clef;
use crate::smufl::glyphs::flag::Flag;
use crate::smufl::glyphs::notehead::Notehead;
use crate::smufl::glyphs::rest::Rest;
use crate::smufl::smufl_metadata::{Cutouts, SmuflMetadata};
use crate::visual::stem::UpDown;
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

        let stem_anchor_left = anchors.stem_down_nw();
        let stem_anchor_right = anchors.stem_up_se();

        Notehead {
            codepoint,
            bbox,
            cutouts,
            stem_anchor_left,
            stem_anchor_right,
            font: self.font.to_string(),
        }
    }

    pub fn rest(&self, name: &str) -> Rest {
        let codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        let glyph_box = self.meta.glyph_boxes.get(name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        Rest {
            codepoint,
            bbox,
            font: self.font.to_string(),
        }
    }

    pub fn flag(&self, name: &str, dir: &UpDown) -> Flag {
        let codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        let glyph_box = self.meta.glyph_boxes.get(name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        let anchors = self.meta.glyph_anchors.get(name).unwrap();
        let cutouts: Cutouts = anchors.to_cutouts(&bbox);

        let stem_anchor = match dir {
            UpDown::Up => anchors.stem_up_nw().unwrap(),
            UpDown::Down => anchors.stem_down_sw().unwrap(),
        };

        Flag {
            codepoint,
            bbox,
            cutouts,
            stem_anchor,
            font: self.font.to_string(),
        }
    }

    pub fn clef(&self, clef: &crate::score::core::clef::Clef) -> Clef {
        let ref_name = match clef.name.as_str() {
            "Treble" => "gClef",
            "Soprano" => "cClef",
            "Mezzo Soprano" => "cClef",
            "Alto" => "cClef",
            "Tenor" => "cClef",
            "Baritone" => "cClef",
            "Bass" => "fClef",
            "Percussion" => "unpitchedPercussionClef1",
            _ => panic!("{} not recognized as a clef", clef.name),
        };
        let codepoint = self.glyph_names.get(ref_name).unwrap().codepoint_char();

        Clef {
            codepoint,
            line: clef.anchor_line,
            font: self.font.to_string(),
        }
    }

    pub fn bracket_top(&self) -> BracketTop {
        let codepoint = self.glyph_names.get("bracketTop").unwrap().codepoint_char();

        BracketTop {
            codepoint,
            thickness: 0.5,
            font: self.font.to_string(),
        }
    }

    pub fn bracket_bottom(&self) -> BracketBottom {
        let codepoint = self
            .glyph_names
            .get("bracketBottom")
            .unwrap()
            .codepoint_char();

        BracketBottom {
            codepoint,
            thickness: 0.5,
            font: self.font.to_string(),
        }
    }
}
