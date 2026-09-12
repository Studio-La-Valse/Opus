#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    use lib::drawable::elements::polygon::Polygon;
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::engrave::engrave;
    use lib::score::score_defaults::ScoreDefaults;
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::layoutable::LayoutParams;
    use lib::score::visual::note::Note;
    use lib::score::visual::part::Part;
    use lib::score::visual::score::Score;
    use lib::score::visual::stem::UpDown;
    use lib::score::visual::tie::{Tie, TieAnchor, TieMetrics, TieSide, tie_arc};
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

    /// A notehead-sized anchor at `x` / `y`, on the middle line of the first
    /// staff.
    fn anchor(x: f32, y: f32) -> TieAnchor {
        TieAnchor {
            left: XY { x, y },
            width: 11.8,
            scale: 1.0,
            color: Color::BLACK,
            staff: 1.into(),
            staff_line: 4,
        }
    }

    /// The arc a tie resolved to, which every case here expects to exist.
    fn shape(tie: &Tie) -> &Polygon {
        tie.shape
            .as_ref()
            .expect("the tie should have drawn an arc")
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

    /// Spans are chosen from the metrics rather than written down, so the test
    /// follows the ratio and the two limits wherever they are tuned to.
    #[test]
    fn arc_height_is_clamped_at_both_extremes() {
        let m = metrics();

        // Half the span the floor starts at, so the ratio lands well under it.
        let short = m.height_min / m.height_ratio / 2.;
        assert_eq!(m.height(short), m.height_min);

        // Twice the span the ceiling starts at.
        let long = m.height_max / m.height_ratio * 2.;
        assert_eq!(m.height(long), m.height_max);

        // And between the two, the ratio applies as written.
        let middle = (m.height_min + m.height_max) / 2.;
        let dx = middle / m.height_ratio;
        assert!(
            (m.height(dx) - middle).abs() < 0.01,
            "expected {middle}, got {}",
            m.height(dx)
        );
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

    /// A very short arc -- two adjacent sixteenths, say -- is still a complete
    /// tie shape: tapered at both ends, thickest in the middle, and lifted off
    /// the line its ends sit on rather than flattened into one.
    #[test]
    fn a_short_tie_is_still_a_full_tapered_arc() {
        let m = metrics();
        // Short enough that the height floor is what settles the bulge.
        let span = m.height_min / m.height_ratio / 2.;
        let shape = tie_arc(
            XY { x: 0., y: 0. },
            XY { x: span, y: 0. },
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

    // ----------------------------------------------------------------- arrange

    #[test]
    fn a_whole_tie_runs_between_the_two_noteheads_it_joins() {
        let m = metrics();
        let (start, end) = (anchor(100., 200.), anchor(300., 200.));

        let mut tie = Tie::new(Some(TieSide::Over));
        tie.arrange(&start, &end, None, &m);

        let (lo, hi) = x_span(&shape(&tie).pts);
        // Clear of both noteheads by the gap, and no further.
        assert!((lo - (100. + start.width + m.note_gap)).abs() < 0.5);
        assert!((hi - (300. - m.note_gap)).abs() < 0.5);
    }

    /// A tie takes the side the document named, and only falls back on the stem
    /// when it named none.
    #[test]
    fn a_named_side_wins_over_the_one_the_stem_implies() {
        let m = metrics();
        let (start, end) = (anchor(100., 200.), anchor(300., 200.));

        // A stem up would infer `Under`, which bulges to a larger y.
        let mut named = Tie::new(Some(TieSide::Over));
        named.arrange(&start, &end, Some(UpDown::Up), &m);

        let mut inferred = Tie::new(None);
        inferred.arrange(&start, &end, Some(UpDown::Up), &m);

        let apex = |tie: &Tie| shape(tie).pts.iter().fold(f32::MIN, |a, p| a.max(p.y));
        assert!(apex(&named) < 200., "the named side should bulge up");
        assert!(apex(&inferred) > 200., "the stem should imply a down bulge");
    }

    /// The half of a broken tie that *is* drawn: a complete arc, tapered at both
    /// of its own ends rather than sliced at an apex, leaving the note and
    /// running out to the limit it was given.
    #[test]
    fn a_tie_with_no_note_to_reach_runs_out_to_its_limit() {
        let m = metrics();
        let start = anchor(100., 200.);

        let mut tie = Tie::new(Some(TieSide::Over));
        tie.arrange_open(&start, None, 400., &m);

        let pts = &shape(&tie).pts;
        assert!((thickness_at(pts, 0) - m.endpoint_thickness).abs() < 0.01);
        assert!((thickness_at(pts, 16) - m.endpoint_thickness).abs() < 0.01);
        assert!((thickness_at(pts, 8) - m.midpoint_thickness).abs() < 0.01);

        let (lo, hi) = x_span(pts);
        assert!((lo - (100. + start.width + m.note_gap)).abs() < 0.5);
        assert!((hi - 400.).abs() < 0.5, "should reach its limit, got {hi}");

        // Both ends sit level with the note it left; only the middle lifts off.
        assert!((pts[0].y - 200.).abs() < m.vertical_offset + m.endpoint_thickness);
        assert!((pts[16].y - 200.).abs() < m.vertical_offset + m.endpoint_thickness);
    }

    #[test]
    fn a_tie_with_nowhere_to_go_draws_nothing() {
        let m = metrics();

        // An end that is not to the right of its start.
        let mut backwards = Tie::new(Some(TieSide::Over));
        backwards.arrange(&anchor(300., 200.), &anchor(100., 200.), None, &m);
        assert!(backwards.shape.is_none());

        // A note sitting past the limit its tie would run out to, which is what
        // a note crowded right against the end of its part looks like.
        let mut cramped = Tie::new(Some(TieSide::Over));
        cramped.arrange_open(&anchor(400., 200.), None, 390., &m);
        assert!(cramped.shape.is_none());
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

    /// Every note the compositor would draw, in walk order.
    fn notes(score: &Score) -> impl Iterator<Item = &Note> {
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
    }

    /// Every arc the compositor would paint, gathered the same way it gathers
    /// them.
    fn drawn(score: &Score) -> usize {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .map(|part| part.ties().count())
            .sum()
    }

    /// Every part of the score, in walk order.
    fn parts(score: &Score) -> impl Iterator<Item = &Part> {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
    }

    /// Every tie in a real score draws an arc, whether or not it found the note
    /// it was reaching for -- so `drawn` counts exactly the ties in the tree.
    #[test]
    fn engraving_a_real_score_draws_an_arc_for_every_tie() {
        let engraved = engrave_actor_prelude();

        let ties = notes(&engraved.score)
            .filter_map(|note| note.tie.as_ref())
            .count();

        assert!(
            ties > 0,
            "ActorPreludeSample has 186 <tied> elements; none reached the tree"
        );
        assert_eq!(drawn(&engraved.score), ties);
    }

    /// A tie whose end note fell on the next system runs out to the end of its
    /// part instead. The fixture is long enough to contain some, so the path is
    /// exercised rather than merely assumed.
    #[test]
    fn a_tie_with_no_note_left_to_reach_runs_out_to_the_end_of_its_part() {
        let engraved = engrave_actor_prelude();
        let m = metrics();

        let mut ran_out = 0;
        for part in parts(&engraved.score) {
            let edge = part.xy.x + part.width - m.break_inset;

            for shape in part.ties() {
                let (_, hi) = x_span(&shape.pts);
                // Within the half-thickness the tapered outline adds past the
                // arc's own endpoint.
                if (hi - edge).abs() < m.endpoint_thickness {
                    ran_out += 1;
                }
            }
        }

        assert!(
            ran_out > 0,
            "a 4-page score with 186 <tied> elements should carry at least one tie \
             across a system break"
        );
    }

    /// The wasm render path re-runs `arrange_score` on a cached `Score` for every
    /// frame, so the tie pass must assign rather than accumulate.
    #[test]
    fn re_arranging_does_not_accumulate_tie_segments() {
        let mut engraved = engrave_actor_prelude();

        let first = drawn(&engraved.score);
        assert!(first > 0);

        lib::score::engrave::arrange_score(
            &mut engraved.score,
            &engraved.layout,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_| {},
        );

        assert_eq!(drawn(&engraved.score), first);
    }
}
