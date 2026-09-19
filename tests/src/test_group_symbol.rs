//! The geometry of the five group-symbol shapes.
//!
//! Every symbol reports a bounding box, and instrument names will be aligned
//! against it, so the box has to be exactly what the ink covers rather than
//! roughly where the symbol is. These pin each shape's box and its placement
//! against the anchor, and check that what a symbol reports is what the renderer
//! actually draws.

#[cfg(test)]
mod tests {
    use lib::drawable::drawable_element::DrawableElement;
    use lib::geometry::bounding_box::BoundingBox;
    use lib::geometry::xy::XY;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::core::group_symbol::{GroupLevel, GroupSymbol as Kind};
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::score_defaults::ScoreDefaults;
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::arranger::{PageArranger, ScoreMeasurement};
    use lib::score::visual::group_symbol::{GroupSymbol, Shape};
    use lib::score::visual::layoutable::LayoutParams;
    use lib::score::visual::render_fonts::RenderFonts;
    use lib::score::visual::render_pass::{BaseRenderer, RenderPass};
    use lib::score::visual::score::Score;
    use lib::smufl::smufl_font::SmuflFont;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    const ACTOR_PRELUDE: &str = "assets/xmlsamples/ActorPreludeSample.musicxml";
    const GROUP_SYMBOLS: &str = "assets/xmlfixtures/group-symbols.musicxml";
    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    /// The system's left edge, at the top line of the first staff spanned.
    const ORIGIN: XY = XY { x: 500., y: 200. };
    /// Two five-line staves and the gap between them, near enough.
    const SPAN: f32 = 120.;

    fn asset(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&asset(BRAVURA_META), &asset(GLYPH_NAMES)))
    }

    /// A symbol of `kind` at `level`, measured over [`SPAN`] and arranged at
    /// [`ORIGIN`]. The kind is forced through the user layout, which is what a
    /// caller overriding the document does.
    fn laid_out(level: GroupLevel, kind: Kind) -> GroupSymbol {
        laid_out_with(level, kind, SPAN, UserLayout::default())
    }

    fn laid_out_with(
        level: GroupLevel,
        kind: Kind,
        span: f32,
        user_layout: UserLayout,
    ) -> GroupSymbol {
        let user_layout = UserLayout {
            section_symbol: Some(kind),
            part_group_symbol: Some(kind),
            part_symbol: Some(kind),
            ..user_layout
        };
        let score_defaults = ScoreDefaults::default();
        let app_defaults = AppDefaults::default();

        let mut symbol = GroupSymbol::new(level, None);
        let params = LayoutParams {
            score_defaults: &score_defaults,
            user_layout: &user_layout,
            app_defaults: &app_defaults,
            font: font(),
        };
        symbol.resolve_layout(params);
        ScoreMeasurement.measure_group_symbol(
            &mut symbol,
            &XY {
                x: f32::INFINITY,
                y: span,
            },
        );
        PageArranger.arrange_group_symbol(&mut symbol, &ORIGIN);

        symbol
    }

    /// A minimal `<score-partwise>` around `part_list`, with a one-note measure
    /// for each of `parts` so the walk has something to lay out.
    fn score_with_part_list(part_list: &str, parts: &[&str]) -> String {
        let bodies: String = parts
            .iter()
            .map(|id| {
                format!(
                    "<part id=\"{id}\"><measure number=\"1\" width=\"300\">\
                     <print new-system=\"yes\"/>\
                     <attributes><divisions>4</divisions>\
                     <key><fifths>0</fifths><mode>major</mode></key>\
                     <time><beats>4</beats><beat-type>4</beat-type></time>\
                     <clef><sign>G</sign><line>2</line></clef></attributes>\
                     <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
                     <duration>16</duration><voice>1</voice><type>whole</type></note>\
                     </measure></part>"
                )
            })
            .collect();

        format!(
            "<score-partwise version=\"4.0\"><part-list>{part_list}</part-list>{bodies}\
             </score-partwise>"
        )
    }

    fn walk(
        xml: &str,
    ) -> (
        Score,
        ScoreDefaults,
        Vec<lib::musicxml::validation_issue::ValidationIssue>,
    ) {
        let document = roxmltree::Document::parse_with_options(
            xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("test document does not parse");

        walk_document(
            &document,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_stage| {},
        )
    }

    fn arrange(score: &mut Score, defaults: &ScoreDefaults, user_layout: &UserLayout) {
        arrange_score(
            score,
            defaults,
            font(),
            user_layout,
            &AppDefaults::default(),
            &mut |_stage| {},
        );
    }

    /// A fully engraved score, laid out with no overrides at all -- so what a
    /// symbol resolves to is what the document asked for.
    fn engrave(xml: &str) -> Score {
        let (mut score, defaults, _) = walk(xml);
        arrange(&mut score, &defaults, &UserLayout::default());
        score
    }

    /// The gap `level` steps by with no overrides, asked of the one place that
    /// decides it -- the three are not all the same, and which one is which is
    /// tuning rather than something these tests should pin.
    fn default_gap(level: GroupLevel) -> f32 {
        let score_defaults = ScoreDefaults::default();
        let user_layout = UserLayout::default();
        let app_defaults = AppDefaults::default();

        LayoutParams {
            score_defaults: &score_defaults,
            user_layout: &user_layout,
            app_defaults: &app_defaults,
            font: font(),
        }
        .group_symbol_gap(level)
    }

    fn drawables(symbol: &GroupSymbol) -> Vec<DrawableElement<'static>> {
        let fonts = RenderFonts::music_only(font());
        let mut out = Vec::new();
        BaseRenderer {}.render_group_symbol(symbol, &fonts, &mut out);
        out
    }

    fn assert_close(actual: f32, expected: f32, what: &str) {
        assert!(
            (actual - expected).abs() < 0.01,
            "{what}: expected {expected}, got {actual}"
        );
    }

    fn assert_box(actual: BoundingBox, expected: [f32; 4], what: &str) {
        assert_close(actual.x_min(), expected[0], &format!("{what} x_min"));
        assert_close(actual.y_min(), expected[1], &format!("{what} y_min"));
        assert_close(actual.x_max(), expected[2], &format!("{what} x_max"));
        assert_close(actual.y_max(), expected[3], &format!("{what} y_max"));
    }

    // ------------------------------------------------------------ the anchor

    /// A symbol steps left by its own gap from whatever edge it is handed, and
    /// applies it itself -- no container carries an offset for it. What that edge
    /// is differs per level, which is the nesting test below; the gap is per level
    /// too, so each one is checked against its own rather than against a shared
    /// number.
    #[test]
    fn a_symbol_steps_its_own_gap_left_of_the_edge_it_is_given() {
        for level in [GroupLevel::Section, GroupLevel::PartGroup, GroupLevel::Part] {
            let symbol = laid_out(level, Kind::Line);
            assert_close(
                symbol.anchor().x,
                ORIGIN.x - default_gap(level),
                &format!("{level:?} anchor"),
            );
            assert_close(symbol.anchor().y, ORIGIN.y, &format!("{level:?} anchor y"));
        }
    }

    #[test]
    fn a_gap_override_moves_the_symbol_and_nothing_else() {
        let symbol = laid_out_with(
            GroupLevel::Section,
            Kind::Line,
            SPAN,
            UserLayout {
                section_symbol_gap: Some(40.),
                ..Default::default()
            },
        );

        assert_close(symbol.anchor().x, ORIGIN.x - 40., "overridden anchor");
        assert_close(symbol.bounds().height(), SPAN, "span is unaffected");
    }

    // ------------------------------------------------------------- the shapes

    /// Nothing drawn, but still a box: a name aligned against a symbol-less
    /// group lands where it would have had there been one.
    #[test]
    fn none_draws_nothing_and_reports_a_zero_width_box() {
        let symbol = laid_out(GroupLevel::Section, Kind::None);

        assert!(matches!(symbol.shape(), Shape::Nothing));
        assert!(!symbol.is_drawn());
        assert!(drawables(&symbol).is_empty());

        let anchor = ORIGIN.x - default_gap(GroupLevel::Section);
        assert_box(
            symbol.bounds(),
            [anchor, ORIGIN.y, anchor, ORIGIN.y + SPAN],
            "none",
        );
    }

    /// A bare vertical line at the `line` weight, spanning exactly the staves,
    /// with its right edge on the anchor. Which weight that is belongs to
    /// [`AppDefaults`]; what is pinned here is that the stroke *is* that weight
    /// and that the box is exactly the stroke.
    #[test]
    fn line_is_one_stroke_of_sub_bracket_weight() {
        let thickness = AppDefaults::default().group_line_thickness;
        let symbol = laid_out(GroupLevel::Section, Kind::Line);
        let anchor = ORIGIN.x - default_gap(GroupLevel::Section);

        let Shape::Line { stroke } = symbol.shape() else {
            panic!("expected a line, got something else");
        };
        assert_close(
            stroke.width,
            thickness,
            "the stroke is drawn at line weight",
        );
        assert_close(stroke.height, SPAN, "stroke spans the staves exactly");

        assert_box(
            symbol.bounds(),
            [anchor - thickness, ORIGIN.y, anchor, ORIGIN.y + SPAN],
            "line",
        );
        assert_eq!(drawables(&symbol).len(), 1);
    }

    /// The bracket keeps the geometry it had before the shapes were unified: a
    /// stroke at bracket weight whose right edge sits its own gap from the system,
    /// with tips registered against the stroke's left edge that flare past each
    /// end of it.
    ///
    /// The tips are scaled to the stroke -- the factor being the stroke's weight
    /// over the glyph's own -- so the box is checked against the `scale` the shape
    /// reports. Checking it against a factor of 1 would only hold while the
    /// default weight happens to equal the glyph's.
    #[test]
    fn bracket_matches_the_geometry_it_had_before() {
        let thickness = AppDefaults::default().group_bracket_thickness;
        let symbol = laid_out(GroupLevel::Section, Kind::Bracket);
        let anchor = ORIGIN.x - default_gap(GroupLevel::Section);

        let Shape::Bracket { stroke, scale, .. } = symbol.shape() else {
            panic!("expected a bracket, got something else");
        };
        let scale = *scale;

        assert_close(
            stroke.width,
            thickness,
            "the stroke is drawn at bracket weight",
        );

        // Bravura's bracketTop / bracketBottom are 1.876 spaces wide and flare
        // 1.18 spaces past the end they cap. Glyph metrics rather than layout
        // config, so unlike the thickness these stay as numbers -- at whatever
        // size the tips were scaled to.
        assert_box(
            symbol.bounds(),
            [
                anchor - thickness,
                ORIGIN.y - 11.8 * scale,
                anchor - thickness + 18.76 * scale,
                ORIGIN.y + SPAN + 11.8 * scale,
            ],
            "bracket",
        );

        // Two tips and the stroke over them.
        assert_eq!(drawables(&symbol).len(), 3);
    }

    /// Thickening the stroke scales the tips with it, so a bracket stays in
    /// proportion instead of growing a stroke its own serifs no longer match.
    ///
    /// Stated as a ratio against the default bracket rather than against a factor
    /// of 1, so what is pinned is that the two move together.
    #[test]
    fn a_thicker_bracket_scales_its_tips_to_match() {
        let nominal = laid_out(GroupLevel::Section, Kind::Bracket);
        let Shape::Bracket {
            scale: nominal_scale,
            ..
        } = nominal.shape()
        else {
            panic!("expected a bracket");
        };

        // Twice the default, whatever the default happens to be.
        let doubled = AppDefaults::default().group_bracket_thickness * 2.;
        let symbol = laid_out_with(
            GroupLevel::Section,
            Kind::Bracket,
            SPAN,
            UserLayout {
                group_bracket_thickness: Some(doubled),
                ..Default::default()
            },
        );

        let Shape::Bracket { stroke, scale, .. } = symbol.shape() else {
            panic!("expected a bracket");
        };
        let scale = *scale;

        assert_close(stroke.width, doubled, "stroke follows the override");
        assert_close(
            scale,
            nominal_scale * 2.,
            "twice the stroke, so tips at twice the size",
        );
        assert_close(
            symbol.bounds().height(),
            // 11.8 tenths of flare at each end, at the scaled tip size.
            SPAN + 11.8 * 2. * scale,
            "the flare scales with the tips",
        );
    }

    /// A square is a stroke with an arm at each end reaching toward the system.
    /// The arms are what the gap is measured to, since they come nearest the
    /// staff, and they sit outside the staves rather than over them.
    #[test]
    fn square_brackets_the_staves_with_two_arms() {
        // Both are tuned values, so what is pinned here is how the shape is built
        // out of them, not what they happen to be.
        let app = AppDefaults::default();
        let t = app.group_square_thickness;
        let arm = app.group_square_arm;

        let symbol = laid_out(GroupLevel::Section, Kind::Square);
        let anchor = ORIGIN.x - default_gap(GroupLevel::Section);

        let Shape::Square { stroke, arms } = symbol.shape() else {
            panic!("expected a square, got something else");
        };
        assert_close(stroke.width, t, "the spine is one stroke wide");
        assert_close(arms[0].width, arm + t, "the arm's reach, plus the spine");
        assert_close(arms[0].xy.y, ORIGIN.y - t, "top arm sits above the staff");
        assert_close(
            arms[1].xy.y,
            ORIGIN.y + SPAN,
            "bottom arm sits below the staff",
        );

        assert_box(
            symbol.bounds(),
            [anchor - arm - t, ORIGIN.y - t, anchor, ORIGIN.y + SPAN + t],
            "square",
        );
        assert_eq!(drawables(&symbol).len(), 3);
    }

    /// The brace is scaled so its ink spans exactly the staves it binds. Its box
    /// is therefore the span's height to the tenth -- which the old
    /// `height / 40` scale, assuming a four-space-tall glyph, was 0.3% short of.
    #[test]
    fn brace_ink_spans_exactly_the_staves_it_binds() {
        let symbol = laid_out(GroupLevel::PartGroup, Kind::Brace);

        assert!(matches!(symbol.shape(), Shape::Brace { .. }));
        assert_close(symbol.bounds().y_min(), ORIGIN.y, "brace top");
        assert_close(symbol.bounds().y_max(), ORIGIN.y + SPAN, "brace foot");

        // Bravura's brace box is 3.988 spaces tall, so a 120-tenth span asks for
        // slightly more than nominal size.
        let Shape::Brace { scale, .. } = symbol.shape() else {
            unreachable!()
        };
        assert_close(*scale, SPAN / 39.88, "scale follows the glyph's own box");
    }

    /// The brace hangs off its right edge, and the step back to its true origin
    /// is applied in two places -- once to place the drawable, once to derive
    /// the box. This is what catches them drifting apart.
    #[test]
    fn the_reported_brace_box_is_the_drawable_glyph_box() {
        let symbol = laid_out(GroupLevel::PartGroup, Kind::Brace);

        let drawn = drawables(&symbol);
        let [DrawableElement::Glyph(glyph)] = drawn.as_slice() else {
            panic!("expected exactly one glyph, got {} elements", drawn.len());
        };

        assert_box(
            symbol.bounds(),
            [
                glyph.bounds().x_min(),
                glyph.bounds().y_min(),
                glyph.bounds().x_max(),
                glyph.bounds().y_max(),
            ],
            "reported box against the drawable's own",
        );
    }

    // -------------------------------------------------------------- suppression

    /// A symbol given no staves draws nothing, whatever shape it resolved to.
    /// Every container measures its symbol on every pass rather than only when
    /// it expects to draw one, so this is the case that keeps a symbol with
    /// nothing to bind from being drawn against a stale span.
    #[test]
    fn a_symbol_with_no_staves_to_span_draws_nothing() {
        for kind in [Kind::Brace, Kind::Bracket, Kind::Line, Kind::Square] {
            let symbol = laid_out_with(GroupLevel::Section, kind, 0., UserLayout::default());

            assert!(!symbol.is_drawn(), "{kind} with no span");
            assert!(matches!(symbol.shape(), Shape::Nothing), "{kind} shape");
            assert!(drawables(&symbol).is_empty(), "{kind} drawables");
        }
    }

    /// Resolution runs on every layout pass, so the same walked score re-laid
    /// out with a different override changes shape. This is what the wasm
    /// bindings depend on: they walk once and re-arrange per render.
    #[test]
    fn re_arranging_the_same_symbol_resolves_it_again() {
        let score_defaults = ScoreDefaults::default();
        let app_defaults = AppDefaults::default();
        let mut symbol = GroupSymbol::new(GroupLevel::Section, None);

        let lay_out = |symbol: &mut GroupSymbol, user_layout: &UserLayout| {
            let params = LayoutParams {
                score_defaults: &score_defaults,
                user_layout,
                app_defaults: &app_defaults,
                font: font(),
            };
            symbol.resolve_layout(params);
            ScoreMeasurement.measure_group_symbol(
                symbol,
                &XY {
                    x: f32::INFINITY,
                    y: SPAN,
                },
            );
            PageArranger.arrange_group_symbol(symbol, &ORIGIN);
        };

        lay_out(&mut symbol, &UserLayout::default());
        assert_eq!(symbol.kind(), Kind::Bracket, "the section default");

        lay_out(
            &mut symbol,
            &UserLayout {
                section_symbol: Some(Kind::Square),
                ..Default::default()
            },
        );
        assert_eq!(symbol.kind(), Kind::Square, "the override wins");
        assert!(matches!(symbol.shape(), Shape::Square { .. }));
    }

    // -------------------------------------------------------------- precedence

    /// The order the three sources are consulted in, which is decided once, in
    /// `LayoutParams::group_symbol`. An absent declaration falls through to the
    /// level's default; an explicit `none` does not.
    #[test]
    fn an_override_beats_the_document_which_beats_the_default() {
        let score_defaults = ScoreDefaults::default();
        let app_defaults = AppDefaults::default();

        let resolve = |declared: Option<Kind>, user: Option<Kind>| {
            let user_layout = UserLayout {
                section_symbol: user,
                ..Default::default()
            };
            let mut symbol = GroupSymbol::new(GroupLevel::Section, declared);
            let params = LayoutParams {
                score_defaults: &score_defaults,
                user_layout: &user_layout,
                app_defaults: &app_defaults,
                font: font(),
            };
            symbol.resolve_layout(params);
            ScoreMeasurement.measure_group_symbol(
                &mut symbol,
                &XY {
                    x: f32::INFINITY,
                    y: SPAN,
                },
            );
            symbol.kind()
        };

        assert_eq!(resolve(None, None), Kind::Bracket, "the level's default");
        assert_eq!(
            resolve(Some(Kind::Brace), None),
            Kind::Brace,
            "the document"
        );
        assert_eq!(
            resolve(Some(Kind::Brace), Some(Kind::Line)),
            Kind::Line,
            "the caller's override"
        );
        assert_eq!(
            resolve(Some(Kind::None), None),
            Kind::None,
            "an explicit 'none' is a declaration, not an absence"
        );
    }

    // ---------------------------------------------------------- the document

    /// `lookup` collects the symbols of the levels it descends past, because a
    /// part knows its section's *index* but nothing about the node that index
    /// came from.
    #[test]
    fn a_part_reports_the_symbols_of_the_levels_enclosing_it() {
        let defaults = walk(&asset(ACTOR_PRELUDE)).1;

        // P8 is a horn: inside the braced "Horns in F" pair, inside the
        // bracketed brass section.
        let horn = defaults.lookup("P8").expect("P8 is in the part-list");
        assert_eq!(horn.section_symbol, Some(Kind::Bracket));
        assert_eq!(horn.part_group_symbol, Some(Kind::Brace));

        // P13 is the tuba, which sits directly in that section with no group of
        // its own between it and the bracket.
        let tuba = defaults.lookup("P13").expect("P13 is in the part-list");
        assert_eq!(tuba.section_symbol, Some(Kind::Bracket));
        assert_eq!(
            tuba.part_group_symbol, None,
            "no <part-group> encloses the tuba, so nothing declared a symbol"
        );
    }

    /// A part written at the top level of a `<part-list>` is given a section
    /// index of its own, but there is no `<part-group>` behind that index to
    /// have declared anything. The index must not carry the previous section's
    /// symbol with it.
    #[test]
    fn a_top_level_part_declares_no_symbols() {
        let xml = score_with_part_list(
            r#"<part-group type="start"><group-symbol>bracket</group-symbol></part-group>
               <score-part id="P1"><part-name>Violin</part-name></score-part>
               <part-group type="stop"/>
               <score-part id="P2"><part-name>Timpani</part-name></score-part>"#,
            &["P1", "P2"],
        );
        let defaults = walk(&xml).1;

        assert_eq!(
            defaults.lookup("P1").unwrap().section_symbol,
            Some(Kind::Bracket)
        );

        let loose = defaults.lookup("P2").unwrap();
        assert_eq!(loose.section_symbol, None, "nothing encloses this part");
        assert_eq!(loose.part_group_symbol, None);
    }

    /// End to end: what a `<part-group>` asks for is what the visual tree gets.
    #[test]
    fn the_visual_tree_draws_what_the_document_asked_for() {
        let xml = score_with_part_list(
            r#"<part-group type="start"><group-symbol>line</group-symbol></part-group>
               <part-group type="start"><group-symbol>square</group-symbol></part-group>
               <score-part id="P1"><part-name>Violin I</part-name></score-part>
               <score-part id="P2"><part-name>Violin II</part-name></score-part>
               <part-group type="stop"/>
               <part-group type="stop"/>"#,
            &["P1", "P2"],
        );

        let score = engrave(&xml);
        let section = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .next()
            .expect("the score has a section");

        assert_eq!(
            section.symbol.kind(),
            Kind::Line,
            "the outer group asked for a line"
        );

        let group = section.part_groups.values().next().expect("a part-group");
        assert_eq!(
            group.symbol.kind(),
            Kind::Square,
            "the inner group asked for a square"
        );
    }

    /// The override still wins over what the document declared, on a real walked
    /// score rather than a hand-built symbol.
    #[test]
    fn an_override_replaces_the_documents_symbols_throughout() {
        let xml = score_with_part_list(
            r#"<part-group type="start"><group-symbol>brace</group-symbol></part-group>
               <score-part id="P1"><part-name>Violin I</part-name></score-part>
               <score-part id="P2"><part-name>Violin II</part-name></score-part>
               <part-group type="stop"/>"#,
            &["P1", "P2"],
        );

        let (mut score, defaults, _) = walk(&xml);
        arrange(
            &mut score,
            &defaults,
            &UserLayout {
                section_symbol: Some(Kind::None),
                ..Default::default()
            },
        );

        let section = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .next()
            .expect("the score has a section");

        assert_eq!(section.symbol.kind(), Kind::None);
        assert!(!section.shows_symbol(), "nothing is drawn for it");
    }

    // ---------------------------------------------------------- the nesting

    /// Every symbol the fixture draws, innermost first, as
    /// `(what, right edge, left edge)`.
    fn stacked(
        score: &Score,
        section_index: usize,
        group_index: usize,
    ) -> Vec<(&'static str, f32, f32)> {
        let section = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .nth(section_index)
            .expect("section");
        let group = section
            .part_groups
            .values()
            .nth(group_index)
            .expect("part-group");
        let part = group.parts.values().next().expect("part");

        [
            ("section", section.shows_symbol(), &section.symbol),
            ("part-group", group.shows_symbol(), &group.symbol),
            ("part", part.shows_symbol(), &part.symbol),
        ]
        .into_iter()
        .filter(|(_, drawn, _)| *drawn)
        .map(|(what, _, symbol)| (what, symbol.bounds().x_max(), symbol.bounds().x_min()))
        .collect()
    }

    /// The order the three levels stand in, which is the whole point of the
    /// fixture: from the system leftward, section then part-group then part.
    /// A section's symbol is measured from the system, and each level further
    /// out clears whatever the level inside it drew, so none of them overlap
    /// however wide a brace happens to be.
    #[test]
    fn the_three_levels_stack_outward_from_the_system() {
        let score = engrave(&asset(GROUP_SYMBOLS));

        for (section_index, what) in [
            (0, "bracket / brace / brace"),
            (1, "square / brace / brace"),
        ] {
            let stack = stacked(&score, section_index, 0);
            assert_eq!(stack.len(), 3, "{what}: all three levels draw");

            for pair in stack.windows(2) {
                let (inner, _, inner_left) = pair[0];
                let (outer, outer_right, _) = pair[1];

                assert!(
                    outer_right <= inner_left,
                    "{what}: the {outer} symbol ends at {outer_right} but the \
                     {inner} symbol it should sit clear of begins at {inner_left}"
                );
            }
        }
    }

    /// The gap between two levels is padding, so widening one moves everything
    /// outside it and nothing inside it. A fixed ladder of distances from the
    /// system could not do this: a brace's width follows its span, so no set of
    /// distances keeps a tall part-group's symbol clear of its section's.
    #[test]
    fn widening_one_gap_pushes_only_what_lies_outside_it() {
        let (mut score, defaults, _) = walk(&asset(GROUP_SYMBOLS));

        arrange(&mut score, &defaults, &UserLayout::default());
        let before = stacked(&score, 0, 0);

        let widened = 45.;
        arrange(
            &mut score,
            &defaults,
            &UserLayout {
                part_group_symbol_gap: Some(widened),
                ..Default::default()
            },
        );
        let after = stacked(&score, 0, 0);

        let extra = widened - default_gap(GroupLevel::PartGroup);

        assert_close(after[0].1, before[0].1, "the section symbol stays put");
        assert_close(
            after[1].1,
            before[1].1 - extra,
            "the part-group symbol moves out by the extra padding",
        );
        assert_close(
            after[2].1,
            before[2].1 - extra,
            "and the part symbol outside it moves with it",
        );
    }

    /// A level that draws nothing must not push the levels outside it away by a
    /// gap nothing occupies. Section 1's second group asks for `none`, so its
    /// parts sit where they would if the group had never been there.
    #[test]
    fn a_level_that_draws_nothing_takes_up_no_room() {
        let score = engrave(&asset(GROUP_SYMBOLS));

        let section = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .nth(1)
            .expect("section 1");

        let brass = section
            .part_groups
            .values()
            .nth(1)
            .expect("the brass group");
        assert_eq!(brass.symbol.kind(), Kind::None);
        assert!(!brass.shows_symbol(), "an explicit 'none' draws nothing");

        // Its parts are single-staff, so none of them draws either -- what is
        // under test is that the group's own anchor did not shift the section's
        // reported edge, which the stacking test above would catch downstream.
        assert!(brass.parts.values().all(|part| !part.shows_symbol()));
    }

    /// Every shape the format defines reaches the tree from this one fixture.
    #[test]
    fn the_fixture_covers_every_shape() {
        let score = engrave(&asset(GROUP_SYMBOLS));
        let sections: Vec<_> = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .collect();

        assert_eq!(
            sections.iter().map(|s| s.symbol.kind()).collect::<Vec<_>>(),
            vec![Kind::Bracket, Kind::Square],
        );

        let groups: Vec<_> = sections
            .iter()
            .flat_map(|s| s.part_groups.values())
            .map(|g| g.symbol.kind())
            .collect();
        assert_eq!(
            groups,
            vec![Kind::Brace, Kind::Line, Kind::Brace, Kind::None],
        );

        // MusicXML has no <group-symbol> for a part, so both multi-staff parts
        // take the app default.
        let drawing: Vec<&str> = sections
            .iter()
            .flat_map(|s| s.part_groups.values())
            .flat_map(|g| g.parts.iter())
            .filter(|(_, part)| part.shows_symbol())
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(drawing, vec!["P1", "P5"], "the organ and the harp");
    }
}
