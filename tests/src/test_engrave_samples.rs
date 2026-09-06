//! Regression guards for sample documents that used to abort the render walk.
//!
//! Each of these named a construct the parser had no arm for and panicked on,
//! rather than engraving something imperfect. The samples are deliberately
//! awkward exports and are never to be repaired, so the guard is simply that
//! each one still walks and arranges end to end.

#[cfg(test)]
mod tests {
    use cli::commands::read_musicxml;
    use std::fs::read_to_string;
    use wasm::{RenderOptions, WasmScore};

    fn asset(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    /// Walks and arranges one sample the way the wasm entry point does, which is
    /// the whole pipeline short of writing an output file.
    fn engrave(sample: &str) {
        let musicxml = read_musicxml(&format!(
            "{}/../assets/xmlsamples/{sample}.musicxml",
            env!("CARGO_MANIFEST_DIR")
        ));
        let meta_json = asset("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = asset("assets/smufl/metadata/glyphnames.json");

        let mut score = WasmScore::new(&musicxml, &meta_json, &glyph_names_json)
            .unwrap_or_else(|e| panic!("{sample} failed to build: {e:?}"));

        let mut output = score.render_with(&RenderOptions::default());
        assert!(
            !output.geometry().is_empty(),
            "{sample} engraved to no geometry at all"
        );
    }

    /// `<beam number="2">forward hook</beam>`. Only the "backward hook" spelling
    /// had an arm; the forward one hit the catch-all panic.
    #[test]
    fn a_score_with_forward_hook_beams_engraves() {
        engrave("Telemann");
    }

    /// `<stem>none</stem>` on an explicitly stemless note. `UpDown` only knows
    /// up and down, and the caller unwrapped the parse.
    #[test]
    fn a_score_with_stemless_notes_engraves() {
        engrave("Binchois");
    }

    /// `<sign>TAB</sign>` on the guitar-tablature staff.
    #[test]
    fn a_score_with_a_tablature_staff_engraves() {
        engrave("BrookeWestSample");
    }

    /// UTF-16 exports, which the file read rejected before it ever reached a
    /// parser.
    #[test]
    fn the_utf16_samples_engrave() {
        engrave("MozaChloSample");
        engrave("MozaVeilSample");
    }
}
