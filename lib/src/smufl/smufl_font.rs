use crate::geometry::bounding_box::BoundingBox;
use crate::score::core::accidental::Accidental as AccidentalCore;
use crate::score::core::clef::Clef as ClefCore;
use crate::score::core::time_signature::TimeSignature as TimeSignatureCore;
use crate::score::layout_options::{APP_DEFAULTS, UserLayout};
use crate::score::visual::stem::UpDown;
use crate::smufl::glyph_name::{GlyphName, ToChar, load_glyph_names};
use crate::smufl::glyphs::accidental::Accidental;
use crate::smufl::glyphs::brace::{Brace, BraceStyle};
use crate::smufl::glyphs::bracket::{BracketBottom, BracketTop};
use crate::smufl::glyphs::clef::Clef;
use crate::smufl::glyphs::flag::Flag;
use crate::smufl::glyphs::notehead::Notehead;
use crate::smufl::glyphs::number::{Number, NumberDigit};
use crate::smufl::glyphs::rest::Rest;
use crate::smufl::smufl_metadata::{Cutouts, SmuflMetadata};
use std::collections::HashMap;

pub struct SmuflFont {
    pub meta: SmuflMetadata,
    pub glyph_names: HashMap<String, GlyphName>,
    /// The layout tier this font's `engravingDefaults` recommends, in tenths.
    /// See [`UserLayout::from_engraving_defaults`].
    pub layout: UserLayout,
    glyph_text: HashMap<char, String>,
}

impl SmuflFont {
    pub fn load(meta_json_content: &str, glyph_names_json_content: &str) -> SmuflFont {
        let meta: SmuflMetadata =
            serde_json::from_str(meta_json_content).expect("Invalid SMuFL metadata");
        let glyph_names = load_glyph_names(glyph_names_json_content);

        let mut glyph_text: HashMap<char, String> = HashMap::new();
        for glyph in glyph_names.values() {
            let c = glyph.codepoint_char();
            glyph_text.entry(c).or_insert_with(|| c.to_string());
        }
        for alternates in meta.glyph_alternatives.values() {
            for alternate in &alternates.alternates {
                let c = alternate.codepoint.codepoint_char();
                glyph_text.entry(c).or_insert_with(|| c.to_string());
            }
        }

        let layout = UserLayout::from_engraving_defaults(&meta.engraving_defaults);

        SmuflFont {
            meta,
            glyph_names,
            layout,
            glyph_text,
        }
    }

    pub fn glyph_str(&self, codepoint: char) -> &str {
        self.glyph_text
            .get(&codepoint)
            .unwrap_or_else(|| panic!("no cached glyph text for codepoint {codepoint:?}"))
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
        }
    }

    pub fn rest(&self, name: &str) -> Rest {
        let codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        let glyph_box = self.meta.glyph_boxes.get(name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        Rest { codepoint, bbox }
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
        }
    }

    /// The glyph for `clef`, anchored against a staff of `staff_lines` lines --
    /// which for a clef that names no pitch is what decides where on the staff
    /// it sits. See [`ClefCore::anchor_line`].
    pub fn clef(&self, clef: &ClefCore, staff_lines: usize) -> Clef {
        let ref_name = match clef {
            ClefCore::Treble => "gClef",
            ClefCore::Soprano
            | ClefCore::MezzoSoprano
            | ClefCore::Alto
            | ClefCore::Tenor
            | ClefCore::Baritone => "cClef",
            ClefCore::Bass => "fClef",
            ClefCore::Percussion => "unpitchedPercussionClef1",
            // MusicXML's `<clef>` doesn't say how many strings the tablature
            // has -- `<staff-details><staff-lines>` does, and `staff_lines` now
            // carries it this far -- but SMuFL offers only a four- and a
            // six-string glyph, so the six-string one stands in for every tab
            // staff until the rest of the tablature work picks between them.
            ClefCore::Tab => "6stringTabClef",
        };
        let codepoint = self.glyph_names.get(ref_name).unwrap().codepoint_char();

        let glyph_box = self.meta.glyph_boxes.get(ref_name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        Clef {
            codepoint,
            line: clef.anchor_line(staff_lines),
            bbox,
        }
    }

    pub fn bracket_top(&self) -> BracketTop {
        let codepoint = self.glyph_names.get("bracketTop").unwrap().codepoint_char();
        let bbox: BoundingBox = self.meta.glyph_boxes.get("bracketTop").unwrap().into();

        BracketTop {
            codepoint,
            bbox,
            thickness: self.bracket_glyph_stroke(),
        }
    }

    pub fn bracket_bottom(&self) -> BracketBottom {
        let codepoint = self
            .glyph_names
            .get("bracketBottom")
            .unwrap()
            .codepoint_char();
        let bbox: BoundingBox = self.meta.glyph_boxes.get("bracketBottom").unwrap().into();

        BracketBottom {
            codepoint,
            bbox,
            thickness: self.bracket_glyph_stroke(),
        }
    }

    /// The brace glyph `style` asks for. A font that doesn't offer that
    /// alternate -- it lists none for `brace`, or not this one -- draws its
    /// plain `brace` instead: the style is a preference, and the font is
    /// what can't honour it.
    pub fn brace(&self, style: BraceStyle) -> Brace {
        let mut name = "brace";
        let mut codepoint = self.glyph_names.get(name).unwrap().codepoint_char();

        let alternate = style.alternate_name().and_then(|alternative| {
            self.meta
                .glyph_alternatives
                .get("brace")?
                .alternates
                .iter()
                .find(|v| v.name == alternative)
        });

        if let Some(alternate) = alternate {
            // The metadata keys a chosen alternate's own box and advance by its
            // name, so both have to be read against that rather than against
            // the default brace they replace.
            name = &alternate.name;
            codepoint = alternate.codepoint.codepoint_char();
        }

        let bbox: BoundingBox = self.meta.glyph_boxes.get(name).unwrap().into();
        let advance = *self.meta.glyph_advance_widths.get(name).unwrap();

        Brace {
            codepoint,
            bbox,
            advance,
        }
    }

    pub fn time_signature(&self, time_signature: TimeSignatureCore) -> (Number, Number) {
        let num = self.number(u32::from(time_signature.time));
        let denom = self.number(time_signature.base.as_int().unsigned_abs());

        (num, denom)
    }

    /// `value` spelled in the font's time-signature digits.
    ///
    /// SMuFL defines one glyph per decimal digit (`timeSig0`..`timeSig9`) and
    /// nothing for a whole multi-digit number, so anything from 10 up -- 12/8,
    /// and every `x/16`, `x/32`, `x/64` -- is a sequence rather than a lookup.
    pub fn number(&self, value: u32) -> Number {
        let digits = if value == 0 {
            vec![0]
        } else {
            let mut digits = Vec::new();
            let mut rest = value;
            while rest > 0 {
                digits.push(rest % 10);
                rest /= 10;
            }
            digits.reverse();
            digits
        };

        Number {
            digits: digits.into_iter().map(|d| self.number_digit(d)).collect(),
        }
    }

    fn number_digit(&self, digit: u32) -> NumberDigit {
        let name = format!("timeSig{digit}");

        let codepoint = self
            .glyph_names
            .get(name.as_str())
            .unwrap_or_else(|| panic!("SMuFL font has no glyph named '{name}'"))
            .codepoint_char();

        let glyph_box = self
            .meta
            .glyph_boxes
            .get(name.as_str())
            .unwrap_or_else(|| panic!("SMuFL metadata has no bounding box for '{name}'"));
        let bbox: BoundingBox = glyph_box.into();

        // A font whose metadata omits `glyphAdvanceWidths` falls back to the
        // ink width, which sets adjacent digits flush against each other. Only
        // noticeable on a multi-digit number, and better than refusing to draw.
        let advance = self
            .meta
            .glyph_advance_widths
            .get(name.as_str())
            .copied()
            .unwrap_or_else(|| bbox.width());

        NumberDigit {
            codepoint,
            bbox,
            advance,
        }
    }

    /// Thickness in tenths of the vertical stroke the bracket tip glyphs are
    /// drawn against: the font's `bracketThickness`. A property of the glyphs,
    /// not a preference, so no user or document override applies.
    fn bracket_glyph_stroke(&self) -> f32 {
        self.layout
            .group_bracket
            .thickness
            .unwrap_or(APP_DEFAULTS.group_bracket.thickness)
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

        let glyph_box = self.meta.glyph_boxes.get(name).unwrap();
        let bbox: BoundingBox = glyph_box.into();

        let cutouts: Option<Cutouts> = self
            .meta
            .glyph_anchors
            .get(name)
            .map(|v| v.to_cutouts(&bbox));

        Accidental {
            codepoint,

            bbox,
            cutouts,
        }
    }
}
