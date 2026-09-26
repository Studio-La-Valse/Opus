//! `<note-size>`: the reduction a grace or cue note is drawn at.
//!
//! The size has two halves that are settled at different times. The document
//! walk records only what the note *is* (its [`NoteKind`]) plus the scaling its
//! staff applies to content; the factor that kind stands for is a layout
//! decision, resolved on every `arrange_score`. These tests pin both halves, and
//! the seam between them: a score walked once must resize when it is arranged
//! again under a different layout.

#[cfg(test)]
mod tests {
    use lib::score::core::note_kind::NoteKind;
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::layout_options::{APP_DEFAULTS, NoteSizeLayout, UserLayout};
    use lib::score::score_defaults::ScoreDefaults;
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

    /// An eighth note, which earns a flag, with one augmentation dot and an
    /// accidental -- so that every part of the note a reduction has to reach is
    /// present.
    fn dotted_eighth(kind: &str, x: u32) -> String {
        let duration = if kind.contains("grace") {
            ""
        } else {
            "<duration>3</duration>"
        };

        format!(
            "<note default-x=\"{x}\">{kind}\
             <pitch><step>C</step><alter>1</alter><octave>5</octave></pitch>{duration}\
             <voice>1</voice><type>eighth</type><dot/><stem>up</stem>\
             <accidental>sharp</accidental></note>"
        )
    }

    fn walk(xml: &str) -> (Score, ScoreDefaults) {
        let document = Document::parse(xml).expect("test document does not parse");
        let (score, defaults, _messages) =
            walk_document(&document, font(), &UserLayout::default(), &mut |_stage| {});

        (score, defaults)
    }

    /// The full pipeline: walk the document, then arrange it under `user_layout`
    /// -- which is when a note's size is actually decided.
    fn engrave(xml: &str, user_layout: &UserLayout) -> Score {
        let (mut score, defaults) = walk(xml);
        arrange(&mut score, &defaults, user_layout);

        score
    }

    fn arrange(score: &mut Score, defaults: &ScoreDefaults, user_layout: &UserLayout) {
        arrange_score(score, defaults, font(), user_layout, &mut |_stage| {});
    }

    /// Every note in the score, in document order.
    fn notes(score: &Score) -> Vec<&lib::score::visual::note::Note> {
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
            .collect()
    }

    fn note_scales(score: &Score) -> Vec<f32> {
        notes(score).iter().map(|note| note.scale).collect()
    }

    fn rest_scales(score: &Score) -> Vec<f32> {
        score
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
            .collect()
    }

    #[test]
    fn note_size_is_read_as_a_fraction_of_normal() {
        let (_, defaults) = walk(&score_xml(
            "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
            &note("", 80),
        ));

        assert_eq!(defaults.appearance.grace, Some(0.5));
        assert_eq!(defaults.appearance.cue, Some(0.7));
    }

    /// MusicXML also defines "grace-cue" and "large"; neither is drawn, and an
    /// unrecognised type is ignored the way an unrecognised `<line-width>` is.
    #[test]
    fn an_unhandled_note_size_type_is_ignored() {
        let (_, defaults) = walk(&score_xml(
            "<note-size type=\"large\">150</note-size><note-size type=\"grace-cue\">40</note-size>",
            &note("", 80),
        ));

        assert_eq!(defaults.appearance.grace, None);
        assert_eq!(defaults.appearance.cue, None);
    }

    /// The walk records what the note *is*, never the factor that stands for --
    /// the factor belongs to a render, and the walk's result is reused across
    /// many of them.
    #[test]
    fn the_walk_records_the_kind_and_leaves_the_factor_alone() {
        let (score, _) = walk(&score_xml(
            "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
            &format!(
                "{}{}{}",
                note("", 80),
                note("<grace/>", 120),
                note("<cue/>", 160)
            ),
        ));

        let kinds: Vec<NoteKind> = notes(&score).iter().map(|note| note.size.kind).collect();
        assert_eq!(
            kinds,
            vec![NoteKind::Normal, NoteKind::Grace, NoteKind::Cue]
        );

        // No `<staff-size>`, so the one thing the walk *can* settle is 1.
        assert!(notes(&score).iter().all(|n| n.size.content_scale == 1.0));
    }

    #[test]
    fn a_declared_note_size_scales_the_note_it_names() {
        let score = engrave(
            &score_xml(
                "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
                &format!(
                    "{}{}{}",
                    note("", 80),
                    note("<grace/>", 120),
                    note("<cue/>", 160)
                ),
            ),
            &UserLayout::default(),
        );

        let scales = note_scales(&score);
        assert_eq!(scales.len(), 3, "got {scales:?}");
        assert_eq!(scales[0], 1.0, "a normal note is not reduced");
        assert_eq!(scales[1], 0.5, "the declared grace size");
        assert_eq!(scales[2], 0.7, "the declared cue size");
    }

    /// A document that declares no `<note-size>` gets the app default.
    #[test]
    fn an_undeclared_note_size_falls_back_to_the_app_default() {
        let app = &APP_DEFAULTS.note_size;
        let score = engrave(
            &score_xml(
                "",
                &format!("{}{}", note("<grace/>", 80), note("<cue/>", 120)),
            ),
            &UserLayout::default(),
        );

        assert_eq!(note_scales(&score), vec![app.grace, app.cue]);
    }

    /// Rests carry the same reduction as notes -- a cue passage's rests are
    /// small too.
    #[test]
    fn a_cue_rest_is_reduced_like_a_cue_note() {
        let rest = "<note default-x=\"80\"><cue/><rest/><duration>4</duration>\
                    <voice>1</voice><type>quarter</type></note>";
        let score = engrave(
            &score_xml("<note-size type=\"cue\">60</note-size>", rest),
            &UserLayout::default(),
        );

        assert_eq!(rest_scales(&score), vec![0.6]);
    }

    /// The two are mutually exclusive in the format. A note carrying both is
    /// read as a grace note rather than compounding the two reductions.
    #[test]
    fn grace_wins_over_cue_on_a_note_carrying_both() {
        let score = engrave(
            &score_xml(
                "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
                &note("<grace/><cue/>", 80),
            ),
            &UserLayout::default(),
        );

        assert_eq!(note_scales(&score), vec![0.5]);
    }

    /// A caller's override wins over what the document declared. This is what
    /// the split exists for: `<note-size>` is a document default, not a
    /// commitment the engraver has to honour.
    #[test]
    fn a_user_override_beats_the_documents_declared_note_size() {
        let xml = score_xml(
            "<note-size type=\"grace\">50</note-size><note-size type=\"cue\">70</note-size>",
            &format!("{}{}", note("<grace/>", 80), note("<cue/>", 120)),
        );

        let score = engrave(
            &xml,
            &UserLayout {
                note_size: NoteSizeLayout {
                    grace: Some(0.25),
                    cue: Some(0.9),
                },
                ..Default::default()
            },
        );

        assert_eq!(note_scales(&score), vec![0.25, 0.9]);
    }

    /// The pipeline walks once and arranges many times -- the wasm bindings
    /// cache the walk and re-run only `arrange_score` per render. A note's size
    /// therefore has to follow the layout of the render asking, not the one that
    /// happened to come first.
    #[test]
    fn re_arranging_the_same_score_resizes_its_grace_notes() {
        let xml = score_xml(
            "<note-size type=\"grace\">50</note-size>",
            &note("<grace/>", 80),
        );
        let (mut score, defaults) = walk(&xml);

        arrange(&mut score, &defaults, &UserLayout::default());
        assert_eq!(note_scales(&score), vec![0.5], "the document's own size");

        arrange(
            &mut score,
            &defaults,
            &UserLayout {
                note_size: NoteSizeLayout {
                    grace: Some(0.8),
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        assert_eq!(
            note_scales(&score),
            vec![0.8],
            "the override, on a re-render"
        );

        arrange(&mut score, &defaults, &UserLayout::default());
        assert_eq!(
            note_scales(&score),
            vec![0.5],
            "and back again -- nothing was baked in"
        );
    }

    /// Everything hanging off a reduced note shrinks by the same number.
    /// Resolving the factor separately per element is how a grace note ends up
    /// with, say, full-size dots on a half-size notehead.
    #[test]
    fn every_part_of_a_grace_note_is_reduced_by_the_same_factor() {
        let score = engrave(
            &score_xml(
                "<note-size type=\"grace\">50</note-size>",
                &dotted_eighth("<grace/>", 80),
            ),
            &UserLayout::default(),
        );

        let chords: Vec<_> = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.measures.values())
            .flat_map(|measure| measure.chords.values().flatten())
            .collect();

        let chord = chords.first().expect("the grace chord should exist");
        let note = chord.notes.first().expect("the grace note should exist");
        let stem = chord.stem.as_ref().expect("an eighth note carries a stem");
        let flag = stem
            .flag
            .as_ref()
            .expect("an unbeamed eighth carries a flag");
        let dot = note.dots.first().expect("the note is dotted");
        let accidental = note
            .accidental
            .as_ref()
            .expect("the note carries an accidental");

        assert_eq!(note.scale, 0.5, "notehead");
        assert_eq!(dot.scale, 0.5, "augmentation dot");
        assert_eq!(stem.scale, 0.5, "stem");
        assert_eq!(flag.scale, 0.5, "flag");
        assert_eq!(accidental.scale, 0.5, "accidental");
    }

    /// A reduced accidental also has to be *drawn* smaller, not merely marked
    /// so: the scale reaches the glyph's own measured size, and the gap it
    /// leaves before the notehead shrinks with it rather than staying a
    /// full-size note's.
    #[test]
    fn a_reduced_accidental_is_narrower_and_sits_closer_to_its_notehead() {
        let engrave_at = |grace: f32| {
            let score = engrave(
                &score_xml("", &dotted_eighth("<grace/>", 80)),
                &UserLayout {
                    note_size: NoteSizeLayout {
                        grace: Some(grace),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );

            let notes = notes(&score);
            let note = notes.first().expect("the grace note should exist");
            let accidental = note
                .accidental
                .as_ref()
                .expect("the note carries an accidental");

            (accidental.width, note.xy.x - accidental.xy.x)
        };

        let (full_width, full_reach) = engrave_at(1.);
        let (half_width, half_reach) = engrave_at(0.5);

        assert!(full_width > 0.);
        assert!(
            (half_width - full_width / 2.).abs() < 1e-4,
            "a half-size accidental is half as wide: {half_width} against {full_width}"
        );
        assert!(
            (half_reach - full_reach / 2.).abs() < 1e-4,
            "and reaches half as far left of the notehead: {half_reach} against {full_reach}"
        );
    }
}
