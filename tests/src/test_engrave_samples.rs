//! Regression guards for documents that used to abort the render walk.
//!
//! Each one names a construct the parser had no arm for and panicked on, rather
//! than engraving something imperfect. The guard is simply that the document
//! still walks and arranges end to end -- these are the cases where the failure
//! was total, so anything at all coming out the far side is the assertion.
//!
//! Two sources, and the distinction matters:
//!
//! - `assets/xmlsamples` are the official MusicXML sample files, cloned as they
//!   ship. Several are deliberately awkward exports and none of them is ever
//!   edited: when one fails, the parser is what changes.
//! - `assets/xmlfixtures` are documents written for this engine, to reach a case
//!   no official sample happens to cover.

#[cfg(test)]
mod tests {
    use cli::commands::read_musicxml;
    use std::fs::read_to_string;
    use wasm::{RenderOptions, WasmMusicFont, WasmScore};

    fn asset(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const LELAND_META: &str = "assets/smufl/Leland-main/leland_metadata.json";
    const MAESTRO_META: &str = "assets/smufl/Maestro-main/Finale Maestro.json";

    /// Walks and arranges one document the way the wasm entry point does, which
    /// is the whole pipeline short of writing an output file. `document` is a
    /// repo-relative path.
    fn engrave(document: &str) {
        engrave_in(document, BRAVURA_META);
    }

    /// [`engrave`] in the font whose metadata is at `meta`.
    fn engrave_in(document: &str, meta: &str) {
        let musicxml = read_musicxml(&format!("{}/../{document}", env!("CARGO_MANIFEST_DIR")));
        let font = WasmMusicFont::new(&asset(meta));

        let mut score = WasmScore::new(&musicxml)
            .unwrap_or_else(|e| panic!("{document} failed to build: {e:?}"));

        let mut output = score.render_with(&RenderOptions::default(), &font);
        assert!(
            !output.geometry().is_empty(),
            "{document} engraved to no geometry at all"
        );
    }

    fn sample(name: &str) {
        engrave(&format!("assets/xmlsamples/{name}.musicxml"));
    }

    fn fixture(name: &str) {
        engrave(&format!("assets/xmlfixtures/{name}.musicxml"));
    }

    /// `<beam number="2">forward hook</beam>`. Only the "backward hook" spelling
    /// had an arm; the forward one hit the catch-all panic.
    #[test]
    fn a_score_with_forward_hook_beams_engraves() {
        sample("Telemann");
    }

    /// `<stem>none</stem>` on an explicitly stemless note. `UpDown` only knows
    /// up and down, and the caller unwrapped the parse.
    #[test]
    fn a_score_with_stemless_notes_engraves() {
        sample("Binchois");
    }

    /// `<sign>TAB</sign>` on the guitar-tablature staff.
    #[test]
    fn a_score_with_a_tablature_staff_engraves() {
        sample("BrookeWestSample");
    }

    /// UTF-16 exports, which the file read rejected before it ever reached a
    /// parser.
    #[test]
    fn the_utf16_samples_engrave() {
        sample("MozaChloSample");
        sample("MozaVeilSample");
    }

    /// 17/16 -- two glyphs over two. No official sample carries a time
    /// signature that needs more than one glyph a side, so the lookup for
    /// `timeSig17` / `timeSig16` failed with nothing to catch it. The unit
    /// tests in `test_time_signature` cover the layout; this covers the walk
    /// actually reaching it.
    #[test]
    fn a_score_with_a_two_digit_time_signature_engraves() {
        fixture("beams");
    }

    /// 4/4, 12/8, 3/16, 15/32 and 1/64 in consecutive measures: every
    /// combination of one and two digits a side, plus a signature change
    /// mid-score.
    #[test]
    fn a_score_of_assorted_time_signatures_engraves() {
        fixture("timesigs");
    }

    /// A beam level opened and never closed, at both the secondary level and at
    /// level 1. `create_beams` used to unwrap its way to the closing stem and
    /// panic when there wasn't one.
    #[test]
    fn a_score_with_an_unclosed_beam_level_engraves() {
        fixture("unclosed-beam");
    }

    /// Every `<group-symbol>` value at once, over an uneven structure: groups of
    /// one, two, three and four parts, parts of one, two and three staves, a
    /// part directly inside a section and one enclosed by nothing at all.
    #[test]
    fn a_score_of_every_group_symbol_engraves() {
        fixture("group-symbols");
    }

    /// Beam groups that run straight through a barline, and one that runs through a
    /// system break, on a piano grand staff whose beams also cross between the two
    /// staves of the grand staff.
    ///
    /// The cross-measure group is the case that panicked outright:
    /// `create_beam_groups` met a `<beam number="1">end</beam>` with no group
    /// open, because the group had been flushed at the previous barline.
    #[test]
    fn a_score_with_beams_across_barlines_engraves() {
        fixture("crazy-beams");
    }

    /// A full orchestral score, which is the largest document the fixtures hold
    /// and the one that exercises the most levels of grouping at once.
    #[test]
    fn an_orchestral_score_engraves() {
        fixture("stresstest");
    }

    /// Every sample and fixture, repo-relative.
    fn every_document() -> Vec<String> {
        let mut documents = Vec::new();
        for dir in ["assets/xmlsamples", "assets/xmlfixtures"] {
            let path = format!("{}/../{dir}", env!("CARGO_MANIFEST_DIR"));
            for entry in std::fs::read_dir(&path).unwrap() {
                let name = entry.unwrap().file_name().into_string().unwrap();
                if name.ends_with(".musicxml") {
                    documents.push(format!("{dir}/{name}"));
                }
            }
        }
        documents.sort();
        documents
    }

    /// Leland ships no `glyphAdvanceWidths` and leaves the anchors off
    /// `noteheadDoubleWhole`; neither may stop a document engraving.
    #[test]
    fn every_document_engraves_in_leland() {
        for document in every_document() {
            engrave_in(&document, LELAND_META);
        }
    }

    /// Finale Maestro ships neither advance widths nor any glyph alternates, so
    /// every brace falls back to the plain one.
    #[test]
    fn every_document_engraves_in_finale_maestro() {
        for document in every_document() {
            engrave_in(&document, MAESTRO_META);
        }
    }
}
