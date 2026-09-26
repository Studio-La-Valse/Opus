#[cfg(test)]
mod tests {
    use lib::drawable::canvas::CanvasPainter;
    use lib::drawable::canvas::flat_buffer::{FlatBufferCanvas, TAG_GLYPH, TAG_TEXT};
    use lib::drawable::drawable_element::{DrawableElement, Scale, compute_bounds};
    use lib::drawable::elements::glyph::Glyph;
    use lib::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
    use lib::geometry::bounding_box::BoundingBox;
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;
    use lib::score::core::clef::Clef as ClefCore;
    use lib::score::visual::arranger::ScoreMeasurement;
    use lib::score::visual::clef::Clef;
    use lib::score::visual::placed::Placed;
    use lib::score::visual::staff::Staff;
    use lib::smufl::glyphs::brace::BraceStyle;
    use lib::smufl::smufl_font::SmuflFont;
    use lib::smufl::smufl_glyph::{SmuflGlyph, staff_space};
    use std::fs::{read, read_to_string};
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

    /// Origin (10, 20), with a box normalized to 4 world units: 2 x 6 of them
    /// starting 4 above the origin, so 8 x 24 spanning y 4..28 in world space.
    fn a_glyph() -> Glyph<'static> {
        Glyph::new(
            "\u{E050}",
            FontSpec::plain("Bravura"),
            40.0,
            Color::BLACK,
            XY { x: 10.0, y: 20.0 },
            &BoundingBox {
                xy: XY { x: 0.0, y: -4.0 },
                size: XY { x: 2.0, y: 6.0 },
            },
            4.0,
        )
    }

    /// The constructor is the only way in, so the box can only ever be the one
    /// the origin and the normalized metadata put there.
    #[test]
    fn the_constructor_places_the_normalized_box_against_the_origin() {
        let glyph = a_glyph();

        assert_eq!((glyph.origin().x, glyph.origin().y), (10.0, 20.0));
        assert_eq!(
            (glyph.bounds().x_min(), glyph.bounds().y_min()),
            (10.0, 4.0)
        );
        assert_eq!(
            (glyph.bounds().width(), glyph.bounds().height()),
            (8.0, 24.0)
        );
    }

    /// The whole reason a glyph is its own element: it measures as the ink it
    /// puts on the page, where a text run can only measure as its anchor.
    #[test]
    fn a_glyph_measures_as_its_ink_box_and_a_text_as_its_anchor() {
        let glyph: Vec<DrawableElement<'_>> = vec![a_glyph().into()];
        assert_eq!(compute_bounds(&glyph), (10.0, 4.0, 18.0, 28.0));

        let text: Vec<DrawableElement<'_>> = vec![
            Text {
                text: "a",
                color: Color::BLACK,
                font_size: 40.0,
                font: FontSpec::plain("serif"),
                bounds: BoundingBox::point(XY { x: 10.0, y: 20.0 }),
                vertical_alignment: VerticalAlign::Bottom,
                horizontal_alignment: HorizontalAlign::Left,
                background: None,
            }
            .into(),
        ];
        assert_eq!(compute_bounds(&text), (10.0, 20.0, 10.0, 20.0));
    }

    /// Scaling has to carry the box with the origin, or the glyph would report
    /// ink somewhere it is no longer drawn.
    #[test]
    fn scaling_a_glyph_moves_its_box_with_it() {
        let scaled = a_glyph().scale(0.5, XY::ZERO);

        assert_eq!((scaled.origin().x, scaled.origin().y), (5.0, 10.0));
        assert_eq!(scaled.font_size, 20.0);
        assert_eq!((scaled.bounds().xy.x, scaled.bounds().xy.y), (5.0, 2.0));
        assert_eq!(
            (scaled.bounds().width(), scaled.bounds().height()),
            (4.0, 12.0)
        );
    }

    /// The glyph record is shorter than a text record -- no alignment fields --
    /// and both draw their content from the one blob, in the order the canvas
    /// saw them.
    #[test]
    fn the_flat_buffer_glyph_record_carries_no_alignment() {
        let elements: Vec<DrawableElement<'_>> = vec![
            a_glyph().into(),
            Text {
                text: "b",
                color: Color::BLACK,
                font_size: 12.0,
                font: FontSpec::plain("Bravura"),
                bounds: BoundingBox::point(XY { x: 1.0, y: 2.0 }),
                vertical_alignment: VerticalAlign::Top,
                horizontal_alignment: HorizontalAlign::Left,
                background: None,
            }
            .into(),
        ];

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);

        assert_eq!(
            flat.geometry,
            vec![
                TAG_GLYPH, 10.0, 20.0, 40.0, 0.0, 0.0, 0.0, 1.0, 0.0, //
                TAG_TEXT, 1.0, 2.0, 12.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
            ]
        );
        assert_eq!(
            flat.text_blob,
            format!(
                "\u{E050}{}b",
                lib::drawable::canvas::flat_buffer::TEXT_DELIMITER
            )
        );
    }

    /// The box a glyph reports and the box the debug overlay draws have to be
    /// the same box, or the two would disagree about where a clef is. Both now
    /// come out of `BoundingBox::placed`, so this pins that they agree.
    #[test]
    fn a_glyph_box_agrees_with_the_visual_element_that_placed_it() {
        let smufl_clef = font().clef(&ClefCore::Treble, Staff::DEFAULT_LINES);

        let mut clef = Clef::new(smufl_clef.clone());
        clef.xy = XY { x: 40.0, y: 60.0 };
        clef.rescale(Clef::COURTESY_SCALE);
        ScoreMeasurement.measure_clef(&mut clef);

        let expected = clef.scaled_box();
        let actual = smufl_clef
            .as_glyph(font(), Color::BLACK, clef.xy, clef.scale)
            .bounds();

        assert_eq!((actual.xy.x, actual.xy.y), (expected.xy.x, expected.xy.y));
        assert_eq!(
            (actual.width(), actual.height()),
            (expected.width(), expected.height())
        );
        // ... and the element's own reserved width is that same box.
        assert_eq!(clef.width, actual.width());
    }

    /// A glyph is placed on its origin, and the origin is the SMuFL
    /// registration point -- on the baseline, at the left of the advance. The
    /// metadata box hangs off it, so a clef's ink starts above the baseline and
    /// runs below it.
    #[test]
    fn a_glyph_is_placed_on_its_origin_with_the_box_hung_off_it() {
        let smufl_clef = font().clef(&ClefCore::Treble, Staff::DEFAULT_LINES);
        let origin = XY { x: 100.0, y: 200.0 };
        let glyph = smufl_clef.as_glyph(font(), Color::BLACK, origin, 1.0);

        assert_eq!((glyph.origin().x, glyph.origin().y), (100.0, 200.0));

        // A treble clef's ink runs from 4.392 staff spaces above the baseline
        // to 2.632 below it, and starts at the origin's own x.
        let unit = staff_space(1.0);
        assert_eq!(glyph.bounds().x_min(), 100.0);
        assert_eq!(glyph.bounds().y_min(), 200.0 - 4.392 * unit);
        assert_eq!(glyph.bounds().y_max(), 200.0 + 2.632 * unit);
    }

    /// The brace is the one glyph placed by its right edge rather than its
    /// origin, because it hangs to the left of the system it braces.
    #[test]
    fn a_brace_is_placed_by_its_right_edge() {
        let brace = font().brace(BraceStyle::Default);
        let at = XY { x: 500.0, y: 300.0 };
        let glyph = brace.as_glyph(font(), Color::BLACK, at, 2.0);

        let unit = staff_space(2.0);
        assert_eq!(glyph.origin().x, 500.0 - brace.advance * unit);
        assert_eq!(glyph.origin().y, 300.0);
        // Its ink ends just short of where it was hung, by the right side
        // bearing the advance includes and the box does not.
        assert!(glyph.bounds().x_max() < 500.0);
        assert!(glyph.bounds().x_max() > 500.0 - 0.05 * unit);
    }

    /// Each style reaches its own alternate, with that glyph's own box and
    /// advance rather than the plain brace's.
    #[test]
    fn a_brace_style_selects_the_fonts_alternate() {
        let plain = font().brace(BraceStyle::Default);
        assert_eq!(plain.codepoint, '\u{E000}');

        for (style, codepoint) in [
            (BraceStyle::Small, '\u{F400}'),
            (BraceStyle::Large, '\u{F401}'),
            (BraceStyle::Larger, '\u{F402}'),
            (BraceStyle::Flat, '\u{F403}'),
        ] {
            let brace = font().brace(style);
            let name = style.alternate_name().unwrap();
            let expected: BoundingBox = font().meta.glyph_boxes.get(name).unwrap().into();

            assert_eq!(brace.codepoint, codepoint, "{style:?}");
            assert_eq!(
                (brace.bbox.width(), brace.bbox.height()),
                (expected.width(), expected.height()),
                "{style:?}"
            );
            assert_eq!(
                brace.advance,
                font().meta.glyph_advance_widths[name],
                "{style:?}"
            );
        }
    }

    /// A font that lists no brace alternates -- Finale Maestro lists none at
    /// all -- still loads, and draws its plain brace for any style.
    #[test]
    fn a_font_without_the_alternate_draws_the_plain_brace() {
        let mut meta: serde_json::Value = serde_json::from_str(&asset(
            "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json",
        ))
        .unwrap();
        meta.as_object_mut().unwrap().remove("glyphsWithAlternates");
        let font = SmuflFont::load(
            &meta.to_string(),
            &asset("assets/smufl/metadata/glyphnames.json"),
        );

        let brace = font.brace(BraceStyle::Large);
        assert_eq!(brace.codepoint, '\u{E000}');
        assert_eq!(brace.advance, font.brace(BraceStyle::Default).advance);
    }

    #[test]
    fn a_brace_style_parses_its_own_spelling_and_nothing_else() {
        assert_eq!("larger".parse::<BraceStyle>().unwrap(), BraceStyle::Larger);
        assert_eq!(" Flat ".parse::<BraceStyle>().unwrap(), BraceStyle::Flat);
        assert!("braceLarge".parse::<BraceStyle>().is_err());
        assert!(serde_json::from_str::<BraceStyle>("\"huge\"").is_err());
    }

    /// The brace's placement steps back by the advance width the *metadata*
    /// states, but what actually shifts the glyph when a renderer lays it out is
    /// the advance in the *font*. They have to be the same number, so this pins
    /// the metadata against the font file it was generated from -- read in the
    /// em square SMuFL registers against, four staff spaces.
    #[test]
    fn the_metadata_advance_width_matches_the_font_it_describes() {
        use ttf_parser::Face;

        let bytes = read(format!(
            "{}/../assets/smufl/bravura-bravura-1.392/redist/otf/Bravura.otf",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("Bravura.otf");
        let face = Face::parse(&bytes, 0).expect("a parseable face");

        let em = f32::from(face.units_per_em());
        let spaces_per_em = Staff::SPACES as f32;

        for (codepoint, advance) in [
            ('\u{E000}', font().brace(BraceStyle::Default).advance),
            ('\u{F401}', font().brace(BraceStyle::Large).advance),
            ('\u{E080}', font().number(0).digits[0].advance),
        ] {
            let gid = face.glyph_index(codepoint).expect("a glyph in Bravura");
            let from_font =
                f32::from(face.glyph_hor_advance(gid).expect("an advance")) / em * spaces_per_em;

            assert!(
                (from_font - advance).abs() < 1e-4,
                "{codepoint:?}: metadata says {advance}, font says {from_font}"
            );
        }
    }

    /// Every element that places glyph metadata does it through one derivation,
    /// so a point and a box scaled off the same element land consistently.
    #[test]
    fn placed_scales_points_and_boxes_by_the_same_unit() {
        let mut clef = Clef::new(font().clef(&ClefCore::Treble, Staff::DEFAULT_LINES));
        clef.xy = XY { x: 7.0, y: 11.0 };
        clef.rescale(0.5);

        let unit = clef.unit();
        assert_eq!(unit, Staff::DEFAULT_SPACE_SIZE * 0.5);

        let pt = clef.scale_pt(&XY { x: 2.0, y: -3.0 });
        assert_eq!((pt.x, pt.y), (7.0 + 2.0 * unit, 11.0 - 3.0 * unit));

        let bbox = clef.scale_box(&BoundingBox {
            xy: XY { x: 2.0, y: -3.0 },
            size: XY { x: 4.0, y: 6.0 },
        });
        assert_eq!((bbox.xy.x, bbox.xy.y), (pt.x, pt.y));
        assert_eq!((bbox.width(), bbox.height()), (4.0 * unit, 6.0 * unit));
    }
}
