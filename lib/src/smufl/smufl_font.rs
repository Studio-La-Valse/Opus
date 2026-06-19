use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::smufl::smufl_metadata::SmuflMetadata;
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

        let notehead = &self.meta.glyph_boxes.notehead_black;
        let bbox = Some(notehead.into());

        SmuflGlyph {
            font: self.name.to_string(),
            codepoint,
            bbox,
        }
    }
}
