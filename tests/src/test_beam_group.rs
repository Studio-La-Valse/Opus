//! The beam pass, from the run it is handed down to what each group draws.
//!
//! Two stages, and each is where a different kind of mistake would land: a run is
//! chunked into groups by the beam types the document declares -- which is also
//! where a group reaching past the part it was collected from is recognised -- and
//! each group then resolves its levels to spans. The first is pure list handling
//! and is unit-tested here; the second only means anything in tenths, so it is
//! checked against `crazy-beams.musicxml` engraved end to end.

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
    use lib::score::visual::beam_arranger::{
        BeamGroup, Cut, LevelEnd, beam_level_ends_at, create_beam_groups, infer_direction,
    };
    use lib::score::visual::chord::Chord;
    use lib::score::visual::note_scale::NoteScale;
    use lib::score::visual::score::Score;
    use lib::score::visual::stem::{BeamType, Stem, UpDown};
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

    /// The run the pass is handed: one voice's chords from one part, in measure
    /// order.
    fn run(chords: &mut [Chord]) -> Vec<&mut Chord> {
        chords.iter_mut().collect()
    }

    /// A group that begins and ends inside this part, and so draws no stub.
    const WHOLE: Cut = Cut {
        left: false,
        right: false,
    };
    /// A group whose `begin` is on the previous system.
    const OPENS_BEFORE: Cut = Cut {
        left: true,
        right: false,
    };
    /// A group that closes on the next system.
    const CLOSES_AFTER: Cut = Cut {
        left: false,
        right: true,
    };
    /// A group that only passes through this part, reaching past it both ways.
    const PASSES_THROUGH: Cut = Cut {
        left: true,
        right: true,
    };

    /// How many chords each group holds and how far past this part it reaches,
    /// which between them are everything the grouping decides.
    fn group_shapes(groups: &[BeamGroup<'_>]) -> Vec<(usize, Cut)> {
        groups
            .iter()
            .map(|group| (group.chords.len(), group.cut))
            .collect()
    }

    // ---------------------------------------------------------------- grouping

    #[test]
    fn a_group_runs_from_its_begin_to_its_end() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(3, WHOLE)]);
    }

    /// The run is a whole part's, so a group may legally span a barline: two
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

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(4, WHOLE)]);
    }

    /// Formerly `panic!("Cannot start a beam group when one is already open")`.
    /// The second `begin` says a group starts there, so whatever was open ended --
    /// and it is *not* treated as running on to the next system, because a group
    /// that only the middle of the run leaves open is a contradiction in the
    /// document rather than a break.
    #[test]
    fn a_second_begin_closes_the_group_that_was_open() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::Start),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(2, WHOLE), (2, WHOLE)]);
    }

    /// Formerly `panic!("Cannot continue a beam group when none is open")` -- the
    /// panic `crazy-beams.musicxml` used to hit at the first barline it crossed.
    #[test]
    fn a_continue_with_nothing_open_opens_a_group() {
        let mut chords = [beamed(BeamType::Continue), beamed(BeamType::End)];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(2, OPENS_BEFORE)]);
    }

    /// Formerly `panic!("Cannot end a beam group when none is open")`.
    #[test]
    fn an_end_with_nothing_open_opens_and_closes_a_group() {
        let mut chords = [beamed(BeamType::End), beamed(BeamType::Start)];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(
            group_shapes(&groups),
            vec![(1, OPENS_BEFORE), (1, CLOSES_AFTER)],
            "an `end` first and a `begin` last are each half of a group the run \
             does not hold all of"
        );
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

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(3, WHOLE), (2, WHOLE)]);
    }

    // ---------------------------------------------------------- reaching beyond

    // A part is one system's worth of one instrument, so a beam group crossing a
    // system break is two groups -- one in each part -- and neither sees the whole
    // of it. Neither has to: the group's `begin` missing from the front of a run,
    // or its `end` missing from the back, is the break seen from one side.

    /// A run whose first group opens with no `begin` of its own opened on the
    /// previous system, so its levels reach back past the first stem.
    #[test]
    fn a_run_opening_without_a_begin_reaches_back_to_the_previous_system() {
        let mut chords = [
            beamed(BeamType::Continue),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(3, OPENS_BEFORE)]);
    }

    /// A run whose last group never closes closes on the next system, so its
    /// levels reach on past the last stem.
    #[test]
    fn a_run_left_open_reaches_on_to_the_next_system() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::Continue),
            beamed(BeamType::Continue),
        ];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(3, CLOSES_AFTER)]);
    }

    /// Both at once: a group long enough to take up a whole system in the middle
    /// of it, which arrives from one break and leaves through the next. This is
    /// also the shape a lone chord stranded between two breaks takes.
    #[test]
    fn a_run_that_neither_opens_nor_closes_reaches_both_ways() {
        let mut chords = [beamed(BeamType::Continue), beamed(BeamType::Continue)];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(2, PASSES_THROUGH)]);
    }

    /// A cross-page group is not a case of its own: a page break is a system
    /// break, so each side is an ordinary part whose run is missing one end.
    #[test]
    fn a_single_chord_carrying_only_an_end_is_a_whole_run_on_its_own() {
        let mut chords = [beamed(BeamType::End)];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(1, OPENS_BEFORE)]);
    }

    /// Only the *edges* of a run can be a break. A `continue` with nothing open
    /// part-way through means the document contradicts itself -- there is a group
    /// before it that closed properly -- so it recovers into a group of its own and
    /// earns no stub.
    #[test]
    fn a_contradiction_in_the_middle_of_a_run_earns_no_stub() {
        let mut chords = [
            beamed(BeamType::Start),
            beamed(BeamType::End),
            beamed(BeamType::Continue),
            beamed(BeamType::End),
        ];

        let groups = create_beam_groups(run(&mut chords));

        assert_eq!(group_shapes(&groups), vec![(2, WHOLE), (2, WHOLE)]);
    }

    // ------------------------------------------------------------- level spans

    #[test]
    fn a_level_that_ends_in_the_group_is_closed_at_that_stem() {
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
    /// also what a level running off the edge of a part looks like -- only the
    /// caller, which knows whether the group was cut, can tell the two apart.
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

        assert_eq!(infer_direction(&run(&mut chords)), Some(UpDown::Down));
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

        assert_eq!(infer_direction(&run(&mut chords)), Some(UpDown::Up));
    }

    #[test]
    fn a_group_of_nothing_but_stemless_chords_has_no_direction() {
        let mut chords = [stemless(), stemless()];

        assert_eq!(infer_direction(&run(&mut chords)), None);
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
    /// Deliberately relative rather than a table of tenths. What the pass claims
    /// is that a beam reaches *past its outermost stem* when its group carries on
    /// past the system, and *through a barline* when the group spans one -- both of
    /// which stay true however the fixture is spaced.
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
                let mut spans = Vec::new();

                for section in system.sections.values() {
                    for group in section.part_groups.values() {
                        for part in group.parts.values() {
                            spans.extend(part.beams.iter().map(|beam| {
                                (
                                    beam.pts.iter().map(|p| p.x).fold(f32::MAX, f32::min),
                                    beam.pts.iter().map(|p| p.x).fold(f32::MIN, f32::max),
                                )
                            }));

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
                    spans,
                });
            }
        }

        out
    }

    /// A beam group left open at the end of a system runs a stub out past its last
    /// stem to the break, and the group arriving on the next system runs one back
    /// in past its first stem.
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

                let beams: Vec<_> = system
                    .sections
                    .values()
                    .flat_map(|section| section.part_groups.values())
                    .flat_map(|group| group.parts.values())
                    .flat_map(|part| part.beams.iter())
                    .collect();

                assert!(
                    !beams.is_empty(),
                    "every system of the fixture carries beams"
                );

                for beam in beams {
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
}
