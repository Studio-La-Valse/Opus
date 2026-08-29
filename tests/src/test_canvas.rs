#[cfg(test)]
mod tests {
    use lib::drawable::canvas::svg::SvgCanvas;
    use lib::drawable::canvas::{Canvas, CanvasPainter};
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::line::Line;
    use lib::drawable::elements::polygon::Polygon;
    use lib::drawable::elements::rect::Rect;
    use lib::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
    use lib::geometry::color::Color;
    use lib::geometry::xy::XY;

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

        fn draw_line(&mut self, _line: &Line) {
            self.calls.push("line".to_string());
        }

        fn draw_rect(&mut self, _rect: &Rect) {
            self.calls.push("rect".to_string());
        }

        fn draw_text(&mut self, _text: &Text<'_>) {
            self.calls.push("text".to_string());
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
            Text {
                text: "a & b",
                color: Color::BLACK,
                font_size: 12.0,
                font: "Bravura",
                xy: XY { x: 2.0, y: 3.0 },
                vertical_alignment: VerticalAlign::Middle,
                horizontal_alignment: HorizontalAlign::Center,
            }
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
                "text",
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
        let svg = CanvasPainter::new(SvgCanvas::new()).paint(&one_of_each());

        assert!(svg.starts_with(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 5""#));
        assert!(svg.contains(
            r##"<line x1="0" y1="0" x2="10" y2="4" stroke="#000000FF" stroke-width="1" />"##
        ));
        assert!(svg.contains(
            r##"<rect x="1" y="1" width="8" height="2" fill="#FFFFFFFF" stroke="#000000FF" stroke-width="0.5" />"##
        ));
        assert!(svg.contains(">a &amp; b</text>"));
        assert!(svg.contains(
            r##"<polygon points="0,0 4,0 2,5" fill="#00FF00FF" stroke="none" stroke-width="0" />"##
        ));
        assert!(svg.ends_with("</svg>"));
    }
}
