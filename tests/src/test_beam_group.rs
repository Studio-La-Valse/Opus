//! The beam pass, from the run it is handed down to what each fragment draws.
//!
//! Three stages, and each one is where a different kind of mistake would land: a
//! run is chunked into groups by the beam types the document declares, a group is
//! chunked into fragments by the system breaks it crosses, and a fragment then
//! resolves each of its levels to a span. The first two are pure list handling and
//! are unit-tested here; the third only means anything in tenths, so it is checked
//! against `crazy-beams.musicxml` engraved end to end.

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    use cli::commands::read_musicxml;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::core::duration_base::BaseDuration;
    use lib::score::core::note_kind::NoteKind;
    use lib::score::engrave::engrave;
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::arranger::{BeamArranger, ScoreArranger};
    use lib::score::visual::beam::{
        Beamable, LevelEnd, beam_level_ends_at, create_beam_groups, infer_direction,
        split_at_system_breaks,
    };
    use lib::score::visual::chord::Chord;
    use lib::score::visual::layoutable::LayoutParams;
    use lib::score::visual::note_scale::NoteScale;
    use lib::score::visual::score::Score;
    use lib::score::visual::stem::{BeamType, Stem, UpDown};
    use lib::score::visual::system::SystemKey;
    use lib::smufl::smufl_font::SmuflFont;

    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    fn asset(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&asset(BRAVURA_META), &asset(GLYPH_NAMES)))
    }

    /// A chord whose stem declares `beams`, one entry per level. Nothing else
    /// about the chord matters to the grouping or the level walk.
    fn chord(direction: UpDown, beams: &[(u32, BeamType)]) -> Chord {
        let mut stem = Stem::new(
            direction,
            BaseDuration::Sixteenth,
            0.into(),
            NoteScale::new(1., NoteKind::Normal),
            None,
        );

        for (level, kind) in beams {
            stem.beams.insert(*level, *kind);
        }

        Chord {
            stem: Some(stem),
            ..Default::default()
        }
    }

    /// A stem-up chord carrying one beam level, which is all the grouping reads.
    fn beamed(kind: BeamType) -> Chord {
        chord(UpDown::Up, &[(1, kind)])
    }

    /// A chord with no stem at all -- a whole note, or one written
    /// `<stem>none</stem>`. It carries no beam, so it can never be beamed.
    fn stemless() -> Chord {
        Chord::default()
    }

    /// The run the pass is handed: each chord paired with the system it landed on.
    fn run<'a>(chords: &'a mut [Chord], keys: &[SystemKey]) -> Vec<Beamable<'a>> {
        assert_eq!(chords.len(), keys.len(), "one key per chord");

        chords
            .iter_mut()
            .zip(keys)
            .map(|(chord, key)| Beamable { key: *key, chord })
            .collect()
    }

    /// Every chord of the run on one and the same system, for the tests that are
    /// about grouping rather than breaking.
    fn one_system(chords: &mut [Chord]) -> Vec<Beamable<'_>> {
        let keys = vec![(1, 1); chords.len()];
        run(chords, &keys)
    }

    fn group_sizes(groups: &[Vec<Beamable<'_>>]) -> Vec<usize> {
        groups.iter().map(|group| group.len()).collect()
    }

    // ---------------------------------------------------------------- grouping

    #[test]
    fn a_group_runs_from_its_begin_to_its_end() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(one_system(&mut chords));

        assert_eq!(group_sizes(&groups), vec![3]);
    }

    /// The run is now a whole part's, so a group may legally span a barline: two
    /// measures' worth of chords that read `begin ... end` are one group, not two.
    #[test]
    fn a_group_spans_whatever_barlines_lie_inside_it() {
        // begin, continue | continue, end -- a barline between the second and
        // third chord changes nothing about the sequence.
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(one_system(&mut chords));

        assert_eq!(group_sizes(&groups), vec![4]);
    }

    /// Formerly `panic!("Cannot start a beam group when one is already open")`.
    /// The second `begin` says a group starts there, so whatever was open ended.
    #[test]
    fn a_second_begin_closes_the_group_that_was_open() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::Start),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(one_system(&mut chords));

        assert_eq!(group_sizes(&groups), vec![2, 2]);
    }

    /// Formerly `panic!("Cannot continue a beam group when none is open")` -- the
    /// panic `crazy-beams.musicxml` used to hit at the first barline it crossed.
    #[test]
    fn a_continue_with_nothing_open_opens_a_group() {
        let mut chords = [beamed(BeamType::Continue), beamed(BeamType::End)];

        let groups = create_beam_groups(one_system(&mut chords));

        assert_eq!(group_sizes(&groups), vec![2]);
    }

    /// Formerly `panic!("Cannot end a beam group when none is open")`.
    #[test]
    fn an_end_with_nothing_open_opens_and_closes_a_group() {
        let mut chords = [beamed(BeamType::End), beamed(BeamType::Start)];

        let groups = create_beam_groups(one_system(&mut chords));

        assert_eq!(group_sizes(&groups), vec![1, 1]);
    }

    /// A chord carrying no beam at all stands alone, and closes whatever it lands
    /// in. A chord with no stem is the same case: it cannot carry one.
    #[test]
    fn a_chord_with_no_beam_closes_the_group_it_lands_in() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            stemless(),
            beamed(BeamType::Start),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(one_system(&mut chords));

        assert_eq!(group_sizes(&groups), vec![3, 2]);
    }

    // ------------------------------------------------------------- fragmenting

    #[test]
    fn a_group_inside_one_system_is_a_single_uncut_fragment() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];

        let fragments = split_at_system_breaks(one_system(&mut chords));

        assert_eq!(fragments.len(), 1);
        assert_eq!(fragments[0].chords.len(), 3);
        assert!(!fragments[0].cut.left, "nothing precedes it");
        assert!(!fragments[0].cut.right, "nothing follows it");
    }

    #[test]
    fn a_group_across_a_system_break_splits_at_it() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];
        let keys = [(1, 1), (1, 1), (1, 2), (1, 2)];

        let fragments = split_at_system_breaks(run(&mut chords, &keys));

        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].key, (1, 1));
        assert_eq!(fragments[0].chords.len(), 2);
        assert_eq!(fragments[1].key, (1, 2));
        assert_eq!(fragments[1].chords.len(), 2);

        assert!(!fragments[0].cut.left && fragments[0].cut.right);
        assert!(fragments[1].cut.left && !fragments[1].cut.right);
    }

    /// A page break *is* a system break, so this needs no handling of its own:
    /// the two fragments simply file under two different pages.
    #[test]
    fn a_group_across_a_page_break_splits_the_same_way() {
        let mut chords = [beamed(BeamType::Start), beamed(BeamType::End)];
        let keys = [(1, 2), (2, 1)];

        let fragments = split_at_system_breaks(run(&mut chords, &keys));

        assert_eq!(fragments.len(), 2);
        assert_eq!(fragments[0].key, (1, 2));
        assert_eq!(fragments[1].key, (2, 1));
        assert!(fragments[0].cut.right && fragments[1].cut.left);
    }

    /// A group reaching into three systems is cut on both sides in the middle.
    #[test]
    fn a_fragment_between_two_others_is_cut_on_both_sides() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];
        let keys = [(1, 1), (1, 2), (1, 3)];

        let fragments = split_at_system_breaks(run(&mut chords, &keys));

        assert_eq!(fragments.len(), 3);
        assert!(fragments[1].cut.left && fragments[1].cut.right);
    }

    // ------------------------------------------------------------- level spans

    #[test]
    fn a_level_that_ends_in_the_fragment_is_closed_at_that_stem() {
        let chords = [
            chord(UpDown::Up, &[(1, BeamType::Start), (2, BeamType::Start)]),
            chord(UpDown::Up, &[(1, BeamType::Continue), (2, BeamType::End)]),
            chord(UpDown::Up, &[(1, BeamType::End)]),
        ];
        let mut chords = chords;
        let borrowed: Vec<&mut Chord> = chords.iter_mut().collect();

        let end = beam_level_ends_at(&borrowed, 0, &2);

        match end {
            LevelEnd::ClosedAt(stem) => {
                assert_eq!(stem.beams.get(&2), Some(&BeamType::End));
            }
            LevelEnd::OpenAt(_) => panic!("level 2 declares an end, so it is closed"),
        }
    }

    /// An unterminated level: the document declares `begin`, `continue` and no
    /// `end`, so the level is taken to run to the last stem carrying it. Which is
    /// also what a level running off the edge of a fragment looks like -- only the
    /// caller, which knows whether the fragment was cut, can tell the two apart.
    #[test]
    fn a_level_with_no_end_is_open_at_its_last_carrier() {
        let mut chords = [
            chord(UpDown::Up, &[(1, BeamType::Start), (2, BeamType::Start)]),
            chord(
                UpDown::Up,
                &[(1, BeamType::Continue), (2, BeamType::Continue)],
            ),
            chord(UpDown::Up, &[(1, BeamType::End)]),
        ];
        let borrowed: Vec<&mut Chord> = chords.iter_mut().collect();

        let end = beam_level_ends_at(&borrowed, 0, &2);

        match end {
            LevelEnd::OpenAt(Some(stem)) => {
                assert_eq!(
                    stem.beams.get(&2),
                    Some(&BeamType::Continue),
                    "the last stem carrying level 2 is the middle one"
                );
            }
            LevelEnd::OpenAt(None) => panic!("one later stem does carry level 2"),
            LevelEnd::ClosedAt(_) => panic!("level 2 declares no end"),
        }
    }

    #[test]
    fn a_level_nothing_else_carries_is_open_at_nothing() {
        let mut chords = [
            chord(UpDown::Up, &[(1, BeamType::Start), (2, BeamType::Start)]),
            chord(UpDown::Up, &[(1, BeamType::End)]),
        ];
        let borrowed: Vec<&mut Chord> = chords.iter_mut().collect();

        assert!(matches!(
            beam_level_ends_at(&borrowed, 0, &2),
            LevelEnd::OpenAt(None)
        ));
    }

    // --------------------------------------------------------------- direction

    /// With every stem pointing the same way the stack grows away from them, so
    /// the second beam sits further from the noteheads than the first.
    #[test]
    fn a_group_whose_stems_agree_stacks_away_from_them() {
        let mut chords = [beamed(BeamType::Start), beamed(BeamType::End)];

        let group = one_system(&mut chords);

        assert_eq!(infer_direction(&group), Some(UpDown::Down));
    }

    /// Cross-staff beaming: the stems of one staff point up and the other's down,
    /// and the stack grows towards the odd one out rather than away from it. This
    /// is the case `crazy-beams.musicxml` renders, and the one thing about the
    /// pass that moving it out of `PartMeasure` most easily could have broken.
    #[test]
    fn a_cross_staff_group_stacks_towards_the_odd_stem_out() {
        let mut chords = [
            chord(UpDown::Up, &[(1, BeamType::Start)]),
            chord(UpDown::Down, &[(1, BeamType::End)]),
        ];

        let group = one_system(&mut chords);

        assert_eq!(infer_direction(&group), Some(UpDown::Up));
    }

    #[test]
    fn a_group_of_nothing_but_stemless_chords_has_no_direction() {
        let mut chords = [stemless(), stemless()];

        let group = one_system(&mut chords);

        assert_eq!(infer_direction(&group), None);
    }

    // ------------------------------------------------------------- in tenths

    fn engraved(fixture: &str) -> Score {
        let path = format!(
            "{}/../assets/xmlfixtures/{fixture}.musicxml",
            env!("CARGO_MANIFEST_DIR")
        );
        let source = read_musicxml(&path);
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let document =
            roxmltree::Document::parse_with_options(&source, options).expect("fixture parses");

        engrave(
            &document,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_| {},
        )
        .score
    }

    /// One system's drawn beams, with the two things a span has to be read
    /// against: how far the stems on that system reach, and where its internal
    /// barlines fall.
    ///
    /// Deliberately relative rather than a table of tenths. What the fragment
    /// rule claims is that a beam reaches *past its outermost stem* when its group
    /// carries on past the system, and *through a barline* when the group spans
    /// one -- both of which stay true however the fixture is spaced.
    struct SystemBeams {
        measures: Vec<u32>,
        stem_left: f32,
        stem_right: f32,
        barlines: Vec<f32>,
        spans: Vec<(f32, f32)>,
    }

    fn system_beams(score: &Score) -> Vec<SystemBeams> {
        let mut out = Vec::new();

        for page in score.pages.values() {
            for system in page.systems.values() {
                let mut stem_left = f32::MAX;
                let mut stem_right = f32::MIN;
                let mut barlines = Vec::new();

                for section in system.sections.values() {
                    for group in section.part_groups.values() {
                        for part in group.parts.values() {
                            for measure in part.measures.values() {
                                barlines.push(measure.origin.x + measure.width);

                                for chord in measure.chords.values().flatten() {
                                    if let Some(stem) = chord.stem.as_ref() {
                                        stem_left = stem_left.min(stem.xy.x);
                                        stem_right = stem_right.max(stem.xy.x);
                                    }
                                }
                            }
                        }
                    }
                }

                // Every part repeats the same measure spans, and the system's own
                // right edge is not a barline between two of its measures.
                barlines.sort_by(|a, b| a.partial_cmp(b).unwrap());
                barlines.dedup_by(|a, b| (*a - *b).abs() < 1e-3);
                barlines.pop();

                out.push(SystemBeams {
                    measures: system.measures.keys().copied().collect(),
                    stem_left,
                    stem_right,
                    barlines,
                    spans: system
                        .beams
                        .iter()
                        .map(|beam| {
                            (
                                beam.pts.iter().map(|p| p.x).fold(f32::MAX, f32::min),
                                beam.pts.iter().map(|p| p.x).fold(f32::MIN, f32::max),
                            )
                        })
                        .collect(),
                });
            }
        }

        out
    }

    /// A beam group left open at the end of a system runs a stub out past its last
    /// stem to the break, and the fragment arriving on the next system runs one
    /// back in past its first stem.
    ///
    /// `crazy-beams.musicxml` crosses its measure 2 / 3 system break with exactly
    /// one group, which carries both beam levels -- so there is one stub per level
    /// on each side and nothing else reaches past the stems at all.
    #[test]
    fn a_group_across_a_system_break_reaches_past_the_stems_on_both_sides() {
        let score = engraved("crazy-beams");
        let systems = system_beams(&score);

        let before = &systems[0];
        let after = &systems[1];
        assert_eq!(before.measures, vec![1, 2], "the system the group leaves");
        assert_eq!(after.measures, vec![3, 4], "the system it arrives on");

        let out = before
            .spans
            .iter()
            .filter(|(_, right)| *right > before.stem_right)
            .count();
        assert_eq!(
            out, 2,
            "one stub per beam level past the last stem of measure 2"
        );

        let back_in = after
            .spans
            .iter()
            .filter(|(left, _)| *left < after.stem_left)
            .count();
        assert_eq!(
            back_in, 2,
            "one stub per beam level back past the first stem of measure 3"
        );
    }

    /// A group crossing a barline with no break in it is drawn as one unbroken beam
    /// per level, straight through the barline -- not as two beams abutting it,
    /// which is what a measure-at-a-time pass could produce at best.
    #[test]
    fn a_group_across_a_barline_is_one_unbroken_beam() {
        let score = engraved("crazy-beams");
        let systems = system_beams(&score);

        let system = &systems[0];
        assert_eq!(
            system.barlines.len(),
            1,
            "measures 1 and 2 with one barline between them"
        );
        let barline = system.barlines[0];

        let crossing = system
            .spans
            .iter()
            .filter(|(left, right)| *left < barline && *right > barline)
            .count();

        assert_eq!(
            crossing, 2,
            "one segment per beam level, drawn straight through the barline"
        );
    }

    /// `crazy-beams.musicxml` is a piano part whose beam groups cross both the
    /// barline and the two staves of the grand staff. The cross-staff beaming is
    /// not what this refactor is for, but it runs through all the code the refactor
    /// moved, and the shape of it is unmistakable: because the group's stems point
    /// up from the lower staff and down from the upper one, every beam in the
    /// document is drawn below the upper staff and no lower than the bottom line of
    /// the lower one. A group that had lost its cross-staff reading would beam one
    /// staff on its own and land clear outside that band.
    #[test]
    fn the_cross_staff_beams_stay_between_the_two_staves() {
        let score = engraved("crazy-beams");

        let mut systems = 0;

        for page in score.pages.values() {
            for system in page.systems.values() {
                let upper = system.find_first_visible_staff();
                let lower = system.find_last_visible_staff();
                let below_upper = upper.xy.y + upper.height();
                let above_lower_bottom = lower.xy.y + lower.height();

                assert!(
                    !system.beams.is_empty(),
                    "every system of the fixture carries beams"
                );

                for beam in system.beams.iter() {
                    for point in beam.pts.iter() {
                        assert!(
                            point.y > below_upper && point.y < above_lower_bottom,
                            "beam point at y {} is outside the {below_upper}..\
                             {above_lower_bottom} cross-staff band",
                            point.y
                        );
                    }
                }

                systems += 1;
            }
        }

        assert_eq!(systems, 4, "two systems on each of two pages");
    }

    // ------------------------------------------------------------ idempotency

    fn engrave_crazy_beams() -> lib::score::engrave::EngravedScore {
        let path = format!(
            "{}/../assets/xmlfixtures/crazy-beams.musicxml",
            env!("CARGO_MANIFEST_DIR")
        );
        let source = read_musicxml(&path);
        let options = roxmltree::ParsingOptions {
            allow_dtd: true,
            ..roxmltree::ParsingOptions::default()
        };
        let document =
            roxmltree::Document::parse_with_options(&source, options).expect("fixture parses");

        engrave(
            &document,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_| {},
        )
    }

    /// Every stem length and beam span in the score, in document order, so a
    /// second arrange can be compared point-for-point against the first.
    fn stems_and_beams(score: &Score) -> (Vec<f32>, Vec<Vec<(f32, f32)>>) {
        let mut lengths = Vec::new();
        for page in score.pages.values() {
            for system in page.systems.values() {
                for section in system.sections.values() {
                    for group in section.part_groups.values() {
                        for part in group.parts.values() {
                            for measure in part.measures.values() {
                                for chord in measure.chords.values().flatten() {
                                    if let Some(stem) = chord.stem.as_ref() {
                                        lengths.push(stem.length);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let spans = system_beams(score).into_iter().map(|s| s.spans).collect();

        (lengths, spans)
    }

    /// The wasm render path re-runs `BeamArranger` on a cached, already-beamed
    /// `Score` for every frame -- so unlike `TieArranger` and
    /// `ClefChangeArranger`, which never move anything they read, `BeamArranger`
    /// has to actively undo the stem lengths it wrote last time before it fits
    /// a ray again. See the `# Idempotency` note on `beam_arranger`.
    #[test]
    fn re_arranging_does_not_move_beams_or_stems() {
        let engraved = engrave_crazy_beams();
        let mut score = engraved.score;

        let user_layout = UserLayout::default();
        let app_defaults = AppDefaults::default();
        let params = LayoutParams {
            score_defaults: &engraved.layout,
            user_layout: &user_layout,
            app_defaults: &app_defaults,
            font: font(),
        };

        let first = stems_and_beams(&score);
        assert!(!first.0.is_empty(), "fixture has stemmed chords");
        assert!(!first.1.is_empty(), "fixture has beams");

        BeamArranger.arrange(&mut score, params);

        let second = stems_and_beams(&score);
        assert_eq!(second.0, first.0, "stem lengths moved on a second arrange");
        assert_eq!(second.1, first.1, "beam spans moved on a second arrange");
    }
}
