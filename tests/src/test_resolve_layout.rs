//! Every element in the tree resolves its own appearance, and the recursion
//! that reaches it has to be kept in step with the recursion `measure` uses --
//! see `Layoutable::resolve_layout`. A container that forgets to forward
//! `resolve_layout` to a new child leaves that child at whatever colour it was
//! constructed with, rather than the one the layout actually asked for.

#[cfg(test)]
mod tests {
    use lib::geometry::color::Color;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::chord::Chord;
    use lib::score::visual::note::Note;
    use lib::score::visual::rest::Rest;
    use lib::score::visual::score::Score;
    use lib::score::visual::staff_measure::StaffMeasure;
    use lib::score::visual::stem::Stem;
    use lib::smufl::smufl_font::SmuflFont;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    const ACTOR_PRELUDE: &str = "assets/xmlsamples/ActorPreludeSample.musicxml";
    /// Brought in alongside [`ACTOR_PRELUDE`] specifically for its key
    /// signature: three sharps, where the prelude is in C throughout and so
    /// never gives a `KeySignature` an accidental to carry.
    const BRAHMS: &str = "assets/xmlsamples/BrahWiMeSample.musicxml";

    fn asset(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
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

    /// A real score, engraved with every foreground-coloured element forced to
    /// [`Color::RED`] -- a shade nothing defaults to at construction, so an
    /// element a container forgot to forward `resolve_layout` to is caught
    /// still holding its own construction-time colour instead of silently
    /// passing.
    fn engraved_in_red(fixture: &str) -> Score {
        let xml = asset(fixture);
        let document = roxmltree::Document::parse_with_options(
            &xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("fixture does not parse");

        let user_layout = UserLayout {
            foreground_color: Some(Color::RED),
            ..Default::default()
        };
        let app_defaults = AppDefaults::default();

        let (mut score, defaults, _) = walk_document(
            &document,
            font(),
            &user_layout,
            &app_defaults,
            &mut |_stage| {},
        );
        arrange_score(
            &mut score,
            &defaults,
            font(),
            &user_layout,
            &app_defaults,
            &mut |_stage| {},
        );

        score
    }

    fn assert_red(actual: Color, what: &str) {
        assert_eq!(
            actual.to_hex(),
            Color::RED.to_hex(),
            "{what} did not resolve to the overridden foreground colour"
        );
    }

    fn assert_stem(stem: &Stem, what: &str) {
        assert_red(stem.color, what);
        if let Some(flag) = &stem.flag {
            assert_red(flag.color, &format!("{what} flag"));
        }
    }

    fn assert_note(note: &Note, what: &str) {
        assert_red(note.color, what);
        if let Some(accidental) = &note.accidental {
            assert_red(accidental.color, &format!("{what} accidental"));
        }
        for (i, dot) in note.dots.iter().enumerate() {
            assert_red(dot.color, &format!("{what} dot {i}"));
        }
    }

    fn assert_rest(rest: &Rest, what: &str) {
        assert_red(rest.color, what);
        if let Some(clef) = &rest.clef_change {
            assert_red(clef.color, &format!("{what} clef change"));
        }
        for (i, dot) in rest.dots.iter().enumerate() {
            assert_red(dot.color, &format!("{what} dot {i}"));
        }
    }

    fn assert_chord(chord: &Chord, what: &str) {
        assert_red(chord.color, what);
        for (i, (_, note)) in chord.notes.iter().enumerate() {
            assert_note(note, &format!("{what} note {i}"));
        }
        if let Some(stem) = &chord.stem {
            assert_stem(stem, &format!("{what} stem"));
        }
        for (staff, clef) in chord.clef_change.iter() {
            assert_red(
                clef.color,
                &format!("{what} clef change on staff {staff:?}"),
            );
        }
    }

    fn assert_staff_measure(measure: &StaffMeasure, what: &str) {
        if let Some(clef) = &measure.clef_start {
            assert_red(clef.color, &format!("{what} clef_start"));
        }
        if let Some(time_signature) = &measure.time_signature_start {
            assert_red(
                time_signature.color,
                &format!("{what} time_signature_start"),
            );
        }
        for (i, (_, accidental)) in measure.key_signature_start.accidentals.iter().enumerate() {
            assert_red(
                accidental.color,
                &format!("{what} key signature accidental {i}"),
            );
        }
        if let Some(time_signature) = &measure.time_signature_end {
            assert_red(time_signature.color, &format!("{what} time_signature_end"));
        }
        if let Some(clef) = &measure.clef_end {
            assert_red(clef.color, &format!("{what} clef_end"));
        }
        for (i, rest) in measure.rests.iter().enumerate() {
            assert_rest(rest, &format!("{what} rest {i}"));
        }
    }

    /// Walks the whole tree a real score arranges into, checking every element
    /// that resolves its colour from `foreground_color` against
    /// [`Color::RED`]. Not every branch fires on every fixture -- a part with
    /// no accidentals leaves its key signature's accidental list empty -- but
    /// every branch that *does* fire is checked, which is what makes a missed
    /// `resolve_layout` forward visible instead of silently passing.
    fn assert_every_element_is_red(score: &Score) {
        assert!(!score.pages.is_empty(), "the fixture engraved to nothing");

        for page in score.pages.values() {
            assert_red(page.foreground, "page foreground");

            for system in page.systems.values() {
                assert_red(system.color, "system");

                for measure in system.measures.values() {
                    assert_red(measure.color, "system measure");
                }

                for section in system.sections.values() {
                    assert_red(section.symbol.color(), "section symbol");

                    for measure in section.measures.values() {
                        assert_red(measure.color, "section measure");
                    }

                    for group in section.part_groups.values() {
                        assert_red(group.symbol.color(), "part-group symbol");

                        for part in group.parts.values() {
                            assert_red(part.symbol.color(), "part symbol");

                            for staff in part.staves.values() {
                                assert_red(staff.color, "staff");

                                for measure in staff.measures.values() {
                                    assert_staff_measure(measure, "staff measure");
                                }
                            }

                            for measure in part.measures.values() {
                                assert_red(measure.color, "part measure");

                                for (voice, chords) in measure.chords.iter() {
                                    for (i, chord) in chords.iter().enumerate() {
                                        assert_chord(chord, &format!("voice {voice:?} chord {i}"));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// `BRAHMS` covers real key-signature accidentals; `ACTOR_PRELUDE` covers
    /// the rest -- see the comment on each constant.
    #[test]
    fn every_element_a_real_score_arranges_carries_the_overridden_foreground_color() {
        for fixture in [ACTOR_PRELUDE, BRAHMS] {
            assert_every_element_is_red(&engraved_in_red(fixture));
        }
    }
}
