#[cfg(test)]
mod tests {
    use lib::drawable::canvas::svg::SvgCanvas;
    use lib::drawable::canvas::{Canvas, CanvasPainter};
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::circle::Circle;
    use lib::drawable::elements::glyph::Glyph;
    use lib::drawable::elements::line::Line;
    use lib::drawable::elements::polygon::Polygon;
    use lib::drawable::elements::rect::Rect;
    use lib::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
    use lib::geometry::bounding_box::BoundingBox;
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;
    use lib::score::visual::render_compositor::RenderedPage;

    /// A `Canvas` that just records the sequence of lifecycle calls it receives.
    #[derive(Default)]
    struct RecordingCanvas {
        calls: Vec<String>,
    }

    impl Canvas for RecordingCanvas {
        type Output = Vec<String>;

        fn begin(&mut self, bounds: (f32, f32, f32, f32)) {
            self.calls.push(format!("begin{bounds:?}"));
        }

        fn begin_page(&mut self, width: f32, height: f32) {
            self.calls.push(format!("begin_page({width}, {height})"));
        }

        fn draw_line(&mut self, _line: &Line) {
            self.calls.push("line".to_string());
        }

        fn draw_rect(&mut self, _rect: &Rect) {
            self.calls.push("rect".to_string());
        }

        fn draw_circle(&mut self, _circle: &Circle) {
            self.calls.push("circle".to_string());
        }

        fn draw_text(&mut self, _text: &Text<'_>) {
            self.calls.push("text".to_string());
        }

        fn draw_glyph(&mut self, _glyph: &Glyph<'_>) {
            self.calls.push("glyph".to_string());
        }

        fn draw_polygon(&mut self, _polygon: &Polygon) {
            self.calls.push("polygon".to_string());
        }

        fn finish(mut self) -> Vec<String> {
            self.calls.push("finish".to_string());
            self.calls
        }
    }

    fn one_of_each() -> Vec<DrawableElement<'static>> {
        vec![
            Line {
                start: XY { x: 0.0, y: 0.0 },
                end: XY { x: 10.0, y: 4.0 },
                stroke_color: Color::BLACK,
                stroke_width: 1.0,
            }
            .into(),
            Rect {
                xy: XY { x: 1.0, y: 1.0 },
                width: 8.0,
                height: 2.0,
                color: Color::WHITE,
                stroke_color: Some(Color::BLACK),
                stroke_width: Some(0.5),
            }
            .into(),
            Circle {
                xy: XY { x: 5.0, y: 2.0 },
                radius: 1.5,
                color: Color::WHITE,
                stroke_color: Some(Color::BLACK),
                stroke_width: Some(0.25),
            }
            .into(),
            Text {
                text: "a & b",
                color: Color::BLACK,
                font_size: 12.0,
                font: FontSpec::plain("Bravura"),
                bounds: BoundingBox::point(XY { x: 2.0, y: 3.0 }),
                vertical_alignment: VerticalAlign::Middle,
                horizontal_alignment: HorizontalAlign::Center,
                background: None,
            }
            .into(),
            Glyph::new(
                "\u{E050}",
                FontSpec::plain("Bravura"),
                12.0,
                Color::BLACK,
                XY { x: 2.0, y: 4.0 },
                // Two units up from the origin and two square: inside the other
                // elements' extent, so it does not widen the painter's bounds.
                &BoundingBox {
                    xy: XY { x: 0.0, y: -2.0 },
                    size: XY { x: 2.0, y: 2.0 },
                },
                1.0,
            )
            .into(),
            Polygon {
                pts: vec![
                    XY { x: 0.0, y: 0.0 },
                    XY { x: 4.0, y: 0.0 },
                    XY { x: 2.0, y: 5.0 },
                ],
                color: Color::GREEN,
                stroke_color: None,
                stroke_width: None,
            }
            .into(),
        ]
    }

    #[test]
    fn painter_runs_begin_then_one_call_per_element_then_finish() {
        let calls = CanvasPainter::new(RecordingCanvas::default()).paint(&one_of_each());

        assert_eq!(
            calls,
            vec![
                "begin(0.0, 0.0, 10.0, 5.0)",
                "line",
                "rect",
                "circle",
                "text",
                "glyph",
                "polygon",
                "finish",
            ]
        );
    }

    #[test]
    fn empty_input_still_begins_and_finishes() {
        let calls = CanvasPainter::new(RecordingCanvas::default()).paint(&[]);

        assert_eq!(
            calls.first().map(String::as_str),
            Some("begin(3.4028235e38, 3.4028235e38, -3.4028235e38, -3.4028235e38)")
        );
        assert_eq!(calls.last().map(String::as_str), Some("finish"));
    }

    #[test]
    fn svg_canvas_emits_a_document_for_every_shape() {
        let svg = CanvasPainter::new(SvgCanvas::new((0.0, 0.0, 10.0, 5.0))).paint(&one_of_each());

        assert!(svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 5""#));
        assert!(svg.contains(
            r##"<line x1="0" y1="0" x2="10" y2="4" stroke="#000000FF" stroke-width="1" />"##
        ));
        assert!(svg.contains(
            r##"<rect x="1" y="1" width="8" height="2" fill="#FFFFFFFF" stroke="#000000FF" stroke-width="0.5" />"##
        ));
        assert!(svg.contains(
            r##"<circle cx="5" cy="2" r="1.5" fill="#FFFFFFFF" stroke="#000000FF" stroke-width="0.25" />"##
        ));
        assert!(svg.contains(r#"font-family="Bravura" font-weight="normal" font-style="normal""#));
        assert!(svg.contains(">a &amp; b</text>"));
        // A glyph goes out as text too, but always start-anchored on the
        // baseline, drawn from the origin it arrived with.
        assert!(svg.contains(
            r##"<text x="2" y="4" fill="#000000FF" font-size="12" font-family="Bravura" font-weight="normal" font-style="normal" text-anchor="start" dominant-baseline="baseline">"##
        ));
        assert!(svg.contains(
            r##"<polygon points="0,0 4,0 2,5" fill="#00FF00FF" stroke="none" stroke-width="0" />"##
        ));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn svg_canvas_emits_its_view_box_verbatim_ignoring_element_bounds() {
        // Elements sit inside a small box near the origin; the canvas is told
        // the page is a large, offset rectangle. The header must reflect the
        // page, not the elements.
        let svg =
            CanvasPainter::new(SvgCanvas::new((1400.0, 0.0, 1360.0, 1760.0))).paint(&one_of_each());

        assert!(svg.starts_with(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="1400 0 1360 1760" width="1360" height="1760">"#
        ));
        // Elements keep their global coordinates -- no translation to page-local.
        assert!(svg.contains(r#"x1="0" y1="0" x2="10" y2="4""#));
    }

    #[test]
    fn paint_pages_brackets_each_page_between_begin_and_finish() {
        // Every page is page-local, so two pages' elements can legitimately
        // occupy the same coordinates -- there is no cross-page offset to
        // keep them apart.
        let page = |number| RenderedPage {
            number,
            width: 100.0,
            height: 50.0,
            elements: vec![
                Line {
                    start: XY { x: 0.0, y: 0.0 },
                    end: XY { x: 10.0, y: 4.0 },
                    stroke_color: Color::BLACK,
                    stroke_width: 1.0,
                }
                .into(),
            ],
        };
        let pages = [page(1), page(2)];

        let calls = CanvasPainter::new(RecordingCanvas::default()).paint_pages(&pages);

        assert_eq!(
            calls,
            vec![
                // begin's bounds span every page's (superimposed) elements.
                "begin(0.0, 0.0, 10.0, 4.0)",
                "begin_page(100, 50)",
                "line",
                "begin_page(100, 50)",
                "line",
                "finish",
            ]
        );
    }
}
