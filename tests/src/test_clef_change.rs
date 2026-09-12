//! Mid-measure clef changes: the rule that places one, and the pass that finds
//! every one an anchor.
//!
//! A clef change is recorded flat on the score and drawn onto a `System`, the
//! way ties and beams are, because what it needs to be placed -- a staff, and a
//! note or rest in a different branch of the tree -- has no single owner. The
//! tests here cover the two halves of that: the placement rule on its own, and
//! the walk that pairs each change with the anchor it was written in front of.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    use lib::score::app_defaults::AppDefaults;
    use lib::score::core::staff_idx::StaffIdx;
    use lib::score::engrave::{EngravedScore, engrave};
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::clef::{Clef, ClefAnchor};
    use lib::score::visual::note::NoteId;
    use lib::score::visual::score::Score;
    use lib::score::visual::staff::Staff;
    use lib::smufl::smufl_font::SmuflFont;

    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    /// The official samples that actually write a clef part-way through a
    /// measure. Most do not: a clef almost always opens a measure, which is a
    /// different element drawn at the previous barline.
    const WITH_CLEF_CHANGES: [&str; 3] = [
        "assets/xmlsamples/BeetAnGeSample.musicxml",
        "assets/xmlsamples/BrahWiMeSample.musicxml",
        "assets/xmlsamples/DebuMandSample.musicxml",
    ];

    fn fixture(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&fixture(BRAVURA_META), &fixture(GLYPH_NAMES)))
    }

    fn engraved(relative: &str) -> EngravedScore {
        let musicxml = fixture(relative);
        let document = roxmltree::Document::parse_with_options(
            &musicxml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("failed to parse fixture");

        engrave(
            &document,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_| {},
        )
    }

    /// Every clef drawn as a mid-measure change, across the whole score.
    fn placed(score: &Score) -> Vec<&Clef> {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.clef_changes.iter())
            .collect()
    }

    /// Where every note and rest in the score ended up, by id -- the anchors a
    /// clef change can name.
    fn anchors(score: &Score) -> HashMap<NoteId, f32> {
        let mut out = HashMap::new();

        for page in score.pages.values() {
            for system in page.systems.values() {
                for section in system.sections.values() {
                    for group in section.part_groups.values() {
                        for part in group.parts.values() {
                            for staff in part.staves.values() {
                                for measure in staff.measures.values() {
                                    for rest in measure.rests.iter() {
                                        out.insert(rest.id, rest.xy.x);
                                    }
                                }
                            }

                            for measure in part.measures.values() {
                                for chord in measure.chords.values().flatten() {
                                    for note in chord.notes.iter() {
                                        out.insert(note.id, note.xy.x);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        out
    }

    // ------------------------------------------------------------ placement

    /// A clef of the given sign on a normal five-line staff, sized at full staff
    /// scale. The scaling is not optional: a freshly built `Clef` has no width
    /// at all until it is scaled, and every placement rule reads the width back.
    fn clef(which: lib::score::core::clef::Clef) -> Clef {
        let mut clef = Clef::new(font().clef(&which, 5));
        clef.rescale(1.0);
        clef
    }

    /// A courtesy clef lands with its right edge exactly the gap left of
    /// whatever it is announcing, and on the line its own clef names.
    #[test]
    fn a_clef_anchored_before_something_grows_leftwards_from_the_gap() {
        let mut clef = clef(lib::score::core::clef::Clef::Bass);
        clef.place(ClefAnchor::GapBefore(400.), 1000., 1.0);

        assert!(clef.width > 0., "a zero-wide clef would prove nothing here");
        assert_eq!(
            clef.xy.x + clef.width + Clef::COURTESY_GAP,
            400.,
            "the clef's right edge should be one gap left of the anchor"
        );

        // The bass clef is fixed to the second line from the top, which is two
        // half-spaces down from the staff's top line.
        let expected = 1000. + clef.clef.line as f32 * (Staff::DEFAULT_SPACE_SIZE / 2.);
        assert_eq!(clef.xy.y, expected, "the clef should sit on its own line");
    }

    /// An opening clef is the other way round: the anchor is its left edge, and
    /// it grows rightwards into the key signature's column.
    #[test]
    fn a_clef_anchored_at_a_column_grows_rightwards_from_it() {
        let mut clef = clef(lib::score::core::clef::Clef::Bass);
        clef.place(ClefAnchor::LeftEdgeAt(400.), 1000., 1.0);

        assert!(clef.width > 0., "a zero-wide clef would prove nothing here");
        assert_eq!(clef.xy.x, 400., "the clef's left edge should be the anchor");

        let expected = 1000. + clef.clef.line as f32 * (Staff::DEFAULT_SPACE_SIZE / 2.);
        assert_eq!(clef.xy.y, expected, "the clef should sit on its own line");
    }

    /// The line a clef sits on is measured in the *staff's* half-spaces, not in
    /// the clef's own -- a courtesy clef is drawn smaller than the staff it
    /// belongs to, and must still land on one of its lines.
    #[test]
    fn the_line_a_clef_sits_on_scales_with_the_staff_not_the_glyph() {
        let mut full = clef(lib::score::core::clef::Clef::Bass);
        full.place(ClefAnchor::LeftEdgeAt(0.), 0., 1.0);

        let mut courtesy = clef(lib::score::core::clef::Clef::Bass);
        courtesy.rescale(Clef::COURTESY_SCALE);
        courtesy.place(ClefAnchor::LeftEdgeAt(0.), 0., 1.0);

        assert!(courtesy.width < full.width, "the courtesy clef is narrower");
        assert_eq!(
            courtesy.xy.y, full.xy.y,
            "but it sits on exactly the same staff line"
        );
    }

    /// A change is drawn at courtesy size, reduced again by whatever its staff
    /// is scaled by -- and the reduction has to be applied before the width is
    /// read back, or the glyph is positioned against a size it isn't drawn at.
    #[test]
    fn a_narrower_clef_still_meets_its_gap_exactly() {
        let mut full = clef(lib::score::core::clef::Clef::Treble);
        full.rescale(Clef::COURTESY_SCALE);
        full.place(ClefAnchor::GapBefore(400.), 0., 1.0);

        let mut half = clef(lib::score::core::clef::Clef::Treble);
        half.rescale(Clef::COURTESY_SCALE * 0.5);
        half.place(ClefAnchor::GapBefore(400.), 0., 0.5);

        assert!(
            half.width < full.width,
            "a clef on a half-scale staff should be drawn narrower"
        );
        assert_eq!(
            half.xy.x + half.width + Clef::COURTESY_GAP,
            400.,
            "the narrower clef should still meet the gap exactly"
        );
    }

    // ------------------------------------------------------------ end to end

    /// The samples that carry mid-measure clefs record them, and every recorded
    /// change finds its anchor and gets drawn. A change that silently fails to
    /// resolve would leave the staff reading in the wrong clef from there on,
    /// which is worse than most things this engine can get wrong.
    #[test]
    fn every_recorded_clef_change_is_placed_on_a_system() {
        for sample in WITH_CLEF_CHANGES {
            let score = engraved(sample).score;

            assert!(
                !score.clef_changes.is_empty(),
                "{sample} writes a clef part-way through a measure; none was recorded"
            );
            assert_eq!(
                placed(&score).len(),
                score.clef_changes.len(),
                "{sample} lost a clef change between the walk and the arrange"
            );
        }
    }

    /// Each drawn clef stands in front of the note or rest its change named.
    ///
    /// Checked as a set rather than pair by pair, because the drawn clefs carry
    /// no back-reference to the change they came from -- which is the point of
    /// filing them on the system.
    #[test]
    fn every_clef_change_is_drawn_in_front_of_the_anchor_it_names() {
        for sample in WITH_CLEF_CHANGES {
            let score = engraved(sample).score;
            let anchors = anchors(&score);

            let mut expected: Vec<f32> = score
                .clef_changes
                .iter()
                .map(|change| {
                    *anchors
                        .get(&change.anchor)
                        .unwrap_or_else(|| panic!("{sample}: change anchored to a missing note"))
                })
                .collect();

            let mut actual: Vec<f32> = placed(&score)
                .iter()
                .map(|clef| clef.xy.x + clef.width + Clef::COURTESY_GAP)
                .collect();

            expected.sort_by(f32::total_cmp);
            actual.sort_by(f32::total_cmp);

            for (expected, actual) in expected.iter().zip(actual.iter()) {
                assert!(
                    (expected - actual).abs() < 0.01,
                    "{sample}: a clef change was drawn at {actual}, not in front of its anchor at {expected}"
                );
            }
        }
    }

    /// A change lands on the staff it names, not on the staff its anchor sits
    /// on: a part's staves share one stream of `<note>` elements, so the two are
    /// not the same question.
    #[test]
    fn every_clef_change_is_drawn_on_the_staff_it_names() {
        for sample in WITH_CLEF_CHANGES {
            let score = engraved(sample).score;
            let drawn: Vec<f32> = placed(&score).iter().map(|clef| clef.xy.y).collect();

            // Every staff of the score, as the band of y it occupies with a
            // half-staff of slack either side -- a clef sits on one of its lines,
            // never beyond the staff it belongs to.
            let bands: Vec<(f32, f32)> = score
                .pages
                .values()
                .flat_map(|page| page.systems.values())
                .flat_map(|system| system.visible_staves())
                .map(|staff| {
                    let slack = staff.height() / 2.;
                    (staff.xy.y - slack, staff.xy.y + staff.height() + slack)
                })
                .collect();

            for y in drawn {
                assert!(
                    bands.iter().any(|(top, bottom)| y >= *top && y <= *bottom),
                    "{sample}: a clef change was drawn at y {y}, which is on no staff"
                );
            }
        }
    }

    /// Re-arranging a cached score has to leave the same clefs behind, the way
    /// re-running the tie and beam passes does -- the wasm render path arranges
    /// one score over and over for whatever layout is asked for.
    #[test]
    fn arranging_twice_leaves_the_same_clef_changes() {
        for sample in WITH_CLEF_CHANGES {
            let engraved = engraved(sample);
            let once: Vec<(f32, f32)> = placed(&engraved.score)
                .iter()
                .map(|clef| (clef.xy.x, clef.xy.y))
                .collect();

            let defaults = engraved.layout;
            let mut score = engraved.score;
            lib::score::engrave::arrange_score(
                &mut score,
                &defaults,
                font(),
                &UserLayout::default(),
                &AppDefaults::default(),
                &mut |_| {},
            );

            let twice: Vec<(f32, f32)> = placed(&score)
                .iter()
                .map(|clef| (clef.xy.x, clef.xy.y))
                .collect();

            assert_eq!(once, twice, "{sample} moved its clef changes on re-arrange");
        }
    }

    /// A staff index the part does not have is a malformed `<clef number=…>`.
    /// It has to drop out rather than take an arranged neighbour's place, and
    /// certainly rather than panic part-way through a render.
    #[test]
    fn a_clef_change_naming_a_staff_the_part_lacks_is_dropped() {
        let sample = WITH_CLEF_CHANGES[0];
        let mut engraved = engraved(sample);

        for change in engraved.score.clef_changes.iter_mut() {
            change.staff = StaffIdx::from(99);
        }

        lib::score::engrave::arrange_score(
            &mut engraved.score,
            &engraved.layout,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_| {},
        );

        assert!(
            placed(&engraved.score).is_empty(),
            "a change naming a staff that does not exist was drawn anyway"
        );
    }
}
