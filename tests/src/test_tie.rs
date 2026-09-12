#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::engrave::engrave;
    use lib::score::score_defaults::ScoreDefaults;
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::layoutable::LayoutParams;
    use lib::score::visual::note::NoteId;
    use lib::score::visual::stem::UpDown;
    use lib::score::visual::system::SystemKey;
    use lib::score::visual::tie::{Tie, TieMetrics, TieSide, tie_arc};
    use lib::score::visual::tie_arranger::{
        NoteAnchor, SystemExtent, collect_note_anchors, split_tie,
    };
    use lib::smufl::smufl_font::SmuflFont;

    const ACTOR_PRELUDE: &str = "assets/xmlsamples/ActorPreludeSample.musicxml";
    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    fn fixture(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&fixture(BRAVURA_META), &fixture(GLYPH_NAMES)))
    }

    fn metrics() -> TieMetrics {
        let score_defaults = ScoreDefaults::default();
        let user_layout = UserLayout::default();
        let app_defaults = AppDefaults::default();

        TieMetrics::resolve(LayoutParams {
            score_defaults: &score_defaults,
            user_layout: &user_layout,
            app_defaults: &app_defaults,
            font: font(),
            abbreviate_names: false,
        })
    }

    /// A notehead-sized anchor on `key`, at `x` / `y`, with no stem. Its measure
    /// is assumed to run well past it; use [`in_measure_ending_at`] when the
    /// measure's right edge is what's under test.
    fn anchor(key: SystemKey, x: f32, y: f32) -> NoteAnchor {
        NoteAnchor {
            key,
            left: XY { x, y },
            width: 11.8,
            measure_right: x + 100.,
            scale: 1.0,
            staff_line: 4,
            stem: None,
            color: Color::BLACK,
        }
    }

    fn in_measure_ending_at(mut a: NoteAnchor, measure_right: f32) -> NoteAnchor {
        a.measure_right = measure_right;
        a
    }

    fn tie(start: NoteId, end: NoteId, side: Option<TieSide>) -> Tie {
        Tie { start, end, side }
    }

    fn extent(left: f32, right: f32) -> SystemExtent {
        SystemExtent { left, right }
    }

    /// The `(min_x, max_x)` a shape's points span.
    fn x_span(pts: &[XY]) -> (f32, f32) {
        pts.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
            (lo.min(p.x), hi.max(p.x))
        })
    }

    /// The outer/inner gap of a `tie_arc` polygon at sample `i`. The polygon is
    /// the outer edge forward followed by the inner edge reversed, so sample `i`
    /// of the outer edge pairs with `len - 1 - i`.
    fn thickness_at(pts: &[XY], i: usize) -> f32 {
        let outer = pts[i];
        let inner = pts[pts.len() - 1 - i];
        (outer - inner).length()
    }

    // ---------------------------------------------------------------- geometry

    #[test]
    fn tie_arc_is_a_closed_ring_of_two_sampled_edges() {
        let m = metrics();
        let shape = tie_arc(
            XY { x: 0., y: 0. },
            XY { x: 60., y: 0. },
            TieSide::Over,
            1.0,
            &m,
            Color::BLACK,
        );

        // 17 samples per edge (TIE_SAMPLES = 16, inclusive of both ends), two edges.
        assert_eq!(shape.pts.len(), 34);
        // A tie is a filled shape, never stroked.
        assert!(shape.stroke_width.is_none());
        assert!(shape.stroke_color.is_none());
    }

    #[test]
    fn an_over_tie_bulges_up_and_an_under_tie_bulges_down() {
        let m = metrics();
        let (a, b) = (XY { x: 0., y: 100. }, XY { x: 60., y: 100. });

        let over = tie_arc(a, b, TieSide::Over, 1.0, &m, Color::BLACK);
        let under = tie_arc(a, b, TieSide::Under, 1.0, &m, Color::BLACK);

        // y grows downward, so "up" is a smaller y.
        let over_min = over.pts.iter().fold(f32::MAX, |acc, p| acc.min(p.y));
        let under_max = under.pts.iter().fold(f32::MIN, |acc, p| acc.max(p.y));

        assert!(over_min < 100., "over tie should rise above its endpoints");
        assert!(
            under_max > 100.,
            "under tie should fall below its endpoints"
        );
    }

    #[test]
    fn arc_height_is_clamped_at_both_extremes() {
        let m = metrics();

        // 10 tenths * 0.15 = 1.5, below the 5.0 floor.
        assert_eq!(m.height(10.), m.height_min);
        // 400 tenths * 0.15 = 60.0, above the 16.0 ceiling.
        assert_eq!(m.height(400.), m.height_max);
        // 60 tenths * 0.15 = 9.0, inside the range.
        assert_eq!(m.height(60.), 9.);
    }

    #[test]
    fn the_arc_tapers_from_endpoint_to_midpoint_thickness() {
        let m = metrics();
        let shape = tie_arc(
            XY { x: 0., y: 0. },
            XY { x: 80., y: 0. },
            TieSide::Over,
            1.0,
            &m,
            Color::BLACK,
        );

        let at_start = thickness_at(&shape.pts, 0);
        let at_middle = thickness_at(&shape.pts, 8);
        let at_end = thickness_at(&shape.pts, 16);

        assert!((at_start - m.endpoint_thickness).abs() < 0.01);
        assert!((at_end - m.endpoint_thickness).abs() < 0.01);
        assert!((at_middle - m.midpoint_thickness).abs() < 0.01);
    }

    /// A short arc -- the length each half of a broken tie gets -- is still a
    /// complete tie shape: tapered at both ends, thickest in the middle. This is
    /// what makes a split tie read as two full segments rather than one arc
    /// sliced at its apex.
    #[test]
    fn a_short_fragment_is_still_a_full_tapered_arc() {
        let m = metrics();
        let shape = tie_arc(
            XY { x: 0., y: 0. },
            XY {
                x: m.break_fragment,
                y: 0.,
            },
            TieSide::Over,
            1.0,
            &m,
            Color::BLACK,
        );

        assert!((thickness_at(&shape.pts, 0) - m.endpoint_thickness).abs() < 0.01);
        assert!((thickness_at(&shape.pts, 16) - m.endpoint_thickness).abs() < 0.01);
        assert!((thickness_at(&shape.pts, 8) - m.midpoint_thickness).abs() < 0.01);

        // Both ends sit on the baseline; only the middle lifts off it.
        assert!((shape.pts[0].y).abs() < m.endpoint_thickness);
        assert!((shape.pts[16].y).abs() < m.endpoint_thickness);
        assert!(shape.pts.iter().fold(f32::MAX, |a, p| a.min(p.y)) < -1.);
    }

    #[test]
    fn thickness_scales_with_the_note() {
        let m = metrics();
        let grace = tie_arc(
            XY { x: 0., y: 0. },
            XY { x: 80., y: 0. },
            TieSide::Over,
            0.5,
            &m,
            Color::BLACK,
        );

        assert!((thickness_at(&grace.pts, 8) - m.midpoint_thickness / 2.).abs() < 0.01);
    }

    #[test]
    fn side_is_inferred_away_from_the_stem_then_from_the_middle_line() {
        assert_eq!(TieSide::infer(Some(UpDown::Up), 4), TieSide::Under);
        assert_eq!(TieSide::infer(Some(UpDown::Down), 4), TieSide::Over);
        // Stemless: staff lines run 0 (top) to 9 (bottom), 4 is the middle line.
        assert_eq!(TieSide::infer(None, 2), TieSide::Over);
        assert_eq!(TieSide::infer(None, 7), TieSide::Under);
    }

    #[test]
    fn side_parses_orientation_then_placement() {
        assert_eq!(TieSide::parse(Some("over"), None), Some(TieSide::Over));
        assert_eq!(TieSide::parse(Some("under"), None), Some(TieSide::Under));
        assert_eq!(TieSide::parse(None, Some("above")), Some(TieSide::Over));
        assert_eq!(TieSide::parse(None, Some("below")), Some(TieSide::Under));
        // Orientation wins over a contradictory placement.
        assert_eq!(
            TieSide::parse(Some("over"), Some("below")),
            Some(TieSide::Over)
        );
        assert_eq!(TieSide::parse(None, None), None);
    }

    // ------------------------------------------------------------------- split

    #[test]
    fn a_tie_inside_one_system_is_a_single_segment() {
        let m = metrics();
        let key = (1, 1);

        let anchors = HashMap::from([
            (NoteId::from(0), anchor(key, 100., 200.)),
            (NoteId::from(1), anchor(key, 300., 200.)),
        ]);
        let extents = HashMap::from([(key, extent(50., 500.))]);

        let segments = split_tie(
            &tie(NoteId::from(0), NoteId::from(1), None),
            &anchors,
            &extents,
            &m,
        );

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].0, key);
    }

    #[test]
    fn a_tie_across_a_system_break_becomes_two_fragments() {
        let m = metrics();
        let (first, second) = ((1, 1), (1, 2));

        // The start note sits early in a measure that runs to x = 480.
        let start = in_measure_ending_at(anchor(first, 400., 200.), 480.);
        let end = anchor(second, 80., 600.);
        let anchors = HashMap::from([(NoteId::from(0), start), (NoteId::from(1), end)]);
        let extents = HashMap::from([(first, extent(50., 500.)), (second, extent(50., 500.))]);

        let segments = split_tie(
            &tie(NoteId::from(0), NoteId::from(1), Some(TieSide::Over)),
            &anchors,
            &extents,
            &m,
        );

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, first);
        assert_eq!(segments[1].0, second);

        // Each fragment is a complete tie shape, not half of one: tapered to
        // `endpoint_thickness` at *both* of its own ends, and at full
        // `midpoint_thickness` in between. This is the property that
        // distinguishes the engraved convention from slicing one arc at its apex.
        for (_, segment) in &segments {
            let pts = &segment.shape.pts;
            assert!((thickness_at(pts, 0) - m.endpoint_thickness).abs() < 0.01);
            assert!((thickness_at(pts, 16) - m.endpoint_thickness).abs() < 0.01);
            assert!((thickness_at(pts, 8) - m.midpoint_thickness).abs() < 0.01);
        }

        // The opening fragment takes the space available to it -- out to the end
        // of its own measure, less the barline margin. The closing one is a
        // short fixed stub. They are deliberately different lengths.
        let (opening, closing) = (&segments[0].1.shape, &segments[1].1.shape);

        let (open_lo, open_hi) = x_span(&opening.pts);
        assert!(open_lo >= 400., "opening fragment starts at its note");
        assert!(
            (open_hi - (480. - m.break_inset)).abs() < 0.5,
            "opening fragment should reach the end of its measure, got {open_hi}"
        );

        let (close_lo, close_hi) = x_span(&closing.pts);
        assert!(close_lo >= 50., "closing fragment stays inside its system");
        assert!(close_hi <= 80. + 0.5, "closing fragment ends at its note");
        assert!(
            (open_hi - open_lo) > (close_hi - close_lo),
            "the opening fragment should be the longer of the two"
        );
    }

    /// Runs `split_tie` for a start note at `note_x` in a measure ending at
    /// `measure_right`, on a system spanning `50 .. system_right`, and returns
    /// the opening fragment's x-span.
    fn opening_fragment(
        m: &TieMetrics,
        note_x: f32,
        measure_right: f32,
        system_right: f32,
    ) -> (f32, f32) {
        let (first, second) = ((1, 1), (1, 2));
        let anchors = HashMap::from([
            (
                NoteId::from(0),
                in_measure_ending_at(anchor(first, note_x, 200.), measure_right),
            ),
            (NoteId::from(1), anchor(second, 80., 200.)),
        ]);
        let extents = HashMap::from([
            (first, extent(50., system_right)),
            (second, extent(50., 500.)),
        ]);

        let segments = split_tie(
            &tie(NoteId::from(0), NoteId::from(1), None),
            &anchors,
            &extents,
            m,
        );
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, first);
        x_span(&segments[0].1.shape.pts)
    }

    /// A note falling right at the end of a short measure still gets a visible
    /// arc rather than a degenerate sliver, even though that means nudging just
    /// past the barline.
    #[test]
    fn a_cramped_opening_fragment_falls_back_to_the_stub_length() {
        let m = metrics();
        // Measure ends at 300, but the system runs on to 500, so there is room.
        let (lo, hi) = opening_fragment(&m, 270., 300., 500.);

        assert!(
            hi - lo >= m.break_fragment - 0.5,
            "opening fragment collapsed to {} tenths",
            hi - lo
        );
    }

    /// ...but the system's own right edge is a hard limit that the fallback
    /// above must never push the fragment past.
    #[test]
    fn an_opening_fragment_never_leaves_its_system() {
        let m = metrics();
        // Note crowded against both the barline and the end of the system.
        let (_, hi) = opening_fragment(&m, 470., 490., 500.);

        assert!(
            hi <= 500. + 0.5,
            "fragment ran past the system edge to {hi}"
        );
    }

    /// A tie across a *page* break is not a separate case: the two systems
    /// simply live under different page keys, and each fragment files itself
    /// under its own page.
    #[test]
    fn a_tie_across_a_page_break_also_becomes_two_fragments() {
        let m = metrics();
        let (last_of_p1, first_of_p2) = ((1, 4), (2, 5));

        let anchors = HashMap::from([
            (NoteId::from(0), anchor(last_of_p1, 400., 200.)),
            (NoteId::from(1), anchor(first_of_p2, 80., 200.)),
        ]);
        let extents = HashMap::from([
            (last_of_p1, extent(50., 500.)),
            (first_of_p2, extent(50., 500.)),
        ]);

        let segments = split_tie(
            &tie(NoteId::from(0), NoteId::from(1), None),
            &anchors,
            &extents,
            &m,
        );

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].0, last_of_p1);
        assert_eq!(segments[1].0, first_of_p2);
    }

    #[test]
    fn undrawable_ties_produce_no_segments() {
        let m = metrics();
        let key = (1, 1);
        let extents = HashMap::from([(key, extent(50., 500.)), ((1, 2), extent(50., 500.))]);

        // An endpoint whose note was never built (no `default-x`).
        let only_start = HashMap::from([(NoteId::from(0), anchor(key, 100., 200.))]);
        assert!(
            split_tie(
                &tie(NoteId::from(0), NoteId::from(1), None),
                &only_start,
                &extents,
                &m
            )
            .is_empty()
        );

        // An end that sorts before its start.
        let backwards = HashMap::from([
            (NoteId::from(0), anchor((1, 2), 100., 200.)),
            (NoteId::from(1), anchor(key, 100., 200.)),
        ]);
        assert!(
            split_tie(
                &tie(NoteId::from(0), NoteId::from(1), None),
                &backwards,
                &extents,
                &m
            )
            .is_empty()
        );

        // Same system, but the end is not to the right of the start.
        let overlapping = HashMap::from([
            (NoteId::from(0), anchor(key, 300., 200.)),
            (NoteId::from(1), anchor(key, 100., 200.)),
        ]);
        assert!(
            split_tie(
                &tie(NoteId::from(0), NoteId::from(1), None),
                &overlapping,
                &extents,
                &m
            )
            .is_empty()
        );
    }

    // -------------------------------------------------------------- end to end

    fn engrave_actor_prelude() -> lib::score::engrave::EngravedScore {
        let musicxml = fixture(ACTOR_PRELUDE);
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

    #[test]
    fn engraving_a_real_score_matches_ties_and_resolves_both_endpoints() {
        let engraved = engrave_actor_prelude();
        let anchors = collect_note_anchors(&engraved.score);

        assert!(
            !engraved.score.ties.is_empty(),
            "ActorPreludeSample has 186 <tied> elements; none were matched"
        );

        for tie in &engraved.score.ties {
            assert_ne!(tie.start, tie.end, "a tie must join two distinct notes");
            assert!(
                anchors.contains_key(&tie.start),
                "tie start {:?} has no note in the tree",
                tie.start
            );
            assert!(
                anchors.contains_key(&tie.end),
                "tie end {:?} has no note in the tree",
                tie.end
            );
        }
    }

    #[test]
    fn every_tie_contributes_one_segment_per_system_it_touches() {
        let engraved = engrave_actor_prelude();
        let anchors = collect_note_anchors(&engraved.score);

        let mut expected = 0;
        let mut broken = 0;
        for tie in &engraved.score.ties {
            let (start, end) = (&anchors[&tie.start], &anchors[&tie.end]);
            if start.key == end.key {
                expected += 1;
            } else {
                expected += 2;
                broken += 1;
            }
        }

        let drawn: usize = engraved
            .score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .map(|system| system.ties.len())
            .sum();

        assert_eq!(drawn, expected);
        assert!(
            broken > 0,
            "a 4-page score with 186 <tied> elements should break at least one tie \
             across a system; the split path went untested"
        );
    }

    /// The wasm render path re-runs `arrange_score` on a cached `Score` for every
    /// frame, so the tie pass must assign rather than accumulate.
    #[test]
    fn re_arranging_does_not_accumulate_tie_segments() {
        let mut engraved = engrave_actor_prelude();

        let count = |score: &lib::score::visual::score::Score| -> usize {
            score
                .pages
                .values()
                .flat_map(|page| page.systems.values())
                .map(|system| system.ties.len())
                .sum()
        };

        let first = count(&engraved.score);
        assert!(first > 0);

        lib::score::engrave::arrange_score(
            &mut engraved.score,
            &engraved.layout,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_| {},
        );

        assert_eq!(count(&engraved.score), first);
    }
}
