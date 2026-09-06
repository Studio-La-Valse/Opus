#[cfg(test)]
mod tests {
    use lib::score::app_defaults::AppDefaults;
    use lib::score::engrave::walk_document;
    use lib::score::score_defaults::ScoreDefaults;
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::score::Score;
    use lib::smufl::smufl_font::SmuflFont;
    use roxmltree::Document;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    fn asset(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| {
            SmuflFont::load(
                &asset("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json"),
                &asset("assets/smufl/metadata/glyphnames.json"),
            )
        })
    }

    /// A one-part, one-measure score. `appearance` goes inside `<defaults>`,
    /// `notes` inside the measure.
    fn score_xml(appearance: &str, notes: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <defaults><scaling><millimeters>7</millimeters><tenths>40</tenths></scaling>\
             <appearance>{appearance}</appearance></defaults>\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef><sign>G</sign><line>2</line></clef></attributes>\
             {notes}</measure></part></score-partwise>"
        )
    }

    /// `kind` is the element that marks the note: `<grace/>`, `<cue/>` or
    /// nothing at all. A grace note carries no `<duration>`.
    fn note(kind: &str, x: u32) -> String {
        let duration = if kind.contains("grace") {
            ""
        } else {
            "<duration>4</duration>"
        };

        format!(
            "<note default-x=\"{x}\">{kind}\
             <pitch><step>C</step><octave>5</octave></pitch>{duration}\
             <voice>1</voice><type>quarter</type><stem>up</stem></note>"
        )
    }

    fn walk(xml: &str) -> (Score, ScoreDefaults) {
        let document = Document::parse(xml).expect("test document does not parse");
        let (score, defaults, _messages) = walk_document(
            &document,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_stage| {},
        );

        (score, defaults)
    }

    /// Every note the walk built, in document order, by the scale it was built
    /// at.
    fn note_scales(xml: &str) -> Vec<f32> {
        let (score, _) = walk(xml);

        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.measures.values())
            .flat_map(|measure| measure.chords.values().flatten())
            .flat_map(|chord| chord.notes.iter())
            .map(|note| note.scale)
            .collect()
    }

    #[test]
    fn note_size_is_read_as_a_fraction_of_normal() {
        let (_, defaults) = walk(&score_xml(
            "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
            &note("", 80),
        ));

        assert_eq!(defaults.appearance.note_size_grace, Some(0.5));
        assert_eq!(defaults.appearance.note_size_cue, Some(0.7));
    }

    /// MusicXML also defines "grace-cue" and "large"; neither is drawn, and an
    /// unrecognised type is ignored the way an unrecognised `<line-width>` is.
    #[test]
    fn an_unhandled_note_size_type_is_ignored() {
        let (_, defaults) = walk(&score_xml(
            "<note-size type=\"large\">150</note-size><note-size type=\"grace-cue\">40</note-size>",
            &note("", 80),
        ));

        assert_eq!(defaults.appearance.note_size_grace, None);
        assert_eq!(defaults.appearance.note_size_cue, None);
    }

    #[test]
    fn a_declared_note_size_scales_the_note_it_names() {
        let scales = note_scales(&score_xml(
            "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
            &format!(
                "{}{}{}",
                note("", 80),
                note("<grace/>", 120),
                note("<cue/>", 160)
            ),
        ));

        assert_eq!(scales.len(), 3, "got {scales:?}");
        assert_eq!(scales[0], 1.0, "a normal note is not reduced");
        assert_eq!(scales[1], 0.5, "the declared grace size");
        assert_eq!(scales[2], 0.7, "the declared cue size");
    }

    /// A document that declares no `<note-size>` gets the app default, which is
    /// what every score got before `<note-size>` was read at all.
    #[test]
    fn an_undeclared_note_size_falls_back_to_the_app_default() {
        let app = AppDefaults::default();
        let scales = note_scales(&score_xml(
            "",
            &format!("{}{}", note("<grace/>", 80), note("<cue/>", 120)),
        ));

        assert_eq!(scales, vec![app.note_size_grace, app.note_size_cue]);
    }

    /// Rests carry the same reduction as notes -- a cue passage's rests are
    /// small too.
    #[test]
    fn a_cue_rest_is_reduced_like_a_cue_note() {
        let rest = "<note default-x=\"80\"><cue/><rest/><duration>4</duration>\
                    <voice>1</voice><type>quarter</type></note>";
        let (score, _) = walk(&score_xml("<note-size type=\"cue\">60</note-size>", rest));

        let scales: Vec<f32> = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.staves.values())
            .flat_map(|staff| staff.measures.values())
            .flat_map(|measure| measure.rests.iter())
            .map(|rest| rest.scale)
            .collect();

        assert_eq!(scales, vec![0.6]);
    }

    /// The two are mutually exclusive in the format. A note carrying both is
    /// read as a grace note rather than compounding the two reductions.
    #[test]
    fn grace_wins_over_cue_on_a_note_carrying_both() {
        let scales = note_scales(&score_xml(
            "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
            &note("<grace/><cue/>", 80),
        ));

        assert_eq!(scales, vec![0.5]);
    }
}
