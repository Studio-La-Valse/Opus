//! Which music font a score is engraved in: what the document asks for, and
//! how that is weighed against the user's choice and what is available.

#[cfg(test)]
mod tests {
    use cli::commands::read_musicxml;
    use lib::score::engrave::walk_document;
    use lib::smufl::font_choice::{MusicFontError, choose_music_font};
    use roxmltree::{Document, ParsingOptions};
    use std::sync::OnceLock;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn available() -> &'static [String] {
        static AVAILABLE: OnceLock<Vec<String>> = OnceLock::new();
        AVAILABLE.get_or_init(|| names(&["Bravura", "Leland", "Finale Maestro"]))
    }

    /// The `<music-font>` list a document's walk records.
    fn document_music_font(xml: &str) -> Vec<String> {
        let options = ParsingOptions {
            allow_dtd: true,
            ..ParsingOptions::default()
        };
        let document = Document::parse_with_options(xml, options).unwrap();
        let (_score, defaults, _messages) = walk_document(&document, &mut |_| {});
        defaults.music_font
    }

    // ------------------------------------------------------------ choosing

    #[test]
    fn the_user_overrides_the_document() {
        let chosen = choose_music_font(Some("Leland"), &names(&["Bravura"]), available());
        assert_eq!(chosen, Ok("Leland"));
    }

    #[test]
    fn the_document_overrides_the_default() {
        let chosen = choose_music_font(None, &names(&["Leland"]), available());
        assert_eq!(chosen, Ok("Leland"));
    }

    #[test]
    fn without_either_the_default_is_bravura() {
        assert_eq!(choose_music_font(None, &[], available()), Ok("Bravura"));
    }

    /// Finale writes its own legacy name, followed by the generic `engraved`
    /// that matches no font at all.
    #[test]
    fn maestro_is_finale_maestro_and_generics_are_skipped() {
        let chosen = choose_music_font(None, &names(&["Maestro", "engraved"]), available());
        assert_eq!(chosen, Ok("Finale Maestro"));

        let chosen = choose_music_font(None, &names(&["engraved", "Leland"]), available());
        assert_eq!(chosen, Ok("Leland"));
    }

    /// Names are matched regardless of case, and come back spelled the way the
    /// available font spells itself.
    #[test]
    fn names_match_case_insensitively() {
        let chosen = choose_music_font(Some("finale maestro"), &[], available());
        assert_eq!(chosen, Ok("Finale Maestro"));

        let chosen = choose_music_font(None, &names(&["LELAND"]), available());
        assert_eq!(chosen, Ok("Leland"));
    }

    /// An unavailable document font falls through to the default.
    #[test]
    fn a_document_font_that_is_missing_falls_back_to_the_default() {
        let chosen = choose_music_font(None, &names(&["Petaluma"]), available());
        assert_eq!(chosen, Ok("Bravura"));
    }

    /// An explicit user choice is never swapped for another font.
    #[test]
    fn a_user_font_that_is_missing_is_an_error() {
        let chosen = choose_music_font(Some("Petaluma"), &names(&["Leland"]), available());
        assert_eq!(chosen, Err(MusicFontError::Unavailable("Petaluma".into())));
    }

    #[test]
    fn nothing_available_is_an_error() {
        let available = names(&["Petaluma"]);
        let chosen = choose_music_font(None, &names(&["Leland"]), &available);
        assert_eq!(chosen, Err(MusicFontError::NoneAvailable));
    }

    // ------------------------------------------------------------ reading

    #[test]
    fn the_walk_records_the_music_font_list_in_order() {
        let xml = read_musicxml(&format!(
            "{}/../assets/xmlsamples/MozartTrio.musicxml",
            env!("CARGO_MANIFEST_DIR")
        ));
        assert_eq!(document_music_font(&xml), names(&["Maestro", "engraved"]));
    }

    #[test]
    fn a_document_without_a_music_font_records_none() {
        let xml = "<score-partwise version=\"4.0\">\
             <defaults><scaling><millimeters>7</millimeters><tenths>40</tenths></scaling></defaults>\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\"/></part>\
             </score-partwise>";
        assert!(document_music_font(xml).is_empty());
    }

    /// Whitespace and quotes around a name are not part of it.
    #[test]
    fn names_are_trimmed_of_whitespace_and_quotes() {
        let xml = "<score-partwise version=\"4.0\">\
             <defaults><scaling><millimeters>7</millimeters><tenths>40</tenths></scaling>\
             <music-font font-family=\" 'Finale Maestro' , Bravura,\"/></defaults>\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\"/></part>\
             </score-partwise>";
        assert_eq!(
            document_music_font(xml),
            names(&["Finale Maestro", "Bravura"])
        );
    }
}
