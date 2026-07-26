use crate::bounding_box::BoundingBox;
use crate::score::core::accidental::Accidental as AccidentalCore;
use crate::score::core::clef::Clef as ClefCore;
use crate::score::core::time_signature::TimeSignature as TimeSignatureCore;
use crate::smufl::glyph_name::{GlyphName, ToChar, load_glyph_names};
use crate::smufl::glyphs::accidental::Accidental;
use crate::smufl::glyphs::brace::Brace;
use crate::smufl::glyphs::bracket::{BracketBottom, BracketTop};
use crate::smufl::glyphs::clef::Clef;
use crate::smufl::glyphs::flag::Flag;
use crate::smufl::glyphs::notehead::Notehead;
use crate::smufl::glyphs::number::Number;
use crate::smufl::glyphs::rest::Rest;
use crate::smufl::smufl_metadata::{Cutouts, SmuflMetadata};
use crate::visual::stem::UpDown;
use std::collections::HashMap;
use std::fs;

pub struct SmuflFont {
    pub meta: SmuflMetadata,
    pub glyph_names: HashMap<String, GlyphName>,
}

impl SmuflFont {
    pub fn load(path_meta_json: &str, path_glyphs_json: &str) -> SmuflFont {
        let data = fs::read_to_string(path_meta_json).expect("Cannot read metadata.json");
        let meta = serde_json::from_str(&data).expect("Invalid SMuFL metadata");
        let glyph_names = load_glyph_names(path_glyphs_json);

        SmuflFont { meta, glyph_names }
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
            font: self.meta.font.to_string(),
        }
    }

    pub fn rest(&self, name: &str) -> Rest {
        let codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        let glyph_box = self.meta.glyph_boxes.get(name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        Rest {
            codepoint,
            bbox,
            font: self.meta.font.to_string(),
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
            font: self.meta.font.to_string(),
        }
    }

    pub fn clef(&self, clef: &ClefCore) -> Clef {
        let ref_name = match clef {
            ClefCore::Treble => "gClef",
            ClefCore::Soprano
            | ClefCore::MezzoSoprano
            | ClefCore::Alto
            | ClefCore::Tenor
            | ClefCore::Baritone => "cClef",
            ClefCore::Bass => "fClef",
            ClefCore::Percussion => "unpitchedPercussionClef1",
        };
        let codepoint = self.glyph_names.get(ref_name).unwrap().codepoint_char();

        Clef {
            codepoint,
            line: clef.anchor_line(),
            font: self.meta.font.to_string(),
        }
    }

    pub fn bracket_top(&self) -> BracketTop {
        let codepoint = self.glyph_names.get("bracketTop").unwrap().codepoint_char();

        BracketTop {
            codepoint,
            thickness: 0.5,
            font: self.meta.font.to_string(),
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
            font: self.meta.font.to_string(),
        }
    }

    pub fn brace(&self, alternative: Option<&str>) -> Brace {
        let mut codepoint = self.glyph_names.get("brace").unwrap().codepoint_char();

        if let Some(alternative) = alternative {
            let alternate = self.meta.glyph_alternatives.get("brace");

            if let Some(alternate) = alternate {
                codepoint = alternate
                    .alternates
                    .iter()
                    .find(|v| v.name == alternative)
                    .unwrap()
                    .codepoint
                    .codepoint_char();
            }
        }

        Brace {
            codepoint,
            font: self.meta.font.to_string(),
        }
    }

    pub fn time_signature(&self, time_signature: TimeSignatureCore) -> (Number, Number) {
        let name = "timeSig".to_string() + time_signature.time.to_string().as_str();
        let codepoint = self
            .glyph_names
            .get(name.as_str())
            .unwrap()
            .codepoint_char();
        let num = Number {
            codepoint,
            font: self.meta.font.to_string(),
        };

        let name = "timeSig".to_string() + time_signature.base.as_int().to_string().as_str();
        let codepoint = self
            .glyph_names
            .get(name.as_str())
            .unwrap()
            .codepoint_char();
        let denom = Number {
            codepoint,
            font: self.meta.font.to_string(),
        };

        (num, denom)
    }

    pub fn accidental(&self, accidental: AccidentalCore) -> Accidental {
        let name = match accidental {
            AccidentalCore::Natural => "accidentalNatural",
            AccidentalCore::Sharp => "accidentalSharp",
            AccidentalCore::DoubleSharp => "accidentalDoubleSharp",
            AccidentalCore::Flat => "accidentalFlat",
            AccidentalCore::DoubleFlat => "accidentalDoubleFlat",
        };

        let codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        Accidental {
            codepoint,
            font: self.meta.font.to_string(),
        }
    }
}
