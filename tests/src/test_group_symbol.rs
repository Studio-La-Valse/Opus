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
    use lib::score::visual::group_symbol::{GroupSymbol, Shape};
    use lib::score::visual::layoutable::{LayoutParams, Layoutable};
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
        symbol.measure(
            &XY {
                x: f32::INFINITY,
                y: span,
            },
            LayoutParams {
                score_defaults: &score_defaults,
                user_layout: &user_layout,
                app_defaults: &app_defaults,
                font: font(),
            },
        );
        symbol.arrange(&ORIGIN);

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

    /// The gap is per level, and a symbol applies it itself -- no container
    /// carries an offset for it. The three defaults are what the containers used
    /// to hardcode.
    #[test]
    fn each_level_sets_its_symbol_the_default_distance_from_the_system() {
        let cases = [
            (GroupLevel::Section, 5.),
            (GroupLevel::PartGroup, 15.),
            (GroupLevel::Part, 5.),
        ];

        for (level, gap) in cases {
            let symbol = laid_out(level, Kind::Line);
            assert_close(
                symbol.anchor().x,
                ORIGIN.x - gap,
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

        let anchor = ORIGIN.x - 5.;
        assert_box(
            symbol.bounds(),
            [anchor, ORIGIN.y, anchor, ORIGIN.y + SPAN],
            "none",
        );
    }

    /// A bare vertical line at sub-bracket weight, spanning exactly the staves,
    /// with its right edge on the anchor.
    #[test]
    fn line_is_one_stroke_of_sub_bracket_weight() {
        let symbol = laid_out(GroupLevel::Section, Kind::Line);
        let anchor = ORIGIN.x - 5.;

        let Shape::Line { stroke } = symbol.shape() else {
            panic!("expected a line, got something else");
        };
        assert_close(stroke.width, 1.6, "Bravura subBracketThickness in tenths");
        assert_close(stroke.height, SPAN, "stroke spans the staves exactly");

        assert_box(
            symbol.bounds(),
            [anchor - 1.6, ORIGIN.y, anchor, ORIGIN.y + SPAN],
            "line",
        );
        assert_eq!(drawables(&symbol).len(), 1);
    }

    /// The bracket keeps the geometry it had before the shapes were unified: a
    /// 5-tenth stroke whose right edge is 5 tenths from the system, with tips
    /// that flare rightward past the anchor and 11.8 tenths beyond each end.
    #[test]
    fn bracket_matches_the_geometry_it_had_before() {
        let symbol = laid_out(GroupLevel::Section, Kind::Bracket);
        let anchor = ORIGIN.x - 5.;

        let Shape::Bracket { stroke, scale, .. } = symbol.shape() else {
            panic!("expected a bracket, got something else");
        };
        assert_close(stroke.width, 5., "Bravura bracketThickness in tenths");
        assert_close(*scale, 1., "tips are nominal size at the default thickness");

        // Bravura's bracketTop / bracketBottom are 1.876 spaces wide and flare
        // 1.18 spaces past the end they cap.
        assert_box(
            symbol.bounds(),
            [
                anchor - 5.,
                ORIGIN.y - 11.8,
                anchor - 5. + 18.76,
                ORIGIN.y + SPAN + 11.8,
            ],
            "bracket",
        );

        // Two tips and the stroke over them.
        assert_eq!(drawables(&symbol).len(), 3);
    }

    /// Thickening the stroke scales the tips with it, so a bracket stays in
    /// proportion instead of growing a stroke its own serifs no longer match.
    #[test]
    fn a_thicker_bracket_scales_its_tips_to_match() {
        let symbol = laid_out_with(
            GroupLevel::Section,
            Kind::Bracket,
            SPAN,
            UserLayout {
                group_bracket_thickness: Some(10.),
                ..Default::default()
            },
        );

        let Shape::Bracket { stroke, scale, .. } = symbol.shape() else {
            panic!("expected a bracket");
        };
        assert_close(stroke.width, 10., "stroke follows the override");
        assert_close(*scale, 2., "twice Bravura's own 5-tenth stroke");
        assert_close(
            symbol.bounds().height(),
            SPAN + 11.8 * 2. * 2.,
            "the flare scales with the tips",
        );
    }

    /// A square is a stroke with an arm at each end reaching toward the system.
    /// The arms are what the gap is measured to, since they come nearest the
    /// staff, and they sit outside the staves rather than over them.
    #[test]
    fn square_brackets_the_staves_with_two_arms() {
        let symbol = laid_out(GroupLevel::Section, Kind::Square);
        let anchor = ORIGIN.x - 5.;

        let Shape::Square { stroke, arms } = symbol.shape() else {
            panic!("expected a square, got something else");
        };
        assert_close(stroke.width, 5., "square is drawn at bracket weight");
        assert_close(arms[0].width, 15., "one staff space, plus the stroke");
        assert_close(arms[0].xy.y, ORIGIN.y - 5., "top arm sits above the staff");
        assert_close(
            arms[1].xy.y,
            ORIGIN.y + SPAN,
            "bottom arm sits below the staff",
        );

        assert_box(
            symbol.bounds(),
            [anchor - 15., ORIGIN.y - 5., anchor, ORIGIN.y + SPAN + 5.],
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
            symbol.measure(
                &XY {
                    x: f32::INFINITY,
                    y: SPAN,
                },
                LayoutParams {
                    score_defaults: &score_defaults,
                    user_layout,
                    app_defaults: &app_defaults,
                    font: font(),
                },
            );
            symbol.arrange(&ORIGIN);
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
            symbol.measure(
                &XY {
                    x: f32::INFINITY,
                    y: SPAN,
                },
                LayoutParams {
                    score_defaults: &score_defaults,
                    user_layout: &user_layout,
                    app_defaults: &app_defaults,
                    font: font(),
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

    /// The whole fixture at once: every shape resolves, and the four levels that
    /// are expected to draw nothing draw nothing. See the comment at the top of
    /// `group-symbols.musicxml` for what each level holds.
    #[test]
    fn the_group_symbols_fixture_resolves_every_shape() {
        let score = engrave(&asset(GROUP_SYMBOLS));
        let sections: Vec<_> = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .collect();

        assert_eq!(sections.len(), 4, "three grouped sections and P17's own");

        let kinds: Vec<_> = sections.iter().map(|s| s.symbol.kind()).collect();
        assert_eq!(
            kinds,
            vec![Kind::Bracket, Kind::Line, Kind::Square, Kind::Bracket],
            "the last is P17's, which declared nothing and took the default"
        );

        let drawn: Vec<_> = sections.iter().map(|s| s.shows_symbol()).collect();
        assert_eq!(
            drawn,
            vec![true, true, true, false],
            "P17's section holds one part-group, so it binds nothing"
        );

        // Section 1's first group asks for `none` outright, and its four horns
        // make it the one group large enough to draw that still draws nothing.
        let horns = sections[1].part_groups.values().next().unwrap();
        assert_eq!(horns.parts.len(), 4);
        assert_eq!(horns.symbol.kind(), Kind::None);
        assert!(!horns.shows_symbol());

        // Section 0's last group is the lone bassoon, which the builder gives a
        // part-group of its own -- one part, so nothing to bind.
        let bassoon = sections[0].part_groups.values().last().unwrap();
        assert_eq!(bassoon.parts.len(), 1);
        assert!(!bassoon.shows_symbol());

        // Every shape reaches the tree somewhere.
        let group_kinds: Vec<_> = sections
            .iter()
            .flat_map(|s| s.part_groups.values())
            .map(|g| g.symbol.kind())
            .collect();
        for expected in [Kind::Square, Kind::Brace, Kind::None, Kind::Line] {
            assert!(
                group_kinds.contains(&expected),
                "no part-group resolved to {expected}: {group_kinds:?}"
            );
        }
    }

    /// A multi-staff part draws a symbol of its own, inside whatever its group
    /// draws. MusicXML has no `<group-symbol>` for a part, so all four of the
    /// fixture's multi-staff parts take the app default.
    #[test]
    fn multi_staff_parts_draw_their_own_symbol() {
        let score = engrave(&asset(GROUP_SYMBOLS));

        let parts: Vec<_> = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.iter())
            .collect();

        let drawing: Vec<&str> = parts
            .iter()
            .filter(|(_, part)| part.shows_symbol())
            .map(|(id, _)| id.as_str())
            .collect();

        assert_eq!(
            drawing,
            vec!["P12", "P13", "P14", "P15"],
            "the trombone, organ, celesta and harp are the multi-staff parts"
        );

        for (_, part) in parts.iter().filter(|(_, p)| p.shows_symbol()) {
            assert_eq!(part.symbol.kind(), Kind::Brace);
        }
    }
}
