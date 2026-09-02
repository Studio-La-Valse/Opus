#[cfg(test)]
mod tests {
    use std::fs::{read, read_to_string};

    use lib::drawable::canvas::CanvasPainter;
    use lib::drawable::canvas::pdf::{EmbeddedFont, FontSet, PdfPage, PdfPageCanvas, write_pdf};
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::circle::Circle;
    use lib::drawable::elements::line::Line;
    use lib::drawable::elements::polygon::Polygon;
    use lib::drawable::elements::rect::Rect;
    use lib::drawable::elements::text::{
        FontSpec, FontStyle, FontWeight, HorizontalAlign, Text, VerticalAlign,
    };
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;
    use lib::score::visual::render_compositor::RenderCompositor;
    use lib::score::visual::render_fonts::RenderFonts;
    use lib::score::visual::score::Score;
    use lib::smufl::smufl_font::SmuflFont;
    use ttf_parser::Face;

    /// A single-font [`FontSet`] over the bundled Bravura, borrowing `bytes`.
    fn bravura_font_set(bytes: &[u8]) -> FontSet<'_> {
        FontSet::single(EmbeddedFont {
            family: "Bravura".to_string(),
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            base_font: "Bravura".to_string(),
            face: Face::parse(bytes, 0).expect("Bravura.otf parses"),
            program: bytes,
        })
    }

    const BRAVURA_OTF: &str = "assets/smufl/bravura-bravura-1.392/redist/otf/Bravura.otf";
    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    /// `gClef`, the treble clef -- a glyph Bravura is guaranteed to carry.
    const G_CLEF: char = '\u{E050}';

    fn fixture_bytes(relative: &str) -> Vec<u8> {
        read(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn fixture_string(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn smufl_font() -> SmuflFont {
        SmuflFont::load(&fixture_string(BRAVURA_META), &fixture_string(GLYPH_NAMES))
    }

    /// Renders one page's worth of elements through [`PdfPageCanvas`] with a
    /// 0.45 pt/tenth scale and a page pinned at the origin, and hands back the
    /// content stream as a lossy string for operator assertions.
    fn paint_page(font_bytes: &[u8], elements: &[DrawableElement<'_>]) -> (PdfPage, String) {
        let fonts = bravura_font_set(font_bytes);
        let canvas = PdfPageCanvas::new(XY::ZERO, (100.0, 200.0), 0.45, &fonts);
        let page = CanvasPainter::new(canvas).paint(elements);
        let content = String::from_utf8_lossy(&page.content).into_owned();
        (page, content)
    }

    #[test]
    fn begin_emits_the_tenths_to_points_flip_matrix_first() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let (page, content) = paint_page(&font_bytes, &[]);

        // 100 x 200 tenths at 0.45 pt/tenth -> a 45 x 90 pt media box.
        assert_eq!(page.media_box, [0.0, 0.0, 45.0, 90.0]);
        // CTM: scale 0.45, y-flip (negative d), pivot at the page height.
        assert!(
            content.contains("0.45 0 0 -0.45 0 90 cm"),
            "content did not start with the flip matrix: {content:?}"
        );
    }

    #[test]
    fn line_becomes_a_stroked_path_in_raw_score_coordinates() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let elements: Vec<DrawableElement<'_>> = vec![
            Line {
                start: XY { x: 10.0, y: 20.0 },
                end: XY { x: 30.0, y: 20.0 },
                stroke_color: Color::RED,
                stroke_width: 2.0,
            }
            .into(),
        ];

        let (_, content) = paint_page(&font_bytes, &elements);

        assert!(content.contains("1 0 0 RG"), "stroke color: {content:?}");
        assert!(content.contains("2 w"), "line width: {content:?}");
        assert!(content.contains("10 20 m"), "move_to: {content:?}");
        assert!(content.contains("30 20 l"), "line_to: {content:?}");
        assert!(content.trim_end().ends_with('S'), "stroke op: {content:?}");
    }

    #[test]
    fn rect_without_stroke_is_filled_only() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let elements: Vec<DrawableElement<'_>> = vec![
            Rect {
                xy: XY { x: 1.0, y: 2.0 },
                width: 10.0,
                height: 20.0,
                color: Color::WHITE,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ];

        let (_, content) = paint_page(&font_bytes, &elements);

        assert!(content.contains("1 1 1 rg"), "fill color: {content:?}");
        assert!(content.contains("1 2 10 20 re"), "rect op: {content:?}");
        assert!(content.trim_end().ends_with('f'), "fill op: {content:?}");
        assert!(!content.contains(" B"), "should not stroke: {content:?}");
    }

    #[test]
    fn rect_with_stroke_is_filled_and_stroked() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let elements: Vec<DrawableElement<'_>> = vec![
            Rect {
                xy: XY { x: 0.0, y: 0.0 },
                width: 5.0,
                height: 5.0,
                color: Color::WHITE,
                stroke_color: Some(Color::BLACK),
                stroke_width: Some(1.0),
            }
            .into(),
        ];

        let (_, content) = paint_page(&font_bytes, &elements);

        assert!(content.contains("0 0 5 5 re"), "rect op: {content:?}");
        assert!(
            content.trim_end().ends_with('B'),
            "fill+stroke op: {content:?}"
        );
    }

    #[test]
    fn polygon_becomes_a_closed_filled_path() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let elements: Vec<DrawableElement<'_>> = vec![
            Polygon {
                pts: vec![
                    XY { x: 0.0, y: 0.0 },
                    XY { x: 4.0, y: 0.0 },
                    XY { x: 2.0, y: 3.0 },
                ],
                color: Color::GREEN,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ];

        let (_, content) = paint_page(&font_bytes, &elements);

        assert!(content.contains("0 0 m"), "first point: {content:?}");
        assert!(content.contains("4 0 l"), "second point: {content:?}");
        assert!(content.contains("2 3 l"), "third point: {content:?}");
        assert!(content.lines().any(|l| l == "h"), "close_path: {content:?}");
        assert!(content.trim_end().ends_with('f'), "fill op: {content:?}");
    }

    #[test]
    fn circle_becomes_a_closed_bezier_path() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let elements: Vec<DrawableElement<'_>> = vec![
            Circle {
                xy: XY { x: 10.0, y: 10.0 },
                radius: 4.0,
                color: Color::GREEN,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ];

        let (_, content) = paint_page(&font_bytes, &elements);

        assert!(
            content.contains("14 10 m"),
            "start at right edge: {content:?}"
        );
        assert!(
            content.lines().any(|l| l.ends_with(" c")),
            "cubic segments: {content:?}"
        );
        assert!(content.lines().any(|l| l == "h"), "close_path: {content:?}");
        assert!(content.trim_end().ends_with('f'), "fill op: {content:?}");
    }

    #[test]
    fn text_emits_a_text_object_with_the_counter_flip_matrix_and_records_the_glyph() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let face = Face::parse(&font_bytes, 0).unwrap();
        let expected_gid = face.glyph_index(G_CLEF).expect("Bravura has gClef").0;

        let elements: Vec<DrawableElement<'_>> = vec![
            Text {
                text: "\u{E050}",
                color: Color::BLACK,
                font_size: 40.0,
                font: FontSpec::plain("Bravura"),
                xy: XY { x: 12.0, y: 34.0 },
                vertical_alignment: VerticalAlign::Bottom,
                horizontal_alignment: HorizontalAlign::Left,
            }
            .into(),
        ];

        let (page, content) = paint_page(&font_bytes, &elements);

        assert!(content.contains("BT"), "begin_text: {content:?}");
        assert!(content.contains("/F0 40 Tf"), "set_font: {content:?}");
        assert!(
            content.contains("1 0 0 -1 12 34 Tm"),
            "text matrix (left/baseline aligned): {content:?}"
        );
        assert!(content.contains("Tj"), "show: {content:?}");
        assert!(content.contains("ET"), "end_text: {content:?}");
        assert!(
            page.used_glyphs[&0].contains(&expected_gid),
            "expected glyph {expected_gid} in {:?}",
            page.used_glyphs
        );
    }

    #[test]
    fn horizontal_center_alignment_shifts_the_text_origin_left_by_half_the_advance() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let face = Face::parse(&font_bytes, 0).unwrap();
        let gid = face.glyph_index(G_CLEF).unwrap();
        let advance = face.glyph_hor_advance(gid).unwrap() as f32;
        let font_size = 40.0_f32;
        let expected_x = 100.0 - (advance * font_size / face.units_per_em() as f32) / 2.0;

        let elements: Vec<DrawableElement<'_>> = vec![
            Text {
                text: "\u{E050}",
                color: Color::BLACK,
                font_size,
                font: FontSpec::plain("Bravura"),
                xy: XY { x: 100.0, y: 50.0 },
                vertical_alignment: VerticalAlign::Bottom,
                horizontal_alignment: HorizontalAlign::Center,
            }
            .into(),
        ];

        let (_, content) = paint_page(&font_bytes, &elements);
        let needle = format!("1 0 0 -1 {expected_x} 50 Tm");
        assert!(
            content.contains(&needle),
            "expected centered matrix {needle:?} in {content:?}"
        );
    }

    #[test]
    fn walk_pages_keeps_one_bundle_per_page() {
        let font = smufl_font();

        let mut score = Score::default();
        {
            let page = score.page_or_insert(1);
            page.xy = XY { x: 0.0, y: 0.0 };
            page.width = 1360.0;
            page.height = 1760.0;
        }
        {
            let page = score.page_or_insert(2);
            page.xy = XY { x: 1400.0, y: 0.0 };
            page.width = 1360.0;
            page.height = 1760.0;
        }

        let compositor = RenderCompositor::base();

        let fonts = RenderFonts::music_only(&font);
        let pages = compositor.walk_pages(&score, &fonts);
        assert_eq!(pages.len(), 2);
        assert_eq!((pages[0].number, pages[1].number), (1, 2));
        assert_eq!((pages[0].origin.x, pages[0].origin.y), (0.0, 0.0));
        assert_eq!((pages[1].origin.x, pages[1].origin.y), (1400.0, 0.0));
        assert_eq!(pages[0].width, 1360.0);
        // BaseRenderer::render_page emits exactly the page background rect.
        assert_eq!(pages[0].elements.len(), 1);
        assert_eq!(pages[1].elements.len(), 1);
    }

    #[test]
    fn write_pdf_produces_a_loadable_multi_page_document() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let fonts = bravura_font_set(&font_bytes);

        let make_page = |origin: XY| {
            let elements: Vec<DrawableElement<'_>> = vec![
                Rect {
                    xy: origin,
                    width: 100.0,
                    height: 200.0,
                    color: Color::WHITE,
                    stroke_color: Some(Color::BLACK),
                    stroke_width: Some(1.0),
                }
                .into(),
                Text {
                    text: "\u{E050}",
                    color: Color::BLACK,
                    font_size: 40.0,
                    font: FontSpec::plain("Bravura"),
                    xy: XY {
                        x: origin.x + 10.0,
                        y: origin.y + 40.0,
                    },
                    vertical_alignment: VerticalAlign::Bottom,
                    horizontal_alignment: HorizontalAlign::Left,
                }
                .into(),
            ];
            let canvas = PdfPageCanvas::new(origin, (100.0, 200.0), 0.45, &fonts);
            CanvasPainter::new(canvas).paint(&elements)
        };

        let pages = vec![
            make_page(XY { x: 0.0, y: 0.0 }),
            make_page(XY { x: 120.0, y: 0.0 }),
        ];

        let bytes = write_pdf(&pages, &fonts);
        let text = String::from_utf8_lossy(&bytes);

        assert!(bytes.starts_with(b"%PDF-"), "PDF header missing");
        assert!(
            text.contains("startxref"),
            "cross-reference pointer missing"
        );
        assert!(text.trim_end().ends_with("%%EOF"), "PDF trailer missing");

        // Two page objects in a tree that knows its own count.
        assert_eq!(
            count(&text, "/Type /Page\n"),
            2,
            "expected two page objects"
        );
        assert!(text.contains("/Count 2"), "page tree count");
        // Both pages carry the 45 x 90 pt media box (100 x 200 tenths * 0.45).
        assert_eq!(count(&text, "/MediaBox [0 0 45 90]"), 2, "media boxes");

        // The music font is embedded once as an OpenType CID font the pages
        // reach through Identity-H.
        assert!(text.contains("/Identity-H"), "encoding");
        assert!(text.contains("/CIDFontType0"), "descendant font subtype");
        assert!(text.contains("/OpenType"), "font file subtype");
        assert_eq!(count(&text, "/FontFile3"), 1, "font embedded exactly once");
    }

    /// Non-overlapping occurrences of `needle` in `haystack`.
    fn count(haystack: &str, needle: &str) -> usize {
        haystack.matches(needle).count()
    }

    #[test]
    fn write_pdf_of_no_pages_is_still_a_valid_pdf() {
        let font_bytes = fixture_bytes(BRAVURA_OTF);
        let fonts = bravura_font_set(&font_bytes);
        let bytes = write_pdf(&[], &fonts);
        let text = String::from_utf8_lossy(&bytes);

        assert!(bytes.starts_with(b"%PDF-"));
        assert!(text.trim_end().ends_with("%%EOF"));
        assert!(text.contains("/Count 0"), "empty page tree");
        assert!(!text.contains("/Type /Page\n"), "no page objects");
        // Nothing drew text, so no font is embedded.
        assert!(!text.contains("/FontFile3"), "no font when no text");
    }
}
